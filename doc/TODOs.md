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

## ✓ Phase 6 — Variable-Section Sweep

`sweep_sections(path, [profile, ...])` DSL function backed by
`BRepOffsetAPI_MakePipeShell`.  Each origin-centred profile is automatically
translated to the corresponding spine point (evenly-distributed along the
spline parameter) and swept with `WithCorrection=true` so cross-sections stay
perpendicular to the spine tangent.  Falls back to `BRepOffsetAPI_ThruSections`
when `MakeSolid()` fails on highly-curved spines (e.g., the teapot handle
C-arc).  See `tests/teapot_dsl.rs` (`sweep_sections_*` tests).

---

## ✓ Utah Teapot Sample

`samples/07_teapot.rb` — rebuilt from the Newell triangle mesh
(`doc/images/utah_teapot.obj`, sourced from https://graphics.cs.utah.edu/teapot/, ×3.0 scale).  Body via `loft` (8 OBJ-derived
cross-sections, widest r=6.00 at Z=2.40); handle via `sweep_sections` along
a 7-point C-arc with flared flanges (r=1.40) at the body-wall attachment
points; spout via tapered `loft`; lid via `loft` dome + `sphere` knob.
Body height = 6.60 units (rim at Z=6.60).
Validated by `tests/teapot_sample.rs` (5 tests).

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
Week 5+: Tier 5 (Phase 8)
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

## Architecture Notes

See `CLAUDE.md` and `doc/development.md` for the full architecture and
development guide.

- **Memory:** each `Shape` is a `Box<occt::Shape>` raw pointer in mRuby
  `RData void*`; the `dfree` GC callback drops it. No SlotMap, no reference
  counting.
- **Preview:** OCCT tessellation → GLB → `axum` HTTP → Three.js browser viewer
  → WebSocket live reload. Web-based preview is the long-term approach; a
  native egui/wgpu viewer is not planned.
