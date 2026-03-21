<p align="center"><img src="doc/images/rrcad-logo.png" alt="rrcad logo" width="400"></p>

# rrcad

> **Work in Progress** — this project is in early development and not yet ready for use.

A 3D CAD language expressed in Ruby. Write `.rb` scripts to describe solid geometry; the engine evaluates them through an embedded mRuby VM, builds exact BRep models with OpenCASCADE (OCCT), and exports to STEP, STL, or glTF.

```ruby
body = spline_2d([
  [0.0, 0.0], [2.8, 0.3], [3.5, 1.2], [4.2, 3.5],
  [3.8, 6.0], [3.2, 7.2], [2.8, 7.8], [0.0, 7.8],
]).revolve(360)

spout_path = spline_3d([[4.0,0.0,2.5],[5.5,0.0,3.5],[8.5,0.0,7.0]])
spout = circle(0.7).sweep(spout_path)

teapot = solid { body.fuse(spout) }
teapot.export("teapot.step")
```

See [`samples/`](samples/) for more complete examples.

![Web-based preview of rendered object](doc/images/Screenshot%202026-03-21%20at%2010-59-34%20rrcad%20preview.png)

## Stack

| Layer | Technology | Role |
|-------|-----------|------|
| DSL | mRuby | Embedded Ruby scripting engine |
| Glue | Rust | Binding layer, memory ownership, CLI |
| Geometry | OpenCASCADE (OCCT) | BRep modeling, boolean ops, export |
| Preview | Three.js + axum | Browser-based live 3D viewer (`--preview` mode) |

## Architecture

```
Ruby DSL (.rb script)
      │ mRuby VM
Rust binding layer
  • native.rs: extern "C" entry points called from glue.c
  • glue.c: C shim hiding mrb_value from Rust
  • Shape: Box<occt::Shape> raw pointer stored in mRuby RData void*
  • dfree callback drops the Box when mRuby GC collects the object
      │ cxx bridge (C++ ABI)
OCCT geometry kernel
  • BRep modeling & boolean ops
  • Tessellation (BRepMesh_IncrementalMesh)
  • STEP / STL / glTF export
```

**Memory model:** Each native `Shape` value wraps a heap-allocated `Box<occt::Shape>`. The raw pointer is stored directly in the mRuby `RData void*` slot. The `dfree` GC callback calls `rrcad_shape_drop` to run `drop(Box::from_raw(ptr))`. No SlotMap, no cross-language reference counting.

## Building

```sh
cargo build
cargo run                          # start REPL
cargo run -- script.rb             # run a .rb script
cargo run -- --preview script.rb   # live browser preview (auto-reloads on save)
cargo test
```

Requires OCCT 7.7+ headers and libraries, and mRuby built as a static library. See [`doc/development.md`](doc/development.md) for full build setup instructions.

## Roadmap

See [`doc/TODOs.md`](doc/TODOs.md) for the phased implementation plan:

- **Phase 0** ✓ — OCCT Rust bindings via `cxx` (primitives, boolean ops, fillets, transforms, STEP/STL/glTF export)
- **Phase 1** ✓ — mRuby embedded; end-to-end Ruby → STEP; REPL with tab-completion and `help`
- **Phase 2** ✓ — DSL enrichment (transforms, fillets, mirror, assemblies, sketches, extrude/revolve)
- **Phase 3** ✓ — Spline profiles + sweep; sub-shape selectors (`.faces`, `.edges`); live browser preview (`axum` + Three.js + WebSocket file watcher; `--preview` CLI)
- **Phase 4** (in progress) — OCCT coverage: `cone`, `torus`, `wedge`, `polygon`, `ellipse`, `arc` done; `loft`, `shell`, `offset`, selective fillet/chamfer, patterns, STEP import, bounding box / volume queries remaining
- **Phase 5** — Parametric design and constraints

## Third-party components

| Component | Version | License |
|-----------|---------|---------|
| [mRuby](https://github.com/mruby/mruby) | 3.4.0 | [MIT](https://github.com/mruby/mruby/blob/master/LICENSE) |
| [OpenCASCADE (OCCT)](https://dev.opencascade.org/) | 7.9 (system) | [LGPL-2.1 with OCCT exception](https://dev.opencascade.org/doc/overview/html/occt__lgpl__exception_8txt.html) |

## License

rrcad is free software: you can redistribute it and/or modify it under the
terms of the **GNU General Public License version 3 or later** (GPL-3.0-or-later)
as published by the Free Software Foundation.

See [`LICENSE`](LICENSE) for the full license text.
