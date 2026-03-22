# rrcad — Implementation History

A Ruby DSL-driven 3D CAD language. Rust as the glue layer, mRuby as the
scripting engine, OCCT as the geometry kernel.

---

## ✓ Phase 0 — OCCT Minimal Rust Bindings

`cxx` bridge to OCCT 7.9. Primitives (`box`, `cylinder`, `sphere`), boolean
ops (`fuse`, `cut`, `common`), fillets, chamfers, transforms, and STEP/STL/glTF
export. See `src/occt/`.

---

## ✓ Phase 1 — mRuby Embedded in Rust

mRuby 3.4.0 vendored; C glue shim (`glue.c`) hides `mrb_value` from Rust.
`MrubyVm` RAII wrapper. Native `Shape` class backed by `Box<occt::Shape>` raw
pointer in mRuby `RData void*`. DSL prelude auto-loaded via `include_str!`.
REPL with readline and history. See `tests/e2e_dsl.rs`.

---

## ✓ Phase 2 — DSL Enrichment

Transforms: `.translate`, `.rotate`, `.scale`, `.mirror`. Modifiers: `.fillet`,
`.chamfer`. Sketch ops: `.extrude`, `.revolve`. 2D faces: `rect`, `circle`.
`solid do…end` block. `Assembly` with `place`. REPL tab-completion and `help`.
See `tests/phase2_dsl.rs`.

---

## ✓ Phase 3 — Splines, Sweep, Live Preview

Spline profiles (`spline_2d`, `spline_3d`) and pipe sweep (`.sweep`) via
`GeomAPI_Interpolate` + `BRepOffsetAPI_MakePipe`. Sub-shape selectors:
`.faces`, `.edges`. Live preview: `rrcad --preview <script.rb>` — `axum` HTTP
server + Three.js viewer + WebSocket live reload via `notify`. `preview(shape)`
is a no-op outside preview mode. See `tests/teapot_dsl.rs`, `tests/phase3_selectors.rs`.

---

## ✓ Phase 4 — OCCT Coverage (OpenSCAD / CadQuery parity)

Additional primitives: `cone`, `torus`, `wedge`. 2D profiles: `polygon`,
`ellipse`, `arc`. 3-D ops: `loft`, `.shell`, `.offset`, `.extrude(twist/scale)`.
Non-uniform scale (`.scale(sx,sy,sz)`). Selective fillet/chamfer by edge
selector. Patterns: `linear_pattern`, `polar_pattern`. Vertex selector.
Direction-based face selector (`">Z"` / `"<X"` etc.). OBJ export. STEP/STL
import. Query: `.bounding_box`, `.volume`, `.surface_area`. OCCT API hardening:
builder-style booleans with fuzzy tolerance, parallel tessellation, GLB TRS
transform format, `BRepCheck_Analyzer` validity guard before export.
See `tests/phase4_3d_ops.rs`, `tests/occt_layer.rs`.

---

## ✓ Phase 5 — Parametric Design & Assembly

`param :name, default:, range:` DSL declaration with `--param key=value` CLI
override. Design table batch export via `--design-table table.csv`. Per-shape
sRGB color (`.color(r,g,b)`) written into GLB/glTF/OBJ via `XCAFDoc_ColorTool`.
Assembly mating (`Shape#mate`, `Assembly#mate`) using OCCT planar face geometry:
normal alignment + centroid translation, with optional gap/interference offset.
Spline tangent constraints (`tangents:` keyword on `spline_2d`/`spline_3d`).
Feature removal (`.simplify(min_feature_size)`) via `BRepAlgoAPI_Defeaturing`.
See `tests/phase5_params.rs`, `tests/e2e_dsl.rs`.

---

## ✓ Phase 6 — Variable-Section Sweep &amp; Teapot Rebuild

`sweep_sections(path, [profile, ...])` DSL function backed by
`BRepOffsetAPI_MakePipeShell`.  Each origin-centred profile is automatically
translated to the corresponding spine point (evenly-distributed along the
spline parameter) and swept with `WithCorrection=true` so cross-sections stay
perpendicular to the spine tangent.  Falls back to `BRepOffsetAPI_ThruSections`
when `MakeSolid()` fails on highly-curved spines (e.g., the teapot handle
C-arc).  See `tests/teapot_dsl.rs` (`sweep_sections_*` tests).

`bezier_patch([pt0..pt15])` — builds a single bicubic Bézier face from
16 control points (4×4 row-major grid) using `Geom_BezierSurface` +
`BRepBuilderAPI_MakeFace`.  `sew([faces], tolerance:)` — assembles multiple
Bézier faces into a closed shell/solid via `BRepBuilderAPI_Sewing` +
`BRepBuilderAPI_MakeSolid`.  Primary use case: Utah Teapot from Newell patches.

---

## ✓ Utah Teapot Sample

`samples/07_teapot.rb` — rebuilt from the original Newell Bézier patch data
(sourced from https://users.cs.utah.edu/~dejohnso/models/teapot.html, ×3.0 scale).
All 28 bicubic Bézier patches from the Newell / Blinn dataset.  Coordinate
transform Y-up → Z-up: `pt(x,y_s,z_s)` → rrcad `[x, z_s, y_s]`.  Patches
sewn with `BRepBuilderAPI_Sewing` (tolerance 1e-3) into a continuous surface;
`scale(3.0)` → rim at Z≈6.75, lid knob at Z=9.0.  Open at the base (no bottom
disc — consistent with the original Newell definition).
Validated by `tests/teapot_sample.rs` (9 tests including `bezier_patch` and
`sew` unit tests).

---

## Phase 7 — Improve OCCT Coverage & Compatibility

### Tier 1 — Quick wins, high ROI (complete existing patterns)

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 1 | **Asymmetric chamfer** | `.chamfer(d1, d2)` or `.chamfer(d1..d2)` | `BRepFilletAPI_MakeChamfer::Add(edge, d1, d2)` |
| 2 | **2D profile offset** | `.offset_2d(d)` | `BRepOffsetAPI_MakeOffset` on a Face/Wire |
| 3 | **Grid pattern** | `grid_pattern(s, nx, ny, dx, dy)` | Pure Rust composition over `linear_pattern` |
| 4 | **Multi-shape fuse/cut** | `fuse_all([a,b,c])`, `cut_all([a,b,c])` | Fold-left in Rust, no new C++ |

### Tier 2 — Validation & introspection (robustness for real workflows)

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 5 | **Shape type query** | `.shape_type` → `:solid/:shell/:face/:wire/:edge/:vertex` | `shape.ShapeType()` → `TopAbs_ShapeEnum` |
| 6 | **Closed / manifold check** | `.closed?`, `.manifold?` | `ShapeAnalysis` + edge-sharing loop |
| 7 | **Centroid** | `.centroid` → `[x, y, z]` | `BRepGProp::VolumeProperties` (already have `volume`) |
| 8 | **Topology validation** | `.validate` → `:ok` or error list | `BRepCheck_Analyzer` (already used in export guard) |

### Tier 3 — Surface modeling (next frontier)

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 9 | **Ruled surface** | `ruled_surface(wire_a, wire_b)` | `BRepFill_RuledSurface` |
| 10 | **Fill surface** | `fill_surface([boundary_wires])` | `BRepFill_Filling` |
| 11 | **Slice by plane** | `.slice(plane: :xy, z: 5.0)` → Face | `BRepAlgoAPI_Section` |

### Tier 4 — Interop (legacy CAD exchange)

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 12 | **IGES import** | `import_iges("file.igs")` | `IGESControl_Reader` |
| 13 | **IGES export** | `shape.export("file.igs")` | `IGESControl_Writer` |
| 14 | **SVG export** (2D projection) | `shape.export("file.svg")` | Orthographic projection → polylines |

### Tier 5 — Phase 8 candidates (include only if time allows)

- **Draft angle extrude** — `.extrude(h, draft: 5.0)` — tapered walls for moulded parts
- **Path pattern** — `path_pattern(shape, spline_path, n)` — copies at equal arc-length steps along a curve
- **Closest point / collision** — `.distance_to(other)` → float — `BRepExtrema_DistShapeShape`
- **Moment of inertia** — `.inertia` → tensor — `BRepGProp::VolumeProperties`
- **2D Boolean on sketches** — `sketch_a.fuse_2d(sketch_b)` — `BRepAlgoAPI_*` restricted to Face shapes
- **Wire fillet** (pre-profile) — `.fillet_wire(r)` — round 2D sketch corners before extrude
- **Pipe with guide curve** — advanced sweep; `BRepFill_PipeShell` with guide surface
- **Convex hull** — `.convex_hull` — bounding convex solid (OpenSCAD `hull()` parity)

### Implementation order

```
Week 1:  Tier 1 (#1–4)  — asymmetric chamfer, offset_2d, grid_pattern, fuse_all/cut_all
Week 2:  Tier 2 (#5–8)  — shape_type, closed?/manifold?, centroid, validate
Week 3:  Tier 3 (#9–11) — ruled_surface, fill_surface, slice
Week 4:  Tier 4 (#12–14) — IGES import/export, SVG export
Week 5+: Tier 5 / Phase 8
```

### Notes

- Tier 1 + 2 require almost no new C++ — completing existing patterns in Rust/glue.c.
- Surface modeling (Tier 3) introduces face-only shapes (not solids); `BRepCheck_Analyzer`
  is the safety net. `BRepFill_Filling` boundary wires must be closed.
- IGES follows the same reader/writer pattern as STEP — no architectural change.
- SVG export requires orthographic projection; a visible-edges-only wireframe is a good v1.
- Draft angle extrude is common for manufacturing but needs its own OCCT path; punted to
  Phase 8 to keep Phase 7 focused.

---

## Phase 8 — Part Design: Sketch-on-Face, Pad & Pocket

**Goal:** close the gap with FreeCAD's Part Design workbench for "CAD as code" workflows.
FreeCAD's core loop is: select a face → sketch in its plane → pad (extrude outward) or
pocket (cut inward).  This phase brings that loop to the Ruby DSL, making rrcad a credible
alternative for mechanical part modelling without a GUI.

### The core DSL pattern

```ruby
plate = box(100, 60, 10)

# Pocket: cut a rounded slot from the top face
result = plate.pocket(:top, depth: 8) do
  rect(40, 20).fillet_wire(4)          # 2D sketch in face-local coords
end

# Pad: add a boss on the bottom face
result = plate.pad(:bottom, height: 6) do
  circle(10)
end

# Arbitrary face by index or direction string
result = plate.pocket(">X", depth: 5) do
  circle(4).translate(0, 5)            # face-local X/Y
  circle(4).translate(0, -5)
end
```

Face-local coordinate system: origin at the face centroid, X/Y along the face
tangent directions, Z along the outward normal.  All 2D shapes in the block are
interpreted in this local frame.  The implementation transforms them to world
coordinates before extrude/cut.

### Tier 1 — Core Part Design primitives

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 1 | **Pad** | `.pad(face_sel, height:) { sketch }` | `BRepPrimAPI_MakePrism` along face normal + `BRepAlgoAPI_Fuse` |
| 2 | **Pocket** | `.pocket(face_sel, depth:) { sketch }` | `BRepPrimAPI_MakePrism` along −normal + `BRepAlgoAPI_Cut` |
| 3 | **Wire fillet** | `.fillet_wire(r)` on a Face/Wire | `BRepFilletAPI_MakeFillet2d` |
| 4 | **Datum plane** | `datum_plane(origin:, normal:, x_dir:)` | `gp_Ax3` + `BRepBuilderAPI_MakeFace(gp_Pln)` — returns a reusable plane shape for `.pad`/`.pocket` |

Implementation of face-local transform (shared by pad + pocket):
1. `BRep_Tool::Surface(face)` → cast to `Geom_Plane` → get `gp_Ax3`
2. `BRepGProp::SurfaceProperties` → face centroid as `gp_Pnt` origin
3. `gp_Trsf::SetTransformation(ax3)` → maps world → face-local (invert for local → world)
4. `BRepBuilderAPI_Transform(sketch, trsf)` → sketch in world coords
5. `BRepPrimAPI_MakePrism(sketch_face, normal_vec * depth)` → tool solid
6. `BRepAlgoAPI_Fuse` (pad) or `BRepAlgoAPI_Cut` (pocket)

### Tier 2 — Manufacturing features

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 5 | **Draft angle** | `.extrude(h, draft: angle_deg)` | `BRepOffsetAPI_DraftAngle` |
| 6 | **Helix path** | `helix(radius:, pitch:, height:)` → Wire | `BRepBuilderAPI_MakeEdge` on `Geom_Line` + `BRepBuilderAPI_MakeWire` with helical param |
| 7 | **Thread** | `thread(solid, face_sel, pitch:, depth:)` | helix path + triangular profile + `.sweep` + `.cut` from base solid |
| 8 | **Counterbore / countersink** | `cbore(d:, cbore_d:, cbore_h:)` sketch macro | compound `rect`+`circle` sketch — pure Ruby DSL, no new C++ |

### Tier 3 — Inspection & clearance

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 9 | **Distance between shapes** | `.distance_to(other)` → Float | `BRepExtrema_DistShapeShape` |
| 10 | **Moment of inertia** | `.inertia` → `{ixx:, iyy:, izz:, ixy:, …}` | `BRepGProp::VolumeProperties` → `GProp_GProps::MatrixOfInertia` |
| 11 | **Thickness map** | `.min_thickness` → Float | `BRepExtrema_DistShapeShape` on shell vs offset shell |

### Tier 4 — 2D drawing output

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 12 | **Slice to face** | `.slice(plane: :xy, at: 5.0)` → Face | `BRepAlgoAPI_Section` |
| 13 | **SVG export** | `shape.export("part.svg", view: :top)` | `HLRBRep_Algo` (hidden-line removal) + `HLRBRep_HLRToShape` → polylines → SVG |
| 14 | **DXF export** | `shape.export("part.dxf")` | slice → wire edges → `DXF_Writer` (lightweight hand-rolled or via `IFSelect`) |

SVG via HLRBRep: `HLRBRep_Algo` computes visible / hidden edges from a given
projection direction; `HLRBRep_HLRToShape` extracts them as `TopoDS_Edge` collections;
each edge is tessellated into polyline segments and serialised as SVG `<path>` elements.
This is the same pipeline FreeCAD's TechDraw uses internally.

### Tier 5 — Advanced composition

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 15 | **Boolean fragment** | `fragment([a, b, c])` → Array of solids | `BRepAlgoAPI_BuilderAlgo` (general fuser that keeps all fragments) |
| 16 | **Convex hull** | `.convex_hull` | `BRepBuilderAPI_Copy` + `BRepAlgoAPI_Fuse` fold (no native OCCT hull; wrap qhull or approximate via point cloud + loft) |
| 17 | **Path pattern** | `path_pattern(shape, path, n)` | `BRepGProp_SurfaceProperties` arc-length param → `n` equally spaced transforms |
| 18 | **Pipe with guide** | `sweep(profile, path, guide:)` | `BRepFill_PipeShell::SetMode(guide_wire)` |

### Implementation order

```
Sprint 1: Tier 1 (#1–4) — pad, pocket, fillet_wire, datum_plane
          These four together unlock the core FreeCAD Part Design loop in rrcad.

Sprint 2: Tier 2 (#5–8) — draft angle, helix, thread, cbore macro
          Manufacturing features; helix is the prerequisite for thread.

Sprint 3: Tier 3 (#9–11) — distance_to, inertia, min_thickness
          Inspection / clearance checks; pure OCCT queries, no new topology.

Sprint 4: Tier 4 (#12–14) — slice, SVG export, DXF export
          2D drawing output; SVG via HLRBRep is the most complex item here.

Sprint 5: Tier 5 (#15–18) — fragment, convex hull, path pattern, guided sweep
          Advanced composition; lower priority, implement as demand arises.
```

### Notes

- **pad / pocket are the highest-priority items** — they alone cover the majority of
  FreeCAD Part Design workflows that users would want to do in code.
- `fillet_wire` is a prerequisite for rounded pockets (slot with rounded ends, etc.)
  and must land in the same sprint as pad/pocket.
- The face-local transform logic (`gp_Ax3` → `gp_Trsf`) is shared by pad, pocket, and
  datum_plane; implement it once as a C++ helper and reuse.
- SVG export via `HLRBRep_Algo` is the single largest new subsystem; it should be a
  separate sub-task with its own bridge class (similar to `ThruSectionsBuilder`).
- Thread is a compound feature (helix + profile sweep + cut) that can be implemented
  entirely in the Ruby DSL once helix is available — no new C++ needed for the thread
  feature itself.
- DXF v1 can be a hand-rolled ASCII writer (just LINE and ARC entities from sliced edges);
  no need for a full DXF library.

---

## Architecture Notes

See `CLAUDE.md` and `doc/development.md` for the full architecture and
development guide.

- **Memory:** each `Shape` is a `Box<occt::Shape>` raw pointer in mRuby
  `RData void*`; the `dfree` GC callback drops it. No SlotMap, no reference
  counting.
- **Preview:** OCCT tessellation → GLB → `axum` HTTP → Three.js browser viewer
  → WebSocket live reload. Web-based preview is the long-term approach; a
  native egui/wgpu viewer is not planned.
