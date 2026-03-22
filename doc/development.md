# rrcad Developer Guide

## Table of Contents

- [Prerequisites](#prerequisites)
- [Repository Layout](#repository-layout)
- [Build System](#build-system)
- [Architecture](#architecture)
- [Layer-by-layer Walkthrough](#layer-by-layer-walkthrough)
- [Adding New OCCT Bindings](#adding-new-occt-bindings)
- [Adding New Ruby DSL Methods](#adding-new-ruby-dsl-methods)
- [Testing](#testing)
- [Code Style](#code-style)

---

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust toolchain | stable, edition 2024 | compiler + cargo |
| Ruby + rake | any modern | builds mRuby |
| g++ / clang++ | C++17-capable | OCCT bridge |
| clang-format | any | C++ formatting (enforced via hook) |
| OCCT dev headers | 7.7+ (7.9 tested) | geometry kernel |

**Ubuntu / Debian:**

```sh
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build tools + OCCT
sudo apt-get install -y \
  ruby rake \
  clang-format \
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
│   ├── lib.rs              # library root (re-exports occt, preview, ruby)
│   ├── main.rs             # CLI entry point (REPL + script + --preview modes)
│   ├── ruby/               # mRuby integration
│   │   ├── mod.rs          # re-exports ffi, vm, native submodules
│   │   ├── ffi.rs          # extern "C" declarations for libmruby + glue.c
│   │   ├── vm.rs           # MrubyVm: safe RAII wrapper
│   │   ├── native.rs       # Rust extern "C" fns called from glue.c
│   │   ├── glue.c          # C shim hiding mrb_value from Rust
│   │   └── prelude.rb      # DSL prelude embedded via include_str!
│   ├── occt/               # OCCT geometry bindings
│   │   ├── mod.rs          # cxx::bridge + safe Shape wrapper + tests
│   │   ├── bridge.h        # C++ header: OcctShape class + fn declarations
│   │   └── bridge.cpp      # C++ implementation of all OCCT operations
│   └── preview/            # Live browser preview (Phase 3)
│       ├── mod.rs          # PreviewState, PREVIEW global, start()
│       ├── server.rs       # axum routes: /, /model.glb, /ws
│       └── viewer.html     # Three.js viewer (embedded via include_str!)
├── samples/                # DSL example scripts
│   ├── README.md
│   ├── 01_hello_box.rb … 07_teapot.rb
│   ├── 08_parametric_box.rb  # Phase 5: param DSL demo
│   └── 08_box_sizes.csv      # design-table CSV for the parametric box
├── tests/                  # integration test suites
│   ├── occt_layer.rs       # OCCT Rust API smoke tests
│   ├── vm_layer.rs         # MrubyVm eval smoke tests
│   ├── prelude_layer.rs    # DSL prelude + native override tests
│   ├── e2e_dsl.rs          # Phase 1–5 end-to-end tests (export, color, mate, simplify)
│   ├── phase2_dsl.rs       # Phase 2 end-to-end tests
│   ├── teapot_dsl.rs       # Phase 3 spline/sweep (incl. tangent variants)
│   ├── teapot_sample.rs    # Full Utah teapot sample smoke tests
│   ├── phase3_selectors.rs # Phase 3 face/edge sub-shape selector tests
│   ├── phase4_3d_ops.rs    # Phase 4: shell, offset, loft, extrude_ex, patterns
│   └── phase5_params.rs    # Phase 5: param DSL, --param overrides, design table
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
`mrb_load_string` and registers the native Shape class; it hides `mrb_value`
from Rust, keeping the FFI surface minimal.

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
`TKOffset` · `TKShHealing` · `TKMesh` · `TKCDF` · `TKCAF` · `TKLCAF` ·
`TKXCAF` · `TKXSBase` · `TKDESTEP` · `TKDESTL` · `TKDEGLTF` · `TKRWMesh`

---

## Architecture

```
Ruby DSL (.rb script)
      │
   mRuby VM  (vendor/mruby — pinned to 3.4.0)
      │
   src/ruby/prelude.rb      ← DSL prelude (embedded via include_str!)
   src/ruby/glue.c          ← C shim; hides mrb_value from Rust
   src/ruby/native.rs       ← extern "C" entry points (called from glue.c)
   src/ruby/ffi.rs          ← extern "C" declarations on the Rust side
   src/ruby/vm.rs           ← MrubyVm: safe RAII wrapper
      │
   src/occt/mod.rs          ← cxx::bridge + Shape wrapper
   src/occt/bridge.h        ← OcctShape class declaration
   src/occt/bridge.cpp      ← OCCT C++ implementations
      │
   OCCT 7.9 (system-installed)
   BRep modeling · boolean ops · splines · tessellation
   STEP / STL / glTF export
```

**Memory ownership:** Each native `Shape` is a heap-allocated `Box<occt::Shape>`.
The raw pointer is stored directly in the mRuby `RData void*` slot —
there is **no SlotMap**. When mRuby's GC collects a `Shape` object it calls
`shape_dfree` → `rrcad_shape_drop` → `drop(Box::from_raw(ptr))`.
No cross-language reference counting.

**Startup sequence:**
1. `MrubyVm::new()` opens `mrb_state`.
2. Evaluates `prelude.rb` (embedded via `include_str!`), which defines
   `Shape`, `Assembly`, and Kernel stub methods.
3. Calls `rrcad_register_shape_class(mrb)` (in `glue.c`), which overrides the
   prelude stubs with native C/Rust implementations. All implemented methods
   are fully native; prelude stubs remain only for operations not yet wired up.

---

## Layer-by-layer Walkthrough

### mRuby layer (`src/ruby/`)

`glue.c` is the seam between C and Rust. It exposes:

```c
// Evaluate Ruby source — returns inspected result or NULL + error message.
const char *rrcad_mrb_eval(mrb_state *mrb, const char *code,
                            const char **error_out);

// Register native Shape class and all DSL methods.
// Called once per MrubyVm after the prelude runs.
void rrcad_register_shape_class(mrb_state *mrb);
```

Each native DSL method in `glue.c` extracts arguments with `mrb_get_args`,
calls the corresponding `extern "C"` function declared in `native.rs`, wraps
the resulting raw pointer with `shape_from_ptr`, and returns the new `mrb_value`.

Errors flow back through an `*error_out` parameter: Rust writes a pointer to
a thread-local `CString`; the C handler checks it and calls `mrb_raise`.

`ffi.rs` wraps the C declarations. `vm.rs` wraps those in a safe `MrubyVm`
struct that manages the `mrb_state *` lifetime via `Drop`.

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

**Spline construction (Phase 3 + Tier 4):**

`make_spline_2d` and `make_spline_3d` both use `GeomAPI_Interpolate` with a
`TColgp_HArray1OfPnt` to fit a BSpline through the given control points.
`make_spline_2d` additionally closes the profile with a straight edge (if
endpoints differ) and builds a `Face` via `BRepBuilderAPI_MakeFace` on the
XZ plane — suitable for `revolve`. `make_spline_3d` returns a bare `Wire`
— suitable for `sweep`.

`make_spline_2d_tan` and `make_spline_3d_tan` are the tangent-constrained
variants: they call `GeomAPI_Interpolate::Load(startTangent, endTangent)`
before `Perform()` to suppress endpoint oscillation on short splines. The DSL
exposes these via the optional `tangents:` keyword argument.

**Pipe sweep (Phase 3):**

`shape_sweep` asserts the path is a `TopAbs_WIRE` then delegates directly to
`BRepOffsetAPI_MakePipe` (`TKOffset`).

**glTF / GLB export pipeline:**

Both `export_gltf` (text JSON + companion `.bin`) and `export_glb` (binary,
single file) share the same three-step pipeline that STEP/STL do not require:
1. Tessellate with `BRepMesh_IncrementalMesh` (linear deflection controls quality).
2. Create an XDE document (`XCAFApp_Application::GetApplication()` singleton,
   then `NewDocument("BinXCAF", doc)`).
3. Add the shape to `XCAFDoc_ShapeTool`, then write with `RWGltf_CafWriter`
   (`isBinary=false` for glTF, `isBinary=true` for GLB).

The live preview uses `export_glb` exclusively — a single `.glb` file is
served at `GET /model.glb` without needing to coordinate a companion `.bin`.

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
from a new TK library, add `println!("cargo:rustc-link-lib=TKTheLib");` to
the link list in `build.rs`.

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

## Adding New Ruby DSL Methods

To wire a new OCCT binding into the Ruby DSL:

**1. Add an `extern "C"` entry point in `src/ruby/native.rs`:**

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_shell(
    ptr: *mut c_void,
    offset: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.shell(offset) {
        Ok(s) => Box::into_raw(Box::new(s)) as *mut c_void,
        Err(e) => { unsafe { set_err(error_out, &e) }; std::ptr::null_mut() }
    }
}
```

**2. Forward-declare the function in `src/ruby/glue.c`:**

```c
extern void* rrcad_shape_shell(void* ptr, double offset, const char** error_out);
```

**3. Add a C handler in `glue.c`:**

```c
static mrb_value mrb_rrcad_shape_shell(mrb_state* mrb, mrb_value self) {
    mrb_float offset;
    mrb_get_args(mrb, "f", &offset);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_shell(ptr, (double)offset, &err);
    if (err) mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}
```

**4. Register the method in `rrcad_register_shape_class`:**

```c
mrb_define_method(mrb, shape_class, "shell", mrb_rrcad_shape_shell, MRB_ARGS_REQ(1));
```

**5. Add a prelude stub in `src/ruby/prelude.rb`** (for documentation and
graceful failure before the native override runs):

```ruby
def shell(_offset)
  raise NotImplementedError, "Shape#shell is not yet implemented"
end
```

**6. Add to the `SHAPE_METHODS` completion list in `src/main.rs` and update
`HELP_TEXT`.**

**7. Write a test.**

---

## Testing

```sh
cargo test                        # all tests
cargo test --test teapot_dsl      # single integration test file
cargo test smoke                  # name-filter: OCCT smoke tests
cargo clippy                      # lints
```

| Test file | What it covers |
|-----------|----------------|
| `src/occt/mod.rs` (inline) | OCCT Rust API: box→fillet→STEP, boolean cut, color, mate |
| `tests/occt_layer.rs` | All OCCT primitives, booleans, transforms, fillets, export |
| `tests/vm_layer.rs` | `MrubyVm` eval: types, errors, persistence, multiple VMs |
| `tests/prelude_layer.rs` | DSL prelude stubs; native overrides for all implemented methods (Phases 1–5); Assembly |
| `tests/e2e_dsl.rs` | Phase 1–5 end-to-end: export formats, color, mate, simplify |
| `tests/phase2_dsl.rs` | Phase 2 end-to-end: transforms, mirror, rect/circle, extrude/revolve |
| `tests/teapot_dsl.rs` | Phase 3: spline_2d/3d (incl. tangent variants), sweep |
| `tests/teapot_sample.rs` | Full Utah teapot sample: 4 part tests + 1 assembly |
| `tests/phase3_selectors.rs` | Phase 3: `.faces(:top|:bottom|:side|:all)`, `.edges(:vertical|:horizontal|:all)` |
| `tests/phase4_3d_ops.rs` | Phase 4: shell, offset, loft, extrude_ex (twist/scale), linear/polar patterns |
| `tests/phase5_params.rs` | Phase 5: `param` DSL, `--param` overrides, design table batch export |

Output files are written to `std::env::temp_dir()` (typically `/tmp` on Linux).

---

## Code Style

Formatting is **enforced automatically** by hooks in `.claude/settings.json`.
After every Write or Edit tool call, the hook runs the appropriate formatter
on the saved file — no manual step is needed.

### Rust

`rustfmt` runs automatically on every `*.rs` file. `cargo clippy` must also pass clean.

### C++

`clang-format -i` runs automatically on every `*.h` / `*.cpp` file. The
project config is in `.clang-format` at the repository root (LLVM base
style, 100-column limit, 4-space indent, left pointer alignment).

To check manually (e.g. in CI):

```sh
clang-format --dry-run -Werror src/occt/bridge.h src/occt/bridge.cpp
```

**Additional C++ rules:**
- Keep all bridge code in `namespace rrcad`.
- Throw `std::runtime_error` on failure — cxx converts it to a Rust `Err`.
- Every new OCCT binding must check `IsDone()` (or the equivalent return
  status) and throw on failure — do not silently return a null shape.
- Add a `cargo test` smoke test for every new shape operation.
