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

## Phase 10 — Usability and Robust Parametric CAD

These are forward-looking CAD enhancements intended to move `rrcad` from a
powerful scripted geometry engine toward a more complete, inspectable CAD
workflow.

**Constraint-based sketching ✓ MVP COMPLETE:** `sketch do ... end` builds
closed polygon profiles from points and lines, with constraint propagation
for `fixed`, `horizontal`, `vertical`, `coincident`, `dimension`,
`equal_length`, `parallel`, `perpendicular`, `symmetric`, `mirror_x`,
`mirror_y`, and `tangent` (line-to-circle, with `side:` keyword for
axis-aligned lines and verification mode for fully-resolved geometry).
Construction geometry: named construction points via `point(:name, x, y)`,
`construction_point(:name, x, y)`, `ref(:name)`, and `self[:name]`;
`midpoint` construction points; non-profile `construction_line(a, b)`
references; and `polar_point([:name,] center, radius, angle_deg)` for bolt
circles and other polar layouts. The MVP works with `.extrude`, `.pad`, and
`.pocket`, and supports exact circle profiles via `circle_at(center,
radius)`, translated arc wires via `arc_at(center, radius, start_deg,
end_deg)`, constrained `rectangle(origin, width, height)` /
`centered_rectangle(center, width, height)` helpers, and axis-aligned
`slot_between(a, b, radius)` profiles. Solver diagnostics name the
involved points and report actual vs expected values for conflicting
constraints, and the "did not converge" failure lists every unresolved
point with its missing coordinates.

**Feature history / parametric model tree:** Represent modeling operations as a
regeneratable feature graph instead of only immutable shape results. This would
enable dependency tracking, clearer regeneration failures, better debug output,
and eventually GUI editing.

**Named faces, edges, and datums:** Add persistent names for selected topology
and reference geometry, so scripts can target `:mounting_face` or
`:front_boss_axis` instead of relying only on broad selectors such as `:top`,
`:vertical`, or `">Z"`.

**Better diagnostics ✓ COMPLETE (initial scope):** Errors from boolean
operations (`fuse`/`cut`/`common`), fillets and chamfers (including
selector variants and variable-radius/asymmetric forms), import/export
(`import_step`/`import_stl`, all seven export formats),
`extrude`/`extrude_ex`/`extrude_draft`/`revolve`,
`shell`/`offset`/`offset_2d`/`simplify`,
`sweep`/`sweep_guide`/`sweep_sections`, `loft`, and Part Design
operations (`pad`, `pocket`) carry call-site context — the operation
name, its numeric parameters and selector, the operand shape kind via
`summarize`, and any file path/view name. The most common failures
(`fillet`/`chamfer` radius too large, `extrude` on a solid, `shell`
thickness too thick, `pad`/`pocket` planar-face requirement,
`sweep`/`sweep_guide` profile-and-path types, `import_step`/`import_stl`
missing/unreadable files) also carry an actionable one-line hint
prefixed with `hint:`. Sample message:
`fillet(r=10) on solid failed: ...\n  hint: radius likely exceeds the
smallest adjacent face/edge; try a smaller value or use fillet_sel with
an edge selector`.

**Assembly constraints beyond `mate` ◐ STARTED:** Added transform-based
helpers that cover most of the planned constraint set without introducing a
full constraint solver: `Assembly#distance_mate` (named air-gap variant of
`mate`); `Assembly#axis_align(from: [p1, p2], to: [q1, q2])` (coaxial /
concentric / axis-alignment by point-pair axes); `Assembly#angle_mate(...,
angle:, pivot:, axis_dir:)` (mate + extra rotation about a chosen axis for
rotation-lock); and `Shape#rotate_about(point, axis_dir, angle_deg)` as a
general primitive. Coincident-plane is the existing `mate(..., offset: 0)`.

**Units system ✓ COMPLETE:** Initial `Numeric` helpers are implemented in the
Ruby prelude: `1.6.mm`, `2.inch`, `1.cm`, `0.5.m`, `15.deg`, and
`Math::PI.rad`.

**Tolerance and manufacturing profiles ✓ COMPLETE (initial scope):**
Hole tools: `clearance_hole`, `tap_drill`, `heat_set_insert`,
`socket_head_cbore`, `flat_head_csink`. Bearing bores: `bearing_bore` for
`:b608`/`:b623`/`:b624`/`:b625`/`:b626`/`:b688`/`:b695`/`:b6000`/`:b6001`
with `:press`/`:slip` fit. Shaft fits: `shaft(diameter, length:, fit:)`
with `:nominal`/`:press`/`:slip`/`:running`. Standard fasteners:
`screw(size, length:, style:)` in `:socket`/`:button`/`:flat` styles. All
size-keyed helpers accept both metric (`:m2`–`:m5`, ISO 4762 / 7380 /
10642) and imperial UNC/UNF (`:"4-40"`, `:"6-32"`, `:"8-32"`, `:"10-32"`,
`:"10-24"`, `:"1/4-20"`, `:"5/16-18"`, `:"3/8-16"`, ASME B18 / ANSI B18.3)
fastener names.

**Preview inspection UX ◐ STARTED:** The browser preview now writes and serves
a `metadata.json` sidecar for each `preview(shape)` update and shows a compact
model properties panel with shape type, validation status, bounding-box size,
volume, and surface area. The viewer menu also has section plane controls
for X/Y/Z clipping with an offset slider, plus a measurement mode that picks
two model points and reports their 3-D distance. Remaining work: face/edge
hover IDs, click-to-print selectors, exploded assembly view, and debug overlays
for failed operations.

**2-D drawing improvements ◐ STARTED:** SVG and DXF output now accept explicit
`scale:` selection for projected drawing geometry. Both support
`center_marks: true` cylinder centres; SVG now also supports `hidden: true`
dashed hidden-line output and `dimensions: true` width/height labels, while DXF
writes hidden edges on a `HIDDEN` layer and also supports `dimensions: true`.
SVG and DXF also now have `view: :sheet` for a 3-view sheet layout and
`title_block: true` metadata blocks, with axis-aware sheet dimensions.
Remaining work: standards-style callouts and tolerance annotations.

**CAM / 3-D printing checks ◐ STARTED:** Additive helpers landed in the
Ruby prelude: `mass_estimate(part, density:)` computes a rough mass in
grams from `part.volume × density / 1000` (PLA default 1.24 g/cm³);
`print_volume_check(part, x:, y:, z:)` reports `fits` plus per-axis
overflow against a rectangular build volume by comparing `bounding_box`
extents; and `overhang_faces(part, max_angle_deg:)` returns faces whose
outward normal tips downward more than the threshold (assumes the part is
+Z-up); `draft_faces(part, axis:, min_draft_deg:)` lists faces with
insufficient mould-draft along a chosen pull axis (`asin(|n·axis|) <
min_draft_deg`, naturally excluding top/bottom faces). The underlying
primitive `Shape#normal` is exposed by a new OCCT bridge call
(`shape_face_normal`) using `BRepLProp_SLProps` at the face's
parameter-space midpoint, with the normal flipped for `TopAbs_REVERSED`
faces. Minimum wall thickness is already available via
`Shape#min_thickness`. Hole orientation is now supported through a new
`shape_cylinder_axis` bridge call (extracting `gp_Cylinder::Axis()` and
radius for `GeomAbs_Cylinder` faces); the `Shape#cylinder_axis` accessor
returns `{origin:, axis:, radius:}`, and the `hole_axes(part, orientation:
:vertical | :horizontal)` DSL helper enumerates and filters cylindrical
faces by axis direction.

---

## Future Works

Open items deferred from completed or partially-complete tracks above.
These are not blocking and can be picked up when the use cases become
important.

- **Sketch — DOF-counting analyser** for redundant-constraint detection
  before solving, and a richer 2-D drawing/preview layer to visualise
  the sketch graph. (Extends *Constraint-based sketching*.)
- **Diagnostics — failure debug exports** of intermediate geometry, and
  richer hints driven by inspecting operand bounding-box extents.
  (Extends *Better diagnostics*.)
- **Assembly — full constraint solver** that resolves under-constrained
  poses iteratively, in addition to the transform-based helpers already
  shipped. (Extends *Assembly constraints beyond `mate`*.)
- **Units — dimensional analysis** with richer unit-aware values,
  beyond the current `Numeric` helpers. (Extends *Units system*.)
- **Hardware helpers — washer/nut bodies.** Imperial sizes (#4-40 through
  3/8-16) are now supported across `clearance_hole`, `tap_drill`,
  `heat_set_insert`, `socket_head_cbore`, `flat_head_csink`, and `screw`.
  (Extends *Tolerance and manufacturing profiles*.)
- **CAM — unsupported islands** via slice-based connectivity analysis.
  (Extends *CAM / 3-D printing checks*.)

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
