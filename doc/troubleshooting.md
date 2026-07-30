# rrcad Troubleshooting Guide

---

## Build Failures

### `rake: command not found` or `mRuby build failed`

`build.rs` needs `rake` to compile mRuby from source.

```sh
gem install rake          # if Ruby is installed but rake is missing
sudo apt-get install ruby # Ubuntu/Debian — includes rake
```

Run `cargo clean && cargo build` after fixing.

---

### `libmruby.a: No such file or directory`

The mRuby submodule has not been initialised, or a previous `rake` build
failed silently.

```sh
git submodule update --init vendor/mruby
cd vendor/mruby && rake    # build manually to see full output
```

---

### `BRepPrimAPI_MakeBox.hxx: No such file or directory`

OCCT development headers are not installed.

```sh
sudo apt-get install -y \
  libocct-foundation-dev \
  libocct-modeling-data-dev \
  libocct-modeling-algorithms-dev \
  libocct-data-exchange-dev \
  libocct-ocaf-dev \
  libocct-visualization-dev \
  fontconfig fonts-dejavu-core
```

OCCT headers must be under `/usr/include/opencascade/`. Verify:

```sh
find /usr/include -name "BRepPrimAPI_MakeBox.hxx"
```

---

### `error[cxxbridge]: block must be declared unsafe extern "C++"`

You added a new function to the `extern "C++"` block in `src/occt/mod.rs`
but forgot the `unsafe` qualifier. The block must read:

```rust
unsafe extern "C++" {
    ...
}
```

In cxx, `unsafe extern "C++"` marks the *block* as one whose functions are
safe to call from Rust (you are asserting the C++ is sound). It does **not**
mean the functions require `unsafe {}` at the call site.

---

### `undefined reference to 'rrcad::...'` (linker error)

A function is declared in `bridge.h` and the `cxx::bridge` block but its
implementation is missing from `bridge.cpp`, or it was accidentally placed
outside `namespace rrcad`.

Check that every function listed in `bridge.h` has a matching definition
in `bridge.cpp` and that both are wrapped in `namespace rrcad { ... }`.

---

### `undefined symbol: TK...` (linker error)

A new OCCT call in `bridge.cpp` uses a class from a library that is not
linked. Find the relevant toolkit:

```sh
dpkg -S $(find /usr/include/opencascade -name "TheHeader.hxx")
```

Then add `println!("cargo:rustc-link-lib=TKTheLib");` to the link list in
`build.rs` and run `cargo build` again.

Beware headers that alias another class: `Font_BRepFont.hxx` is a thin
wrapper over `StdPrs_BRepFont`, whose symbols live in `TKV3d` — not in
`TKService` where the `Font_` package name suggests. When the header trick
points at the wrong toolkit, search the symbol itself:

```sh
for lib in /usr/lib/x86_64-linux-gnu/libTK*.so; do
  nm -D --defined-only "$lib" 2>/dev/null | grep -q TheSymbol && echo "$lib"
done
```

---

### `cxx-build` fails to find `rust/cxx.h`

This usually means `cxx` and `cxx-build` are on different patch versions.
Ensure both are `"1.0"` in `Cargo.toml` (cargo will resolve them to the
same patch release):

```toml
[dependencies]
cxx = "1.0"

[build-dependencies]
cxx-build = "1.0"
```

---

## Runtime / Test Failures

### `BRepFilletAPI_MakeFillet failed — degenerate edges`

Fillet iterates every edge of the shape with `TopExp_Explorer` and attempts
to round it. This can fail when:

- The radius is larger than the smallest edge length or face.
- The shape came from a boolean cut that left zero-length or near-degenerate
  edges.
- The fillet radius is exactly zero (no-op but OCCT may reject it).

**Workarounds:**
- Use a smaller radius.
- Call `fillet` before boolean operations when possible.
- For post-boolean fillets, shape healing will be added in a later phase.

---

### `BRepAlgoAPI_Fuse/Cut/Common failed`

OCCT boolean operations can fail if:

- The two input shapes are disjoint (for `common`) or identical (for `cut`).
- The shapes share a face but have incompatible topology (floating-point
  tolerance issues).

Try translating one shape slightly so it overlaps cleanly, or check that
both shapes are valid solids (closed shells).

---

### `STEPControl_Writer::Write failed`

- The target directory does not exist — OCCT will not create it. Create the
  directory first.
- The path contains non-ASCII characters on some platforms — use ASCII paths
  for STEP output for now.

---

### `text: no font found for 'sans-serif'`

`text()` resolves its default face through fontconfig, so a headless or
minimal system with no fonts installed cannot render glyphs. Install any
font family (`fonts-dejavu-core` is enough) or pass `font:` with a family
name that exists on the machine or a path to a `.ttf`/`.otf` file:

```ruby
text("V2", size: 6, font: "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf")
```

`text: could not load font file '…'` means the `font:` argument looked like
a path (contains `/` or ends in `.ttf`/`.otf`) but the file is missing or
not a parseable font.

---

### `RWGltf_CafWriter::Perform failed`

The glTF pipeline runs tessellation first (`BRepMesh_IncrementalMesh`). If
the shape is degenerate, tessellation may produce no triangles and the
writer will fail.

Also check that the output directory exists.

Also check that `linear_deflection:` is positive and not extremely small
(e.g. `1e-10`), which asks for a mesh with more triangles than memory allows.
It defaults to 0.1 mm; a reasonable value for anything else is `size / 1000`,
where `size` is the largest dimension.

---

### `export_3mf: shape tessellated to no triangles`

The shape has no surface to print. Usually it is a wire or an edge — a
`helix`, a `polyline`, a sketch that was never turned into a face — or a
compound that ended up empty after a boolean removed everything.

Check with `shape.shape_type` and `shape.volume`. If the intent was to export
a path rather than a body, `sweep` or `pipe` it into a solid first; if a
boolean emptied the shape, the cutting tool was larger than expected.

The export fails rather than writing an empty package, which a slicer would
open and display nothing for.

---

### REPL prints garbled results or crashes

The result and error strings returned by `rrcad_mrb_eval()` are owned by
mRuby's GC. They must be copied (to `String`) before the next `eval()` call
or a GC cycle. The current implementation in `vm.rs` does this correctly via
`CStr::to_string_lossy().into_owned()`.

If you modify `glue.c`, ensure the returned `const char *` is always
pointing to mRuby-managed memory (from `mrb_str_to_cstr`) rather than
stack memory.

---

### `gp_Dir` exception: `Standard_ConstructionError` (zero-vector axis)

`Shape::rotate` with `(ax=0, ay=0, az=0, ...)` will return `Err(...)` because
`gp_Dir` requires a non-zero vector. This is expected behaviour. Guard against
it at the call site:

```rust
if ax == 0.0 && ay == 0.0 && az == 0.0 {
    return Err("rotation axis must be non-zero".to_string());
}
```

---

## Incremental Build Tips

- mRuby is rebuilt only if `vendor/mruby/build/host/lib/libmruby.a` is
  missing. Changing mRuby source files will **not** trigger a rebuild
  automatically; delete the `.a` file and run `cargo build`.

- The OCCT bridge is recompiled when `src/occt/mod.rs`, `src/occt/bridge.h`,
  or `src/occt/bridge.cpp` change (tracked by `rerun-if-changed` in
  `build.rs`).

- `cargo clean` removes all build artefacts including `libmruby.a`, so the
  next build will recompile mRuby from scratch (takes ~1–2 minutes).
