<p align="center"><img src="doc/images/rrcad-logo.png" alt="rrcad logo" width="400"></p>

# rrcad

A 3D CAD language expressed in Ruby. Write `.rb` scripts to describe solid geometry; the engine evaluates them through an embedded mRuby VM, builds exact BRep models with OpenCASCADE (OCCT), and exports to STEP, STL, 3MF, glTF/GLB, OBJ, and 2-D SVG/DXF drawings.

```ruby
# Outer cylinder — radius 30 mm, height 80 mm
bucket = cylinder(30, 80)

# Pocket the top face 70 mm deep, leaving 5 mm walls and a 10 mm base
bucket = bucket.pocket(:top, depth: 70) { circle(25) }

bucket.export("bucket.stl")
preview bucket
```

See [`samples/`](samples/) for more complete examples.

For standalone CAD projects, copy [`rrcad.toml.example`](rrcad.toml.example)
to `rrcad.toml` in your project root to set default `preview_port` and
`[params]` values.

> **3D preview:** open [`doc/bucket.stl`](doc/bucket.stl) or [`doc/split_tkl_keyboard.stl`](doc/split_tkl_keyboard.stl) in GitHub to view the models interactively in your browser.

![Three.js live preview of the bucket sample](doc/images/Screenshot%202026-03-22%20at%2014-39-04%20rrcad%20preview.png)
![Three.js live preview of the split TKL keyboard — fully assembled side-by-side view](doc/images/Screenshot%202026-03-26%20at%2009-03-09%20rrcad%20preview.png)

## Stack

| Layer | Technology | Role |
|-------|-----------|------|
| DSL | mRuby | Embedded Ruby scripting engine |
| Glue | Rust | Binding layer, memory ownership, CLI |
| Geometry | OpenCASCADE (OCCT) | BRep modeling, boolean ops, export |
| Preview | Three.js + axum | Browser-based live 3D viewer (`--preview` mode) |
| MCP | rmcp + stdio | Model Context Protocol server (`--mcp` mode) |

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
  • Hidden-line removal for 2-D drawings (HLRBRep_PolyAlgo)
  • STEP / STL / 3MF / glTF / GLB / OBJ / SVG / DXF export
```

The OCCT Rust layer is intentionally split across small modules under
`src/occt/` so that shape construction, feature history, diagnostics, preview
metadata, and export helpers can evolve independently.

**Memory model:** Each native `Shape` value wraps a heap-allocated `Box<occt::Shape>`. The raw pointer is stored directly in the mRuby `RData void*` slot. The `dfree` GC callback calls `rrcad_shape_drop` to run `drop(Box::from_raw(ptr))`. No SlotMap, no cross-language reference counting.

## Install

Build and install the release binary into Cargo's default installation path:

```sh
cargo build --release && cargo install --path .
```

This uses the optimized release build and then installs the `rrcad` binary into
the standard Cargo bin directory for your user account.

## Running

```sh
rrcad                              # start REPL
rrcad script.rb                    # run a .rb script
rrcad --preview script.rb          # live browser preview (auto-reloads on save)
rrcad --mcp                        # MCP server over stdio (for Claude Desktop / Claude Code)
```

If you have not installed the binary yet, the equivalent `cargo run` forms are
`cargo run`, `cargo run -- script.rb`, `cargo run -- --preview script.rb`, and
`cargo run -- --mcp`.

For a task-oriented walkthrough of the DSL with worked examples for CAD
engineers, see the [User Guide](doc/user-guide.md).

## Building

```sh
cargo build
cargo test
```

For long-running commands, `./scripts/observe.sh cargo test` tees output to a
timestamped log file under `/tmp/rrcad-logs` and prints a heartbeat while the
command is still running.

Requires OCCT 7.7+ headers and libraries, and mRuby built as a static library. See [`doc/development.md`](doc/development.md) for full build setup instructions.

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
