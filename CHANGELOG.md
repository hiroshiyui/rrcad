# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Binary STL output** (`src/occt/bridge.cpp`, `src/ruby/glue.c`): `.stl` was
  the most-used export in the tool and the only one still written as text.
  `StlAPI_Writer` defaults to ASCII and the writer never said otherwise, so
  every part came out several times larger than it needed to be for no benefit
  — a filleted cylinder at `linear_deflection: 0.02` (4,808 triangles) wrote
  1,216,440 bytes where binary writes 240,484, the same mesh 5.1x smaller.
  Binary spends a fixed 50 bytes per triangle; text spends whatever the decimal
  digits need.

  Binary is now the default. `export("part.stl", ascii: true)` opts back into
  text for diffing or for a toolchain that cannot read binary. This changes the
  bytes existing scripts produce, though not the geometry: every slicer, mesh
  tool, and `import_stl` reads either encoding. 7 tests in
  `tests/export_stl_binary.rs`, which check the format's self-describing
  structure (header, `uint32` count, 50-byte stride all agreeing), that the
  header does not begin with `solid` — the sniff a parser uses to tell the
  encodings apart — and that the binary triangle count matches the count the
  text encoding spells out for the same shape at the same deflection.

- **Feature tree in the live preview** (`src/preview/mod.rs`,
  `src/preview/viewer.html`): the `metadata.json` sidecar now carries the
  shape's `feature_graph` parsed into nodes, and the viewer renders it as a
  **Features** panel beneath the model properties. A feature graph is a DAG,
  not a list — a boolean draws from two branches — so the panel lays it out the
  way a CAD tree does: the chain leading to the previewed shape flush left, a
  branch feeding into it indented beneath the step that consumes it, and the
  merged-in node named on that row. A linear model therefore does not indent at
  all. Clicking a row shows its full recorded entry, which carries detail the
  short label drops. The panel is read-only: the script stays the place the
  model is edited, and saving re-runs it and redraws the tree.

- **`thicken`** (`src/occt/bridge.cpp`, `src/ruby/glue.c`): `surface.thicken(t)`
  gives a Face or Shell a wall and returns a solid — the counterpart to
  `shell`, which takes material out of one. It is how a lofted or filled
  surface becomes a part that can be machined or printed. The wall grows along
  the surface normal, a negative thickness builds on the other side, and
  curvature is preserved: thickening a cylinder's side face gives a tube, not
  an extrusion. A Solid is refused, naming `offset` and `shell` as the two
  operations that were probably meant.

  Uses `MakeThickSolidBySimple` rather than the `MakeThickSolidByJoin` that
  `shell` uses: ByJoin is the hollowing algorithm and expects a solid to remove
  material from, so handed an open surface it offsets the faces without closing
  the sides and returns a zero-volume shape that looks right in a viewer. The
  result's orientation is then corrected from the sign of its volume, because a
  face that was never cut from a solid has no side that is meaningfully
  "inside", and getting it backwards produces a solid that removes material
  where it should add it. 9 tests in `tests/thicken.rs`.

- **3MF export** — `part.export("part.3mf")`. STL is the format every slicer
  reads and the reason a print occasionally comes out at the wrong scale: an
  STL says `10` and nothing about what `10` means, and it merges every body
  into one triangle soup. A 3MF declares `unit="millimeter"` once, gives each
  solid in the shape its own `<object>`, and carries `Shape#color` as an sRGB
  material. OCCT has no 3MF writer and a 3MF is a ZIP, so the work is split
  along the line each side is good at: C++ (`shape_3mf_model` in
  `src/occt/bridge.cpp`) tessellates and returns the model XML, and Rust
  (`src/occt/threemf.rs`) wraps it in the OPC package. Per-face triangulations
  are welded on exact node coordinates — never on a tolerance, which would
  collapse a thin wall — and reversed faces are wound back, so the result is a
  closed surface with outward normals rather than a pile of loose quads that
  merely looks right in a viewer. Colour is one per shape, because that is
  where `Shape#color` puts it; a multi-coloured assembly still needs a file per
  part. A shape with no surface is refused rather than written as an empty
  package. Adds the `zip` dependency (deflate only). 14 tests in
  `tests/export_3mf.rs` and `src/occt/threemf.rs`.

- **Sheet metal** (Phase 11 Track D, `src/ruby/prelude.rb`):
  `sheet_metal(thickness:, radius:, k_factor:) { |s| ... }` builds a folded
  part from a recipe of bends — `s.base(w, h)` for the plate and
  `s.flange(side, length:, angle:, radius:, from:, to:)` for walls folded up
  off `:xmin` / `:xmax` / `:ymin` / `:ymax`. `to_shape` gives the folded solid;
  `flat` develops the blank and `export_flat` writes it as a 1:1 cut file
  through `export_outline`. The recipe is recorded rather than the geometry
  alone because the blank cannot be recovered from a folded solid: unfolding
  needs to know where each bend line ran and how tight it is. Bend allowance is
  `angle × (radius + k_factor × thickness)`, the neutral-axis arc, and `bends`
  reports what each fold consumed. A flange narrowed with `from:` / `to:` gets
  bend relief automatically — `:rectangular` or `:obround` — notched into the
  solid and the blank alike. Two flanges that would run into a shared corner
  are refused at the call, since they would meet at a point with no material
  joining them and the blank would pinch to nothing while the folded solid
  still looked right. `length:` is the leg past the bend, not the overall
  height, so opening the radius does not silently shorten the wall. Holes are
  not developed, by choice: a hole in a bend zone moves and distorts. Entirely
  in the prelude, on existing primitives — no new FFI surface. 30 tests in
  `tests/sheet_metal.rs`, checked against hand-computed trigonometry rather
  than against each other. New sample `samples/11_sheet_metal_tray.rb`.

- **Parts lists and balloon callouts on assembly drawings** (Phase 11 Track C,
  `src/occt/bridge.cpp`, `src/ruby/prelude.rb`): `asm.export("panel.svg",
  view: :sheet, bom: true, balloons: true)` draws the bill of materials as a
  table below the drawing and marks each component with a numbered balloon
  whose leader lands on that part, keyed to the table's item numbers. One
  balloon per *component* rather than per part — the table already states the
  quantity. Balloons ring the geometry, ordered by each part's bearing from the
  centre so the leaders fan out without crossing, and attach to the top view on
  a three-view sheet. Per-component data cannot travel as scalar export options
  (the row count is not known until the assembly is walked), so it crosses the
  FFI as tab- and newline-delimited records; a delimiter inside a component
  name is replaced rather than silently shifting every column after it. Cells
  are XML-escaped, so a component named `M6 <A&B>` no longer produces an SVG
  that fails to parse. SVG emits `bom` and `balloons` groups; DXF uses `BOM`
  and `BALLOON` layers. Both options are drawing-only and ignored by the solid
  formats. This completes Phase 11 Track C.

- **`Assembly#export` accepts drawing options** (`src/ruby/prelude.rb`): it
  took only a path, so `view:`, `section:`, `dimensions:`, and every other
  option was silently dropped — an assembly could produce a STEP file but not a
  sheet, which is the drawing an assembly most needs. Options are now forwarded
  untouched to the fused `Shape`, so `asm.export("rig.svg", view: :sheet,
  title_block: true)` works. Also unblocks the BOM-on-sheet item in Track C.

- **Ordinate dimensioning on 2-D drawings** (`src/occt/bridge.cpp`,
  `src/ruby/glue.c`): `part.export("plate.svg", ordinate: true)` measures every
  located feature from a single datum corner and labels it, so a drawing says
  where the holes are and not just how big the part is. Ordinate form was
  chosen over a chain of dimensions between neighbours because a chain
  accumulates tolerance across the part and stops being readable past a few
  features. The datum is the lower-left of the projected geometry rather than
  the model origin, so a part modelled far from the origin still reads its own
  dimensions, and the numbers match a part clamped against a corner stop.
  Labels stay in model units regardless of `scale:`, and features sharing a
  coordinate collapse to a single ordinate so a row of holes gets one dimension
  rather than four stacked on each other. The located set is cylindrical faces
  whose axis points along the view direction — the same features
  `center_marks:` and `callouts:` act on, including corner fillets. SVG emits
  an `ordinates` group; DXF an `ORDINATE` layer with right-aligned `TEXT` so a
  rotated label grows away from the drawing instead of back over it. Composes
  with `dimensions:` and works per view on `view: :sheet`.

- **Detail views on 2-D drawings** (`src/occt/bridge.cpp`, `src/ruby/glue.c`):
  `part.export("part.svg", view: :top, detail: { at: [68, 38], radius: 8,
  scale: 4 })` clips a circular region out of the projection, magnifies it, and
  draws it beside the parent view inside a border circle captioned
  `DETAIL A (4:1)`; the parent gains a thin circle marking what was blown up.
  The region is stated in model units on the view's own drawing plane (`:top` →
  X/Y, `:front` → X/Z, `:side` → Y/Z), so the numbers match the ones used to
  model the part and do not shift with the drawing `scale:`. Edges crossing the
  region are cut analytically on the boundary rather than at the nearest
  tessellation vertex, and a polyline that leaves and re-enters is split rather
  than bridged by an edge the part does not have. Hidden lines, section
  outlines, and hatching inside the region are magnified along with the visible
  edges. SVG emits a `detail` group; DXF puts the marker, border, label, and
  caption on a dedicated `DETAIL` layer. The close-up carries no dimensions and
  no centre marks or callouts of its own — both would be placed against the
  parent's geometry at the wrong scale. An empty region is an error rather than
  a blank bubble, and `view: :sheet` is refused since it has no single parent
  view to magnify.

- **Flat cut-file export** (`src/occt/bridge.cpp`, `src/ruby/prelude.rb`):
  `face.export_outline("plate.dxf")` writes the closed loops of one planar
  face at 1:1 with nothing else in the file. This is a different deliverable
  from `export("plate.dxf")`, which draws an HLR projection of the whole solid
  and carries whatever else is visible from that direction — right for a shop
  drawing, wrong for a laser or CNC controller. Circular edges become true
  `CIRCLE` and `ARC` entities rather than strings of short chords, so a bolt
  hole arrives as a hole and a filleted corner as an arc; only free-form
  curves are approximated, bounded by `deflection:`. The outline is taken in
  the face's own plane, so a face tilted in space keeps true size instead of
  foreshortening, and is shifted so its bounding box starts at the origin,
  ready to nest on a sheet. Holes go on a `HOLES` layer separate from
  `PROFILE`, since inside cuts normally run first, and the DXF declares
  millimetres via `$INSUNITS`. `.svg` is supported as a second format. A
  planar face is required: a multi-face solid raises and names how to select
  one, and a curved face is refused.

- **Multi-file projects via `require_relative`** (`src/ruby/loader.rs`,
  `src/ruby/native_loader.rs`, `src/ruby/glue.c`, `src/cli.rs`): a project of
  any size outgrows one script, and until now the alternative was one large
  file or copy-pasted constants — which is how a motor mount and an arm end up
  disagreeing about a bolt pattern. Paths resolve against the directory of the
  file doing the requiring, not the working directory, so a required file can
  pull in its own neighbours and the project runs the same from anywhere. The
  `.rb` suffix is optional. Each file is evaluated at most once, returning
  `false` on later requires, which also makes a require cycle terminate rather
  than recurse. Syntax errors name the file they are in. Under `--preview`
  every required file is watched, not just the entry script, and the watch set
  is recomputed after each reload. Plain `require` has no load path to search,
  so it raises and redirects. Unavailable in MCP mode, guarded both by the
  security prelude and by the loader's own refusal to run without a base
  directory.

- **Stated part mass and assembly inertia** (Phase 11 Track B follow-on,
  `src/ruby/prelude.rb`): `mass_properties` derived mass from `volume ×
  density`, which is right for parts you model and wrong for parts you buy —
  and on a real assembly the bought parts dominate. Placement methods now take
  `mass:` (grams), a datasheet weight that overrides the computed one while
  leaving the envelope geometry in place, so the part keeps taking part in
  clash and clearance checks. Passing `mass:` and `density:` together raises,
  and every row reports `mass_source: :stated` or `:density`. A stated mass on
  a zero-volume shape behaves as a point mass, covering wiring and adhesive
  without a separate concept. `mass_properties` also returns the inertia
  tensor in g·mm², summed by parallel-axis transfer about the centre of mass
  or any `about:` point — validated against the tensor OCCT computes for the
  equivalent fused solid, which agrees to machine precision on all six
  components.

- **Assembly reports** (Phase 11 Track B, `src/ruby/prelude.rb`): an assembly
  that solves cleanly can still be wrong, and until now it produced none of
  the deliverables an assembly exists for. Three reports close that gap.
  `Assembly#interferences` intersects every pair of components and reports
  real overlaps with volume and centroid, worst first; a flush `mate` is not a
  clash, since two boxes sharing a face overlap in zero volume. Passing
  `clearance:` also demands an air gap, skipping parts that touch on purpose
  unless `ignore_contact: false`. `Assembly#bom` rolls components up by their
  `component:` key with quantity, volume, and mass, and `#bom_text` renders an
  aligned table; grouping parts of different volume raises rather than
  quietly averaging them. `Assembly#mass_properties` returns total volume,
  mass, and a mass-weighted centre of mass with a per-part breakdown.
  Placement methods (`place`, `part`, `ground`, `mate`, `distance_mate`,
  `axis_align`, `angle_mate`) now accept optional `name:`, `component:`,
  `material:`, and `density:` metadata, and `Assembly#components` enumerates
  ad-hoc placements and solver parts uniformly in their solved positions. A
  built-in material→density table means `material: "aluminium"` alone yields a
  believable mass; an explicit `density:` wins, an unknown material falls back
  rather than failing, and every report echoes the density it used.

- **Spline segments in sketch profiles** (Phase 11 Track A,
  `src/occt/bridge.cpp`, `src/ruby/prelude.rb`): `spline a, b, through: [...]`
  draws a curved segment of a constraint sketch through the given interior
  points, which may be literal `[x, y]` pairs or sketch points the solver
  places. A new `make_profile_2d` bridge function builds the outline as a
  chain of edges — straight runs as line edges, curved ones interpolated with
  `GeomAPI_Interpolate` — so the curve reaches the profile as a real BSpline
  edge rather than a polyline standing in for one, and survives export and
  later features as a curve. A sketch with a spline needs only two segments to
  close, since a curve can close a loop on its own. Corner modifiers set back
  along a straight run and `trim` / `extend` slide an endpoint along one, so
  both reject spline segments by name; corners between two straight segments
  still work in a sketch that has curves elsewhere.

- **Sketch-level patterns** (Phase 11 Track A, `src/ruby/prelude.rb`):
  `linear_pattern(count:, dx:, dy:)`, `polar_pattern(count:, center:,
  angle:)`, and `grid_pattern(nx:, ny:, dx:, dy:)` inside `sketch do ... end`
  replicate the finished profile into one compound profile, so a single
  `extrude`, `pad`, or `pocket` covers every copy — six bolt holes become one
  pocket rather than six. Polar patterns turn about a sketch point, an
  `[x, y]` pair, or the origin, over a partial or full sweep, and a centre
  that names a corner the sketch moved uses its final position. Patterning
  runs after corner modifiers, segment edits, and the offset, so every copy
  carries that shaping. The builder methods shadow the top-level functions of
  the same name and delegate to them when handed a Shape, so
  `polar_pattern(shape, n, angle)` still works inside a sketch block.

- **Profile offset** (Phase 11 Track A, `src/ruby/prelude.rb`,
  `src/occt/bridge.cpp`): `offset(distance)` inside `sketch do ... end` grows
  (positive) or shrinks (negative) the finished profile in its own plane,
  keeping every edge parallel to where it started. It is the last step of
  building the profile, so it applies to a constrained polygon including its
  corner modifiers and segment edits, or to a `circle_at` / `arc_at` /
  `slot_between` profile. A sketch takes one non-zero offset.

- **Sketch segment `trim` and `extend`** (Phase 11 Track A,
  `src/ruby/prelude.rb`): shorten or lengthen an individual segment of a
  line-based sketch — one endpoint slides along the segment while the other
  anchors it, either `by:` a distance or `to:` the intersection with another
  segment's infinite line. `at:` chooses which endpoint moves. Edits apply in
  declaration order after the constraint solver runs, so a later edit sees an
  earlier one and corner modifiers act on the moved corner's final position.
  Collapsed segments, parallel `to:` references, wrong-direction
  intersections (which name the operation the user meant), unregistered point
  pairs, and bad `at:` targets are all rejected before any geometry is built.

- **Sketch corner fillets and chamfers** (Phase 11 Track A,
  `src/ruby/prelude.rb`): `fillet(point, radius)` and
  `chamfer(point, distance)` inside `sketch do ... end` round or bevel an
  individual corner while the 2-D profile is built, so the shaping is part of
  the profile rather than a 3-D fillet applied afterwards. Both accept unit
  values, and oversized modifiers, overlapping setbacks, duplicate modifiers
  on one corner, collinear corners, and modifiers targeting a point outside
  the closed loop are all rejected up front with the offending corner named.

- **Drawing section views** (Phase 11 Track C, `src/occt/bridge.cpp`,
  `src/ruby/native_io.rs`, `src/ruby/glue.c`): `shape.export("part.svg",
  section: :xz)` cuts the solid with an axis-aligned plane and draws the
  exposed cut face with standard 45° hatching — SVG gets `hatch` and
  `section` groups, DXF gets a dedicated `HATCH` layer. Interior holes stay
  unhatched. Omitting `offset:` cuts through the middle of the part rather
  than at the origin, so the shorthand works on parts that start at the
  origin. Non-solids, planes that miss the shape, and unknown plane names are
  each rejected with a specific message.

- **`puts`, `print`, `p`, and `pp` now work in scripts** (`src/ruby/prelude.rb`,
  `src/ruby/native_output.rs`): the embedded interpreter ships without the IO
  gems, so scripts previously had no way to print anything — `puts` raised
  `undefined method`, even though the REPL help advertised it. They are now
  defined in the prelude on a native output primitive, with Ruby's formatting
  rules (array flattening, `nil` as a blank line, no doubled trailing
  newline, `p` returning its argument). Output goes to standard output and is
  removed entirely in MCP mode, where stdout carries the JSON-RPC responses.

- **Export-path confinement tests and documentation**
  (`tests/export_confinement.rs`, `doc/user-guide/10-import-export.md`): the
  intentionally strict import/export path rules — inside the working
  directory only, symlink escapes rejected, target directory must exist,
  applies to reads as well as writes — are now pinned by explicit tests and
  described for users.

### Changed

- **FFI boilerplate consolidated** (`src/ruby/`): the repetitive Ruby-to-native
  wrapper patterns are now generated by a documented `macro_rules!` layer in
  `src/ruby/native.rs` (constructors, shape-returning methods, scalar/array/
  flag/string returns, path-validated import & export, selector count/get
  pairs) and by static helpers in `src/ruby/glue.c` (`raise_if_err`,
  `checked_shape_ptr`, hash-option fetchers, `collect_shape_ptrs`). No
  `extern "C"` name, signature, or error string changed; roughly 1,750 lines
  of duplicated boilerplate were removed.

- **The drawing export travels as one shared struct** (`src/occt/mod.rs`,
  `src/occt/bridge.h`, `src/occt/bridge.cpp`, `src/occt/drawing_ops.rs`,
  `src/ruby/native_io.rs`): the SVG and DXF exporters took a flat list of some
  thirty scalars that six files had to keep in lockstep by hand, and every new
  drawing option made it worse. The request is now a `DrawingSpec` cxx shared
  struct, declared once and generated for both Rust and C++, so the two sides
  of that boundary cannot drift apart; a new option is one field instead of six
  edits. `export_svg` and `export_dxf` collapsed into a single
  `export_drawing(spec, format)`, since the two writers always took an
  identical request. The C ABI hop between `glue.c` and `native_io.rs` stays
  flat deliberately — a struct there would mean a hand-matched memory layout in
  two languages, which fails silently, where a mismatched argument list at
  least fails at the call site. No behaviour change: every drawing option's
  output was byte-compared against the previous build across twelve files.

### Fixed

- **`--preview` panicked instead of starting** (`src/preview/mod.rs`).
  `preview::start` bound the port before creating the Tokio runtime, and
  `bind_listener` ends in `TcpListener::from_std`, which panics outright with
  "there is no reactor running" when there is no reactor to register the socket
  with. The whole preview mode was dead on arrival. The runtime is now created
  first and the bind happens inside its `enter()` guard. It went unnoticed
  because every test that touched `bind_listener` was a `#[tokio::test]`, which
  supplies exactly the runtime the real caller lacks — so the regression test is
  a plain `#[test]` on purpose.

- **`linear_deflection:` on `export` now does something.** It was documented
  for a long time and never read: every mesh came out at a fixed 0.1 mm, so
  `linear_deflection: 0.01` and `linear_deflection: 2.0` produced byte-identical
  files and nothing said so. It now controls tessellation for every mesh format
  (`.3mf`, `.stl`, `.glb`, `.gltf`, `.obj`), defaults to 0.1 mm, accepts
  `deflection:` as a shorter spelling, and is refused if it is not greater than
  zero — zero asks for infinite detail, which OCCT does not report as an error.
  Ignored by STEP and the drawing formats, so one value can cover a batch of
  exports. "Not asked for" crosses the FFI as its own flag rather than as a
  sentinel value, so `linear_deflection: 0` is reported as the mistake it is
  instead of silently falling back to the default.

- **Exporting one shape twice at different mesh qualities gave the first mesh
  both times** (`src/occt/bridge.cpp`). OCCT stores the triangulation on the
  shape and `BRepMesh_IncrementalMesh` keeps one it considers good enough, so
  after a coarse export a finer one silently returned the coarse mesh. The
  exporters now compare against the deflection each face was built to and
  discard the mesh when it differs, while still skipping the work when nothing
  changed — the common case of writing one shape to STEP, STL and GLB in one
  script.

- **Annotation text can no longer corrupt the drawing it is written into**
  (`src/occt/bridge.cpp`): an `&` or `<` in a `datum:` or `feature_control:`
  string produced an SVG that no parser would open — `feature_control:
  "<0.05> A|B"` is an ordinary thing to write — and a newline in either
  desynchronised the DXF group-code stream, since DXF values occupy exactly one
  line and the following line is read as the next code. Both failed silently at
  export: the file was written, weighed the right amount, and only broke when
  something tried to read it. Every SVG text node now goes through `svg_text()`
  and every DXF text value through `dxf_text()`, which flattens breaks to
  spaces so the drawing keeps what the user wrote. Generated labels
  (dimensions, diameter callouts, ordinates, the title block) go through them
  too: deciding per call site whether a string can reach a user is the
  judgement that produced the bug. 7 tests in `tests/drawing_text_safety.rs`.

- **Corrected the documented meaning of `Shape#inertia`** (`src/occt/bridge.cpp`,
  `src/ruby/prelude.rb`, `doc/api.md`): the tensor's off-diagonal entries were
  documented as products of inertia (+∫xy dV) when OCCT in fact returns true
  inertia-tensor entries (−∫xy dV), and the values were described as mass
  moments when they are volume moments in mm⁵ (OCCT integrates at density 1).
  Either misreading silently corrupts an axis transfer or a unit conversion,
  which is exactly what the new assembly inertia rollup depends on. No
  behaviour change — the values were always these; only the description was
  wrong.

- **`Shape#offset_2d` returns a usable profile** (`src/occt/bridge.cpp`):
  `BRepOffsetAPI_MakeOffset` returns bare wires, so a Face went in and a Wire
  came out — the result extruded into an open shell rather than a solid, and
  an inward offset even reported a negative volume. The offset wires are now
  rebuilt into a planar face, the largest enclosed area becoming the outer
  boundary and the rest holes, so an offset profile extrudes, pads, and
  pockets like any other. Profiles with holes offset in both directions, and
  faces OCCT cannot offset whole (an all-circular annulus among them) fall
  back to offsetting each boundary wire separately, with the sign flipped for
  hole wires. An inward offset that consumes the profile now raises instead
  of returning an empty result.

- **MCP printing escape** (`src/mcp/security.rs`): the security prelude
  undefined `puts`/`print`/`p`/`pp` on `Kernel` only. Methods defined by a
  top-level `def` land on `Object`, so once the prelude defined them there
  they stayed reachable inside MCP tool calls. They are now undefined on
  `Object` too, along with the `__rrcad_write` primitive beneath them, and
  MCP additionally routes script output off stdout as a second guard.

- **Design tables fail fast on malformed rows** (`src/cli.rs`): a data row
  whose field count does not match the header is now rejected with the real
  file line number and the expected vs actual counts, instead of being
  silently truncated by `zip()` and producing a subtly wrong part.

## [0.4.0] - 2026-07-28

### Added

- **Project configuration** (`src/project_config.rs`, `rrcad.toml.example`):
  optional `rrcad.toml` files are discovered from the script directory or
  current working directory, walking up parent directories. Supports
  `preview_port` for `--preview` defaults and `[params]` for default `param()`
  overrides; CLI flags still take precedence.
- **Preview auto port selection** (`src/preview/server.rs`, `src/cli.rs`):
  `--preview` now asks the OS for a free port by default (printed at startup)
  instead of requiring a fixed port; `--preview-port` or `rrcad.toml`
  `preview_port` pin a specific one.
- **User guide chapters** (`doc/user-guide/`): the user guide is split into 15
  task-oriented chapters with navigation links; README gained install and
  running sections.
- **Viewer resilience** (`src/preview/viewer.html`): a clear on-page message
  when Three.js cannot be loaded from the CDN (offline use) and automatic
  model reload after a WebSocket reconnect.

### Changed

- **Dependencies**: `axum` 0.7 → 0.8, `rmcp` 0.1 → 2.2, `notify` 6 → 8,
  `rustyline` 14 → 18, `toml` 0.8 → 1; the MCP server and preview WebSocket
  were migrated to the new APIs.
- **Viewer controls** (`src/preview/viewer.html`): orbiting stays enabled
  while measuring, keyboard shortcuts ignore Ctrl/Cmd/Alt combinations, and
  section planes clip only the model (floor, axes, and measure markers stay
  visible).
- **Internal quality**: the SVG/DXF export chain shares one options struct and
  common C++ layout helpers; MCP sandbox filenames and the export-format
  allowlist are named constants; public DSL-facing items carry doc comments;
  the whole tree is `rustfmt`/`clang-format` clean with zero clippy warnings.

### Fixed

- **OCCT exception safety** (`src/occt/bridge.cpp`): pipe-shell, sewing,
  thru-sections, and fragment builders now translate `Standard_Failure` into
  Rust errors instead of aborting the process when given malformed geometry.
- **Config discovery** (`src/project_config.rs`): `rrcad.toml` lookup now
  walks up parent directories correctly when the script is given as a bare
  relative path (e.g. `rrcad --preview part.rb`).
- **CLI validation** (`src/cli.rs`): `--preview-port` without `--preview` is
  now a hard error instead of being silently ignored.
- **MCP hardening** (`src/mcp/security.rs`): VM security constants tightened
  and the security prelude documented as defence-in-depth on top of the
  compile-time `mcp_safe.gembox` sandbox.

### Security

- **rmcp 2.2 upgrade** fixes RUSTSEC-2026-0189 (high severity, DNS rebinding
  in the Streamable HTTP transport) and drops the unmaintained `paste` crate;
  a `rand` unsoundness advisory was also cleared via the axum/tungstenite
  bump. `cargo audit` reports zero advisories.
- **Preview server DNS-rebinding defence** (`src/preview/server.rs`): all
  preview routes (including the WebSocket) reject requests whose `Host` or
  `Origin` headers do not identify the local machine.

## [0.3.0] - 2026-05-15

### Added

- **Hardware helpers — hole tools** (`src/ruby/prelude.rb`,
  `tests/phase8_tier2.rs`): added `clearance_hole(size, depth:)`,
  `tap_drill(size, depth:)`, `heat_set_insert(size, depth:)`,
  `socket_head_cbore(size, depth:, head_depth:)`, and
  `flat_head_csink(size, depth:, angle:)` for `:m2`–`:m5` metric hardware,
  each producing a cylinder/counterbore/countersink suitable for subtractive
  modelling with `.cut`.
- **Hardware helpers — bearing bores** (`src/ruby/prelude.rb`,
  `tests/phase8_tier2.rs`): `bearing_bore(size, depth:, fit:)` produces an
  outer-diameter bore for common deep-groove ball bearings (`:b608`, `:b623`,
  `:b624`, `:b625`, `:b626`, `:b688`, `:b695`, `:b6000`, `:b6001`, or a numeric
  OD), with `:press` (interference) or `:slip` (clearance) fit.
- **Hardware helpers — shaft fits** (`src/ruby/prelude.rb`,
  `tests/phase8_tier2.rs`): `shaft(diameter, length:, fit:)` generates a solid
  mating shaft cylinder with `:nominal`, `:press`, `:slip`, or `:running` fit
  adjustments relative to the nominal diameter.
- **Hardware helpers — standard fasteners** (`src/ruby/prelude.rb`,
  `tests/phase8_tier2.rs`): `screw(size, length:, style:)` generates solid
  fastener bodies for `:m2`–`:m5` in `:socket` (ISO 4762), `:button`
  (ISO 7380), and `:flat` (ISO 10642 90° conical head) styles.
- **Sketch — tangent constraint** (`src/ruby/prelude.rb`,
  `tests/phase10_sketch_constraints.rs`): `tangent(a, b, center, radius,
  side:)` constrains a line segment tangent to a circle. With `side:`
  (`:above`/`:below` for horizontal lines, `:left`/`:right` for vertical
  lines) the solver propagates the unknown perpendicular coordinate on
  either the line or the center; for fully-resolved geometry it verifies the
  point-to-line distance and raises on conflict. Endpoint-driven orientation
  inference lets the constraint work inside open rectangle sketches.
- **Assembly — transform-based constraint helpers** (`src/ruby/prelude.rb`,
  `tests/assembly_constraints.rs`): `Shape#rotate_about(point, axis_dir,
  angle_deg)` rotates around any pivot; `Assembly#distance_mate(shape,
  from:, to:, distance:)` is a named air-gap variant of `mate`;
  `Assembly#axis_align(shape, from: [p1, p2], to: [q1, q2])` rotates and
  translates a shape so a source axis maps to a target axis (covers
  coaxial / concentric / axis-alignment cases); `Assembly#angle_mate(...,
  angle:, pivot:, axis_dir:, offset:)` mates a face flush and then rotates
  about a chosen pivot axis to lock the leftover rotational degree of
  freedom.
- **Sketch — solver diagnostics enriched** (`src/ruby/prelude.rb`,
  `tests/phase10_sketch_constraints.rs`): conflict messages now name the
  involved sketch points and report the actual vs expected values
  (`horizontal`/`vertical`/`coincident`/`dimension`/`equal_length`/
  `tangent` and any `assign_coord`-driven constraint); the "did not
  converge" failure lists every unresolved point with its missing
  coordinates. The original `"conflicting X constraint"` substring is
  preserved at the start of each message for backward-compatible matching.
- **Sketch — polar_point construction helper** (`src/ruby/prelude.rb`,
  `tests/phase10_sketch_constraints.rs`): `polar_point([:name,] center,
  radius, angle_deg)` registers a construction point at polar coordinates
  around `center`; once `center` resolves the solver derives `(cx +
  r·cos θ, cy + r·sin θ)`. Useful for bolt circles and angular layouts.
  Closes the constraint-based-sketching MVP entry in `doc/ROADMAP.md`.
- **CAM / 3-D printing — mass and build-volume helpers**
  (`src/ruby/prelude.rb`, `tests/cam_checks.rs`):
  `mass_estimate(part, density: 1.24)` computes a rough mass in grams from
  `part.volume × density / 1000`; `print_volume_check(part, x:, y:, z:)`
  reports `{fits:, dx:, dy:, dz:, overflow_x/y/z:}` against a rectangular
  build volume.
- **CAM / 3-D printing — face normals exposed + overhang_faces**
  (`src/occt/bridge.{h,cpp}`, `src/occt/mod.rs`, `src/ruby/native.rs`,
  `src/ruby/glue.c`, `src/ruby/prelude.rb`, `tests/cam_checks.rs`): new
  `shape_face_normal` bridge call samples the outward unit normal at the
  face's parameter-space midpoint via `BRepLProp_SLProps`, flipping for
  `TopAbs_REVERSED` faces; surfaced to Ruby as `Shape#normal` returning
  `[nx, ny, nz]`. Built on top: `overhang_faces(part, max_angle_deg: 45)`
  lists faces whose outward normal tips below a horizontal threshold
  (assumes +Z-up build direction).
- **CAM / 3-D printing — draft analysis** (`src/ruby/prelude.rb`,
  `tests/cam_checks.rs`): `draft_faces(part, axis: [0, 0, 1],
  min_draft_deg: 1.0)` lists faces with insufficient mould draft along the
  pull axis (`asin(|n·axis|) < min_draft_deg`). Top/bottom faces are
  naturally excluded; the pull direction is configurable so non-Z pull
  setups work without rotation tricks.
- **CAM / 3-D printing — hole orientation analysis**
  (`src/occt/bridge.{h,cpp}`, `src/occt/mod.rs`, `src/ruby/native.rs`,
  `src/ruby/glue.c`, `src/ruby/prelude.rb`, `tests/cam_checks.rs`): new
  `shape_cylinder_axis` bridge call extracts the axis (origin + unit
  direction) and radius of cylindrical faces via
  `BRepAdaptor_Surface::Cylinder()`. Surfaced to Ruby as
  `Shape#cylinder_axis` returning `{origin:, axis:, radius:}`. Built on
  top: `hole_axes(part, orientation: :vertical | :horizontal,
  tolerance_deg:)` enumerates and filters cylindrical faces by axis
  direction.
- **OCCT diagnostics — call-site context + actionable hints**
  (`src/occt/mod.rs`, `tests/error_diagnostics.rs`): errors from
  Boolean ops (`fuse`/`cut`/`common`), fillets and chamfers (including
  selector / variable-radius / asymmetric variants),
  `extrude`/`revolve`/`shell`/`offset`/`offset_2d`/`simplify`,
  `sweep`/`sweep_guide`/`sweep_sections`, `loft`, Part Design (`pad`,
  `pocket`), and import/export now lead with the operation name, its
  numeric parameters, the operand shape kind (via a `summarize()`
  helper), and any file path or view name. The most common failures
  (`fillet`/`chamfer` radius too large, `extrude` on a solid, `shell`
  thickness too thick, `pad`/`pocket` planar-face requirement,
  `sweep`/`sweep_guide` profile-and-path types,
  `import_step`/`import_stl` missing files) also carry an actionable
  one-line `hint: …` suffix. The original `"X failed: …"` substring is
  preserved so existing substring matchers keep working.

### Fixed

- **CI — install g++ explicitly** (`.github/workflows/rust.yml`):
  Debian sid's `gcc` meta-package no longer pulls `g++` as a transitive
  dependency, so the CI container was missing a C++ compiler and
  `link-cplusplus` / cxx bridge builds failed with `failed to find tool
  "c++"`. Add `g++` to the apt install line.

---

## [0.2.2] - 2026-05-13

### Security

- **MCP timeouts are now process-enforced** (`src/mcp/mod.rs`, `src/main.rs`):
  MCP tool evaluations run in one-shot worker processes that are killed when
  the 30 s timeout expires.  A runaway Ruby loop or long OCCT operation can no
  longer keep running on a blocking thread after the client receives a timeout.
- **MCP prelude hardening extended** (`src/mcp/mod.rs`): the production
  security prelude now removes additional metaprogramming entry points from
  `BasicObject`, `Object`, `Kernel`, and `Module`, including top-level
  `define_method`, closing a dev-build escape path caught by the expanded
  integration tests.

### Changed

- **MCP tests use production hardening** (`tests/mcp_tools.rs`,
  `tests/mcp_stress.rs`): integration and stress tests now construct MCP VMs
  through the real production helper instead of carrying a stale copied prelude,
  and cover blocked file-access and metaprogramming methods.
- **Dependency audit rationale documented** (`audit.toml`): accepted the
  transitive `rand` advisory (`RUSTSEC-2026-0097`) with the project-specific
  rationale pending an upstream axum/tungstenite update.
- **Agent repository guidance added** (`AGENTS.md`): documented project layout,
  build/test commands, coding style, testing expectations, and agent-specific
  instructions for future automation.

---

## [0.2.1] - 2026-05-12

### Security

- **MCP prelude: undef metaprogramming methods** (`src/mcp/mod.rs`): the runtime
  security prelude now also strips `eval`, `instance_eval`, `class_eval`,
  `module_eval`, `send`, `__send__`, `public_send`, `method`, `define_method`,
  `define_singleton_method`, and `binding` from `Kernel`.  Production builds
  with `mcp_safe.gembox` were already safe, but a development build using the
  default gembox could previously reach undef'd methods via `send` or `eval`.
  The loop now runs inside `Kernel.module_eval { … }` so undef'ing `:send`
  mid-iteration cannot break the loop itself.
- **CLI preview tempfile uses CSPRNG** (`src/main.rs`): the temporary GLB
  filename is now derived from `uuid::Uuid::new_v4()` (122 bits of OS-CSPRNG
  entropy) instead of `DefaultHasher(time, pid)`.  The previous path was
  predictable from PID plus approximate launch time, weakening the symlink-
  attack mitigation.

### Fixed

- **Preview server start now returns `Result`** (`src/preview/mod.rs`,
  `src/main.rs`): `preview::start` no longer panics on double-init or tokio
  runtime creation failure — the CLI prints a clear error and exits cleanly.
- **MCP preview port-init race** (`src/mcp/mod.rs`): `MCP_PREVIEW_PORT`
  switched from `std::sync::OnceLock` to `tokio::sync::OnceCell` with
  `get_or_try_init`.  Two concurrent first-time `cad_preview` calls could
  previously each bind a listener and spawn an axum task — only one won the
  `OnceLock::set`, leaking the loser's listener and task.  The async OnceCell
  serialises initialisation so only one initialiser runs.
- **Preview WebSocket reconnect storm** (`src/preview/viewer.html`): the
  browser previously retried every 1 s forever after rrcad exited, pounding a
  dead port.  Reconnect now uses exponential backoff (1 → 2 → 4 → 8 s, capped
  at 8 attempts) and ends with a clear `"server gone (reload page to retry)"`
  status.
- **Preview GLB-load error detail** (`src/preview/viewer.html`): the load-
  error status line now includes the underlying error message instead of the
  bare `"load error"` text, making failures self-diagnosable from the browser.
- **Clippy lints**: `items_after_test_module` in `src/main.rs` (helpers moved
  above the test module), `useless_vec` in `tests/phase5_params.rs`, and
  `len_zero` in `tests/e2e_dsl.rs`.

### Changed

- **REPL tab-completion**: `SHAPE_METHODS` in `src/main.rs` extended with
  `fillet_var`, `bounding_box`, `volume`, `surface_area`, `distance_to`,
  `inertia`, and `min_thickness` so autocomplete matches the documented and
  available native methods.
- **Documentation**: `CLAUDE.md` updated to reflect the per-process randomised
  CLI preview tempfile and the MCP `/tmp/rrcad_mcp/preview.glb` path, and to
  list the `/logo.png` axum route.  `src/preview/server.rs` module doc lists
  `/logo.png`.

---

## [0.2.0] - 2026-03-31

### Added

- **Preview: flat-line view** (`src/preview/viewer.html`): press `F` to toggle a
  flat-line rendering mode — white flat-shaded surfaces (`MeshLambertMaterial`,
  `flatShading: true`) with heavy-gray edge lines drawn at ≥15° creases via
  `EdgesGeometry`.  View mode persists across live-reloads.
- **Preview: hamburger menu** (`src/preview/viewer.html`): a menu button in the
  top-right corner opens a panel with three sections — **View** (Normal / Flat-line
  radio buttons), **Scene** (Showroom / White radio buttons), and **Show** (Axes
  checkbox).  Keyboard shortcuts (`F`, `A`) remain functional and stay in sync with
  the menu state.  Press `Escape` to close the menu.
- **Preview: white scene** (`src/preview/viewer.html`): second scene preset — pure
  white background, white studio floor, flat neutral lighting (bright hemisphere + soft
  directional key only), vignette hidden.  The dark Showroom scene remains the default.
  Status text and menu-button colours adapt automatically for readability on the bright
  background.

### Fixed

- **Preview: GPU memory leaks** (`src/preview/viewer.html`): outgoing model's
  `BufferGeometry` and `EdgesGeometry` instances are now disposed on each live-reload,
  preventing unbounded GPU memory growth during long preview sessions.
- **Preview: GLB load-error message** (`src/preview/viewer.html`): error callback now
  sets status to `"rrcad preview — load error"` (previously showed "waiting for model")
  and logs the error to the browser console.
- **Preview: F shortcut ambiguity** (`src/preview/viewer.html`): the `F` shortcut label
  is now shown only on the inactive view menu item; the active item is already visually
  distinguished and does not need a shortcut hint.
- **MCP server: sandbox directory permissions** (`src/mcp/mod.rs`): sandbox directory
  is now created with mode `0o700` (user-only access) via `DirBuilder::mode`, eliminating
  a TOCTOU window from the previous two-step create-then-chmod approach.
- **MCP server: prelude blocks additional methods** (`src/mcp/mod.rs`): `open`,
  `require`, `require_relative`, and `load` are now removed from `Kernel` in the security
  prelude, preventing scripts from reading host files or loading arbitrary mRuby code.
- **glue.c: malloc leak on mRuby raise** (`src/ruby/glue.c`): multi-shape operations
  (`loft`, `sweep_sections`, `fuse_all`, `cut_all`, `sew`) now use
  `mrb_data_check_get_ptr` (non-raising) with explicit NULL checks and free-before-raise
  to avoid leaking the pointer array when an argument is an invalid type.
- **bridge.cpp: std::exception escape from sewing_build** (`src/occt/bridge.cpp`): added
  a second catch clause (`const std::exception&`) that re-throws, so non-OCCT C++
  exceptions cannot silently cross the `cxx` boundary and abort the process.

### Security

- MCP prelude now blocks `open`, `require`, `require_relative`, and `load` in addition to
  the previously blocked `system`, `exec`, `` ` ``, and `eval`.
- Sandbox directory created with `0o700` permissions (was world-readable until chmod).

---

## [0.1.6] - 2026-03-28

### Changed

- **Preview: studio photography look** (`src/preview/viewer.html`): transformed the
  live viewer into a dark product-photography studio — vertical gradient background
  (cool dark top → faintly warm bottom), a large polished studio floor plane that
  catches shadows and shows faint model reflection, multi-light rig (key/fill/rim/bounce),
  40° telephoto FOV (less distortion than 45°), CSS vignette overlay for cinematic
  corners, per-model shadow frustum fitted to bounding box for crisp shadows, orbit
  floor clamp so the camera cannot go below the studio floor, axes helper (press A to
  toggle), and ACES filmic tone mapping at 1.2× exposure.
- **Preview: tighter camera fit** (`src/preview/viewer.html`): `fitCamera` multipliers
  reduced from 2.5 × 1.4 to 1.15 × 1.0, so the model fills the frame closely on every
  load rather than sitting in a wide empty viewport.
- **Split TKL keyboard — flat-bottom case** (`samples/split_tkl_keyboard.rb`): removed
  the `solid_tent` wedge base entirely.  The case now has a flat bottom; users glue on
  custom-printed tenting feet at whatever angle they prefer.
- **Split TKL keyboard — right bottom-row modifiers widened** (`samples/split_tkl_keyboard.rb`):
  RAlt, Fn, and RCtrl widened from 1 U to 1.25 U each; Space repositioned to 1 U at the
  left edge of the row (flush with the case wall) so all three modifiers pack tightly
  against the spacebar.  Leaves a natural 1.25 U gap before the arrow cluster.

### Fixed

- **Split TKL keyboard — RShift position corrected** (`samples/split_tkl_keyboard.rb`):
  RShift switch centre moved from 5.5 U to 6.0 U so the 2 U keycap sits flush against
  `/` on the left (both edges at 5.0 U) and flush against Up ↑ on the right (both edges
  at 7.0 U).  The previous 5.5 U position caused the RShift keycap to overlap `/` by
  0.5 U.
- **Preview: zoom limits removed** (`src/preview/viewer.html`): `minDistance` and
  `maxDistance` constraints on `OrbitControls` were removed so the camera can orbit
  right up to the model surface for close-up detail inspection.
- **CLI: script paths outside working directory** (`src/main.rs`): the `safe_path`
  CWD restriction on the input script argument was overly strict for CLI use — users
  should be able to run `rrcad --preview ../mykb/script.rb` or any absolute path.
  Removed the guard from `run_script`, `run_preview`, and `run_table`; export paths
  produced *inside* scripts remain confined by `safe_path` in `native.rs`.
- **Split TKL keyboard sample** (`samples/split_tkl_keyboard.rb`): replaced
  fraction-based pillar positions with diagonal midpoints between 4 adjacent
  key centres, giving ~13.5 mm clearance from every switch body edge
  (previously the positions fell directly under Cherry MX switch bodies and
  inside the Raspberry Pi Pico board footprints).  Both halves now have
  4 pillars each.

---

## [0.1.5] - 2026-03-26

### Fixed

- **MCP server: SIGSEGV from concurrent mRuby VMs** — `spawn_blocking` runs closures on a
  thread pool, so a timed-out tool call could leave a mRuby VM alive on one thread while a
  second call spawned a new VM on another. mRuby is not thread-safe; concurrent VMs caused
  SIGSEGV crashes. A new `MRUBY_EVAL_LOCK` (`std::sync::Mutex<()>`) is now acquired at the
  top of every `spawn_blocking` closure, serialising all mRuby/OCCT work regardless of how
  many concurrent calls arrive or how many timed-out threads are still running.
- **MCP server: TOCTOU port-binding race in `cad_preview`** — the old code bound a listener
  to discover a free port, dropped it, then had axum try to rebind the same port. Another
  process could steal the port in that window, causing a `panic!` inside the spawned task.
  The `tokio::net::TcpListener` is now kept alive and passed directly to a new
  `serve_with_listener()` on the axum server, eliminating both the race and a 200 ms sleep.
- **Split TKL keyboard sample** (`samples/split_tkl_keyboard.rb`): added M2.5 button-head
  counterbores (Ø4.8 mm, 1.5 mm deep) to all plate screw vias so screw heads sit flush with
  the plate top face.

### Changed

- **Split TKL keyboard — connectors, bosses, and manufacturing details** (`samples/split_tkl_keyboard.rb`):
  - Replaced RJ-45 inter-half connector with USB-C (safer for hot-plug; no VCC on interconnect
    cable). Left half: USB Micro host port at back wall ¼-width + USB-C interconnect at ¾-width.
    Right half: USB-C interconnect at ¼-width + USB-C host port at ¾-width.
  - Added wall slots for USB-C adapter boards (12×4.2 mm PCB, open-top insertion pocket,
    9×3.5 mm USB-C port opening in back wall).
  - Upgraded corner and mid-edge screw bosses to M2.5 heat-set copper insert compatibility
    (POST_R 2.5 → 3.2 mm, M2_R 1.2 → 1.6 mm; 3D print FIT_TOL 0.2 mm per side).
  - Added bottom-face lead-in step (0.5 mm box-cut method) on all switch cutouts to ease
    Cherry MX switch clip insertion from below.
  - Added `CHAMFER_CASE` chamfer to the solid tent wedge base (applied before fusing with the
    tilted case half to avoid BRepFilletAPI_MakeChamfer failures on complex fused geometry).
  - Preview changed from 2×2 parts layout to a fully assembled side-by-side view (plates
    seated in cases, left and right halves with a 20 mm gap).

### Added

- **MCP stress tests** (`tests/mcp_stress.rs`): 10 new tests covering sequential VM churn,
  error recovery, boundary inputs, deep boolean chains, geometry validation after operations,
  security prelude persistence, and a two-thread lock-serialisation proof.

---

## [0.1.4] - 2026-03-24

### Changed

- **Split TKL keyboard layout refinements** (`samples/split_tkl_keyboard.rb`):
  - Left Fn row (F1–F6) aligned with number row (` 1–6) while maintaining proper Tab/Home/Shift row key widths (1.5U, 1.75U, 2.25U respectively)
  - RJ-45 Ethernet ports repositioned to symmetrical 1/4-width positions on both left and right cases (mirrors USB positioning)

---

## [0.1.3] - 2026-03-24

### Added

- **Split TKL keyboard sample** (`samples/split_tkl_keyboard.rb`): complete
  86-key split TKL mechanical keyboard (Cherry MX, 19.05 mm pitch) with a
  compact right half (≈20.7 cm) that fits a 22 cm print bed. Layout features:
  single nav column, inverted-T arrow cluster on the bottom row,
  PrtSc/ScrLk/Pause on the Fn row alongside F7–F12.
- **M2.5 heat-set insert standoffs** for Raspberry Pi Pico mounting (4 bosses
  per side, 4 mm tall, 3.2 mm Ø press-fit holes). Left Pico rotated 90° so
  the micro-USB port faces the back wall; USB cutout Z-offset raised to align
  with the Pico PCB level.
- **Mid-edge M2 screw bosses** (2 per side) at verified switch-cutout-clear
  edge positions to improve plate–case rigidity beyond the four corner screws.
- **Central screw-less support pillar** (1 per side) at the plate midpoint —
  a solid post rising to 0.2 mm below the plate underside to resist flex under
  typing load without requiring a via hole through the plate.
- `doc/split_tkl_keyboard.stl` — 2×2 preview layout (cases + plates) for
  interactive 3D viewing on GitHub.

---

## [0.1.2] - 2026-03-24

### Added

- **Schmidt ball pen sample** (`samples/pen_schmidt.rb`): four-part pen body
  (barrel, tip, front cap, tail cap) demonstrating `cone`, `rotate`,
  boolean ops, and multi-part layout. Tip-to-barrel joint uses an L-shaped
  tenon & mortise (quarter-turn bayonet) with spring-relief cantilever tabs
  for tactile snap-fit installation. Exports STEP and STL.

### Changed

- **Preview render quality** (`src/preview/viewer.html`): replaced the ad-hoc
  three-directional-light rig (including a blue fill that caused unnatural
  colour casts) with ACES filmic tone mapping, a `RoomEnvironment` PBR
  ambient map, a `HemisphereLight` (cool sky / warm ground), and a single
  clean key light. Shadows upgraded to `PCFSoftShadowMap` at 2048 × 2048
  with bias to eliminate shadow acne.

### Fixed

- **`set_params()` backslash injection** (`src/ruby/vm.rs`): only
  double-quotes were escaped when building the `$_rrcad_params` Ruby hash
  literal from `--param` CLI values. A backslash in a value (e.g.
  `--param path=C:\dir`) produced an unterminated string literal. Backslashes
  are now escaped before double-quotes.
- **Memory-limit doc discrepancy** (`src/mcp/mod.rs`): module-level table
  said `512 MB` but the constant is `2 GB`. Updated to match actual value and
  clarified that the limit is applied once in `start()`, not per-call.

---

## [0.1.1] - 2026-03-23

### Fixed

- **`MRUBY_CONFIG` path doubling**: mruby's Rakefile already prepends
  `build_config/` when resolving `MRUBY_CONFIG`, so passing
  `build_config/rrcad` produced `build_config/build_config/rrcad.rb` and
  broke every CI build from scratch. Fixed by passing the bare name `rrcad`.
  The bug was masked locally by the cached `libmruby.a`.
- **Missing `mruby-compiler` in `mcp_safe` gembox**: `mrb_load_string()` —
  used by `glue.c` to evaluate DSL strings — is implemented in the
  `mruby-compiler` core gem, not in mruby's base C library. Omitting it
  caused a linker error on all clean builds. `mruby-compiler` (C-level
  parser) is distinct from `mruby-eval` (Ruby-level `Kernel#eval`), which
  remains excluded.

### Changed

- Added `scripts/clean-build.sh` and a `pre-push` git hook that
  automatically runs a clean mruby build when `build.rs` or
  `mruby_configs/` are in the outgoing commits, catching build-plumbing
  bugs before they reach CI.

---

## [0.1.0] - 2026-03-23

### Added

- **MCP server** (`rrcad --mcp`): exposes four tools over stdio JSON-RPC —
  `cad_eval`, `cad_export`, `cad_preview`, `cad_validate` — and two resources
  (`rrcad://api`, `rrcad://examples`). Compatible with Claude Desktop and
  Claude Code out of the box.
- MCP server configuration template (`.mcp.json.example`) for easy client setup.
- User guide (`doc/user-guide.md`) covering all run modes including the MCP server.

### Fixed

- **MCP stability**: `setrlimit(RLIMIT_AS)` was called inside every
  `spawn_blocking` closure, permanently capping the entire server process's
  virtual address space to 512 MB after the first tool call. Moved to a single
  call in `start()` and raised the limit to 2 GB so OCCT boolean operations no
  longer crash the server.

### Changed

- rrcad-specific mruby build configs (`rrcad.rb`, `mcp_safe.gembox`) moved from
  `vendor/mruby/build_config/` into `mruby_configs/` in the rrcad repo.
  `build.rs` now copies them into the submodule before invoking `rake`, keeping
  the vendored mruby tree pristine at tag `3.4.0`.

---

## [0.0.1] - 2026-03-23

Initial public release of **rrcad** — a Ruby DSL-driven 3D CAD language backed
by mRuby and OpenCASCADE Technology (OCCT).

### Added

**Core language & runtime**
- mRuby 3.4.0 vendored as a submodule; manual C FFI (`glue.c`) hides `mrb_value` from Rust
- OCCT geometry kernel bound via a hand-written `cxx` bridge (`bridge.h` / `bridge.cpp`)
- Interactive REPL with tab-completion and inline `help` reference
- Script execution mode (`rrcad script.rb`) and live browser preview (`rrcad --preview script.rb`)
- DSL prelude auto-loaded on VM startup

**Primitives**
- 3D solids: `box`, `cylinder`, `sphere`, `cone`, `torus`, `wedge`
- 2D sketch profiles: `rect`, `circle`, `polygon`, `ellipse`, `arc`
- Splines: `spline_2d`, `spline_3d` (with optional `tangents:` constraint)

**Transforms**
- `translate`, `rotate`, `scale` (uniform and non-uniform), `mirror`

**Modifiers**
- `fillet` (constant and variable-radius via Range syntax), `chamfer`, `chamfer_asym`
- `extrude` (with optional `draft:` angle and `twist_deg:`/`scale:` for twisted forms)
- `revolve`, `sweep`, `sweep` with `guide:` auxiliary spine

**Boolean & multi-shape operations**
- `fuse`, `cut`, `common`; `fuse_all`, `cut_all`
- `fragment` — partition overlapping solids into non-overlapping pieces

**Surface modeling**
- `loft`, `sweep_sections` — multi-section sweeps
- `ruled_surface`, `fill_surface` — NURBS surface generation
- `bezier_patch` + `sew` — Bézier patch assembly (Utah Teapot sample)
- `slice` — cross-section extraction

**Part design**
- `pad`, `pocket` — sketch-on-face feature operations
- `fillet_wire` — 2D wire/face corner rounding
- `datum_plane` — reference plane construction
- `helix`, `thread` — helical wire and thread groove
- `cbore`, `csink` — counterbore and countersink hole tools

**Patterns**
- `linear_pattern`, `polar_pattern`, `grid_pattern`
- `path_pattern` — arc-length-spaced copies along a wire path

**Assembly**
- `color(r, g, b)` — sRGB material tagging (written into GLB/glTF/OBJ)
- `mate` — face-based assembly mating with optional gap offset
- `assembly` builder with `place` and keyword `mate`

**Query & introspection**
- `shape_type`, `centroid`, `bounding_box`, `volume`, `surface_area`
- `closed?`, `manifold?`, `validate`
- `faces`, `edges`, `vertices` selectors (symbolic and CadQuery-style direction)
- `distance_to`, `inertia`, `min_thickness`
- `convex_hull`, `simplify`
- `offset`, `offset_2d`

**Export / Import**
- Export: STEP, STL, GLB (binary), glTF (text), OBJ, SVG (HLR projection), DXF R12
- Import: STEP, STL
- SVG/DXF support `:top`, `:front`, `:side` view selectors

**Parametric & batch**
- `param` DSL declaration with default and optional `range:` validation
- `--param name=value` CLI override
- `--design-table table.csv script.rb` batch export

**Security**
- `safe_path` guard on all file I/O — rejects path traversal and paths outside the working directory
- Randomised preview GLB path to prevent symlink attacks
- Integer overflow guard on mRuby→C integer casts

**Developer experience**
- 343 tests across 20 test files (unit, integration, and end-to-end)
- `rustfmt` and `clang-format` enforced automatically via Claude Code hooks
- `CLAUDE.md` with architecture, build instructions, and coding conventions
