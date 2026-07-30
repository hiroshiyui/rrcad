# rrcad — Roadmap

A Ruby DSL-driven 3D CAD language. Rust as the glue layer, mRuby as the
scripting engine, OCCT as the geometry kernel.

Cross-cutting work comes first; the phase history follows, oldest to newest.

## Status at a glance

| Phase | Focus | Status |
|-------|-------|--------|
| [0](#phase-0--occt-minimal-rust-bindings) | OCCT bindings via `cxx` | ✓ Complete |
| [1](#phase-1--mruby-embedded-in-rust) | mRuby embedded, native `Shape` | ✓ Complete |
| [2](#phase-2--dsl-enrichment) | Transforms, modifiers, sketch ops | ✓ Complete |
| [3](#phase-3--splines-sweep-live-preview) | Splines, sweep, live preview | ✓ Complete |
| [4](#phase-4--occt-coverage-openscad--cadquery-parity) | OpenSCAD / CadQuery parity | ✓ Complete |
| [5](#phase-5--parametric-design--assembly) | Parameters, design tables, mating | ✓ Complete |
| [6](#phase-6--variable-section-sweep--teapot-rebuild) | Variable-section sweep, Bézier patches | ✓ Complete |
| [7](#phase-7--wider-occt-coverage) | Introspection, surface modelling | ✓ Complete |
| [8](#phase-8--part-design-manufacturing--composition) | Part Design, manufacturing, 2-D drawings | ✓ Complete |
| [9](#phase-9--model-context-protocol-mcp-server) | MCP server for AI agents | ✓ Complete |
| [10](#phase-10--usability-and-robust-parametric-cad) | Sketch constraints, feature tree, GD&T | ✓ Complete |
| [11](#phase-11--professional-cad-depth) | Professional CAD depth | ✓ Complete |
| [12](#phase-12--quadcopter-readiness) | Quadcopter readiness | ○ Planned |

Phases 0–11 are complete. Phase 12 collects the gaps found by scoping a real
quadcopter drone design against the current feature set. Work considered and
set aside is recorded under
[Deferred and not planned](#deferred-and-not-planned), with the reasoning, so
a decision can be revisited rather than re-litigated.

---

## Project Improvements

Cross-cutting work that does not belong to a phase.

### Open

None. See [Deferred and not planned](#deferred-and-not-planned) for work that
was considered and set aside.

### Done

- [x] Feature-tree browsing in the browser preview. `metadata.json` now carries
      the parsed `feature_graph` — one node per operation, in dependency order —
      and the viewer renders it as a **Features** panel under the model
      properties. Because a feature graph is a DAG rather than a list, the panel
      lays it out the way a CAD tree does: the chain leading to the previewed
      shape flush left, branches feeding a boolean indented beneath the step
      that consumes them, and the merged-in node named on the row. Clicking a
      row shows its full recorded history entry. Read-only by design; editing
      from the viewer is [cancelled](#deferred-and-not-planned).
      Tests: 4 in `src/preview/mod.rs`, including the tree layout exercised
      directly under node.

- [x] The drawing export travels as one `DrawingSpec` shared struct instead of
      a flat list of some thirty scalars repeated through `glue.c` →
      `native_io.rs` → `drawing_ops.rs` → `mod.rs` → `bridge.h` → `bridge.cpp`.
      cxx generates the struct for both languages from a single declaration, so
      the two sides of the C++ boundary cannot drift apart, and a new drawing
      option is now one field rather than six edits. The SVG and DXF writers
      also collapsed into one `export_drawing(spec, format)` — they always took
      an identical request. The C ABI hop stays flat on purpose: a struct there
      would mean a hand-matched memory layout in two languages, which fails
      silently, where a hand-matched argument list at least fails at the call.
      Verified by byte-comparing every drawing option's output against the
      previous build — twelve files, identical.
- [x] Annotation text can no longer corrupt the file it is written into. An
      `&` or `<` in a datum label or feature-control frame produced an SVG no
      parser would open, and a newline in either desynchronised the DXF
      group-code stream, since DXF values are one per line and the next line is
      read as a code. Both failed silently at export. Every SVG text node now
      goes through `svg_text()` and every DXF text value through `dxf_text()`,
      including the ones built from numbers — deciding per call site which
      strings can reach a user is the judgement that produced the bug.
      Tests: 7 in `tests/drawing_text_safety.rs`.
- [x] `Assembly#export` accepts and forwards drawing options. It took only a
      path, so `view:`, `section:`, and every other option was silently dropped
      and an assembly could not produce a sheet. This also unblocks the
      BOM-on-sheet item in Track C.
- [x] `puts` / `print` / `p` / `pp` in scripts, built on a native output
      primitive rather than reintroducing the IO gems. Removed in MCP mode,
      where stdout carries the JSON-RPC responses.
- [x] CSV design-table parsing fails fast on row-width mismatches instead of
      silently truncating with `zip()`.
- [x] Explicit tests and documentation for the strict export-path confinement
      (`tests/export_confinement.rs`, `doc/user-guide/10-import-export.md`).
- [x] Consolidated the repetitive Ruby-to-native FFI wrappers into a macro
      layer (`src/ruby/native.rs`) and static helpers (`src/ruby/glue.c`).
- [x] Project-local `rrcad.toml` with `preview_port` and default `[params]`,
      discovered from the script directory or CWD.
- [x] Preview auto-selects a free port to avoid local conflicts.
- [x] CI coverage for MCP security invariants, including `create_mcp_vm()` and
      the file/process constant removal checks.
- [x] Maintainer note on the OCCT bridge and mRuby lifetime invariants.
- [x] Claude skill definitions in `.claude/skills`, with the repo-local
      `.codex/skills` mirror kept in sync when Codex-callable copies are needed.

---

## Completed phases

### Phase 0 — OCCT Minimal Rust Bindings

`cxx` bridge to OCCT 7.9: primitives (`box`, `cylinder`, `sphere`), booleans
(`fuse`, `cut`, `common`), fillets, chamfers, transforms, and STEP/STL/glTF
export. See `src/occt/`.

### Phase 1 — mRuby Embedded in Rust

mRuby 3.4.0 vendored, with a C glue shim (`glue.c`) hiding `mrb_value` from
Rust and an `MrubyVm` RAII wrapper. Native `Shape` class backed by a
`Box<occt::Shape>` raw pointer in mRuby's `RData void*`. DSL prelude
auto-loaded via `include_str!`. REPL with readline and history.
Tests: `tests/e2e_dsl.rs`.

### Phase 2 — DSL Enrichment

- Transforms: `.translate`, `.rotate`, `.scale`, `.mirror`.
- Modifiers: `.fillet`, `.chamfer`. Sketch ops: `.extrude`, `.revolve`.
- 2-D faces: `rect`, `circle`. `solid do … end` block.
- `Assembly` with `place`; REPL tab-completion and `help`.

Tests: `tests/phase2_dsl.rs`.

### Phase 3 — Splines, Sweep, Live Preview

Spline profiles (`spline_2d`, `spline_3d`) and pipe sweep (`.sweep`) via
`GeomAPI_Interpolate` + `BRepOffsetAPI_MakePipe`. Sub-shape selectors `.faces`
and `.edges`.

Live preview through `rrcad --preview <script.rb>`: `axum` HTTP server, Three.js
viewer, and WebSocket live reload driven by `notify`. `preview(shape)` is a
no-op outside preview mode.

Tests: `tests/teapot_dsl.rs`, `tests/phase3_selectors.rs`.

### Phase 4 — OCCT Coverage (OpenSCAD / CadQuery parity)

- Primitives: `cone`, `torus`, `wedge`. 2-D profiles: `polygon`, `ellipse`, `arc`.
- 3-D ops: `loft`, `.shell`, `.offset`, `.extrude(twist/scale)`, non-uniform
  `.scale(sx, sy, sz)`.
- Selective fillet/chamfer by edge selector; vertex selector; direction-based
  face selectors (`">Z"`, `"<X"`, …).
- Patterns: `linear_pattern`, `polar_pattern`. OBJ export; STEP/STL import.
- Queries: `.bounding_box`, `.volume`, `.surface_area`.
- OCCT hardening: builder-style booleans with fuzzy tolerance, parallel
  tessellation, GLB TRS transform format, and a `BRepCheck_Analyzer` validity
  guard before export.

Tests: `tests/phase4_3d_ops.rs`, `tests/occt_layer.rs`.

### Phase 5 — Parametric Design & Assembly

- `param :name, default:, range:` with `--param key=value` CLI override.
- Design-table batch export via `--design-table table.csv`.
- Per-shape sRGB colour (`.color(r, g, b)`) written into GLB/glTF/OBJ through
  `XCAFDoc_ColorTool`.
- Assembly mating (`Shape#mate`, `Assembly#mate`) from OCCT planar-face
  geometry: normal alignment plus centroid translation, with an optional
  gap/interference offset.
- Spline tangent constraints (`tangents:` on `spline_2d` / `spline_3d`).
- Feature removal via `.simplify(min_feature_size)` (`BRepAlgoAPI_Defeaturing`).

Tests: `tests/phase5_params.rs`, `tests/e2e_dsl.rs`.

### Phase 6 — Variable-Section Sweep & Teapot Rebuild

`sweep_sections(path, [profile, …])` backed by `BRepOffsetAPI_MakePipeShell`.
Each origin-centred profile is translated to its spine point (evenly distributed
along the spline parameter) and swept with `WithCorrection=true`, keeping
cross-sections perpendicular to the spine tangent. Falls back to
`BRepOffsetAPI_ThruSections` when `MakeSolid()` fails on highly curved spines,
such as the teapot handle's C-arc.

`bezier_patch([pt0..pt15])` builds a bicubic Bézier face from a 4×4 row-major
control grid (`Geom_BezierSurface` + `BRepBuilderAPI_MakeFace`).
`sew([faces], tolerance:)` assembles faces into a closed shell or solid
(`BRepBuilderAPI_Sewing` + `BRepBuilderAPI_MakeSolid`).

Tests: `tests/teapot_dsl.rs` (`sweep_sections_*`).

#### Utah Teapot sample

`samples/07_teapot.rb`, rebuilt from the original Newell Bézier data
([source](https://users.cs.utah.edu/~dejohnso/models/teapot.html), ×3.0 scale).
All 28 bicubic patches from the Newell / Blinn dataset, with a Y-up → Z-up
transform (`pt(x, y_s, z_s)` → `[x, z_s, y_s]`). Patches are sewn at tolerance
1e-3 into a continuous surface; at `scale(3.0)` the rim sits at Z≈6.75 and the
lid knob at Z=9.0. Open at the base, consistent with the original definition.

Tests: `tests/teapot_sample.rs` (9, including `bezier_patch` and `sew` units).

### Phase 7 — Wider OCCT Coverage

- Asymmetric chamfer (`.chamfer(d1, d2)`), 2-D profile offset (`.offset_2d`),
  `grid_pattern`, and multi-shape `fuse_all` / `cut_all`.
- Introspection: `.shape_type`, `.closed?`, `.manifold?`, `.centroid`,
  `.validate` (`BRepCheck_Analyzer`).
- Surface modelling: `ruled_surface` (`BRepFill::Shell`), `fill_surface`
  (`BRepFill_Filling`), `.slice` by axis-aligned plane (`BRepAlgoAPI_Section`).

IGES import/export was deprioritised — STEP covers the same workflows. SVG/DXF
2-D drawing landed in Phase 8 Tier 4 instead.

Tests: `tests/phase7_tier1.rs` (12), `tests/phase7_tier2.rs` (12),
`tests/phase7_tier3.rs` (10).

### Phase 8 — Part Design, Manufacturing & Composition

**Tier 1 — Part Design.** `.pad(face, height:) { sketch }` and
`.pocket(face, depth:) { sketch }` via a face-local `gp_Ax3` transform plus
`BRepPrimAPI_MakePrism` and fuse/cut. `.fillet_wire(r)` rounds 2-D sketch
corners before extrude (`BRepFilletAPI_MakeFillet2d`). `datum_plane` builds
reusable reference planes from origin/normal/x-dir.
Tests: `tests/phase8_tier1.rs` (11).

**Tier 2 — Manufacturing.** Draft-angle extrude (`BRepOffsetAPI_DraftAngle`);
`helix(radius:, pitch:, height:)` wire paths (BSpline at 16 samples/turn);
`thread`, `cbore`, and `csink` as Ruby DSL macros over helix + sweep + cut.
Tests: `tests/phase8_tier2.rs` (13).

**Tier 3 — Inspection.** `.distance_to` (`BRepExtrema_DistShapeShape`),
`.inertia` tensor (`BRepGProp::VolumeProperties` → `MatrixOfInertia`), and
`.min_thickness` via inward ray-casting (`IntCurvesFace_ShapeIntersector`).
Tests: `tests/phase8_tier3.rs` (10).

**Tier 4 — 2-D drawing.** `.export("part.svg")` / `.export("part.dxf")` through
`HLRBRep_PolyAlgo` hidden-line removal, in `:top` (default), `:front`, and
`:side` views. SVG emits `<polyline>` with Y-down coordinates; DXF emits `LINE`
entities (R12 ASCII, Y-up).
Tests: `tests/phase8_tier4.rs` (11).

**Tier 5 — Advanced composition.** `fragment([a, b, c])`
(`BRepAlgoAPI_BuilderAlgo`); `.convex_hull` via incremental 3-D QuickHull plus
sewing; `path_pattern(shape, path, n)` using `GCPnts_UniformAbscissa`
arc-length sampling; guided `.sweep(path, guide: wire)` via
`BRepOffsetAPI_MakePipeShell::SetMode`.
Tests: `tests/phase8_tier5.rs` (11).

### Phase 9 — Model Context Protocol (MCP) Server

Public wiring in `src/mcp/mod.rs`, implementation split across helpers under
`src/mcp/`. Start with `cargo run -- --mcp`.

- **Tools:** `cad_eval` (shape properties JSON), `cad_export` (file into
  `/tmp/rrcad_mcp/`), `cad_preview` (Three.js live URL), `cad_validate`
  (`BRepCheck` result).
- **Resources:** `rrcad://api` (`doc/api.md`) and `rrcad://examples`
  (`samples/*.rb`).

**Security** — layered mitigations, in rough order of importance:

1. Restricted mRuby gembox (`mcp_safe.gembox`) — no IO, socket, dir, eval, or
   metaprogramming gems are linked, so the dangerous methods mostly do not
   exist. This, not the runtime prelude, is the real sandbox.
2. Runtime prelude strips `system`, `exec`, `fork`, the printing methods, and
   friends as defence in depth.
3. One-shot worker child process per tool call, killed by the parent when its
   `tokio::time::timeout` fires at 30 s; 2 GB `setrlimit(RLIMIT_AS)`.
4. Export paths confined to `/tmp/rrcad_mcp/` (mode 0700) by `safe_path()`.
5. Fresh VM per call; 64 KB input cap; null-byte filtering.
6. `MRUBY_EVAL_LOCK` serialises mRuby/OCCT work for in-process tests — mRuby is
   not thread-safe and concurrent VMs SIGSEGV.

The `cad_preview` TOCTOU port race was eliminated by keeping the bound
`tokio::net::TcpListener` alive and handing it straight to
`serve_with_listener()`.

Tests: unit tests in `src/mcp/`, `tests/mcp_tools.rs` (15),
`tests/mcp_stress.rs` (10 stress/concurrency).

### Phase 10 — Usability and Robust Parametric CAD

Moved rrcad from a powerful scripted geometry engine toward a complete,
inspectable CAD workflow.

**Constraint-based sketching** (MVP). `sketch do … end` builds closed polygon
profiles from points and lines, with constraint propagation for `fixed`,
`horizontal`, `vertical`, `coincident`, `dimension`, `equal_length`,
`parallel`, `perpendicular`, `symmetric`, `mirror_x`, `mirror_y`, and `tangent`
(line-to-circle, with a `side:` keyword for axis-aligned lines).

- Construction geometry: named points via `point(:name, x, y)`,
  `construction_point`, `ref(:name)`, `self[:name]`; `midpoint` points;
  non-profile `construction_line(a, b)`; and `polar_point` for bolt circles.
- Exact profiles: `circle_at`, `arc_at`, `rectangle`, `centered_rectangle`,
  and axis-aligned `slot_between`. Works with `.extrude`, `.pad`, `.pocket`.
- Diagnostics name the involved points and report actual vs expected values;
  non-convergence lists every unresolved point and its missing coordinates.
  `sketch(diagnostics: true)` attaches a redundancy report; `strict: true`
  raises when redundant constraints are present.

**Feature history / parametric model tree.** Shapes carry a readable modelling
history (`shape.history`) and a regeneratable feature graph with stable node IDs
and dependency edges (`shape.feature_graph`). `shape.rebuild` replays the tree
from the recorded parents. The browser preview renders the tree as a read-only
**Features** panel — see [Project Improvements](#project-improvements); editing
from the viewer is [cancelled](#deferred-and-not-planned).

**Named faces, edges, and datums.** `name_face(:mounting_face, :top)`,
`name_edge(:boss_edges, :vertical)`, and `datum(:fixture_plane, datum_plane(…))`
can be resolved later with `faces(:mounting_face)`, `edges(:boss_edges)`, or
`ref(:fixture_plane)`, instead of relying only on broad selectors.

**Better diagnostics.** Errors carry call-site context — the operation name, its
numeric parameters and selector, the operand shape kind via `summarize`, and any
file path or view name. Covered operations: booleans (`fuse`/`cut`/`common`),
fillets and chamfers (including selector, variable-radius, and asymmetric
forms), `import_step`/`import_stl` and all eight export formats,
`extrude`/`extrude_ex`/`extrude_draft`/`revolve`,
`shell`/`offset`/`offset_2d`/`simplify`,
`sweep`/`sweep_guide`/`sweep_sections`, `loft`, and Part Design `pad`/`pocket`.
The most common failures also carry a `hint:` line, for example:

```
fillet(r=10) on solid failed: …
  hint: radius likely exceeds the smallest adjacent face/edge; try a smaller
  value or use fillet_sel with an edge selector
```

With `RRCAD_DEBUG_EXPORTS=1`, those failure paths also write STEP debug
artifacts into `RRCAD_DEBUG_EXPORTS_DIR` (or the system temp directory) so the
failing geometry can be inspected directly.

**Assembly constraints beyond `mate`.** A declarative rigid-body solver sits
alongside the eager helpers: `assembly("rig") do |a| … end` with
`a.ground :base, base` and `a.part :post, post do |p| mate from: :bottom,
to: face(:base, :top) end` resolves chained parts lazily at `to_shape` /
`export` time, detecting under-constrained parts and conflicting mate chains.
The older `place`, `mate`, `distance_mate`, `axis_align`, and `angle_mate`
helpers remain as compatibility transforms over `Shape#rotate_about`.

**Units.** `Numeric` helpers in the Ruby prelude: `1.6.mm`, `2.inch`, `1.cm`,
`0.5.m`, `15.deg`, `Math::PI.rad`.

**Tolerance and manufacturing profiles.**

- Hole tools: `clearance_hole`, `tap_drill`, `heat_set_insert`,
  `socket_head_cbore`, `flat_head_csink`.
- Bearing bores: `bearing_bore` for `:b608`, `:b623`, `:b624`, `:b625`,
  `:b626`, `:b688`, `:b695`, `:b6000`, `:b6001`, with `:press` / `:slip` fit.
- Shaft fits: `shaft(diameter, length:, fit:)` with `:nominal`, `:press`,
  `:slip`, `:running`.
- Fasteners: `screw(size, length:, style:)` in `:socket`, `:button`, `:flat`.
- Size-keyed helpers accept both metric (`:m2`–`:m5`; ISO 4762 / 7380 / 10642)
  and imperial UNC/UNF names: `:"4-40"`, `:"6-32"`, `:"8-32"`, `:"10-32"`,
  `:"10-24"`, `:"1/4-20"`, `:"5/16-18"`, `:"3/8-16"` (ASME B18 / ANSI B18.3).

**Preview inspection UX.** The viewer serves a `metadata.json` sidecar per
`preview(shape)` update and shows a properties panel (shape type, validity,
bounding box, volume, surface area). The menu adds X/Y/Z section clipping with
an offset slider and a measurement mode reporting 3-D distance between two
picked points. Hovering shows face/edge IDs, clicking prints the best matching
selector, an explode toggle separates top-level parts, and failed updates
surface their error in the inspector.

**2-D drawing improvements.** Explicit `scale:`; `center_marks:` cylinder
centres; `hidden:` dashed hidden lines (SVG) and a `HIDDEN` layer (DXF);
`dimensions:` labels; `view: :sheet` 3-view layouts with `title_block:`
metadata and axis-aware sheet sizing; `tolerance:` in both symmetric `±` and
asymmetric `+…/-…` forms; `callouts:` diameter labels; and framed `datum:` /
`feature_control:` annotations attachable to real faces, with datum lists for
richer geometric tolerancing. `Shape#gdt(standard:)` stores a canonical spec on
the shape, with ASME/ISO ordering resolved at export time.

**CAM / 3-D printing checks.** `mass_estimate(part, density:)` (PLA default
1.24 g/cm³), `print_volume_check(part, x:, y:, z:)`,
`overhang_faces(part, max_angle_deg:)`, `draft_faces(part, axis:,
min_draft_deg:)`, and `hole_axes(part, orientation:, tolerance_deg:)`, over the
`Shape#normal` and `Shape#cylinder_axis` primitives.
`unsupported_islands(part, …)` scans slices layer-by-layer for disconnected
footprints that do not overlap the previous layer.

### Phase 11 — Professional CAD Depth

Phases 0–10 brought rrcad to parity with scripted CAD tools. Phase 11 targets
the gap between "scripted geometry engine" and "tool a mechanical engineer
would choose": the operations people reach for constantly, and the deliverables
a design is expected to produce.

**Complete.** All four tracks, plus multi-file projects, flat cut-file export,
and the opportunistic list that was folded in alongside them.

#### Track A — Sketcher depth

The constraint solver and profile types exist, but the sketcher was missing the
operations used most in practice. **Complete.**

- **Corner `fillet` / `chamfer`** inside `sketch do … end`. Shapes the 2-D
  profile itself, so a rounded outline survives every later pad, pocket, or
  boolean — unlike `Shape#fillet`, which rounds an existing solid.
  Tests: `tests/phase11_sketch_corners.rs` (11).
- **`trim` / `extend`** on sketch segments. One endpoint slides along the
  segment while the other anchors it, either `by:` a distance or `to:` the
  intersection with another segment's infinite line. Edits apply in
  declaration order after the solver runs, in a side table rather than by
  mutating solved points, so corner modifiers see the moved corner and
  `to_profile` stays idempotent.
  Tests: `tests/phase11_sketch_edits.rs` (21).
- **Profile `offset`** — `offset` inside `sketch do … end`, plus a fixed
  `Shape#offset_2d`. The offsetter returns bare wires, so an offset profile
  used to extrude into an open shell instead of a solid; the wires are now
  rebuilt into a planar face, largest as the outer boundary and the rest as
  holes. Profiles with holes offset in both directions, with a per-wire
  fallback (and sign flip) for the faces OCCT cannot offset whole, such as
  an all-circular annulus.
  Tests: `tests/phase11_profile_offset.rs` (18).
- **Sketch-level linear and polar patterns** — `linear_pattern`,
  `polar_pattern`, and `grid_pattern` inside `sketch do … end` replicate
  the finished profile into one compound profile, so a single `extrude`,
  `pad`, or `pocket` covers every copy. Polar patterns turn about a sketch
  point, an `[x, y]` pair, or the origin, over a partial or full sweep.
  The builder methods shadow the top-level functions of the same name and
  delegate to them when passed a Shape, so both forms work in a block.
  Tests: `tests/phase11_sketch_patterns.rs` (23).
- **Spline segments in sketch profiles** — `spline a, b, through: [...]`
  draws a curved segment through interior points that may themselves be
  solver-driven sketch points. A new `make_profile_2d` bridge function
  builds the outline as a chain of edges, so the curve stays an
  interpolated BSpline instead of being flattened into a polyline. Corner
  modifiers and `trim` / `extend` measure along straight runs, so they
  reject spline segments and say why.
  Tests: `tests/phase11_sketch_splines.rs` (20).

#### Track B — Assembly intelligence

The assembly layer had a real constraint solver but produced none of the
deliverables an assembly exists for. All three build on primitives that already
existed (`common`, `distance_to`, `mass_estimate`). **Complete.**

Placement methods now take optional `name:` / `component:` / `material:` /
`density:` metadata, and `Assembly#components` enumerates placed and solved
parts uniformly, which is what the three reports are built on.

- **Interference / clash detection** between parts, with clearance
  reporting — `Assembly#interferences(clearance:, ignore_contact:)` and
  `#clash?`. Pairwise `common` for overlap volume, `distance_to` for gaps.
  A flush mate is not a clash, and deliberate contact is excluded from the
  clearance check by default.
- **Bill-of-materials generation** with quantity rollup — `Assembly#bom`
  rolls components up by their `component:` key; `#bom_text` renders an
  aligned table. A built-in material→density table means `material:`
  alone yields believable masses.
- **Assembly-level mass, volume, and centre-of-mass rollup** —
  `Assembly#mass_properties`, mass-weighted, with a per-part breakdown.

Follow-ons, driven by working through a quadcopter as a test case:

- **`mass:` override** for parts you buy rather than model. Deriving mass
  from `volume × density` is wrong for a bought part whose geometry is
  only an envelope — and on a real vehicle those parts dominate the total.
  The envelope stays, so it keeps clash-checking; only the mass comes from
  the datasheet. A stated mass on a zero-volume shape acts as a point
  mass, which covers wiring and adhesive without a separate concept.
- **Inertia rollup** — `mass_properties` returns the tensor about the
  centre of mass (or any `about:` point) by parallel-axis transfer.
  Validated against the tensor OCCT computes for the equivalent fused
  solid, which agrees to machine precision on all six components.
  Tests: 64 in `tests/phase11_assembly_reports.rs`.

#### Multi-file projects

- **`require_relative`** — a project of any size outgrows one file, and
  copy-pasted constants are how two parts drift out of sync. Resolves
  against the requiring file's directory (not the CWD), evaluates each
  file once so cycles terminate, and reports its load set so `--preview`
  watches the whole project. Disabled in MCP mode by two independent
  guards. Tests: 18 in `tests/multi_file_projects.rs`, 6 in
  `src/ruby/loader.rs`, 3 in `src/cli.rs`, 2 in `src/mcp/security.rs`.

#### Track C — Drawing completeness

SVG/DXF output already had hidden lines, GD&T frames, title blocks, and 3-view
sheets. **Complete.**

- **Section views** with standard 45° hatching, built on `BRepAlgoAPI_Cut`
  against a half-space (the same axis-aligned planes `Shape#slice`
  supports). Even-odd hatch clipping leaves interior holes unhatched. An
  omitted `offset:` cuts the part's mid-plane. SVG emits `hatch` and
  `section` groups; DXF uses a dedicated `HATCH` layer.
  Tests: 7 co-located in `src/occt/drawing_ops.rs` and `src/ruby/native_io.rs`.
- **Detail views** — `detail: { at:, radius:, scale:, label: }` clips a
  circular region out of the projection, magnifies it, and draws it beside
  the parent view inside a captioned border circle; the parent gains a
  marker circle. The region is stated in model units on the view's own
  drawing plane, and edges are cut analytically on the boundary rather than
  at the nearest tessellation vertex. SVG uses a `detail` group, DXF a
  `DETAIL` layer. Refused on `view: :sheet`, which has no single parent.
  Tests: 21 in `tests/detail_views.rs`.
- **Auto-dimensioning of principal features** — `ordinate: true` measures
  every axis-aligned cylindrical feature from a datum corner (the lower-left
  of the projected geometry), the form a plate full of holes actually gets
  on a drawing. Labels stay in model units regardless of the drawing scale,
  and features sharing a coordinate collapse to one ordinate. SVG uses an
  `ordinates` group, DXF an `ORDINATE` layer with right-aligned text.
  Tests: 16 in `tests/auto_dimensioning.rs`.
- **BOM tables with balloon callouts** — `asm.export(path, bom: true,
  balloons: true)` draws the bill of materials below the drawing and marks
  each component with a numbered balloon whose leader lands on it, keyed to
  the table's item numbers. Per-component data crosses the FFI as delimited
  records, since the row count is not known until the assembly is walked.
  Balloons ring the geometry ordered by each part's bearing, so leaders do
  not cross; on a three-view sheet they attach to the top view.
  SVG uses `bom` / `balloons` groups, DXF `BOM` / `BALLOON` layers.
  Tests: 15 in `tests/bom_sheets.rs`.

#### Flat cut-file export

- **`Shape#export_outline`** — `export("x.dxf")` writes an HLR *drawing*
  of a whole solid, which is the wrong deliverable for a cutter. This
  writes the closed loops of one planar face at 1:1 and nothing else,
  with circular edges as true `CIRCLE` / `ARC` entities rather than chord
  approximations, holes on their own layer, and the outline shifted to the
  origin for nesting. Taken in the face's own plane, so a tilted face keeps
  true size. `.dxf` and `.svg`. Track D's flat patterns go out through this
  same writer.
  Tests: 16 in `tests/cut_file_export.rs`.

#### Track D — Sheet metal

A new modelling domain, largely independent of the other tracks. **Complete.**

Sheet metal is built from a recipe rather than sculpted, because the folded
solid and the flat blank are two views of one part and the blank cannot be
recovered from finished geometry — unfolding needs to know where each bend
line ran and how tight it is. `SheetMetal` records the bends and derives both,
so the two deliverables cannot drift apart. It lives entirely in
`prelude.rb`, on the existing primitives and booleans; no new FFI surface.

- **Base and edge flanges** — `sheet_metal(thickness:, radius:) { |s| … }`
  with `s.base(w, h)` and `s.flange(side, length:, angle:, radius:)` off
  `:xmin` / `:xmax` / `:ymin` / `:ymax`. Each flange is built in a local
  frame where it grows along +x from a bend line along +y, then placed on
  its side, so one construction serves all four. The bend is a tube sector
  swept about the bend line — true cylindrical faces, no chord
  approximation — and the wall is laid out flat and folded about that same
  line, which is what the press brake does and what keeps the two agreeing.
  `length:` is the leg past the bend, not the overall height. Angles run
  to 180°, giving a hem.
- **Bend relief and K-factor bend allowance** — a flange narrowed with
  `from:` / `to:` gets a notch at each end of the bend line by default,
  `:rectangular` or `:obround`, cut into the solid and the blank alike.
  Allowance is `angle × (radius + k × thickness)`, the neutral-axis arc.
  Two flanges that would meet at a shared corner are refused at the call:
  they touch at a point with no material joining them, the blank pinches to
  nothing, and the folded solid looks entirely plausible meanwhile.
- **Unfolded flat patterns** — `flat` develops the blank as one planar
  face by tracing the outline counter-clockwise, and `export_flat` writes
  it through `export_outline`, so obround reliefs keep true arcs.
  Holes are deliberately not developed: one in a bend zone moves and
  distorts, and guessing where it lands is worse than not guessing.
  Tests: 30 in `tests/sheet_metal.rs`, against hand-computed geometry
  rather than against each other.

#### Opportunistic additions

Folded in where they fit rather than scheduled:

- 3MF export — carries units, colour, and multi-body properly, unlike STL.
  `export("part.3mf")`. OCCT has no 3MF writer, so the work is split: C++
  tessellates and emits the model XML, Rust wraps it in the OPC ZIP
  (`src/occt/threemf.rs`). Colour is still one per shape, because that is
  where `Shape#color` puts it; per-body colour needs the tag on the
  topology.
- `pattern_along_path` — already present, under the name `path_pattern`
  (`path_pattern(shape, path, n)`). Distributes n arc-length-even copies
  along a Wire or Edge, each oriented to the path tangent. Kept in the list
  by an oversight rather than left undone.
- `knit` — already present, under the name `sew`
  (`sew([face1, face2, …], tolerance:)`). Same oversight.
- `thicken` — `surface.thicken(t)` gives a Face or Shell a wall and returns
  a solid, the counterpart to `shell` hollowing one out. Uses
  `BRepOffsetAPI_MakeThickSolid::MakeThickSolidBySimple`; the *ByJoin*
  variant `shell` uses is the hollowing algorithm and hands back a
  zero-volume shape for an open surface. The result's orientation is
  corrected from the sign of its volume, since a face that was never cut
  from a solid has no side that is meaningfully "inside".
  Tests: 9 in `tests/thicken.rs`.
- Binary STL — `.stl` is the most-used export and was the only one still
  written as text, several times larger for no benefit (1,216,440 bytes vs
  240,484 for the same 4,808-triangle mesh). Binary is now the default;
  `export("part.stl", ascii: true)` opts back into text. Changes the bytes
  existing scripts write, not the geometry.
  Tests: 7 in `tests/export_stl_binary.rs`.
- `volume` on an open surface — a Shell with a free boundary encloses
  nothing, but OCCT integrates over it anyway and returned a plausible,
  meaningless number (517.9 for a ruled surface between two loops), which
  became a fictional mass in `mass_estimate` and the assembly rollup. Now
  `0.0`, matching Face and Wire. The guard is narrow on purpose: keying off
  `closed?` would reject spheres, booleans and imported meshes, all of
  which OCCT reports as not closed while measuring correctly.
  Tests: 8 in `tests/volume_of_surfaces.rs`.

---

## Planned phases

### Phase 12 — Quadcopter Readiness

Phase 11's assembly reports were validated by working through a quadcopter,
and the same exercise is what scopes this phase: the parts of a real drone —
frame plates, arms, motor mounts, canopy, propellers — that the current
feature set cannot model comfortably, or cannot deliver in the form the next
tool in the chain needs. Ordered so the pure-Ruby items land first.

- [x] **Airfoil profile primitive** — the one quadcopter part that could not
      be modelled was a propeller. A blade is a loft of airfoil sections with
      varying chord and twist along the radius; `loft` already accepted
      pre-rotated, pre-placed 3-D profiles, but there was no way to *make* an
      airfoil section short of hand-feeding dozens of points into
      `spline_2d`. `airfoil(naca: "2412", chord: 20)` generates NACA 4-digit
      sections analytically (closed-trailing-edge polynomial, cosine point
      spacing); `coordinates:` takes published points in Selig order and
      `dat:` the text of a Selig `.dat` file, with a blunt trailing edge
      closed by a straight base. Built as two interpolated BSpline segments
      over `make_profile_2d`, so the surfaces stay smooth through
      extrude/loft while the trailing edge stays a true corner. Pure Ruby in
      the prelude. Unlocks propellers and an aerodynamic canopy.
      Tests: 28 in `tests/phase12_airfoil.rs`.
- [x] **Per-section twist and scale in `sweep_sections`** — the sweep placed
      origin-centred profiles at the spine points but could not rotate or
      scale them per station, so a blade or tapered arm meant manual loft
      bookkeeping. `twist:` and `scale:` take a per-profile Array or a single
      Numeric blended linearly (total twist from 0°; end scale from 1). Done
      in the prelude: the native sweep is now `__rrcad_sweep_sections` and
      the wrapper rotates/scales a copy of each profile in its own plane
      before placement, so no C++ changed and profiles are never mutated.
      A blade is one call: `sweep_sections(spine, [section] * 3,
      twist: [30, 20, 12], scale: [1.0, 0.75, 0.4])`.
      Tests: 14 in `tests/phase12_sweep_twist.rs`.
- [ ] **Nut-pocket helpers** — the hardware library (`clearance_hole`,
      `tap_drill`, `heat_set_insert`, `bearing_bore`) stops short of two
      staples of printed frames: hex recesses for captive metric nuts and
      pockets for standoffs. Same table-driven pure-Ruby pattern as
      `heat_set_insert`: `nut_pocket(:m3)`, with locknut variants.
- [ ] **`shell` with face selection** — `.shell(thickness)` always removes
      the topmost face. A canopy or battery tray needs to choose which
      face(s) to open — `.shell(1.6, open: part.faces("front"))`. OCCT's
      `BRepOffsetAPI_MakeThickSolid` already takes a face list, so the work
      is plumbing the selection through the bridge.
- [ ] **Text emboss / engrave** — no `text()` exists anywhere. Frame plates
      want part labels, version numbers, and motor-rotation arrows — CW/CCW
      markings are genuinely functional on a quad. OCCT provides
      `Font_BRepTextBuilder`; emboss is extrude + fuse, engrave is extrude +
      cut.
- [ ] **Structured (non-fused) assembly STEP export** — `Assembly#export`
      fuses everything into one solid, so reopening the design in
      FreeCAD/Fusion — or handing it to whoever machines the plates — loses
      the motors, arms, and plates as separate components. Write STEP with
      product structure via XCAF (`STEPCAFControl_Writer`), which the GLB
      exporter already neighbours through CAF. Also fixes per-part colour in
      assembly GLB previews.

---

## Deferred and not planned

- **IGES import/export** — deprioritised in Phase 7; STEP covers the same
  workflows.
- **Native egui/wgpu viewer** — not planned. The browser preview is the
  long-term approach.
- **Section arrows, "A-A" labels, half/offset/revolved sections** — annotation
  and advanced section types beyond the Track C geometry work.
- **Pretty-printing** — `pp` is an alias of `p`; there is no pretty-printer.
- **Sheet metal beyond one plate and its edges** — no flange folded off another
  flange, no non-rectangular base, and holes are not developed through a bend.
  A hole in a bend zone moves and distorts as the metal wraps; producing a
  blank with it in the wrong place is worse than producing one without it.
- **Feature-tree editing in the browser preview** — cancelled. Browsing landed
  (see [Project Improvements](#project-improvements)); editing is a different
  and much larger problem. The `.rb` script is the source of truth, so changing
  a parameter from the viewer means either rewriting the user's file — a
  source-transformation problem, not a UI one — or keeping a live VM behind the
  preview server, which the current design deliberately avoids: `--preview`
  re-runs the whole script on save, and the MCP path builds a fresh VM per
  call. Editing the script and saving already round-trips in well under a
  second, so the payoff would not cover either cost.

---

## Architecture notes

See `CLAUDE.md` and `doc/development.md` for the full architecture and
development guide.

- **Memory:** each `Shape` is a `Box<occt::Shape>` raw pointer stored in mRuby's
  `RData void*`; the `dfree` GC callback drops it. No SlotMap, no reference
  counting.
- **Preview:** OCCT tessellation → GLB → `axum` HTTP → Three.js viewer →
  WebSocket live reload.
- **Threading:** mRuby is not thread-safe. `RUST_TEST_THREADS=1` in
  `.cargo/config.toml` keeps tests single-threaded; live `MrubyVm` values must
  never cross threads.
