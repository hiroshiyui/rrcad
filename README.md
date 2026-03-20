<p align="center"><img src="doc/rrcad-logo.png" alt="rrcad logo" width="400"></p>

# rrcad

> **Work in Progress** — this project is in early development and not yet ready for use.

A 3D CAD language expressed in Ruby. Write `.rb` scripts to describe solid geometry; the engine evaluates them through an embedded mRuby VM, builds exact BRep models with OpenCASCADE (OCCT), and exports to STEP, STL, or glTF.

```ruby
part = solid do
  box 10, 20, 30
  fillet 2, edges: :vertical
  cut do
    cylinder r: 5, h: 40, at: [5, 10, 0]
  end
end

part.export("part.step")
preview part   # opens live browser viewer
```

## Stack

| Layer | Technology | Role |
|-------|-----------|------|
| DSL | mRuby | Embedded Ruby scripting engine |
| Glue | Rust | Binding layer, memory ownership, CLI |
| Geometry | OpenCASCADE (OCCT) | BRep modeling, boolean ops, export |
| Preview | Three.js + axum | Browser-based live 3D viewer |

## Architecture

```
Ruby DSL (.rb script)
      │ mRuby VM
Rust binding layer
  • Shape handle: SlotMap<u64, OcctShape>
  • mrb_define_class / mrb_define_method
  • dfree callback drops shape on GC
      │ cxx bridge (C++ ABI)
OCCT geometry kernel
  • BRep modeling & boolean ops
  • Tessellation (BRepMesh_IncrementalMesh)
  • STEP / STL / glTF export
```

**Memory model:** Rust owns all OCCT shapes via a `SlotMap`. mRuby `RData` holds only a `u64` key; GC triggers `dfree` which removes the key and drops the shape. No cross-language reference counting.

**Live preview:** On `preview part`, rrcad tessellates the model to glTF, starts an `axum` HTTP server, and opens a Three.js viewer in the browser. A `notify` watcher re-evaluates the script on every save and pushes the new mesh over WebSocket.

## Building

```sh
cargo build
cargo run -- script.rb
cargo run -- --preview script.rb
cargo test
```

Requires OCCT 7.7+ headers and libraries, and mRuby built as a static library. See [`doc/development.md`](doc/development.md) for full build setup instructions.

## Roadmap

See [`doc/TODOs.md`](doc/TODOs.md) for the phased implementation plan:

- **Phase 0** ✓ — OCCT Rust bindings via `cxx` (primitives, boolean ops, fillets, transforms, STEP/STL/glTF export)
- **Phase 1** — mRuby embedded; end-to-end Ruby → STEP
- **Phase 2** — DSL enrichment (fillets, assemblies, sketches, extrude/revolve)
- **Phase 3** — Live browser preview (Three.js + WebSocket file watcher)
- **Phase 4** — Native egui + wgpu desktop viewer
- **Phase 5** — Parametric design and constraints

## Third-party components

| Component | Version | License |
|-----------|---------|---------|
| [mRuby](https://github.com/mruby/mruby) | 3.4.0 | [MIT](https://github.com/mruby/mruby/blob/master/LICENSE) |

## License

rrcad is free software: you can redistribute it and/or modify it under the
terms of the **GNU General Public License version 3 or later** (GPL-3.0-or-later)
as published by the Free Software Foundation.

See [`LICENSE`](LICENSE) for the full license text.
