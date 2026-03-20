# rrcad TODOs

A Ruby DSL-driven 3D CAD language, with Rust as the glue/performance layer,
mRuby as the embedded scripting engine, and OCCT as the geometry kernel.

---

## Phase 0 — OCCT Minimal Rust Bindings

Goal: build a box, do boolean ops, export STEP from Rust.

- [ ] Set up `cxx` crate and OCCT C++ bridge scaffolding
- [ ] Bind `BRepPrimAPI_MakeBox`, `MakeCylinder`, `MakeSphere`
- [ ] Bind `BRepAlgoAPI_Fuse`, `Cut`, `Common` (boolean ops)
- [ ] Bind `BRepFilletAPI_MakeFillet`, `MakeChamfer`
- [ ] Bind `BRepBuilderAPI_Transform` (translate, rotate, scale)
- [ ] Bind `STEPControl_Writer` for STEP export
- [ ] Bind `StlAPI_Writer` for STL export
- [ ] Bind `RWGltf_CafWriter` for glTF export (OCCT 7.7+)
- [ ] Write a smoke test: build a filleted box, export to STEP

---

## Phase 1 — mRuby Embedded in Rust

Goal: call `box(10,20,30).export("test.step")` from a Ruby script.

- [x] Add mRuby as a C dependency (via `mruby-sys` or manual FFI)
  - Vendored at `vendor/mruby` (submodule, pinned to 3.4.0); built via `rake` in `build.rs`
  - Manual FFI in `src/ruby/ffi.rs`; C glue shim in `src/ruby/glue.c` hides `mrb_value` from Rust
- [x] Bootstrap `mrb_open` / `mrb_close` lifecycle in Rust
  - `src/ruby/vm.rs`: `MrubyVm` struct with `new()` / `eval()` / `Drop`
- [ ] Define a `Shape` Ruby class backed by a Rust-side `SlotMap` of OCCT shapes
  - Use opaque u64 handle in mRuby `RData`; `dfree` callback drops from SlotMap
- [ ] Implement `box`, `cylinder`, `sphere` as top-level Ruby methods
- [ ] Implement `.export(path)` method on `Shape`
- [ ] Implement boolean op methods: `.fuse`, `.cut`, `.common`
- [ ] Execute a `.rb` script file from Rust CLI entrypoint (`rrcad script.rb`)
- [x] Interpreter / REPL mode (`rrcad` or `rrcad --repl`): readline loop wired to mRuby eval, prints `=> <result>`
- [ ] End-to-end test: `ruby_script → mRuby → Rust → OCCT → STEP file`

---

## Phase 2 — DSL Enrichment

Goal: expressive Ruby DSL with transforms, fillets, chamfers, and assemblies.

- [ ] `.fillet(r, edges: :all | :vertical | [...])` method
- [ ] `.chamfer(d, edges: ...)` method
- [ ] `.translate(x, y, z)` / `.rotate(axis, angle)` / `.scale(f)` methods
- [ ] `.mirror(:xy | :xz | :yz)` method
- [ ] `solid do ... end` block DSL
- [ ] `assembly "name" do ... end` block DSL with `place` / `mate` primitives
- [ ] 2D sketch primitives: `rect`, `circle`, `polygon`
- [ ] Extrude / revolve: `.extrude(h)`, `.revolve(angle)`
- [ ] Face/edge selectors: `.faces(:top)`, `.edges(:vertical)` returning sub-Shape handles
- [ ] Error messages: Ruby-level exceptions mapped to user-friendly messages

---

## Phase 3 — Live Preview (Three.js, Plan B)

Goal: save `.rb` → see 3D result in browser instantly.

- [ ] Tessellation pipeline: `BRepMesh_IncrementalMesh` → `Poly_Triangulation` → glTF
  - Two LOD levels: coarse for interaction, fine for final view
- [ ] `axum` HTTP server serving the glTF file and a static Three.js viewer page
- [ ] Three.js viewer (~100 lines):
  - `GLTFLoader` + `OrbitControls` + `GridHelper`
  - `EdgesGeometry` + `LineSegments` for CAD edge overlay
  - Coordinate axis gizmo
- [ ] WebSocket channel: server → browser "mesh updated, reload"
- [ ] `notify` crate watching `.rb` script for changes
- [ ] File-change event triggers: re-execute mRuby → tessellate → write glTF → WS notify
- [ ] `preview part` top-level Ruby method (or CLI flag `--preview`) to launch the server

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
  • mrb_define_class / mrb_define_method
  • Shape handle: SlotMap<u64, OcctShape>
  • dfree callback drops shape on GC
      │ cxx bridge (C++ ABI)
OCCT geometry kernel
  • BRep modeling
  • Tessellation
  • STEP / STL / glTF export
```

**Memory model:** Rust owns all OCCT shapes via `SlotMap`. mRuby `RData` holds
only a `u64` key; GC triggers `dfree` which removes the key and drops the shape.
No cross-language reference counting.

**Rendering (short-term):** OCCT tessellation → glTF → `axum` HTTP → Three.js browser viewer → WebSocket live reload.

**Rendering (long-term):** egui + wgpu native viewer once DSL is stable.
