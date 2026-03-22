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
| `Shape::make_cone(r1, r2, height) -> Result<Shape>` | Cone/frustum along the Z axis (`r1` = base radius, `r2` = top radius) |
| `Shape::make_torus(r1, r2) -> Result<Shape>` | Torus in the XY plane (`r1` = major radius, `r2` = tube radius) |
| `Shape::make_wedge(dx, dy, dz, ltx) -> Result<Shape>` | Wedge with base `dx×dz`, height `dy`, and top face width `ltx` along X |

```rust
let b = Shape::make_box(10.0, 20.0, 30.0)?;
let c = Shape::make_cylinder(5.0, 15.0)?;
let s = Shape::make_sphere(8.0)?;
let k = Shape::make_cone(4.0, 1.0, 10.0)?;
let t = Shape::make_torus(6.0, 1.5)?;
let w = Shape::make_wedge(10.0, 8.0, 6.0, 4.0)?;
```

#### 2D Sketch Faces (for extrude / revolve)

| Method | Description |
|--------|-------------|
| `Shape::make_rect(w, h) -> Result<Shape>` | Rectangular face in the XY plane |
| `Shape::make_circle_face(r) -> Result<Shape>` | Circular face in the XY plane |
| `Shape::make_polygon(pts: &[f64]) -> Result<Shape>` | Closed polygon face in the XY plane. `pts` is a flat `[x0, y0, x1, y1, …]` slice; at least 3 points required. |
| `Shape::make_ellipse_face(rx, ry) -> Result<Shape>` | Elliptic face in the XY plane. OCCT requires major ≥ minor; arguments are swapped automatically if needed. |
| `Shape::make_arc(r, start_deg, end_deg) -> Result<Shape>` | Circular arc `Wire` in the XY plane, counterclockwise from `start_deg` to `end_deg`. Suitable as a sweep path. |
| `Shape::make_spline_2d(pts: &[f64]) -> Result<Shape>` | Closed profile in the XZ plane. `pts` is a flat `[r0, z0, r1, z1, …]` slice. Interpolates a BSpline, closes with a straight edge if the endpoints differ, returns a `Face`. Designed for `.revolve()`. |
| `Shape::make_spline_2d_tan(pts, t0x, t0z, t1x, t1z) -> Result<Shape>` | Same as `make_spline_2d` but with explicit start/end tangent vectors in the XZ plane. Suppresses natural-boundary oscillation on short splines. |

#### 3D Wire Paths (for sweep)

| Method | Description |
|--------|-------------|
| `Shape::make_spline_3d(pts: &[f64]) -> Result<Shape>` | 3D BSpline `Wire`. `pts` is a flat `[x0, y0, z0, x1, y1, z1, …]` slice. Use as the `path` argument to `.sweep()`. |
| `Shape::make_spline_3d_tan(pts, t0x, t0y, t0z, t1x, t1y, t1z) -> Result<Shape>` | Same as `make_spline_3d` with explicit start/end tangent vectors. Suppresses endpoint oscillation. |

#### Import

| Method | Description |
|--------|-------------|
| `Shape::import_step(path: &str) -> Result<Shape>` | Import a STEP file. Returns the first transferred shape from the STEP reader. |
| `Shape::import_stl(path: &str) -> Result<Shape>` | Import an STL file as a triangulated shell. |

#### Loft

| Method | Description |
|--------|-------------|
| `Shape::loft(profiles: &[&Shape], ruled: bool) -> Result<Shape>` | Loft through a sequence of profile faces/wires. `ruled=false` gives smooth (BSpline) blending; `ruled=true` gives straight (ruled) surface between each pair. Uses `BRepOffsetAPI_ThruSections`. |

#### Bézier Patch & Sewing

| Method | Description |
|--------|-------------|
| `Shape::make_bezier_patch(pts: &[f64]) -> Result<Shape>` | Bicubic Bézier face from 16 control points. `pts` is a flat `[x0,y0,z0, x1,y1,z1, …]` slice (48 values, 4×4 row-major). Uses `Geom_BezierSurface` + `BRepBuilderAPI_MakeFace`. |
| `Shape::sew(faces: &[&Shape], tolerance: f64) -> Result<Shape>` | Sew multiple faces into a closed shell/solid via `BRepBuilderAPI_Sewing` + `BRepBuilderAPI_MakeSolid`. Primary use case: assemble Utah Teapot Bézier patches. |

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

Both operations apply to **all edges** by default, or only to edges matching a
selector string.

| Method | Description |
|--------|-------------|
| `.fillet(radius) -> Result<Shape>` | Round every edge to the given radius |
| `.fillet_sel(radius, selector: &str) -> Result<Shape>` | Round only edges matching the selector (`"all"`, `"vertical"`, `"horizontal"`) |
| `.chamfer(dist) -> Result<Shape>` | Bevel every edge by the given distance |
| `.chamfer_sel(dist, selector: &str) -> Result<Shape>` | Bevel only edges matching the selector |

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
| `.scale_xyz(sx, sy, sz) -> Result<Shape>` | Non-uniform scale — independent factor per axis; uses `gp_GTrsf` |
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

### Sub-shape Selectors

| Method | Description |
|--------|-------------|
| `.faces(selector: &str) -> Result<Vec<Shape>>` | All faces matching the selector. Named selectors: `"all"`, `"top"` (normal·Z > 0.5), `"bottom"` (normal·Z < −0.5), `"side"` (all others). Direction-based selectors: `">Z"`, `"<Z"`, `">X"`, `"<X"`, `">Y"`, `"<Y"` — selects faces whose outward normal has a component > 0.5 (or < −0.5) along the given axis. Face orientation is accounted for in both forms. |
| `.edges(selector: &str) -> Result<Vec<Shape>>` | All unique edges matching the selector (deduplicated via `TopTools_IndexedMapOfShape`). Selectors: `"all"`, `"vertical"` (tangent·Z > 0.5), `"horizontal"` (all others). Degenerate edges are excluded. |
| `.vertices(selector: &str) -> Result<Vec<Shape>>` | All unique vertices. Only `"all"` is supported (deduplicated via `TopTools_IndexedMapOfShape`). |

```rust
let top_faces = part.faces("top")?;
let vert_edges = part.edges("vertical")?;
```

---

### Patterns

Both functions return a `TopoDS_Compound` containing `n` copies. Copy `i=0`
is the original (un-translated / un-rotated) position. The compound can be
used directly in boolean operations or exported as-is.

| Method | Description |
|--------|-------------|
| `.linear_pattern(n, dx, dy, dz) -> Result<Shape>` | Copy `i` is translated by `i * (dx, dy, dz)`. `n` must be ≥ 1. |
| `.polar_pattern(n, angle_deg) -> Result<Shape>` | Copy `i` is rotated around the Z axis by `i * (angle_deg / n)` degrees. Use `angle_deg = 360` for evenly-spaced full-circle copies. `n` must be ≥ 1. |

```rust
// 5 bolts spaced 20 mm apart along X
let bolt = Shape::make_cylinder(2.0, 10.0)?;
let row  = bolt.linear_pattern(5, 20.0, 0.0, 0.0)?;

// 6 holes equally spaced around a 30 mm bolt circle
let hole    = Shape::make_cylinder(3.0, 15.0)?.translate(30.0, 0.0, 0.0)?;
let pattern = hole.polar_pattern(6, 360.0)?;
```

---

### Color and Material

| Method | Description |
|--------|-------------|
| `.set_color(r, g, b) -> Result<Shape>` | Returns a copy of the shape with an sRGB color tag attached (`r`, `g`, `b` each in `[0.0, 1.0]`). The color is written to the XDE document during GLB/glTF/OBJ export and is visible in the live preview. The original shape is unchanged. |

```rust
let colored = part.set_color(0.8, 0.5, 0.2)?;   // warm orange
```

---

### Assembly Mating

| Method | Description |
|--------|-------------|
| `.mate(from_face: &Shape, to_face: &Shape, offset: f64) -> Result<Shape>` | Return a copy of `self` rigidly repositioned so that `from_face`'s outward normal aligns antiparallel with `to_face`'s outward normal, and `from_face`'s centroid coincides with `to_face`'s centroid. `offset > 0` leaves a gap; `offset < 0` gives intentional overlap. Both faces must be planar. |

```rust
let base   = Shape::make_box(100.0, 80.0, 10.0)?;
let post   = Shape::make_box(20.0, 20.0, 50.0)?;
let bottom = post.faces("bottom")?;
let top    = base.faces("top")?;
let placed = post.mate(&bottom[0], &top[0], 0.0)?;
```

---

### Feature Removal

| Method | Description |
|--------|-------------|
| `.simplify(min_feature_size: f64) -> Result<Shape>` | Remove small holes and fillets. Faces with surface area < `min_feature_size²` are passed to `BRepAlgoAPI_Defeaturing`. Returns the original shape unchanged if no faces qualify. |

```rust
let simple = part.simplify(1.0)?;   // remove features smaller than ~1 mm
```

---

### Export

| Method | Description |
|--------|-------------|
| `.export_step(path: &str) -> Result<()>` | STEP AP203 boundary-representation file |
| `.export_stl(path: &str) -> Result<()>` | ASCII STL triangulated mesh |
| `.export_gltf(path: &str, linear_deflection: f64) -> Result<()>` | glTF 2.0 (text JSON + companion `.bin`). `linear_deflection` controls tessellation quality (e.g. `0.1` for 0.1 mm). |
| `.export_glb(path: &str, linear_deflection: f64) -> Result<()>` | Binary glTF (GLB). Single self-contained file; used by the live preview server. |
| `.export_obj(path: &str, linear_deflection: f64) -> Result<()>` | Wavefront OBJ text format via `RWObj_CafWriter` (`TKDEOBJ`). Writes a companion `.mtl` material file alongside the `.obj`. |

```rust
part.export_step("/tmp/part.step")?;
part.export_stl("/tmp/part.stl")?;
part.export_gltf("/tmp/part.gltf", 0.1)?;
part.export_glb("/tmp/part.glb", 0.1)?;
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
fn make_cone(r1: f64, r2: f64, height: f64)          -> Result<UniquePtr<OcctShape>>;
fn make_torus(r1: f64, r2: f64)                       -> Result<UniquePtr<OcctShape>>;
fn make_wedge(dx: f64, dy: f64, dz: f64, ltx: f64)  -> Result<UniquePtr<OcctShape>>;

// 2D sketch faces
fn make_rect(w: f64, h: f64)                          -> Result<UniquePtr<OcctShape>>;
fn make_circle_face(r: f64)                            -> Result<UniquePtr<OcctShape>>;
fn make_polygon(pts: &[f64])                           -> Result<UniquePtr<OcctShape>>;
fn make_ellipse_face(rx: f64, ry: f64)                -> Result<UniquePtr<OcctShape>>;
fn make_arc(r: f64, start_deg: f64, end_deg: f64)    -> Result<UniquePtr<OcctShape>>;
fn make_spline_2d(pts: &[f64])                         -> Result<UniquePtr<OcctShape>>;
fn make_spline_2d_tan(pts: &[f64], t0x: f64, t0z: f64,
                      t1x: f64, t1z: f64)              -> Result<UniquePtr<OcctShape>>;
fn make_spline_3d(pts: &[f64])                         -> Result<UniquePtr<OcctShape>>;
fn make_spline_3d_tan(pts: &[f64], t0x: f64, t0y: f64, t0z: f64,
                      t1x: f64, t1y: f64, t1z: f64)   -> Result<UniquePtr<OcctShape>>;

// Import
fn import_step(path: &str)                             -> Result<UniquePtr<OcctShape>>;
fn import_stl (path: &str)                             -> Result<UniquePtr<OcctShape>>;

// Loft
fn thru_sections_new(solid: bool, ruled: bool)         -> UniquePtr<ThruSectionsBuilder>;
fn thru_sections_add(builder: Pin<&mut ThruSectionsBuilder>, profile: &OcctShape);
fn thru_sections_build(builder: Pin<&mut ThruSectionsBuilder>)
                                                       -> Result<UniquePtr<OcctShape>>;

// Bézier patch & sewing
fn make_bezier_patch(pts: &[f64])                      -> Result<UniquePtr<OcctShape>>;
fn sewing_new(tolerance: f64)                          -> Result<UniquePtr<SewingBuilder>>;
fn sewing_add(builder: Pin<&mut SewingBuilder>,
              shape: &OcctShape)                       -> Result<()>;
fn sewing_build(builder: Pin<&mut SewingBuilder>)      -> Result<UniquePtr<OcctShape>>;

// Boolean ops
fn shape_fuse  (a: &OcctShape, b: &OcctShape)         -> Result<UniquePtr<OcctShape>>;
fn shape_cut   (a: &OcctShape, b: &OcctShape)         -> Result<UniquePtr<OcctShape>>;
fn shape_common(a: &OcctShape, b: &OcctShape)         -> Result<UniquePtr<OcctShape>>;

// Fillets / chamfers
fn shape_fillet    (shape: &OcctShape, radius: f64)              -> Result<UniquePtr<OcctShape>>;
fn shape_chamfer   (shape: &OcctShape, dist: f64)                -> Result<UniquePtr<OcctShape>>;
fn shape_fillet_sel(shape: &OcctShape, radius: f64, sel: &str)   -> Result<UniquePtr<OcctShape>>;
fn shape_chamfer_sel(shape: &OcctShape, dist: f64, sel: &str)    -> Result<UniquePtr<OcctShape>>;

// Transforms
fn shape_translate(shape: &OcctShape,
                   dx: f64, dy: f64, dz: f64)         -> Result<UniquePtr<OcctShape>>;
fn shape_rotate(shape: &OcctShape,
                axis_x: f64, axis_y: f64, axis_z: f64,
                angle_deg: f64)                        -> Result<UniquePtr<OcctShape>>;
fn shape_scale    (shape: &OcctShape, factor: f64)    -> Result<UniquePtr<OcctShape>>;
fn shape_scale_xyz(shape: &OcctShape,
                   sx: f64, sy: f64, sz: f64)         -> Result<UniquePtr<OcctShape>>;
fn shape_mirror(shape: &OcctShape, plane: &str)       -> Result<UniquePtr<OcctShape>>;

// Sketch operations
fn shape_extrude(shape: &OcctShape, height: f64)      -> Result<UniquePtr<OcctShape>>;
fn shape_extrude_ex(shape: &OcctShape, height: f64,
                    twist_deg: f64, scale: f64)        -> Result<UniquePtr<OcctShape>>;
fn shape_revolve(shape: &OcctShape, angle_deg: f64)   -> Result<UniquePtr<OcctShape>>;
fn shape_sweep(profile: &OcctShape, path: &OcctShape) -> Result<UniquePtr<OcctShape>>;

// 3-D operations
fn shape_shell   (shape: &OcctShape, thickness: f64)  -> Result<UniquePtr<OcctShape>>;
fn shape_offset  (shape: &OcctShape, distance: f64)   -> Result<UniquePtr<OcctShape>>;
fn shape_simplify(shape: &OcctShape,
                  min_feature_size: f64)               -> Result<UniquePtr<OcctShape>>;

// Color and mating (Phase 5)
fn shape_set_color(shape: &OcctShape,
                   r: f64, g: f64, b: f64)            -> Result<UniquePtr<OcctShape>>;
fn shape_mate(shape: &OcctShape, from_face: &OcctShape,
              to_face: &OcctShape, offset: f64)        -> Result<UniquePtr<OcctShape>>;

// Query
fn shape_bounding_box(shape: &OcctShape, out: &mut [f64]);
fn shape_volume      (shape: &OcctShape)               -> f64;
fn shape_surface_area(shape: &OcctShape)               -> f64;

// Sub-shape selectors
fn shape_faces_count(shape: &OcctShape, selector: &str)              -> Result<i32>;
fn shape_faces_get(shape: &OcctShape, selector: &str, idx: i32)      -> Result<UniquePtr<OcctShape>>;
fn shape_edges_count(shape: &OcctShape, selector: &str)              -> Result<i32>;
fn shape_edges_get(shape: &OcctShape, selector: &str, idx: i32)      -> Result<UniquePtr<OcctShape>>;
fn shape_vertices_count(shape: &OcctShape, selector: &str)           -> Result<i32>;
fn shape_vertices_get(shape: &OcctShape, selector: &str, idx: i32)   -> Result<UniquePtr<OcctShape>>;

// Patterns
fn shape_linear_pattern(shape: &OcctShape, n: i32,
                         dx: f64, dy: f64, dz: f64)   -> Result<UniquePtr<OcctShape>>;
fn shape_polar_pattern (shape: &OcctShape, n: i32,
                         angle_deg: f64)               -> Result<UniquePtr<OcctShape>>;

// Export
fn export_step(shape: &OcctShape, path: &str)                         -> Result<()>;
fn export_stl (shape: &OcctShape, path: &str)                         -> Result<()>;
fn export_gltf(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
fn export_glb (shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
fn export_obj (shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
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
| `cone(r1, r2, h)` | Cone/frustum along Z axis |
| `torus(r1, r2)` | Torus in XY plane |
| `wedge(dx, dy, dz, ltx)` | Wedge primitive |
| `rect(w, h)` | Rectangular face in XY plane |
| `circle(r)` | Circular face in XY plane |
| `polygon([[x,y], ...])` | Closed polygon face in XY plane (≥ 3 points) |
| `ellipse(rx, ry)` | Elliptic face in XY plane |
| `arc(r, start_deg, end_deg)` | Circular arc wire in XY plane (counterclockwise) |
| `spline_2d([[r,z], ...])` | Closed XZ-plane profile (for `revolve`) |
| `spline_2d([[r,z], ...], tangents: [[t0r,t0z],[t1r,t1z]])` | Same with explicit start/end tangent vectors; suppresses endpoint oscillation on short splines |
| `spline_3d([[x,y,z], ...])` | 3D wire path (for `sweep` and `sweep_sections`) |
| `spline_3d([[x,y,z], ...], tangents: [[t0x,t0y,t0z],[t1x,t1y,t1z]])` | Same with explicit tangent vectors |
| `loft([profile1, profile2, ...])` | Loft through a sequence of circle/sketch profiles; `ruled: false` (default) gives smooth blending |
| `sweep_sections(path, [profile1, profile2, ...])` | Variable-section sweep: each origin-centred profile is automatically placed at the corresponding spine point and swept along `path` (a `spline_3d` Wire). Uses `BRepOffsetAPI_MakePipeShell`; falls back to `ThruSections` loft for highly-curved paths. Supports `circle`, `rect`, `polygon`, `ellipse`, and `arc` profiles. |
| `bezier_patch([[x,y,z], ...])` | Build a single bicubic Bézier face from exactly 16 control points (4×4 row-major grid). Uses `Geom_BezierSurface` + `BRepBuilderAPI_MakeFace`. Returns a `Face` suitable for passing to `sew`. |
| `sew([face1, face2, ...], tolerance: 1e-4)` | Assemble multiple faces (typically Bézier patches) into a closed shell or solid via `BRepBuilderAPI_Sewing` + `BRepBuilderAPI_MakeSolid`. `tolerance` controls the maximum edge-gap that is considered coincident. |
| `import_step("file.step")` | Import a STEP file as a Shape |
| `import_stl("file.stl")` | Import an STL file as a triangulated Shape |
| `linear_pattern(shape, n, [dx, dy, dz])` | `n` copies of `shape` translated along vector; copy `i` at `i*[dx,dy,dz]`. Returns a Compound. |
| `polar_pattern(shape, n, angle_deg)` | `n` copies of `shape` rotated around Z; copy `i` at `i*(angle_deg/n)` degrees. Returns a Compound. |
| `param(:name, default: val)` | Declare a named parameter. Returns `val` unless a `--param name=x` CLI override was supplied; coerces string overrides to the default's type (Integer/Float/String). |
| `param(:name, default: val, range: lo..hi)` | Same with range validation; raises `ArgumentError` if the value is outside the range. |
| `solid { ... }` | Block returning its last expression |
| `assembly("name") { \|a\| ... }` | Named assembly. See `Assembly` below. |
| `preview(shape)` | Tessellate and push to live browser preview. No-op when not in `--preview` mode. |

### Shape instance methods

| Method | Description |
|--------|-------------|
| `.fuse(other)` | Union |
| `.cut(other)` | Subtraction |
| `.common(other)` | Intersection |
| `.translate(x, y, z)` | Move |
| `.rotate(ax, ay, az, deg)` | Rotate around axis by degrees |
| `.scale(factor)` | Uniform scale (all axes) |
| `.scale(sx, sy, sz)` | Non-uniform scale — independent factor per axis |
| `.mirror(:xy\|:xz\|:yz)` | Mirror about a coordinate plane |
| `.fillet(r)` | Round all edges |
| `.fillet(r, :selector)` | Round only edges matching selector (`:all` / `:vertical` / `:horizontal`) |
| `.chamfer(d)` | Bevel all edges |
| `.chamfer(d, :selector)` | Bevel only edges matching selector |
| `.extrude(h)` | Extrude face/wire along Z |
| `.extrude(h, twist_deg: 0, scale: 1.0)` | Extrude with optional twist (degrees) and end-scale; uses `ThruSections` when twist/scale are non-default |
| `.revolve(deg=360)` | Revolve around Z axis |
| `.sweep(path)` | Sweep profile along a `spline_3d` wire |
| `.shell(thickness)` | Hollow out a solid by removing the topmost face and offsetting walls inward |
| `.offset(distance)` | Inflate (positive) or deflate (negative) a solid uniformly |
| `.simplify(min_feature_size)` | Remove small holes/fillets; faces with area < `min_feature_size²` are defeatured. Returns the original shape if none qualify. |
| `.color(r, g, b)` | Attach an sRGB color (`r`, `g`, `b` each in `[0.0, 1.0]`); written into GLB/glTF/OBJ export. Returns a new Shape; original unchanged. |
| `.mate(from_face, to_face, offset=0.0)` | Reposition `self` so `from_face` aligns flush against `to_face` (antiparallel normals, coincident centroids). `offset > 0` = gap; `offset < 0` = overlap. Both faces must be planar. |
| `.faces(:top\|:bottom\|:side\|:all)` | Array of matching face sub-shapes (symbol selectors) |
| `.faces(">Z"\|"<X"\|...)` | Direction-based face selector — string form (CadQuery style) |
| `.edges(:vertical\|:horizontal\|:all)` | Array of matching edge sub-shapes (deduplicated) |
| `.vertices(:all)` | Array of all unique vertex sub-shapes |
| `.bounding_box` | Returns `{x:, y:, z:, dx:, dy:, dz:}` — minimum corner and extents |
| `.volume` | Volume of the solid (float) |
| `.surface_area` | Total surface area (float) |
| `.export("out.step")` | Write file; format determined by extension: `.step`/`.stp` → STEP, `.stl` → STL, `.glb` → GLB, `.gltf` → glTF, `.obj` → OBJ |

---

### Assembly

`Assembly` groups named shapes and can position them via face mating.

```ruby
base = box(100, 80, 10)
post = box(20, 20, 50)
asm = assembly("bracket") do |a|
  a.place base
  a.mate post, from: post.faces(:bottom).first,
               to:   base.faces(:top).first
  a.mate post, from: post.faces(:bottom).first,
               to:   base.faces(:top).first, offset: 2.0  # 2 mm gap
end
asm.export("bracket.glb")
```

| Method | Description |
|--------|-------------|
| `a.place(shape)` | Add `shape` at its current position. Returns `shape`. |
| `a.mate(shape, from:, to:, offset: 0.0)` | Reposition `shape` so `from:` face aligns against `to:` face, then add to the assembly. Returns the repositioned shape. |
| `asm.export(path)` | Fuse all shapes and export to file. |

---

## CLI

```sh
rrcad                                          # start REPL (readline, history, tab-completion)
rrcad --repl                                   # same as above
rrcad <script.rb>                              # execute a .rb script and exit
rrcad --param key=value <script.rb>            # override a param() declaration
rrcad --param w=50 --param h=20 <script.rb>   # multiple overrides
rrcad --preview <script.rb>                    # live browser preview; re-evals on every save
rrcad --preview --param w=50 <script.rb>       # preview with parameter override
rrcad --design-table table.csv <script.rb>     # batch export: one file per CSV row
```

**`--param key=value`** — Override a `param()` declaration in the script. The value is a string; it is coerced to the declared default's type (Integer, Float, or String) when the parameter is read. Multiple `--param` flags are allowed.

**`--design-table table.csv`** — Read a CSV or TSV table where the first row is column headers (parameter names) and each data row is one variant. An optional `name` column provides the output filename stem; otherwise rows are numbered. Each data row evaluates the script once with the row's values as `param()` overrides and exports a STEP file named `<name>.step` (or `variant_N.step`). Comments starting with `#` are ignored.

```csv
name,width,height,depth
small,30,20,10
medium,60,40,20
large,90,60,30
```

`--preview` starts an `axum` HTTP server on `http://localhost:3000`, opens the browser, watches the script with `notify`, and calls `preview(shape)` automatically on each re-eval. Ctrl-C to quit.

**Preview server routes:**

| Route | Description |
|-------|-------------|
| `GET /` | Three.js viewer HTML |
| `GET /model.glb` | Current tessellated shape (binary glTF) |
| `GET /logo.png` | rrcad logo (served from embedded bytes) |
| `GET /ws` | WebSocket; server pushes `"reload"` when the model updates |

**REPL commands:**

| Input | Effect |
|-------|--------|
| Any Ruby expression | Evaluates and prints `=> <result>` |
| `help` | Prints DSL quick-reference |
| `exit` / `quit` | Exits |
| Ctrl-D / Ctrl-C | Exits |

Tab-completion is available for top-level DSL identifiers and, after a `.`,
for Shape method names.
