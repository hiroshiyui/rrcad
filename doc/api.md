# rrcad API Reference

This document covers the public Rust API, the underlying cxx bridge, and
the mRuby VM interface. The Ruby DSL itself is defined in Phase 1 and
Phase 2 (see `doc/TODOs.md`).

---

## Rust API — `occt::Shape`

`Shape` is the primary type for geometry work. Every method is immutable
(takes `&self`) and returns a *new* `Shape` — shapes are never mutated
in-place.

All methods return `Result<_, String>`. On C++ failure the error string
contains the message passed to `std::runtime_error` in the bridge.

```rust
use rrcad::occt::Shape;
```

### Constructors

| Method | Description |
|--------|-------------|
| `Shape::make_box(dx, dy, dz) -> Result<Shape>` | Axis-aligned box with corner at origin |
| `Shape::make_cylinder(radius, height) -> Result<Shape>` | Cylinder along the Z axis |
| `Shape::make_sphere(radius) -> Result<Shape>` | Sphere centred at origin |

```rust
let b = Shape::make_box(10.0, 20.0, 30.0)?;
let c = Shape::make_cylinder(5.0, 15.0)?;
let s = Shape::make_sphere(8.0)?;
```

---

### Boolean Operations

All boolean operations return a new `Shape` and leave the inputs unchanged.

| Method | Description |
|--------|-------------|
| `.fuse(&other) -> Result<Shape>` | Union of `self` and `other` |
| `.cut(&other) -> Result<Shape>` | Subtract `other` from `self` |
| `.common(&other) -> Result<Shape>` | Intersection of `self` and `other` |

```rust
let base  = Shape::make_box(20.0, 20.0, 20.0)?;
let hole  = Shape::make_cylinder(4.0, 25.0)?;
let part  = base.cut(&hole)?;
```

---

### Fillets and Chamfers

Both operations apply to **all edges** of the shape. Selective edge
application is planned for Phase 2.

| Method | Description |
|--------|-------------|
| `.fillet(radius) -> Result<Shape>` | Round every edge to the given radius |
| `.chamfer(dist) -> Result<Shape>` | Bevel every edge by the given distance |

```rust
let rounded = part.fillet(2.0)?;
```

> **Note:** Fillet can fail on degenerate topology produced by certain
> boolean operations. See `doc/troubleshooting.md`.

---

### Transforms

Transforms are immutable — each returns a new `Shape`.

| Method | Description |
|--------|-------------|
| `.translate(dx, dy, dz) -> Result<Shape>` | Move by the given vector |
| `.rotate(ax, ay, az, angle_deg) -> Result<Shape>` | Rotate around axis `(ax,ay,az)` by `angle_deg` degrees. Axis need not be pre-normalised. |
| `.scale(factor) -> Result<Shape>` | Uniform scale about the origin |

```rust
let moved   = part.translate(5.0, 0.0, 0.0)?;
let rotated = part.rotate(0.0, 0.0, 1.0, 45.0)?;   // 45° around Z
let scaled  = part.scale(2.0)?;
```

> `rotate` throws if the axis vector is zero (`gp_Dir` construction
> error propagates to Rust as `Err`).

---

### Export

| Method | Description |
|--------|-------------|
| `.export_step(path: &str) -> Result<()>` | STEP AP203 boundary-representation file |
| `.export_stl(path: &str) -> Result<()>` | ASCII STL triangulated mesh |
| `.export_gltf(path: &str, linear_deflection: f64) -> Result<()>` | glTF 2.0 (text JSON, `.gltf`). `linear_deflection` controls tessellation quality (e.g. `0.1` for 0.1 mm on a 10 mm part). |

```rust
part.export_step("/tmp/part.step")?;
part.export_stl("/tmp/part.stl")?;
part.export_gltf("/tmp/part.gltf", 0.1)?;
```

---

## cxx Bridge — `occt::ffi`

The `ffi` module is not intended for direct use; it is the raw cxx bridge
that `Shape` delegates to. It is documented here for contributors adding
new bindings.

All bridge functions live in `namespace rrcad` on the C++ side.

### Opaque Type

```rust
// Rust
type OcctShape;           // opaque; only accessible via UniquePtr<OcctShape>
```

```cpp
// C++ (src/occt/bridge.h)
class rrcad::OcctShape {
    TopoDS_Shape shape_;  // BRep handle-counted shape value
public:
    explicit OcctShape(TopoDS_Shape s) noexcept;
    const TopoDS_Shape& get() const noexcept;
    TopoDS_Shape&       get()       noexcept;
    // non-copyable, non-movable
};
```

### Bridge Function Signatures

```rust
// Primitives
fn make_box(dx: f64, dy: f64, dz: f64)          -> Result<UniquePtr<OcctShape>>;
fn make_cylinder(radius: f64, height: f64)        -> Result<UniquePtr<OcctShape>>;
fn make_sphere(radius: f64)                        -> Result<UniquePtr<OcctShape>>;

// Boolean ops
fn shape_fuse  (a: &OcctShape, b: &OcctShape)     -> Result<UniquePtr<OcctShape>>;
fn shape_cut   (a: &OcctShape, b: &OcctShape)     -> Result<UniquePtr<OcctShape>>;
fn shape_common(a: &OcctShape, b: &OcctShape)     -> Result<UniquePtr<OcctShape>>;

// Fillets / chamfers
fn shape_fillet (shape: &OcctShape, radius: f64)  -> Result<UniquePtr<OcctShape>>;
fn shape_chamfer(shape: &OcctShape, dist: f64)    -> Result<UniquePtr<OcctShape>>;

// Transforms
fn shape_translate(shape: &OcctShape, dx: f64, dy: f64, dz: f64)
                                                   -> Result<UniquePtr<OcctShape>>;
fn shape_rotate(shape: &OcctShape,
                axis_x: f64, axis_y: f64, axis_z: f64,
                angle_deg: f64)                    -> Result<UniquePtr<OcctShape>>;
fn shape_scale(shape: &OcctShape, factor: f64)    -> Result<UniquePtr<OcctShape>>;

// Export
fn export_step(shape: &OcctShape, path: &str)                      -> Result<()>;
fn export_stl (shape: &OcctShape, path: &str)                      -> Result<()>;
fn export_gltf(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
```

---

## mRuby VM — `ruby::vm::MrubyVm`

Wraps an `mrb_state *` lifecycle. Intended as the single interpreter
instance per process.

```rust
use rrcad::ruby::vm::MrubyVm;
```

| Method | Description |
|--------|-------------|
| `MrubyVm::new() -> MrubyVm` | Opens a new interpreter; panics on allocation failure |
| `.eval(code: &str) -> Result<String, String>` | Evaluates Ruby source; returns `Ok(inspect_result)` or `Err(exception_message)` |
| `Drop` | Calls `mrb_close` automatically |

```rust
let mut vm = MrubyVm::new();
match vm.eval("1 + 1") {
    Ok(result) => println!("=> {result}"),   // => 2
    Err(e)     => eprintln!("Error: {e}"),
}
```

The result string is the mRuby `.inspect` representation of the last
evaluated expression. The error string is the inspected exception.

---

## CLI

```
rrcad                   # start REPL (readline, history)
rrcad --repl            # same as above
rrcad <script.rb>       # execute script (Phase 1 — not yet implemented)
```

**REPL commands:**

| Input | Effect |
|-------|--------|
| Any Ruby expression | Evaluates and prints `=> <result>` |
| `exit` / `quit` | Exits |
| Ctrl-D / Ctrl-C | Exits |
