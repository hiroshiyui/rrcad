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
| `Shape::make_profile_2d(pts: &[f64], counts: &[i32], kinds: &[i32]) -> Result<Shape>` | Closed XY profile face from a chain of segments: segment *i* owns `counts[i]` consecutive points and repeats the corner it shares with its neighbour; `kinds[i]` is 0 for a straight run or 1 for a BSpline interpolated through them (`GeomAPI_Interpolate`). Backs constraint sketches that mix `line` and `spline` segments. |
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

### Named Topology

You can attach persistent names to face/edge selectors or to datum/reference
shapes, then resolve them later.

For drawing annotations, `Shape#gdt(standard: :asme) { |g| ... }` stores a
structured GD&T spec on the shape itself. The SVG/DXF exporters use that
stored spec first, then fall back to the older keyword arguments when no
structured spec is present.

| Method | Description |
|--------|-------------|
| `.name_face(name, selector)` | Store a named face selector such as `"top"` or `">Z"` |
| `.name_edge(name, selector)` | Store a named edge selector such as `"vertical"` |
| `.datum(name, shape)` | Attach a reference shape such as a datum plane |
| `.ref(name)` | Resolve a named face, edge, or datum reference |

```rust
part.name_face("mounting_face", "top")?;
part.name_edge("boss_edges", "vertical")?;
part.datum("fixture_plane", Shape::make_rect(10.0, 10.0)?)?;
let top = part.faces("mounting_face")?;
let datum = part.ref("fixture_plane")?;
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

### Validation & Introspection (Phase 7 Tier 2)

| Method | Description |
|--------|-------------|
| `.shape_type_name() -> Result<String>` | Returns the topological type as a string: `"solid"`, `"shell"`, `"face"`, `"wire"`, `"edge"`, `"vertex"`, `"compound"`, `"compsolid"`, or `"other"`. |
| `.history() -> Vec<String>` | Returns the provenance chain of modeling operations that produced the shape, oldest to newest. |
| `.feature_graph() -> String` | Returns the feature tree as tab-separated lines with stable node IDs, parent IDs, labels, and history entries. |
| `.rebuild() -> Result<Shape>` | Replays the stored feature tree and returns a rebuilt shape. |
| `.centroid() -> Result<[f64; 3]>` | Centre of mass as `[x, y, z]`. Dispatches to `BRepGProp::VolumeProperties` (solids/compounds), `SurfaceProperties` (shells/faces), or `LinearProperties` (wires/edges). |
| `.is_closed() -> Result<bool>` | `true` if every edge has at least 2 adjacent faces (no open boundary). Uses `TopTools_IndexedDataMapOfShapeListOfShape`. |
| `.is_manifold() -> Result<bool>` | `true` if every edge has exactly 2 adjacent faces (manifold mesh). |
| `.validate() -> Result<String>` | Runs `BRepCheck_Analyzer`. Returns `"ok"` if valid, or a newline-separated string of error names. |

```rust
assert_eq!(Shape::make_box(10.0, 20.0, 30.0)?.shape_type_name()?, "solid");
let c = Shape::make_box(10.0, 20.0, 30.0)?.centroid()?;
assert!((c[0] - 5.0).abs() < 1e-6);
assert!(Shape::make_box(10.0, 20.0, 30.0)?.is_manifold()?);
assert_eq!(Shape::make_box(10.0, 20.0, 30.0)?.validate()?, "ok");
assert_eq!(
    Shape::make_box(10.0, 20.0, 30.0)?.history(),
    vec!["box(dx=10, dy=20, dz=30)"]
);
```

---

### Surface Modeling (Phase 7 Tier 3)

| Method | Description |
|--------|-------------|
| `Shape::ruled_surface(wire_a: &Shape, wire_b: &Shape) -> Result<Shape>` | Ruled surface (Shell) connecting `wire_a` to `wire_b` via `BRepFill::Shell`. Both must be Wire shapes. |
| `Shape::fill_surface(boundary_wire: &Shape) -> Result<Shape>` | Fills the enclosed region of a closed boundary Wire with a smooth NURBS Face via `BRepFill_Filling`. |
| `.slice(plane: &str, offset: f64) -> Result<Shape>` | Cross-section by an axis-aligned plane (`"xy"`, `"xz"`, `"yz"`) at `offset`. Returns a compound of section edges/wires via `BRepAlgoAPI_Section`. |

---

### Part Design (Phase 8 Tier 1)

| Method | Description |
|--------|-------------|
| `.pad(face_ref: &Shape, sketch: &Shape, height: f64) -> Result<Shape>` | Extrude `sketch` along the outward normal of `face_ref` by `height` and fuse with `self`. Uses `BRepPrimAPI_MakePrism` + `BRepAlgoAPI_Fuse`. The sketch is repositioned from the XY plane onto `face_ref` via `gp_Trsf`. |
| `.pocket(face_ref: &Shape, sketch: &Shape, depth: f64) -> Result<Shape>` | Extrude `sketch` inward along the outward normal of `face_ref` by `depth` and subtract from `self`. Uses `BRepPrimAPI_MakePrism` + `BRepAlgoAPI_Cut`. |
| `.fillet_wire(radius: f64) -> Result<Shape>` | Round all corners of a 2D Wire or Face. Applies `BRepFilletAPI_MakeFillet2d` at each vertex; non-corner vertices are silently skipped. Returns a Face. Raises an error if called on a Solid/Shell. |
| `Shape::make_datum_plane(ox, oy, oz, nx, ny, nz, xx, xy, xz) -> Result<Shape>` | Reference plane Face from `gp_Ax3(origin, normal, x_dir)` + `BRepBuilderAPI_MakeFace(gp_Pln)`. The resulting Face can be passed directly to `.pad` / `.pocket` as `face_ref`. |

```rust
let body = Shape::make_box(100.0, 60.0, 10.0)?;
let top  = body.faces("top")?;
let slot = Shape::make_rect(40.0, 20.0)?.fillet_wire(4.0)?;
let part = body.pocket(&top[0], &slot, 8.0)?;

let plane = Shape::make_datum_plane(0.0, 0.0, 10.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0)?;
let boss  = Shape::make_circle_face(10.0)?;
let part2 = body.pad(&plane, &boss, 6.0)?;
```

---

### Manufacturing features (Phase 8 Tier 2)

| Method | Description |
|--------|-------------|
| `.extrude_draft(height: f64, draft_deg: f64) -> Result<Shape>` | Straight extrude then taper all lateral (non-Z-normal) planar faces via `BRepOffsetAPI_DraftAngle`. Neutral plane is Z=0 (base edges stay fixed). Positive `draft_deg` → wider at base, narrower at top. `draft_deg == 0` falls through to a straight extrude. |
| `Shape::make_helix(radius: f64, pitch: f64, height: f64) -> Result<Shape>` | Helical Wire path built from a `GeomAPI_Interpolate` BSpline (16 samples/turn, max 512 points). Use as the `path` argument to `.sweep()` to create thread tools. |

```rust
// Draft-angle extrude (e.g. for injection-moulded ribs)
let rib = Shape::make_rect(5.0, 40.0)?.extrude_draft(15.0, 2.0)?;

// Helical sweep for a thread tool
let path    = Shape::make_helix(5.0, 1.5, 12.0)?;
let profile = Shape::make_polygon(&[0.0, 0.0,  -0.6, 0.75,  0.0, 1.5])?;
let thread  = profile.sweep(&path)?;
```

### Inspection & clearance (Phase 8 Tier 3)

| Method | Description |
|--------|-------------|
| `.distance_to(other: &Shape) -> Result<f64>` | Minimum distance between two shapes via `BRepExtrema_DistShapeShape`. Returns `0.0` for overlapping or touching shapes. |
| `.inertia() -> Result<[f64; 6]>` | Inertia tensor `[Ixx, Iyy, Izz, Ixy, Ixz, Iyz]` about the shape's own centre of mass, via `BRepGProp::VolumeProperties` → `GProp_GProps::MatrixOfInertia`. OCCT integrates at density 1, so these are *volume* moments (mm⁵); scale by mass / volume for g·mm². Off-diagonals are true tensor entries (−∫xy dV), not products of inertia. |
| `.min_thickness() -> Result<f64>` | Minimum wall thickness of a solid or shell via ray-casting (`IntCurvesFace_ShapeIntersector`): for each face, shoots a ray inward from the UV-centre, returns the shortest non-trivial intersection distance across all faces. Raises an error for non-solid/non-shell shapes. |
| `.face_normal() -> Result<[f64; 3]>` | Outward unit normal of a face shape, sampled at the face's parameter-space midpoint via `BRepLProp_SLProps` and flipped when `TopAbs_REVERSED` so it points out of the parent solid. Errors if the shape is not a face or the normal is undefined. |
| `.cylinder_axis() -> Result<[f64; 7]>` | For a cylindrical face, returns `[ox, oy, oz, ax, ay, az, radius]` — the axis origin, unit direction, and radius via `BRepAdaptor_Surface::Cylinder()`. Errors if the surface is not `GeomAbs_Cylinder`. |

```rust
// Clearance check
let a = Shape::make_box(10.0, 10.0, 10.0)?;
let b = Shape::make_box(5.0, 5.0, 5.0)?.translate(20.0, 0.0, 0.0)?;
let gap: f64 = a.distance_to(&b)?; // ≈ 10.0

// Inertia tensor
let s = Shape::make_sphere(5.0)?;
let [ixx, iyy, izz, ixy, ixz, iyz] = s.inertia()?;

// Wall thickness of a hollow box (shell thickness = 2 mm)
let shell = Shape::make_box(20.0, 20.0, 20.0)?.shell(2.0)?;
let t: f64 = shell.min_thickness()?; // ≈ 2.0
```

```rust
// Ruled surface between two wires
let w1 = Shape::make_spline_3d(&[0.,0.,0., 1.,0.,0., 1.,1.,0.])?;
let w2 = Shape::make_spline_3d(&[0.,0.,5., 1.,0.,5., 1.,1.,5.])?;
let shell = Shape::ruled_surface(&w1, &w2)?;

// Cross-section of a box at z=5
let section = Shape::make_box(10.0, 10.0, 10.0)?.slice("xy", 5.0)?;
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
| `.export_svg(path: &str, view: &str, scale: f64, hidden: bool, center_marks: bool, dimensions: bool, title_block: bool, callouts: bool, datum: &str, feature_control: &str, tolerance_plus: f64, tolerance_minus: f64) -> Result<()>` | SVG 2-D drawing. `view` is `"top"` (default), `"front"`, `"side"`, or `"sheet"` for a 3-view sheet; `scale` defaults to `1.0` and must be positive; `hidden` includes dashed hidden-line geometry; `center_marks` adds cylinder centre marks; `dimensions` adds overall width/height labels, and on `"sheet"` those labels are axis-aware (`X/Y/Z`). `title_block` adds a simple metadata block; `callouts` adds diameter callouts for cylindrical faces; `datum` accepts either a simple label or a hash with `label:` and `face:` / `selector:` to anchor the datum to a real face; `feature_control` accepts a string or a hash with `text:` / `frame:` / `value:` plus `datums:` for attached references; `Shape#gdt` overrides these keyword arguments when present. `tolerance_plus` and `tolerance_minus` control the label formatting, using `±` when equal and `+.../-...` when asymmetric. Uses `HLRBRep_PolyAlgo` + `HLRBRep_PolyHLRToShape`. Delegates to `.export_svg_with_anchor(...)`, which additionally takes datum / feature-control anchor points and `section_plane: &str` (`""`, `"xy"`, `"xz"`, `"yz"`) plus `section_offset: f64` for section views. |
| `.export_dxf(path: &str, view: &str, scale: f64, hidden: bool, center_marks: bool, dimensions: bool, title_block: bool, callouts: bool, datum: &str, feature_control: &str, tolerance_plus: f64, tolerance_minus: f64) -> Result<()>` | DXF R12 ASCII drawing. Same `view`, `scale`, `hidden`, `center_marks`, `dimensions`, `title_block`, `callouts`, datum / feature-control frame, and tolerance options as `export_svg`; `view` also accepts `"sheet"` for a 3-view layout. Hidden entities are written on a `HIDDEN` layer, center marks on a `CENTER` layer, callouts on a `CALLOUT` layer, the datum frame on `GDT`, dimensions on a `DIMENSION` layer, and title block entities on `TITLEBLOCK`. In sheet mode, the dimension labels are axis-aware (`X/Y/Z`) and can include `±` or `+.../-...` notation. `Shape#gdt` overrides these keyword arguments when present. Delegates to `.export_dxf_with_anchor(...)`, which adds the anchor points and the same `section_plane` / `section_offset` parameters as the SVG side. |

### Phase 8 Tier 5 — Advanced Composition

| Method | Description |
|--------|-------------|
| `Shape::fragment_all(shapes: &[&Shape]) -> Result<Shape, String>` | Boolean fragment: splits all shapes at mutual intersection boundaries. Returns a Compound of all non-overlapping pieces. Uses `BRepAlgoAPI_BuilderAlgo` via the `FragmentBuilder` builder pattern. |
| `.convex_hull() -> Result<Shape, String>` | 3-D convex hull of the shape's tessellated mesh vertices. Runs an incremental QuickHull algorithm; sews hull triangles into a BRep solid via `BRepBuilderAPI_Sewing` + `BRepBuilderAPI_MakeSolid`. |
| `.path_pattern(path: &Shape, n: i32) -> Result<Shape, String>` | Distribute `n` arc-length-evenly-spaced copies of `self` along `path` (a Wire or Edge). Each copy is oriented so its local Z-axis aligns with the path tangent. Uses `GCPnts_UniformAbscissa` on `BRepAdaptor_CompCurve`. |
| `.sweep_guide(path: &Shape, guide: &Shape) -> Result<Shape, String>` | Guided sweep: sweeps `self` (a profile Wire or Face) along `path` while keeping the profile orientation locked to the auxiliary `guide` wire. Uses `BRepOffsetAPI_MakePipeShell::SetMode(guide_wire, true, BRepFill_Contact)`. |

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

// Validation & introspection (Phase 7 Tier 2)
fn shape_type_str   (shape: &OcctShape)                -> Result<String>;
fn shape_centroid   (shape: &OcctShape, out: &mut [f64]) -> Result<()>;
fn shape_face_normal(face: &OcctShape, out: &mut [f64]) -> Result<()>;
fn shape_cylinder_axis(face: &OcctShape, out: &mut [f64]) -> Result<()>;
fn shape_is_closed  (shape: &OcctShape)                -> Result<bool>;
fn shape_is_manifold(shape: &OcctShape)                -> Result<bool>;
fn shape_validate_str(shape: &OcctShape)               -> Result<String>;

// Surface modeling (Phase 7 Tier 3)
fn shape_ruled_surface(wire_a: &OcctShape,
                       wire_b: &OcctShape)             -> Result<UniquePtr<OcctShape>>;
fn shape_fill_surface(boundary_wire: &OcctShape)       -> Result<UniquePtr<OcctShape>>;
fn shape_slice(shape: &OcctShape, plane: &str,
               offset: f64)                            -> Result<UniquePtr<OcctShape>>;

// Part Design (Phase 8 Tier 1)
fn shape_pad(body: &OcctShape, face_ref: &OcctShape,
             sketch: &OcctShape, height: f64)          -> Result<UniquePtr<OcctShape>>;
fn shape_pocket(body: &OcctShape, face_ref: &OcctShape,
                sketch: &OcctShape, depth: f64)        -> Result<UniquePtr<OcctShape>>;
fn shape_fillet_wire(profile: &OcctShape,
                     radius: f64)                      -> Result<UniquePtr<OcctShape>>;
fn make_datum_plane(ox: f64, oy: f64, oz: f64,
                    nx: f64, ny: f64, nz: f64,
                    xx: f64, xy: f64, xz: f64)         -> Result<UniquePtr<OcctShape>>;

// Manufacturing (Phase 8 Tier 2)
fn shape_extrude_draft(profile: &OcctShape,
                       height: f64, draft_deg: f64)    -> Result<UniquePtr<OcctShape>>;
fn make_helix(radius: f64, pitch: f64,
              height: f64)                             -> Result<UniquePtr<OcctShape>>;

// Inspection & clearance (Phase 8 Tier 3)
fn shape_distance_to(a: &OcctShape, b: &OcctShape)    -> Result<f64>;
fn shape_inertia(shape: &OcctShape,
                 out: &mut [f64])                      -> Result<()>;
fn shape_min_thickness(shape: &OcctShape)              -> Result<f64>;

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

// Phase 8 Tier 5 — Advanced composition
type FragmentBuilder;
fn fragment_new()                                                          -> Result<UniquePtr<FragmentBuilder>>;
fn fragment_add(builder: Pin<&mut FragmentBuilder>, shape: &OcctShape)    -> Result<()>;
fn fragment_build(builder: Pin<&mut FragmentBuilder>)                     -> Result<UniquePtr<OcctShape>>;
fn shape_convex_hull(shape: &OcctShape)                                   -> Result<UniquePtr<OcctShape>>;
fn shape_path_pattern(shape: &OcctShape, path: &OcctShape, n: i32)       -> Result<UniquePtr<OcctShape>>;
fn shape_sweep_guide(profile: &OcctShape, path: &OcctShape,
                     guide: &OcctShape)                                   -> Result<UniquePtr<OcctShape>>;
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
| `grid_pattern(shape, nx, ny, dx, dy)` | `nx × ny` copies of `shape` in a 2-D grid; copy `(i,j)` at `(i*dx, j*dy, 0)`. Implemented as two nested `linear_pattern` calls. Returns a Compound. |
| `fuse_all([shape1, shape2, ...])` | Fold-left union of two or more shapes. Requires at least 2 shapes. |
| `cut_all(base, [tool1, tool2, ...])` | Subtract each tool from `base` in sequence. Requires at least 1 tool. |
| `ruled_surface(wire_a, wire_b)` | Ruled surface (Shell) spanning from `wire_a` to `wire_b`. Both must be Wire shapes. Uses `BRepFill::Shell`. |
| `fill_surface(boundary_wire)` | Smooth NURBS surface filling the region enclosed by a closed Wire. Uses `BRepFill_Filling` with C0 boundary constraints. |
| `datum_plane(origin: [ox,oy,oz], normal: [nx,ny,nz], x_dir: [xx,xy,xz])` | Reference plane Face at the given origin/normal/x_dir. Can be used as the `face_sel` argument to `.pad` / `.pocket`. |
| `helix(radius:, pitch:, height:)` | Helical Wire path (16 samples/turn). Use as the `path` argument to `.sweep` for thread-profile sweeps. |
| `thread(solid, face_sym, pitch:, depth:)` | Cut a helical groove into `solid`. `face_sym` is reserved (pass `:side`); geometry is inferred from the bounding box. Returns the threaded solid. Pure Ruby DSL — composes `helix` + `polygon` + `.sweep` + `.cut`. |
| `cbore(d:, cbore_d:, cbore_h:, depth:)` | Counterbore 3-D hole tool. Subtract from a plate with `.cut`. All dimensions are diameters. Pure Ruby DSL. |
| `csink(d:, csink_d:, csink_angle:, depth:)` | Countersink 3-D hole tool. `csink_angle` is the cone half-angle in degrees (45° = 90° included angle for flat-head screws). Subtract from a plate with `.cut`. Pure Ruby DSL. |
| `clearance_hole(size, depth:)` | ISO close-clearance hole tool sized for `:m2`, `:m2_5`, `:m3`, `:m4`, `:m5`, or a numeric diameter. Pure Ruby DSL. |
| `tap_drill(size, depth:)` | Metric coarse tap-drill hole tool for `:m2`–`:m5` or a numeric diameter. Pure Ruby DSL. |
| `heat_set_insert(size, depth:)` | Pilot-hole tool sized for common heat-set inserts (`:m2`, `:m2_5`, `:m3`, or a numeric diameter). Pure Ruby DSL. |
| `socket_head_cbore(size, depth:, head_depth:)` | Counterbore tool sized for metric socket-head cap screws (`:m2`–`:m5`). Pure Ruby DSL. |
| `flat_head_csink(size, depth:, angle: 45)` | Countersink tool sized for metric flat-head screws (`:m2`–`:m5`). Pure Ruby DSL. |
| `bearing_bore(size, depth:, fit: :press)` | Outer-diameter bore for common deep-groove ball bearings (`:b608`, `:b623`, `:b624`, `:b625`, `:b626`, `:b688`, `:b695`, `:b6000`, `:b6001`, or numeric OD). `fit:` is `:press` (−0.01 mm interference) or `:slip` (+0.05 mm clearance). Pure Ruby DSL. |
| `shaft(diameter, length:, fit: :nominal)` | Solid mating shaft cylinder at the given nominal diameter with a fit adjustment (`:nominal`, `:press` +0.02, `:slip` −0.02, `:running` −0.05 mm). Pure Ruby DSL. |
| `screw(size, length:, style: :socket)` | Solid fastener body for `:m2`–`:m5`. `style:` is `:socket` (ISO 4762 cylindrical socket-head cap screw), `:button` (ISO 7380 low dome head), or `:flat` (ISO 10642 90° conical head). Pure Ruby DSL. |
| `mass_estimate(part, density: 1.24)` | Rough mass in grams from `part.volume × density / 1000` (mm³ × g/cm³). Default density is PLA; pass ABS 1.04, PETG 1.27, steel 7.85, etc. Pure Ruby DSL. |
| `print_volume_check(part, x:, y:, z:)` | Returns `{fits:, dx:, dy:, dz:, overflow_x:, overflow_y:, overflow_z:}` against a rectangular build volume. Pure Ruby DSL. |
| `overhang_faces(part, max_angle_deg: 45)` | Array of faces whose outward normal tips more than `max_angle_deg` below horizontal (assumes the part is +Z-up). Uses `Shape#normal`. Pure Ruby DSL. |
| `draft_faces(part, axis: [0, 0, 1], min_draft_deg: 1.0)` | Array of faces with insufficient mould draft along the pull axis: `asin(|n·axis|) < min_draft_deg`. Top/bottom faces are naturally excluded (their draft is 90°). Pure Ruby DSL. |
| `hole_axes(part, orientation: nil, tolerance_deg: 5.0)` | Enumerate cylindrical-surface faces of `part` as `{origin:, axis:, radius:}`. Filter with `orientation:` `:vertical` (axis ‖ Z) or `:horizontal` (axis ⊥ Z) within `tolerance_deg`. Pure Ruby DSL on top of `Shape#cylinder_axis`. |
| `unsupported_islands(part, layer_height: 0.2, axis: :z, min_area: 0.0, tolerance: 0.05)` | Slice `part` layer-by-layer and report disconnected footprints that do not overlap the previous layer. Returns an Array of layer hashes with `:offset`, `:components`, and `:unsupported`; each unsupported component includes `:area`, `:centroid`, and `:bbox`. `axis:` accepts `:x`, `:y`, `:z`, or an axis-aligned 3-element vector. Pure Ruby DSL. |
| `param(:name, default: val)` | Declare a named parameter. Returns `val` unless a `--param name=x` CLI override was supplied; coerces string overrides to the default's type (Integer/Float/String or typed length/angle values). |
| `param(:name, default: val, range: lo..hi)` | Same with range validation; raises `ArgumentError` if the value is outside the range. Typed unit defaults and ranges work as expected. |
| `solid { ... }` | Block returning its last expression |
| `assembly("name") { \|a\| ... }` | Named assembly. See `Assembly` below. |
| `preview(shape)` | Tessellate and push to live browser preview. No-op when not in `--preview` mode. |
| `require_relative(path)` | Evaluate another script file, resolved against the directory of the *requiring* file. `.rb` suffix optional. Returns `true` when evaluated, `false` when already loaded — so a file loads at most once and require cycles terminate. Unavailable in MCP mode and in the absence of a script directory. Plain `require` does not exist (no load path); it raises and redirects here. |
| `sketch(diagnostics: false, strict: false) { ... }` | Constraint sketch. Returns the profile Face the block describes, or the block's own return value if it is already a Shape. See [Constraint sketching](#constraint-sketching). |
| `puts(*args)` / `print(*args)` / `p(*args)` / `pp(*args)` | Print to standard output with Ruby's formatting rules (arrays flattened, `nil` as a blank line, no doubled trailing newline); `p` returns its argument. Defined in the prelude on a native primitive, since the embedded interpreter ships without the IO gems. Removed entirely in MCP mode, where stdout carries the JSON-RPC responses. |

### Constraint sketching

`sketch do ... end` evaluates its block against a `SketchBuilder` (block arity
1 receives the builder instead of `instance_eval`). Points may be
under-specified and are resolved by a propagating constraint solver; the
resolved outline is then built into a profile Face ready for `extrude`,
`revolve`, `pad`, or `pocket`.

Line-based sketches must form one closed loop of at least three segments — or
two once one of them is a `spline`, since a curve can close a loop on its own.
A `circle_at` / `arc_at` / `slot_between` call instead defines the whole
profile directly.

Corner modifiers set back along a straight run and `trim` / `extend` slide an
endpoint along one, so both reject spline segments; corners between two
straight segments still work in a sketch that has curves elsewhere.

| Method | Description |
|--------|-------------|
| `point(x, y)` | Anonymous point; either coordinate may be `nil` for the solver to fill in |
| `point(:name, x, y)` / `construction_point(:name, x, y)` | Named point, retrievable with `ref(:name)` or `self[:name]` |
| `line(a, b)` | Profile segment between two points. Returns `[a, b]` |
| `construction_line(a, b)` | Reference segment that shapes nothing itself; returns `[a, b]` for use as a `trim` / `extend` target |
| `spline(a, b, through: [...])` | Curved segment from `a` to `b` through the given interior points (sketch points or `[x, y]` pairs). Becomes a real interpolated BSpline edge, so a curved sketch is built by `make_profile_2d` rather than flattened into a polygon |
| `midpoint([:name,] a, b)` | Point constrained to the middle of `a`–`b` |
| `polar_point([:name,] center, radius, angle_deg)` | Point at polar coordinates around `center`, CCW from +X |
| `rectangle(origin, w, h)` / `centered_rectangle(center, w, h)` | Four constrained corners plus their four segments; returns the corner points |
| `circle_at(center, r)` / `arc_at(center, r, start_deg, end_deg)` / `slot_between(a, b, r)` | Define the profile directly instead of from segments |
| `fixed(p[, x, y])` | Pin a point's coordinates |
| `horizontal(a, b)` / `vertical(a, b)` / `coincident(a, b)` | Axis and coincidence constraints |
| `dimension(a, b, length)` | Fix the distance between two points |
| `equal_length(a, b, c, d)` / `parallel(a, b, c, d)` / `perpendicular(a, b, c, d)` | Relations between two segments |
| `symmetric(a, b, center)` / `mirror_x(src, dst[, axis_y])` / `mirror_y(src, dst[, axis_x])` | Mirror relations |
| `tangent(a, b, center, radius, side: nil)` | Constrain segment `a`–`b` tangent to a circle; propagates for axis-aligned lines with `side:`, otherwise verifies |
| `fillet(point, radius)` | Round a corner of the 2-D profile with a tangent arc |
| `chamfer(point, distance)` | Bevel a corner, setting back `distance` along both adjacent segments |
| `trim(a, b, by:\|to:, at: :end)` | Shorten a segment: one endpoint slides along it `by:` a distance or `to:` the intersection with another segment's infinite line. `at:` (`:end`, `:start`, or an endpoint) picks which end moves. The segment may be given as two points or as the array `line` returns |
| `extend(a, b, by:\|to:, at: :end)` | Lengthen a segment; same arguments as `trim` |
| `offset(distance)` | Grow (positive) or shrink (negative) the finished profile in its own plane. One non-zero offset per sketch; applied after corner modifiers and segment edits |
| `linear_pattern(count:, dx:, dy:)` | Repeat the finished profile along a row; copy *i* at *i* × (`dx`, `dy`). Needs a non-zero `dx:` or `dy:` |
| `polar_pattern(count:, center: nil, angle: 360)` | Repeat it around `center:` (a sketch point, an `[x, y]` pair, or the origin); copy *i* at *i* × (`angle` / `count`) |
| `grid_pattern(nx:, ny:, dx:, dy:)` | Repeat it across a 2-D grid |
| `diagnostics` | Structured solve report (components, estimated DOF, redundant constraints) from the builder, before the profile is built |

Patterning is the last step, after the offset, so every copy carries the
sketch's shaping; the result is one compound profile, so a single `extrude`,
`pad`, or `pocket` applies to all copies. A sketch takes one pattern, and a
`count:` of 1 is a no-op rather than an error. The three pattern names shadow
the top-level functions of the same name; passing a Shape as the first
argument delegates to those, so `polar_pattern(shape, n, angle)` still works
inside a sketch block.

Corner modifiers, segment edits, the offset, and the pattern are all validated
before any geometry is built — oversized or overlapping modifiers, collapsed segments,
parallel `to:` references, wrong-direction intersections, and offsets that
consume the whole profile each raise with the offending point named.

`sketch(diagnostics: true)` attaches the same report to the returned shape as
`.sketch_diagnostics`; `sketch(strict: true)` raises when redundant
constraints are present.

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
| `.chamfer(d)` | Bevel all edges (symmetric) |
| `.chamfer(d, :selector)` | Bevel only edges matching selector (symmetric) |
| `.chamfer_asym(d1, d2)` | Asymmetric bevel all edges: `d1` on the reference face side, `d2` on the other |
| `.chamfer_asym(d1, d2, :selector)` | Asymmetric bevel only edges matching selector |
| `.extrude(h)` | Extrude face/wire along Z |
| `.extrude(h, twist_deg: 0, scale: 1.0)` | Extrude with optional twist (degrees) and end-scale; uses `ThruSections` when twist/scale are non-default |
| `.revolve(deg=360)` | Revolve around Z axis |
| `.sweep(path)` | Sweep profile along a `spline_3d` wire |
| `.shell(thickness)` | Hollow out a solid by removing the topmost face and offsetting walls inward |
| `.offset(distance)` | Inflate (positive) or deflate (negative) a solid uniformly |
| `.offset_2d(distance)` | Offset a 2D Wire or Face inward (negative) or outward (positive) in its own plane. Uses `BRepOffsetAPI_MakeOffset`. A Face returns a Face — the offset wires are rebuilt into a planar profile, so the result extrudes, pads, and pockets like any other profile. Profiles with holes are supported: growing the material shrinks the holes. Raises if an inward offset consumes the whole profile. |
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
| `.shape_type` | Returns a Symbol naming the topological type: `:compound`, `:compsolid`, `:solid`, `:shell`, `:face`, `:wire`, `:edge`, `:vertex` |
| `.centroid` | Returns `[x, y, z]` centre of mass. Uses `BRepGProp::VolumeProperties` for solids/compounds, `SurfaceProperties` for shells/faces, `LinearProperties` for wires/edges. |
| `.closed?` | `true` if every edge is shared by at least 2 faces (no open boundary) |
| `.manifold?` | `true` if every edge is shared by exactly 2 faces (manifold mesh) |
| `.validate` | Runs `BRepCheck_Analyzer`. Returns `:ok` if the shape is valid, or an `Array` of error description strings if not. |
| `.sketch_diagnostics` | Returns the structured sketch diagnostics hash attached by `sketch(diagnostics: true)`, or `nil` if the shape did not come from a diagnostic sketch. |
| `.slice(plane: :xy, z: d)` | Cross-section by an axis-aligned plane. `plane:` is `:xy` (offset along Z), `:xz` (offset along Y), or `:yz` (offset along X). The offset key matches the plane normal axis (`z:` for `:xy`, `y:` for `:xz`, `x:` for `:yz`). Returns a compound of the section edges/wires. Uses `BRepAlgoAPI_Section`. |
| `.pad(face_sel, height:) { sketch }` | Extrude the block's sketch along the outward normal of `face_sel` and fuse with `self`. `face_sel` can be a Symbol (`:top`, `:bottom`, `">X"`, etc.) or an explicit face Shape. The sketch block is evaluated in the XY plane; it is automatically repositioned onto the target face. Returns a Solid. |
| `.pocket(face_sel, depth:) { sketch }` | Same as `pad` but extrudes inward and subtracts from `self`. Returns a Solid. |
| `.fillet_wire(r)` | Round all corners of a 2D Wire or Face by radius `r`. Returns a Face. Raises an error if `self` is a Solid or Shell. |
| `.extrude(h, draft: a)` | Extrude with draft angle `a` degrees. Positive → narrower at top (standard mould taper). Uses `BRepOffsetAPI_DraftAngle` on the straight prism's lateral faces. |
| `.distance_to(other)` | Minimum clearance distance to `other` shape (Float). Returns `0.0` when shapes overlap or touch. Uses `BRepExtrema_DistShapeShape`. |
| `.inertia` | Inertia tensor as a Hash `{ixx:, iyy:, izz:, ixy:, ixz:, iyz:}`, about the shape's own centre of mass (translating the shape does not change it). Volume moments in mm⁵ — OCCT integrates at density 1 — so scale by mass / volume for g·mm². Off-diagonals are true tensor entries (−∫xy dV), not products of inertia. `Assembly#mass_properties` does this conversion and sums across parts. |
| `.min_thickness` | Minimum wall thickness of a Solid or Shell (Float). Uses `IntCurvesFace_ShapeIntersector` ray-casting: shoots a ray inward from each face centroid along the face normal; returns the shortest non-trivial hit distance. Raises `ArgumentError` for non-solid/non-shell shapes. |
| `.normal` (on a Face) | Outward unit normal as `[nx, ny, nz]`. Sampled at the face's parameter-space midpoint via `BRepLProp_SLProps` and flipped when the face orientation is `TopAbs_REVERSED` so the vector points out of the parent solid. Raises if the shape is not a face or the normal is undefined at the sample point. |
| `.cylinder_axis` (on a Face) | For a cylindrical face, returns `{origin: [ox,oy,oz], axis: [ax,ay,az], radius: r}` via `BRepAdaptor_Surface::Cylinder()`. Raises if the shape is not a face or the underlying surface is not a cylinder. |
| `.export("out.step")` | Write file; format determined by extension: `.step`/`.stp` → STEP, `.stl` → STL, `.glb` → GLB, `.gltf` → glTF, `.obj` → OBJ, `.svg` → SVG 2-D drawing, `.dxf` → DXF R12 2-D drawing |
| `.export("out.svg", view: :top\|:front\|:side\|:sheet, scale: 1.0, hidden: false, center_marks: false, dimensions: false, title_block: false, callouts: false, datum: nil, feature_control: nil, tolerance: 0.0)` | SVG 2-D drawing via `HLRBRep_PolyAlgo` hidden-line removal. `:top` (default) looks down −Z; `:front` looks along −Y; `:side` looks along +X; `:sheet` lays out top/front/side views on one SVG page. `scale:` multiplies drawing geometry and must be positive. `hidden: true` adds dashed hidden-line polylines. `center_marks: true` adds cylinder centre marks. `callouts: true` adds diameter callouts for cylindrical faces. `datum:` accepts either a simple label or a hash with `label:` and `face:` / `selector:` to anchor the datum to a real face. `feature_control:` accepts a string or a hash with `text:` / `frame:` / `value:` plus `datums:` for attached references. `dimensions: true` adds width/height labels, and on `:sheet` the labels are axis-aware (`X/Y/Z`). `tolerance: { plus:, minus: }` formats the labels with either `±` or `+.../-...`; numeric shorthand still means symmetric `±`. `title_block: true` adds a small metadata block. `section: :xy|:xz|:yz` (or `section: { plane:, offset: }`) turns the drawing into a section view: the solid is cut with that axis-aligned plane and the exposed cut face is drawn with 45° hatching in `hatch` and `section` groups; an omitted `offset:` cuts the part's mid-plane. Outputs `<polyline>` elements with Y-down SVG coordinates. `detail: { at: [x, y], radius:, scale:, label: }` adds a magnified close-up of one circular region beside the view, with a marker circle on the parent and a `DETAIL <label> (<n>:1)` caption; `at:` and `radius:` are in model units on the view's own plane (`:top` → X/Y, `:front` → X/Z, `:side` → Y/Z). Rejected on `view: :sheet`. `ordinate: true` adds ordinate dimensions: a witness line from every located feature centre out to a baseline below and to the left, labelled with its distance from the datum corner (the lower-left of the projected geometry), in an `ordinates` group. Labels stay in model units regardless of `scale:`; features sharing a coordinate collapse to one ordinate. |
| `.export("out.dxf", view: :top\|:front\|:side\|:sheet, scale: 1.0, hidden: false, center_marks: false, dimensions: false, title_block: false, callouts: false, datum: nil, feature_control: nil, tolerance: 0.0)` | DXF R12 ASCII drawing via the same HLR pipeline. `view: :sheet` lays out top/front/side views on one sheet. `scale:` multiplies drawing geometry and must be positive. `hidden: true` writes hidden `LINE` entities on a `HIDDEN` layer. `center_marks: true` writes centre marks on a `CENTER` layer. `callouts: true` writes diameter callouts on a `CALLOUT` layer. `datum:` accepts either a simple label or a hash with `label:` and `face:` / `selector:` to anchor the datum to a real face. `feature_control:` accepts a string or a hash with `text:` / `frame:` / `value:` plus `datums:` for attached references. `dimensions: true` writes overall width/height labels on a `DIMENSION` layer, and on `:sheet` those labels are axis-aware (`X/Y/Z`). `tolerance: { plus:, minus: }` formats labels with either `±` or `+.../-...`; numeric shorthand still means symmetric `±`. `title_block: true` writes title-block entities on a `TITLEBLOCK` layer. `section:` takes the same plane symbol or `{ plane:, offset: }` hash as the SVG side and writes the hatching on a dedicated `HATCH` layer. Outputs Y-up CAD coordinates. `detail:` takes the same hash as the SVG side and writes the marker, border circle, label, and caption on a dedicated `DETAIL` layer. `ordinate: true` writes the same ordinate dimensions on an `ORDINATE` layer, with right-aligned `TEXT` (group code `72`) so rotated labels grow away from the drawing. |
| `.export_outline(path, deflection: 0.05)` | **Cut file**, not a drawing: the closed loops of one planar face at 1:1, with nothing else in the file. Receiver must be a planar `Face` or a shape holding exactly one face (a solid raises, naming how to select one; a curved face is refused). Circular edges are written as true `CIRCLE` / `ARC` entities so holes and rounded corners stay exact; only free-form curves are approximated, to `deflection:` mm of chord error. The outline is taken in the face's own plane, so a tilted face keeps true size, and is shifted so its bounding box starts at the origin, ready to nest. Holes land on a `HOLES` layer separate from `PROFILE` (SVG: `class="holes"` / `"profile"`). `.dxf` declares mm via `$INSUNITS`; `.svg` is sized in mm. |
| `fragment([a, b, c])` | General Boolean fragment: split all shapes at their mutual intersection boundaries. Returns a Compound of all non-overlapping pieces. Uses `BRepAlgoAPI_BuilderAlgo`. |
| `.convex_hull` | 3-D convex hull of the shape's tessellated mesh vertices. Uses an incremental QuickHull algorithm; returns a BRep solid. |
| `path_pattern(shape, path, n)` | Distribute `n` arc-length-evenly-spaced copies of `shape` along `path` (a Wire). Each copy is oriented so its local Z-axis aligns with the path tangent. |
| `.sweep(path, guide: wire)` | Guided sweep: sweep `self` (profile Wire or Face) along `path` while keeping the profile orientation locked to the auxiliary `guide` Wire. Uses `BRepOffsetAPI_MakePipeShell::SetMode`. |

---

### Assembly

`Assembly` groups named shapes and can position them either eagerly via the
existing transform helpers or lazily via a declarative constraint graph.

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

rig = assembly("rig") do |a|
  a.ground :base, base
  a.part :post, post do
    mate from: :bottom, to: face(:base, :top)
  end
end
rig.to_shape
```

| Method | Description |
|--------|-------------|
| `a.place(shape)` | Add `shape` at its current position. Returns `shape`. |
| `a.place(shape, name:, component:, material:, density:, mass:)` | Same, with reporting metadata. All five keywords are optional and are also accepted by `part`, `ground`, `mate`, `distance_mate`, `axis_align`, and `angle_mate`. Geometry ignores them. `mass:` (grams) states a datasheet weight for a bought part, overriding `volume × density`; passing it together with `density:` raises. |
| `a.ground(name, shape) { ... }` | Register a fixed part in the declarative solver. The first `part` is fixed by default; use `ground` for clarity on the root body. |
| `a.part(name, shape, fixed: nil) { ... }` | Register a named rigid part for lazy solving. The yielded builder supports `mate`, `distance_mate`, and `angle_mate` constraints; the first part defaults to fixed when `fixed:` is omitted. |
| `a.face(part_name, selector)` | Build a face reference for the declarative solver. Use with `a.part(...){ mate from: :bottom, to: face(:base, :top) }`. |
| `asm.solve` | Resolve the declarative assembly graph and return a Hash of part names to positioned Shapes. Raises on under-constrained or conflicting assemblies. |
| `a.mate(shape, from:, to:, offset: 0.0)` | Reposition `shape` so `from:` face aligns against `to:` face, then add to the assembly. Returns the repositioned shape. |
| `a.distance_mate(shape, from:, to:, distance:)` | Variant of `mate` that names the air-gap intent and requires `distance > 0`. Equivalent to `mate(..., offset: distance)`. |
| `a.axis_align(shape, from: [p1, p2], to: [q1, q2])` | Rotate and translate `shape` so the source axis (`p1 → p2` in the shape's frame) maps to the target axis (`q1 → q2` in world coordinates). Useful for coaxial / concentric / axis-alignment placement by point pairs. |
| `a.angle_mate(shape, from:, to:, angle:, pivot:, axis_dir:, offset: 0.0)` | Mate `from:` face flush onto `to:` face (with optional `offset:` gap), then rotate the placed shape by `angle` degrees about an axis through `pivot` in direction `axis_dir`. Locks the rotational DOF left over by a planar mate. |
| `asm.export(path, **opts)` | Fuse all shapes and export to file. When declarative parts are present, the assembly is solved lazily before fusion. Options are forwarded untouched to `Shape#export`, so an assembly takes the same drawing controls a part does (`view:`, `scale:`, `section:`, `dimensions:`, `ordinate:`, `detail:`, `hidden:`, `title_block:`). |

#### Assembly reports

These solve the assembly first, so the shapes they measure are in their final
world positions. None of them mutate the assembly.

| Method | Description |
|--------|-------------|
| `asm.components` | Every component as `{name:, component:, material:, density:, shape:}` — ad-hoc placements first (auto-named `:part_1`, `:part_2`, … when no `name:` was given), then solver parts. |
| `asm.interferences(clearance: 0, ignore_contact: true)` | Check all pairs for solid overlap and, when `clearance:` is given, for an insufficient gap. Returns `{a:, b:, type: :interference, volume:, centroid:}` and `{a:, b:, type: :clearance, distance:, minimum:}` rows, worst first. Parts in contact are skipped by the clearance check unless `ignore_contact: false`. O(n²) booleans. |
| `asm.clash?` | `true` when any two components overlap in solid volume. |
| `asm.bom(density: 1.24)` | Bill of materials rolled up by `component:`. Rows are `{component:, quantity:, material:, density:, unit_volume:, volume:, unit_mass:, mass:, parts:}`, sorted by descending quantity. Raises if one component key groups parts of different volume. |
| `asm.bom_text(density: 1.24)` | `bom` rendered as an aligned text table with a TOTAL row. |
| `asm.mass_properties(density: 1.24, about: nil)` | `{volume:, mass:, center_of_mass:, inertia:, inertia_about:, parts: [...]}`. Mass-weighted centre of mass; volumes in mm³, masses in grams, inertia in g·mm² (×1e−9 for kg·m²). `about:` takes `:origin` or a 3-element point; it defaults to the centre of mass. Overlapping parts double-count their shared volume — run `interferences` first. |

Each part row in `bom` and `mass_properties` carries `mass_source:`
(`:stated` or `:density`) so an overridden mass is never mistaken for a
computed one. A stated mass on a zero-volume shape (a `datum_plane`) acts as a
point mass at that location.

Inertia is summed by parallel-axis transfer from each part's own tensor.
A stated `mass:` rescales that part's inertia distribution to match the
declared weight, which assumes uniform density across its envelope.

Density for each component resolves in order: explicit `density:`, then the
built-in material table (steel 7.85, aluminium 2.70, brass 8.50, PLA 1.24,
ABS 1.04, PETG 1.27, nylon 1.14, delrin 1.41, and others; names are matched
case- and punctuation-insensitively), then the `density:` argument to the
reporting call, then 1.24 g/cm³ — the same PLA default as `mass_estimate`.
An unrecognised material name is not an error, since material strings are
free text; every report echoes the density it actually used.

`Shape#rotate_about(point, axis_dir, angle_deg)` is the underlying transform
primitive used by `angle_mate`: rotate the shape by `angle_deg` around an
axis through `point` (3-element array) pointing in `axis_dir` (3-element
array). Implemented as `translate(−p) → rotate → translate(+p)`.

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

`--preview` starts an `axum` HTTP server on `http://localhost:3000`, opens the browser, watches the script with `notify`, and calls `preview(shape)` automatically on each re-eval. Every file pulled in with `require_relative` is watched too, and the watch set is recomputed after each reload. Ctrl-C to quit.

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
