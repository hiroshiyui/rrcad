# rrcad TODOs

A Ruby DSL-driven 3D CAD language, with Rust as the glue/performance layer,
mRuby as the embedded scripting engine, and OCCT as the geometry kernel.

---

## ✓ Phase 0 — OCCT Minimal Rust Bindings (complete)

`cxx` bridge wired to OCCT 7.9. Primitives (box, cylinder, sphere),
boolean ops (fuse, cut, common), fillets, chamfers, transforms
(translate, rotate, scale), and STEP / STL / glTF export are all bound
and covered by smoke tests. See `src/occt/`.

---

## ✓ Phase 1 — mRuby Embedded in Rust (complete)

mRuby 3.4.0 vendored as a submodule and built via `rake`. Manual FFI in
`src/ruby/ffi.rs`; C glue shim in `src/ruby/glue.c`. `MrubyVm` RAII
wrapper in `src/ruby/vm.rs`. Native `Shape` Ruby class backed by
`Box<occt::Shape>` raw pointer in mRuby `RData void*`; `dfree` callback
drops the Box on GC. Top-level `box`, `cylinder`, `sphere`; `.export`,
`.fuse`, `.cut`, `.common`. DSL prelude auto-loaded via `include_str!`.
REPL (readline, history) + script file execution. See `tests/e2e_dsl.rs`.

---

## ✓ Phase 2 — DSL Enrichment (complete)

`.translate`, `.rotate`, `.scale`, `.mirror(:xy|:xz|:yz)`, `.fillet(r)`,
`.chamfer(d)`, `.extrude(h)`, `.revolve(angle)`. 2D sketch faces: `rect`,
`circle`. `solid do…end` block. `Assembly` class with `place`; `mate`
deferred to Phase 5. Error messages propagated as Ruby `RuntimeError`.
REPL tab-completion and `help` command. See `tests/phase2_dsl.rs`.

---

## ✓ Phase 3 — Live Preview (complete)

Spline profiles (`spline_2d`, `spline_3d`) and pipe sweep (`.sweep`) via
`GeomAPI_Interpolate` + `BRepOffsetAPI_MakePipe`. Sub-shape selectors
`.faces(:top|:bottom|:side|:all)` and `.edges(:vertical|:horizontal|:all)`
using `BRepLProp_SLProps` (orientation-aware normals) and
`TopTools_IndexedMapOfShape` (deduplicated iteration).

Live preview: `rrcad --preview <script.rb>` tessellates to binary glTF (GLB)
via `export_glb`; `axum` HTTP server serves the GLB and a Three.js viewer
page (`GLTFLoader` + `OrbitControls` + auto-fit camera); `notify` watches
the script file and re-evals on save; WebSocket pushes `"reload"` to the
browser. `preview(shape)` is a no-op when not in `--preview` mode so scripts
stay portable. See `tests/teapot_dsl.rs` (6 tests), `tests/phase3_selectors.rs` (16 tests).

---

## Phase 4 — OCCT Coverage (OpenSCAD / CadQuery parity) — in progress

Goal: close the gap between our DSL and what OpenSCAD / CadQuery expose from OCCT.

Completed: primitives `cone`, `torus`, `wedge`; 2D profiles `polygon`, `ellipse`, `arc`.

### 3-D operations
- [ ] `loft([profile1, profile2, ...], ruled: false)` — `BRepOffsetAPI_ThruSections`; solves organic shapes (teapot body, blades, …)
- [ ] `.shell(thickness)` — hollow out a solid; `BRepOffsetAPI_MakeThickSolid::MakeThickSolidByJoin`
  - Note: **not** `BRepOffsetAPI_MakeOffset` — that is for 2D wire offsetting only
- [ ] `.offset(distance)` — inflate / deflate a solid; `BRepOffsetAPI_MakeOffsetShape`
- [ ] `.extrude(h, twist_deg: 0, scale: 1.0)` — extend `shape_extrude` with twist and end-scale;
  use `BRepOffsetAPI_MakePipeShell` with a `GeomFill_EvolvedSection` law (replaces `MakePrism` when twist/scale are nonzero)

### Transforms
- [ ] `.scale(sx, sy, sz)` — non-uniform scale; `BRepBuilderAPI_GTransform` with `gp_GTrsf`
  (current `shape_scale` uses `gp_Trsf::SetScaleFactor` which is uniform-only)

### Selective modifiers
- [ ] `.fillet(r, selector)` — fillet only edges matching a selector string (reuse existing edge-selector machinery)
- [ ] `.chamfer(d, selector)` — same for chamfers

### Patterns
- [ ] `linear_pattern(shape, n, [dx, dy, dz])` — repeat shape n times along a vector
- [ ] `polar_pattern(shape, n, angle_deg)` — rotate n copies around the Z axis

### Import
- [ ] `import_step("file.step")` — `STEPControl_Reader`
- [ ] `import_stl("file.stl")` — `RWStl::ReadFile` (returns `Handle(Poly_Triangulation)`)

### Query / introspection
- [ ] `.bounding_box` — returns `{x:, y:, z:, dx:, dy:, dz:}`; use `BRepBndLib::AddOptimal` (tighter than `BRepBndLib::Add`) + `Bnd_Box`
- [ ] `.volume` — mass properties; `BRepGProp::VolumeProperties` + `GProp_GProps`
- [ ] `.surface_area` — `BRepGProp::SurfaceProperties` + `GProp_GProps`

### Sub-shape selectors (extensions)
- [ ] `vertices` selector on shapes (complement to existing `faces` / `edges`)
- [ ] Direction-based face selector: `faces(">Z")` / `faces("<X")` (CadQuery style)

### Export additions
- [ ] OBJ export — `RWObj_CafWriter` (OCCT 7.6+); same XDE pipeline as glTF, trivial to add

---

## OCCT API Improvements (existing code)

Targeted improvements to the current `bridge.cpp` implementation using newer OCCT APIs.

### Boolean operations — robustness and performance
Use the builder-style API instead of the two-argument constructors (`BRepAlgoAPI_Fuse(a,b)`):
```cpp
op.SetRunParallel(Standard_True);  // multi-threaded; OCCT 7.4+
op.SetFuzzyValue(1e-6);            // handles near-coincident faces; prevents spurious failures
```
Apply to all three of `shape_fuse`, `shape_cut`, `shape_common`.

### Tessellation — parallel meshing
`BRepMesh_IncrementalMesh` accepts a parallel flag (OCCT 7.4+); speeds up complex shapes:
```cpp
BRepMesh_IncrementalMesh mesher(shape, deflection, /*isRelative=*/false, 0.5,
                                /*isParallel=*/Standard_True);
```
Apply in `export_stl`, `export_gltf`, `export_glb`.

### Spline tangent constraints
`GeomAPI_Interpolate::Load(startTangent, endTangent)` produces smoother BSplines with
user-specified end tangents. Currently `make_spline_2d` / `make_spline_3d` use natural
boundary conditions, which can cause oscillation near endpoints (visible on teapot sweep paths).
Expose as optional `tangents:` array argument in the DSL.

### GLB transform format
`RWGltf_CafWriter::SetTransformationFormat(RWGltf_WriterTrsfFormat_TRS)` (OCCT 7.7+)
emits TRS components instead of a 4×4 matrix — the glTF spec default, better for Three.js.
Add to `export_glb` before `Perform`.

### GLB color/material support
Before calling `RWGltf_CafWriter::Perform`, attach colors via `XCAFDoc_ColorTool`:
```cpp
Handle(XCAFDoc_ColorTool) colorTool = XCAFDoc_DocumentTool::ColorTool(doc->Main());
colorTool->SetColor(shapeLabel, Quantity_Color(r, g, b, Quantity_TOC_sRGB),
                   XCAFDoc_ColorSurf);
```
Needed for multi-part assemblies where parts should render in different colors.

### Shape validity check before export
Add `BRepCheck_Analyzer` to `export_step` / `export_glb` for actionable error messages
instead of silent writer failures:
```cpp
#include <BRepCheck_Analyzer.hxx>
BRepCheck_Analyzer checker(shape);
if (!checker.IsValid())
    throw std::runtime_error("shape is topologically invalid — check for degenerate faces");
```

### Feature removal (`BRepAlgoAPI_Defeaturing`, OCCT 7.4+)
Removes small features (holes, fillets) for simplified export-for-simulation meshes.
Not urgent, but useful for a future `.simplify(min_feature_size)` DSL method.

---

## Phase 5 — Parametric Design & Constraints

Goal: scripts with parameters, constraints, and design tables.

- [ ] `param :width, default: 10, range: 1..100` DSL
- [ ] Constraint solver integration (research options: SolveSpace lib, custom)
- [ ] Design table: vary params across rows, export batch of STEP files
- [ ] `--param width=20` CLI override

---

## Utah Teapot Sample (deferred)

A faithful `samples/07_teapot.rb` requires `loft` (Phase 4) to model the body
properly — the current `spline_2d + revolve` approach cannot reproduce the
Newell silhouette without BSpline oscillation.

### Reference data

- **Primary source** — original Newell Bezier patch data and measurements:
  <https://users.cs.utah.edu/~dejohnso/models/teapot.html>
- **Bezier patches** (raw vertex + patch index file):
  <https://users.cs.utah.edu/~dejohnso/models/teapot_bezier>

### Coordinate system (Newell)

- **Y = height axis**; total body height ≈ 3.0 units (rim at Y ≈ 2.25,
  knob tip at Y ≈ 3.15)
- **Max body radius** = 2.0 at Y ≈ 0.9 (40 % of body height)

### Scaling to OCCT (Z-up, body height = 7.5)

| Newell | OCCT | factor |
|--------|------|--------|
| Y (height) | Z | × 3.333 |
| X/Z (radius) | X/Y | × 3.5 |

### Key geometry (Newell units → OCCT)

| Feature | Newell | OCCT |
|---------|--------|------|
| Body bottom | Y=0, r=0 | Z=0, r=0 |
| Foot ring | Y=0.15, r=1.5 | Z=0.50, r=5.25 |
| **Widest** | Y=0.90, r=2.0 | **Z=3.00, r=7.00** |
| Shoulder | Y=1.35, r=1.75 | Z=4.50, r=6.13 |
| Neck | Y=1.65, r=1.40 | Z=5.50, r=4.90 |
| Rim opening | Y=2.25, r=1.40 | Z=7.50, r=4.90 |
| Spout junction | Y≈0.45, X≈1.70 | Z≈1.50, X≈5.95 |
| Spout tip | Y≈2.40, X≈3.50 | Z≈8.00, X≈12.25 |
| Handle bottom | Y≈0.45, X≈−1.50 | Z≈1.50, X≈−5.25 |
| Handle top | Y≈2.10, X≈−1.50 | Z≈7.00, X≈−5.25 |
| Handle max extent | Y≈1.35, X≈−3.00 | Z≈4.50, X≈−10.50 |

### Recommended implementation approach (post Phase 4)

1. Body — `loft` through 6–8 cross-section circles at the Z heights above
2. Spout — `circle(1.2).sweep(spline_3d([...]))` along the 4-point centerline
3. Handle — `circle(1.0).sweep(spline_3d([...]))` along the 5-point centerline
4. Lid — `spline_2d + revolve`, translated to Z=6.0 (rim opening)
5. Knob — `sphere(0.5).translate(0, 0, 7.6)`

---

## Architecture Notes

```
Ruby DSL (.rb script)
      │ mRuby VM
Rust binding layer
  • native.rs: extern "C" entry points
  • glue.c: C shim hiding mrb_value from Rust
  • Shape: Box<occt::Shape> raw pointer in mRuby RData void*
  • dfree callback drops the Box on GC
      │ cxx bridge (C++ ABI)
OCCT geometry kernel
  • BRep modeling · splines
  • Tessellation
  • STEP / STL / glTF export
```

**Memory model:** Each native `Shape` is a heap-allocated `Box<occt::Shape>`.
The raw pointer lives in the mRuby `RData void*` slot. `dfree` drops it.
No SlotMap, no cross-language reference counting.

**Rendering (current):** OCCT tessellation → GLB → `axum` HTTP → Three.js browser viewer → WebSocket live reload. Activated with `rrcad --preview <script.rb>`.

**Rendering:** Web-based preview via axum + Three.js is the long-term approach; native egui/wgpu viewer has been dropped.
