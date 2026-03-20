# rrcad Developer Guide

## Table of Contents

- [Prerequisites](#prerequisites)
- [Repository Layout](#repository-layout)
- [Build System](#build-system)
- [Architecture](#architecture)
- [Layer-by-layer Walkthrough](#layer-by-layer-walkthrough)
- [Adding New OCCT Bindings](#adding-new-occt-bindings)
- [Adding New Ruby Bindings (Phase 1)](#adding-new-ruby-bindings-phase-1)
- [Testing](#testing)
- [Code Style](#code-style)

---

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust toolchain | stable, edition 2024 | compiler + cargo |
| Ruby + rake | any modern | builds mRuby |
| g++ / clang++ | C++17-capable | OCCT bridge |
| OCCT dev headers | 7.7+ (7.9 tested) | geometry kernel |

**Ubuntu / Debian:**

```sh
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build tools + OCCT
sudo apt-get install -y \
  ruby rake \
  libocct-foundation-dev \
  libocct-modeling-data-dev \
  libocct-modeling-algorithms-dev \
  libocct-data-exchange-dev \
  libocct-ocaf-dev
```

**mRuby submodule:**

```sh
git submodule update --init vendor/mruby
```

The first `cargo build` will run `rake` inside `vendor/mruby/` and compile
`libmruby.a`. Subsequent builds skip this step unless the `.a` is missing.

---

## Repository Layout

```
rrcad/
├── Cargo.toml              # crate manifest
├── build.rs                # three-phase build orchestration
├── src/
│   ├── main.rs             # CLI entry point (REPL + script modes)
│   ├── ruby/               # mRuby integration
│   │   ├── mod.rs          # re-exports ffi + vm submodules
│   │   ├── ffi.rs          # extern "C" bindings to libmruby + glue.c
│   │   ├── vm.rs           # MrubyVm: safe RAII wrapper
│   │   └── glue.c          # C shim that hides mrb_value from Rust
│   └── occt/               # OCCT geometry bindings
│       ├── mod.rs          # cxx::bridge + safe Shape wrapper + tests
│       ├── bridge.h        # C++ header: OcctShape class + fn declarations
│       └── bridge.cpp      # C++ implementation of all OCCT operations
├── vendor/
│   └── mruby/              # git submodule — mRuby 3.4.0
└── doc/
    ├── TODOs.md            # phased roadmap
    ├── development.md      # this file
    ├── api.md              # API reference
    └── troubleshooting.md  # common issues
```

---

## Build System

`build.rs` runs three independent compilation pipelines in sequence:

### 1 — mRuby static library

Checks for `vendor/mruby/build/host/lib/libmruby.a`. If missing, runs
`rake` in `vendor/mruby/`. Links `libmruby` (static) and `libm`.

### 2 — C glue shim

Compiles `src/ruby/glue.c` via the `cc` crate. The shim wraps
`mrb_load_string` and hides `mrb_value` from Rust, keeping the FFI surface
minimal.

### 3 — OCCT C++ bridge

```rust
cxx_build::bridge("src/occt/mod.rs")
    .file("src/occt/bridge.cpp")
    .include("/usr/include/opencascade")
    .include("src/occt")   // so bridge.cpp can #include "bridge.h"
    .flag_if_supported("-std=c++17")
    .compile("rrcad_occt_bridge");
```

`cxx-build` parses the `#[cxx::bridge]` macro in `mod.rs`, generates C++
shim code, and compiles it together with `bridge.cpp`. It automatically
adds the cxx runtime headers (`rust/cxx.h`) to the include path.

**Linked OCCT libraries:**
`TKernel` · `TKMath` · `TKG2d` · `TKG3d` · `TKBRep` · `TKGeomBase` ·
`TKGeomAlgo` · `TKTopAlgo` · `TKPrim` · `TKBool` · `TKBO` · `TKFillet` ·
`TKShHealing` · `TKMesh` · `TKCDF` · `TKCAF` · `TKLCAF` · `TKXCAF` ·
`TKXSBase` · `TKDESTEP` · `TKDESTL` · `TKDEGLTF` · `TKRWMesh`

---

## Architecture

```
Ruby DSL (.rb script)
      │
   mRuby VM  (vendor/mruby — pinned to 3.4.0)
      │
   src/ruby/glue.c          ← C shim; hides mrb_value from Rust
   src/ruby/ffi.rs          ← extern "C" bindings
   src/ruby/vm.rs           ← MrubyVm: safe RAII wrapper
      │
   [Phase 1 TODO: Shape Ruby class + SlotMap<u64, OcctShape>]
      │
   src/occt/mod.rs          ← cxx::bridge + Shape wrapper
   src/occt/bridge.h        ← OcctShape class declaration
   src/occt/bridge.cpp      ← OCCT C++ implementations
      │
   OCCT 7.9 (system-installed)
   BRep modeling · boolean ops · tessellation
   STEP / STL / glTF export
```

**Memory ownership rule:** Rust owns all OCCT shapes. In Phase 1, a
`SlotMap<u64, UniquePtr<OcctShape>>` will be the single source of truth.
mRuby `RData` holds only a `u64` key; the `dfree` GC callback removes the
key from the map, dropping the C++ shape automatically. No cross-language
reference counting.

---

## Layer-by-layer Walkthrough

### mRuby layer (`src/ruby/`)

`glue.c` is the seam between C and Rust. It exposes one function:

```c
const char *rrcad_mrb_eval(mrb_state *mrb, const char *code,
                            const char **error_out);
```

On success it returns a pointer to the mRuby-GC-owned inspected result
string and sets `*error_out = NULL`. On exception it returns `NULL` and
sets `*error_out` to the inspected exception string. Both pointers are
owned by mRuby's GC — callers must copy before the next eval or GC cycle.

`ffi.rs` wraps this with raw `extern "C"` declarations. `vm.rs` wraps
those in a safe `MrubyVm` struct that manages the `mrb_state *` lifetime
via `Drop`.

### OCCT layer (`src/occt/`)

**Bridge type — `rrcad::OcctShape`:**

Declared in `bridge.h` and always transferred as
`std::unique_ptr<OcctShape>` across the bridge. It wraps a `TopoDS_Shape`
value (which internally uses OCCT handle-based reference counting, making
the copy on construction cheap). The class is non-copyable and non-movable
so that cxx can manage lifetime exclusively through `UniquePtr<OcctShape>`
on the Rust side.

**Fillet / chamfer edge iteration:**

Both `shape_fillet` and `shape_chamfer` apply to *all* edges using
`TopExp_Explorer`:

```cpp
TopExp_Explorer exp(shape, TopAbs_EDGE);
for (; exp.More(); exp.Next())
    builder.Add(radius, TopoDS::Edge(exp.Current()));
```

The `TopoDS::Edge()` downcast is required — `exp.Current()` returns
`TopoDS_Shape`.

**glTF export pipeline:**

glTF export requires three steps that STEP/STL do not:
1. Tessellate with `BRepMesh_IncrementalMesh` (linear deflection controls quality).
2. Create an XDE document (`XCAFApp_Application::GetApplication()` singleton,
   then `NewDocument("BinXCAF", doc)`).
3. Add the shape to `XCAFDoc_ShapeTool`, then write with `RWGltf_CafWriter`.

---

## Adding New OCCT Bindings

To expose a new OCCT operation (e.g. `BRepOffsetAPI_MakeThickSolid`):

**1. Declare in `src/occt/bridge.h`** (inside `namespace rrcad`):

```cpp
std::unique_ptr<OcctShape> shape_shell(const OcctShape& shape, double offset);
```

**2. Implement in `src/occt/bridge.cpp`** (inside `namespace rrcad`):

```cpp
std::unique_ptr<OcctShape> shape_shell(const OcctShape& s, double offset) {
    // ... OCCT call ...
    if (!op.IsDone())
        throw std::runtime_error("shape_shell failed");
    return wrap(op.Shape());
}
```

Add the required OCCT `#include` at the top of `bridge.cpp`. If it comes
from a new TK library, add `println!("cargo:rustc-link-lib=TKOffset");`
(or the appropriate name) to the `build.rs` link list.

**3. Declare in the `cxx::bridge` block in `src/occt/mod.rs`:**

```rust
fn shape_shell(shape: &OcctShape, offset: f64) -> Result<UniquePtr<OcctShape>>;
```

**4. Add a safe Rust method to `Shape`:**

```rust
pub fn shell(&self, offset: f64) -> Result<Shape, String> {
    ffi::shape_shell(&self.inner, offset)
        .map(|p| Shape { inner: p })
        .map_err(|e| e.to_string())
}
```

**5. Write a test.**

---

## Adding New Ruby Bindings (Phase 1)

> This section describes the *planned* pattern for Phase 1.

1. Define a `Shape` Ruby class in Rust using `mrb_define_class`.
2. Register methods via `mrb_define_method` pointing to C callbacks.
3. Use `mrb_data_object_alloc` to store a `u64` SlotMap key in mRuby's
   `RData`, with a `dfree` callback that removes the key and drops the
   OCCT shape.
4. C callbacks retrieve the `u64` key from `RData`, look up the shape in
   the SlotMap, call the appropriate `src/occt` function, and store the
   new shape key back.

---

## Testing

```sh
cargo test                        # all tests
cargo test smoke                  # OCCT smoke tests only
cargo test <name_substring>       # single test by name
cargo clippy                      # lints
```

Current tests live in `src/occt/mod.rs`:

| Test | What it checks |
|------|----------------|
| `smoke_filleted_box_to_step` | `make_box` → `fillet` → `export_step`; verifies STEP header |
| `smoke_boolean_cut` | `make_box` → `cut(cylinder)` → `export_step` |

Output files are written to `std::env::temp_dir()` (typically `/tmp` on Linux).

---

## Code Style

### Rust

Standard `rustfmt` formatting; `cargo clippy` must pass clean.

### C++

C++ code is formatted with **clang-format**. The project config is in
`.clang-format` at the repository root (LLVM base style, 100-column limit,
4-space indent, left pointer alignment).

**Check formatting (dry run):**

```sh
clang-format --dry-run -Werror src/occt/bridge.h src/occt/bridge.cpp
```

**Apply formatting in-place:**

```sh
clang-format -i src/occt/bridge.h src/occt/bridge.cpp
```

Install clang-format if missing:

```sh
sudo apt-get install -y clang-format
```

**Additional C++ rules:**
- Keep all bridge code in `namespace rrcad`.
- Throw `std::runtime_error` on failure — cxx converts it to a Rust `Err`.
- Every new OCCT binding must check `IsDone()` (or the equivalent return
  status) and throw on failure — do not silently return a null shape.
- Add a `cargo test` smoke test for every new shape operation.
