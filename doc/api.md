# rrcad API Reference

This document covers the public Rust API (`occt::Shape`, `ruby::vm::MrubyVm`),
the underlying cxx bridge signatures, and the CLI.

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

#### Solids

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

#### 2D Sketch Faces (for extrude / revolve)

| Method | Description |
|--------|-------------|
| `Shape::make_rect(w, h) -> Result<Shape>` | Rectangular face in the XY plane |
| `Shape::make_circle_face(r) -> Result<Shape>` | Circular face in the XY plane |
| `Shape::make_spline_2d(pts: &[f64]) -> Result<Shape>` | Closed profile in the XZ plane. `pts` is a flat `[r0, z0, r1, z1, …]` slice. Interpolates a BSpline, closes with a straight edge if the endpoints differ, returns a `Face`. Designed for `.revolve()`. |

#### 3D Wire Paths (for sweep)

| Method | Description |
|--------|-------------|
| `Shape::make_spline_3d(pts: &[f64]) -> Result<Shape>` | 3D BSpline `Wire`. `pts` is a flat `[x0, y0, z0, x1, y1, z1, …]` slice. Use as the `path` argument to `.sweep()`. |

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

Both operations apply to **all edges** of the shape.

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

All transforms are immutable — each returns a new `Shape`.

| Method | Description |
|--------|-------------|
| `.translate(dx, dy, dz) -> Result<Shape>` | Move by the given vector |
| `.rotate(ax, ay, az, angle_deg) -> Result<Shape>` | Rotate around axis `(ax,ay,az)` by `angle_deg` degrees |
| `.scale(factor) -> Result<Shape>` | Uniform scale about the origin |
| `.mirror(plane: &str) -> Result<Shape>` | Mirror about a coordinate plane. `plane` is `"xy"`, `"xz"`, or `"yz"` |

```rust
let moved    = part.translate(5.0, 0.0, 0.0)?;
let rotated  = part.rotate(0.0, 0.0, 1.0, 45.0)?;   // 45° around Z
let scaled   = part.scale(2.0)?;
let mirrored = part.mirror("xz")?;
```

> `rotate` returns `Err` if the axis vector is zero (`gp_Dir` requires a non-zero vector).

---

### Sketch Operations

| Method | Description |
|--------|-------------|
| `.extrude(height) -> Result<Shape>` | Extrude a face/wire along the Z axis by `height` |
| `.revolve(angle_deg) -> Result<Shape>` | Revolve around the Z axis by `angle_deg` degrees (360 for full revolution) |
| `.sweep(path: &Shape) -> Result<Shape>` | Sweep `self` (profile) along `path` (a Wire). Uses `BRepOffsetAPI_MakePipe`. |

```rust
let profile = Shape::make_rect(10.0, 5.0)?;
let solid   = profile.extrude(20.0)?;

let disc    = Shape::make_circle_face(3.0)?;
let ring    = disc.revolve(270.0)?;   // three-quarter revolution

let pts_2d  = vec![0.0_f64, 0.0, 3.0, 1.0, 4.0, 4.0, 0.0, 5.0];
let profile = Shape::make_spline_2d(&pts_2d)?;
let body    = profile.revolve(360.0)?;

let pts_3d  = vec![4.0_f64, 0.0, 0.0,  6.0, 0.0, 3.0,  8.0, 0.0, 6.0];
let path    = Shape::make_spline_3d(&pts_3d)?;
let spout   = Shape::make_circle_face(0.7)?.sweep(&path)?;
```

---

### Export

| Method | Description |
|--------|-------------|
| `.export_step(path: &str) -> Result<()>` | STEP AP203 boundary-representation file |
| `.export_stl(path: &str) -> Result<()>` | ASCII STL triangulated mesh |
| `.export_gltf(path: &str, linear_deflection: f64) -> Result<()>` | glTF 2.0 (text JSON). `linear_deflection` controls tessellation quality (e.g. `0.1` for 0.1 mm). |

```rust
part.export_step("/tmp/part.step")?;
part.export_stl("/tmp/part.stl")?;
part.export_gltf("/tmp/part.gltf", 0.1)?;
```

---

## cxx Bridge — `occt::ffi`

The `ffi` module is the raw cxx bridge that `Shape` delegates to. Documented
here for contributors adding new bindings.

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
fn make_box(dx: f64, dy: f64, dz: f64)              -> Result<UniquePtr<OcctShape>>;
fn make_cylinder(radius: f64, height: f64)            -> Result<UniquePtr<OcctShape>>;
fn make_sphere(radius: f64)                            -> Result<UniquePtr<OcctShape>>;

// 2D sketch faces
fn make_rect(w: f64, h: f64)                          -> Result<UniquePtr<OcctShape>>;
fn make_circle_face(r: f64)                            -> Result<UniquePtr<OcctShape>>;
fn make_spline_2d(pts: &[f64])                         -> Result<UniquePtr<OcctShape>>;
fn make_spline_3d(pts: &[f64])                         -> Result<UniquePtr<OcctShape>>;

// Boolean ops
fn shape_fuse  (a: &OcctShape, b: &OcctShape)         -> Result<UniquePtr<OcctShape>>;
fn shape_cut   (a: &OcctShape, b: &OcctShape)         -> Result<UniquePtr<OcctShape>>;
fn shape_common(a: &OcctShape, b: &OcctShape)         -> Result<UniquePtr<OcctShape>>;

// Fillets / chamfers
fn shape_fillet (shape: &OcctShape, radius: f64)      -> Result<UniquePtr<OcctShape>>;
fn shape_chamfer(shape: &OcctShape, dist: f64)        -> Result<UniquePtr<OcctShape>>;

// Transforms
fn shape_translate(shape: &OcctShape,
                   dx: f64, dy: f64, dz: f64)         -> Result<UniquePtr<OcctShape>>;
fn shape_rotate(shape: &OcctShape,
                axis_x: f64, axis_y: f64, axis_z: f64,
                angle_deg: f64)                        -> Result<UniquePtr<OcctShape>>;
fn shape_scale(shape: &OcctShape, factor: f64)        -> Result<UniquePtr<OcctShape>>;
fn shape_mirror(shape: &OcctShape, plane: &str)       -> Result<UniquePtr<OcctShape>>;

// Sketch operations
fn shape_extrude(shape: &OcctShape, height: f64)      -> Result<UniquePtr<OcctShape>>;
fn shape_revolve(shape: &OcctShape, angle_deg: f64)   -> Result<UniquePtr<OcctShape>>;
fn shape_sweep(profile: &OcctShape, path: &OcctShape) -> Result<UniquePtr<OcctShape>>;

// Export
fn export_step(shape: &OcctShape, path: &str)                         -> Result<()>;
fn export_stl (shape: &OcctShape, path: &str)                         -> Result<()>;
fn export_gltf(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
```

---

## mRuby VM — `ruby::vm::MrubyVm`

Wraps an `mrb_state *` lifecycle. One instance per process.

```rust
use rrcad::ruby::vm::MrubyVm;
```

| Method | Description |
|--------|-------------|
| `MrubyVm::new() -> MrubyVm` | Opens interpreter; evaluates the DSL prelude; registers native Shape class. Panics on allocation failure. |
| `.eval(code: &str) -> Result<String, String>` | Evaluates Ruby source; returns `Ok(inspect_result)` or `Err(exception_message)` |
| `Drop` | Calls `mrb_close` automatically |

```rust
let mut vm = MrubyVm::new();
match vm.eval("box(10, 20, 30).class") {
    Ok(result) => println!("=> {result}"),   // => Shape
    Err(e)     => eprintln!("Error: {e}"),
}
```

The result string is the mRuby `.inspect` representation of the last
evaluated expression. The error string is the inspected exception.

---

## Ruby DSL

The DSL is auto-loaded by `MrubyVm::new()` via `src/ruby/prelude.rb`. No
`require` is needed.

### Top-level methods

| Method | Description |
|--------|-------------|
| `box(dx, dy, dz)` | Rectangular solid |
| `cylinder(r, h)` | Cylinder along Z axis |
| `sphere(r)` | Sphere at origin |
| `rect(w, h)` | Rectangular face in XY plane |
| `circle(r)` | Circular face in XY plane |
| `spline_2d([[r,z], ...])` | Closed XZ-plane profile (for `revolve`) |
| `spline_3d([[x,y,z], ...])` | 3D wire path (for `sweep`) |
| `solid { ... }` | Block returning its last expression |
| `assembly("name") { \|a\| a.place shape }` | Named assembly |

### Shape instance methods

| Method | Description |
|--------|-------------|
| `.fuse(other)` | Union |
| `.cut(other)` | Subtraction |
| `.common(other)` | Intersection |
| `.translate(x, y, z)` | Move |
| `.rotate(ax, ay, az, deg)` | Rotate around axis by degrees |
| `.scale(factor)` | Uniform scale |
| `.mirror(:xy\|:xz\|:yz)` | Mirror about a coordinate plane |
| `.fillet(r)` | Round all edges |
| `.chamfer(d)` | Bevel all edges |
| `.extrude(h)` | Extrude face/wire along Z |
| `.revolve(deg=360)` | Revolve around Z axis |
| `.sweep(path)` | Sweep profile along a `spline_3d` wire |
| `.export("out.step")` | Write STEP file |

---

## CLI

```sh
rrcad                   # start REPL (readline, history, tab-completion)
rrcad --repl            # same as above
rrcad <script.rb>       # execute a .rb script and exit
```

**REPL commands:**

| Input | Effect |
|-------|--------|
| Any Ruby expression | Evaluates and prints `=> <result>` |
| `help` | Prints DSL quick-reference |
| `exit` / `quit` | Exits |
| Ctrl-D / Ctrl-C | Exits |

Tab-completion is available for top-level DSL identifiers and, after a `.`,
for Shape method names.
