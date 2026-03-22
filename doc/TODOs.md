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

## Phase 4 — OCCT Coverage (OpenSCAD / CadQuery parity) — complete

Goal: close the gap between our DSL and what OpenSCAD / CadQuery expose from OCCT.

All 3-D operations shipped. Completed: `loft`, `.shell`, `.offset`, `.extrude(twist/scale)`; primitives `cone`, `torus`, `wedge`; 2D profiles `polygon`, `ellipse`, `arc`; import `import_step`, `import_stl`; query `.bounding_box`, `.volume`, `.surface_area`.

### Transforms
- [x] `.scale(sx, sy, sz)` — non-uniform scale; `BRepBuilderAPI_GTransform` with `gp_GTrsf`

### Selective modifiers
- [x] `.fillet(r, selector)` — fillet only edges matching a selector string
- [x] `.chamfer(d, selector)` — same for chamfers

### Patterns
- [x] `linear_pattern(shape, n, [dx, dy, dz])` — repeat shape n times along a vector
- [x] `polar_pattern(shape, n, angle_deg)` — rotate n copies around the Z axis

### Sub-shape selectors (extensions)
- [x] `vertices` selector on shapes (complement to existing `faces` / `edges`)
- [x] Direction-based face selector: `faces(">Z")` / `faces("<X")` (CadQuery style)

### Export additions
- [x] OBJ export — `RWObj_CafWriter` via `TKDEOBJ`; same XDE pipeline as glTF; `.export("out.obj")` dispatches automatically by extension

---

## OCCT API Improvements (existing code)

Targeted improvements to the current `bridge.cpp` implementation using newer OCCT APIs.
All items below are complete. Items requiring DSL changes have been moved to Phase 5.

### [x] Boolean operations — robustness and performance

**File:** `src/occt/bridge.cpp` — `shape_fuse`, `shape_cut`, `shape_common`

**Problem:** The current 2-argument constructors (`BRepAlgoAPI_Fuse(a, b)`) build
immediately with default settings. This silently fails on near-coincident faces
(e.g. two boxes sharing a wall), a common user mistake.

**Fix:** Switch to the builder-style API — set arguments/tools explicitly, then
configure before calling `Build()`:

```cpp
TopTools_ListOfShape args, tools;
args.Append(a.get());
tools.Append(b.get());
BRepAlgoAPI_Fuse op;
op.SetArguments(args);
op.SetTools(tools);
op.SetRunParallel(Standard_True); // OCCT 7.4+: use TBB thread pool
op.SetFuzzyValue(1e-6);           // tolerance for near-coincident geometry
op.Build();
if (!op.IsDone())
    throw std::runtime_error("BRepAlgoAPI_Fuse failed");
```

Apply the same pattern to `shape_cut` (`BRepAlgoAPI_Cut`) and `shape_common`
(`BRepAlgoAPI_Common`). `TopTools_ListOfShape` is already included.
No header changes needed.

---

### [x] Tessellation — parallel meshing

**File:** `src/occt/bridge.cpp` — `export_stl` and `make_xde_doc` (shared by glTF/GLB/OBJ)

**Problem:** `BRepMesh_IncrementalMesh` runs single-threaded by default. On complex
shapes (teapot, lofted bodies) tessellation is the dominant export cost.

**Fix:** Pass `/*isParallel=*/Standard_True` as the 5th constructor argument (OCCT 7.4+):

```cpp
BRepMesh_IncrementalMesh mesher(shape.get(), deflection,
                                /*isRelative=*/Standard_False,
                                /*angularDeflection=*/0.5,
                                /*isParallel=*/Standard_True);
```

Two call sites: the one in `export_stl` and the one in `make_xde_doc`.

---

### [x] GLB transform format — TRS instead of 4×4 matrix

**File:** `src/occt/bridge.cpp` — `export_glb`

**Problem:** `RWGltf_CafWriter` defaults to `RWGltf_WriterTrsfFormat_Compact`, which
emits a 4×4 matrix for node transforms. The glTF 2.0 spec recommends TRS
(translation/rotation/scale) decomposition; Three.js handles both but TRS is
lighter and more interoperable with animation tools.

**Fix:** One line before `writer.Perform`:

```cpp
#include <RWGltf_WriterTrsfFormat.hxx>  // already pulled in via RWGltf_CafWriter.hxx
writer.SetTransformationFormat(RWGltf_WriterTrsfFormat_TRS);
```

Confirmed available in the installed OCCT 7.9 headers.
Apply only to `export_glb` (the live-preview path); `export_gltf` can follow
the same pattern but is less critical.

---

### [x] Shape validity check before export

**File:** `src/occt/bridge.cpp` — `export_step` and `export_glb`

**Problem:** When OCCT geometry operations produce a topologically invalid shape
(degenerate faces, open shells, self-intersecting edges), the STEP/GLB writers
silently write a corrupt file with no diagnostic. Users see an empty or broken
model with no error message.

**Fix:** Run `BRepCheck_Analyzer` before writing and throw with a clear message:

```cpp
#include <BRepCheck_Analyzer.hxx>
BRepCheck_Analyzer checker(shape.get());
if (!checker.IsValid())
    throw std::runtime_error(
        "shape is topologically invalid (degenerate faces or open shells) "
        "— check upstream boolean operations or fillet radii");
```

Add at the top of `export_step` and `export_glb`. The check is fast for typical
CAD models (milliseconds). No new library dependency — `BRepCheck` is in `TKBRep`
which is already linked.

---

## ✓ Phase 5 — Parametric Design & Constraints (complete)

Goal: scripts with parameters, constraints, and design tables. All tiers complete.

### Tier 1 — Core parametric primitives (implement together)

#### [x] `param` DSL — `param :width, default: 10, range: 1..100`

Foundational primitive; everything else in Phase 5 depends on it. Pure Ruby DSL:
store declared parameters in a global `$params` hash, validate against `range:`,
expose values to the script. No OCCT involvement.

#### [x] `--param width=20` CLI override

Trivially small once `param` exists. Parse `key=value` pairs in `main.rs` and
inject them into the VM before `eval`. Ship together with the `param` DSL as a
single atomic unit.

---

### Tier 1 — Design table (natural next step after params)

#### [x] Design table: vary params across rows, export batch of STEP files

Read a CSV/TSV where each row is a parameter set; iterate rows, eval the script
once per row, export a named STEP file per row (e.g. `bracket_50mm.step`).
Useful for product families. No constraint solver required.

---

### Tier 2 — High visual impact, low risk

#### [x] GLB color/material support

**File:** `src/occt/bridge.cpp` — `make_xde_doc` or `export_glb`

**Problem:** All shapes export with the default grey material. Multi-part
assemblies need per-part colors for useful preview.

**Fix:** Attach colors to XDE shape labels via `XCAFDoc_ColorTool` before
`RWGltf_CafWriter::Perform`. Requires a new DSL method (`.color(r, g, b)`) to
carry color metadata through to the export path. Bridge change is mechanical;
DSL addition is small. Payoff is immediate and visible in the live preview.

---

### Tier 3 — Research-heavy, high value if done well

#### [x] Assembly mating (scoped constraint solver)

Scoped to face-based assembly mating rather than full sketch constraints.
Implemented `Shape#mate(from_face, to_face, offset=0.0)` and
`Assembly#mate(shape, from:, to:, offset: 0.0)` using OCCT geometry directly:

- Outward face normals computed from `Geom_Plane::Axis()` + `TopAbs_REVERSED` orientation.
- Face centroids via `BRepGProp::SurfaceProperties`.
- Rotation (pivoting around from-face centroid) aligns normals antiparallel.
- Translation maps from-face centroid onto to-face centroid.
- `offset` parameter shifts along the to-face outward normal (gap/interference).

Full SolveSpace / `slvs` sketch-level constraint integration remains out of scope —
the `param` DSL + design table covers parametric relationships adequately.

---

### Tier 4 — Nice-to-have, low urgency

#### [x] Spline tangent constraints

**File:** `src/occt/bridge.cpp` — `make_spline_2d_tan`, `make_spline_3d_tan`

`GeomAPI_Interpolate::Load(startTangent, endTangent)` is called before
`Perform()` when the optional `tangents:` keyword is supplied.

```ruby
spline_2d([[0,0],[5,10],[10,5]], tangents: [[1,0],[1,0]])
spline_3d([[0,0,0],[5,5,5],[10,0,0]], tangents: [[1,0,0],[1,0,0]])
```

2D tangents are XZ-plane vectors `[x, z]`; 3D tangents are `[x, y, z]`.
Magnitude is ignored — only direction matters. Without `tangents:` the
natural-boundary (cubic spline) interpolation is used unchanged.

#### [x] Feature removal — `.simplify`

`BRepAlgoAPI_Defeaturing` removes faces with area < `min_feature_size²`.
Returns the original shape unchanged if no faces qualify.

```ruby
part.simplify(1.0)   # remove features smaller than ~1 mm
```

---

## Utah Teapot Sample ✓

`samples/07_teapot.rb` is complete.  Uses `loft` (Phase 4) for the body,
`sweep` for spout and handle, `revolve` for the lid dome, and `sphere` for
the knob.  The body follows Newell cross-section radii/heights scaled to
OCCT coordinates (Z-up, height=7.5).  Validated by `tests/teapot_sample.rs`
(5 tests: 4 part-level + 1 full assembly).

The result is a geometric approximation of the Newell teapot — not an exact
Bezier-patch reproduction, but faithful to the published cross-section
silhouette data.

### Reference data

- **Primary source** — original Newell Bezier patch data and measurements:
  <https://users.cs.utah.edu/~dejohnso/models/teapot.html>
- **Bezier patches** (raw vertex + patch index file):
  <https://users.cs.utah.edu/~dejohnso/models/teapot_bezier>
- Utah Graphics Lab: The Utah Teapot
  <https://graphics.cs.utah.edu/teapot/>

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

See `CLAUDE.md` and `doc/development.md` for the authoritative architecture
and development guide. Key points:

- **Memory:** each `Shape` is a `Box<occt::Shape>` raw pointer in mRuby `RData void*`; `dfree` drops it on GC. No SlotMap, no reference counting.
- **Preview:** OCCT tessellation → GLB → `axum` HTTP → Three.js browser viewer → WebSocket live reload. Activated with `rrcad --preview <script.rb>`. Web-based preview is the long-term approach; a native egui/wgpu viewer is not planned.
