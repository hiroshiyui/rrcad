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

## ✓ Phase 7 — Improve OCCT Coverage & Compatibility

Asymmetric chamfer (`.chamfer(d1, d2)`), 2-D profile offset (`.offset_2d`), grid
pattern (`grid_pattern`), and multi-shape `fuse_all`/`cut_all`.  Shape introspection:
`.shape_type`, `.closed?`, `.manifold?`, `.centroid`, `.validate`
(`BRepCheck_Analyzer`).  Surface modeling: `ruled_surface` (`BRepFill::Shell`),
`fill_surface` (`BRepFill_Filling`), `.slice` by axis-aligned plane
(`BRepAlgoAPI_Section`).  IGES import/export was deprioritised (STEP covers the same
workflows); SVG/DXF 2-D drawing landed in Phase 8 Tier 4 instead.
See `tests/phase7_tier1.rs` (12), `tests/phase7_tier2.rs` (12), `tests/phase7_tier3.rs` (10).

---

## ✓ Phase 8 — Part Design, Manufacturing & Advanced Composition

**Part Design (Tier 1):** `.pad(face, height:) { sketch }` and `.pocket(face, depth:) { sketch }`
via face-local `gp_Ax3` transform + `BRepPrimAPI_MakePrism` + fuse/cut.  `.fillet_wire(r)`
rounds 2-D sketch corners before extrude (`BRepFilletAPI_MakeFillet2d`).  `datum_plane`
constructs reusable reference planes from origin/normal/x-dir.
See `tests/phase8_tier1.rs` (11 tests).

**Manufacturing (Tier 2):** Draft-angle extrude (`BRepOffsetAPI_DraftAngle`);
`helix(radius:, pitch:, height:)` Wire path (BSpline at 16 samples/turn); `thread` and
`cbore`/`csink` as pure Ruby DSL macros built on helix + sweep + cut.
See `tests/phase8_tier2.rs` (13 tests).

**Inspection (Tier 3):** `.distance_to` (`BRepExtrema_DistShapeShape`), `.inertia` tensor
(`BRepGProp::VolumeProperties` → `MatrixOfInertia`), `.min_thickness` via inward
ray-casting (`IntCurvesFace_ShapeIntersector`).
See `tests/phase8_tier3.rs` (10 tests).

**2-D drawing (Tier 4):** `.export("part.svg")` / `.export("part.dxf")` via
`HLRBRep_PolyAlgo` hidden-line removal.  Three view directions: `:top` (default),
`:front`, `:side`.  SVG outputs `<polyline>` with Y-down coordinates; DXF outputs
`LINE` entities (R12 ASCII, Y-up).
See `tests/phase8_tier4.rs` (11 tests).

**Advanced composition (Tier 5):** `fragment([a,b,c])` via `BRepAlgoAPI_BuilderAlgo`;
`.convex_hull` via incremental 3-D QuickHull + sewing; `path_pattern(shape, path, n)`
via `GCPnts_UniformAbscissa` arc-length sampling; guided `.sweep(path, guide: wire)`
via `BRepOffsetAPI_MakePipeShell::SetMode`.
See `tests/phase8_tier5.rs` (11 tests).

---

## Phase 9 — Model Context Protocol (MCP) Server ✓ COMPLETE

**Implemented in** `src/mcp/mod.rs`. Start with `cargo run -- --mcp`.

Tools: `cad_eval` (shape properties JSON), `cad_export` (file to `/tmp/rrcad_mcp/`),
`cad_preview` (Three.js live URL), `cad_validate` (BRepCheck result).
Resources: `rrcad://api` (`doc/api.md`) and `rrcad://examples` (`samples/*.rb`).

Security: 8 mitigations — restricted mRuby gembox (`mcp_safe.gembox`), runtime prelude
strips `system`/`exec`/`fork`/etc., 30 s `tokio::time::timeout`, 2 GB `setrlimit`,
export paths confined to `/tmp/rrcad_mcp/`, fresh VM per call, 64 KB input cap,
`MRUBY_EVAL_LOCK` mutex serialises all mRuby/OCCT work across the tokio thread pool
(prevents SIGSEGV from concurrent VMs when a timed-out call lingers on a pool thread).
TOCTOU port race in `cad_preview` eliminated by keeping the `tokio::net::TcpListener`
alive and passing it directly to `serve_with_listener()`.
Test coverage: 10 unit tests in `src/mcp/mod.rs`, 13 integration tests in
`tests/mcp_tools.rs`, 10 stress/concurrency tests in `tests/mcp_stress.rs`.

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
