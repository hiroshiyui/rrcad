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
stay portable. See `tests/teapot_dsl.rs` (7 tests), `tests/phase3_selectors.rs`
(16 tests), `samples/07_teapot.rb`.

---

## Phase 4 — Native Viewer (egui + wgpu, Plan C)

Goal: replace browser preview with a native desktop viewer with tighter integration.

- [ ] `egui` + `wgpu` + `winit` scaffold (via `eframe`)
- [ ] 3D viewport render pass: camera, orbit controls, mesh draw
- [ ] Face/edge picking (ray cast or ID buffer)
- [ ] Assembly tree panel (egui side panel)
- [ ] Clip plane / cross-section mode
- [ ] Parameter sliders wired back to mRuby globals
- [ ] Migrate tessellation output from glTF to direct wgpu vertex buffers

---

## Phase 5 — Parametric Design & Constraints

Goal: scripts with parameters, constraints, and design tables.

- [ ] `param :width, default: 10, range: 1..100` DSL
- [ ] Constraint solver integration (research options: SolveSpace lib, custom)
- [ ] Design table: vary params across rows, export batch of STEP files
- [ ] `--param width=20` CLI override

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

**Rendering (long-term):** egui + wgpu native viewer (Phase 4) once DSL is stable.
