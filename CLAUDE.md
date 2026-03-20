# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rrcad** is a Ruby DSL-driven 3D CAD language. Users write `.rb` scripts; mRuby executes them; Rust binds mRuby to OCCT (the geometry kernel). See `doc/TODOs.md` for the full phased roadmap.

## Build & Run

```sh
cargo build
cargo run
cargo test
cargo test <test_name>   # run a single test by name substring
cargo clippy
```

## Planned Architecture

```
Ruby DSL (.rb script)
      │ mRuby VM
Rust binding layer          (src/ruby/)
  • mrb_define_class / mrb_define_method
  • Shape handle: SlotMap<u64, OcctShape>
  • dfree callback drops shape on GC
      │ cxx bridge (C++ ABI)
OCCT geometry kernel        (src/occt/)
  • BRep modeling, tessellation
  • STEP / STL / glTF export
```

**Memory model:** Rust owns all OCCT shapes via a `SlotMap`. mRuby `RData` holds only a `u64` key; GC triggers a `dfree` callback that removes the key and drops the shape. No cross-language reference counting.

## Key Technology Choices

- **mRuby FFI** — use `mruby-sys` or raw C FFI (not the `mrusty` crate). Wire Ruby classes to Rust via `mrb_define_class` / `mrb_define_method`.
- **OCCT bindings** — use the `cxx` crate with a hand-written C++ bridge. Bind only what is needed incrementally; do not attempt full OCCT coverage. Header: `src/occt/bridge.h`, implementation: `src/occt/bridge.cpp`.
- **Preview (short-term)** — `axum` HTTP server + WebSocket + Three.js in the browser. OCCT tessellates to glTF via `RWGltf_CafWriter`; `notify` crate watches `.rb` files and pushes reload events over WebSocket.
- **Preview (long-term)** — `egui` + `wgpu` native viewer once the DSL stabilizes.

## Code Style

- **Rust** — standard `rustfmt`; `cargo clippy` must pass clean.
- **C++** — run `clang-format -i` on every C++ file you write or modify before finishing.
  Config is in `.clang-format` (LLVM base, 100-col, 4-space indent).
  Check with: `clang-format --dry-run -Werror src/occt/bridge.h src/occt/bridge.cpp`

## DSL Style

Prefer the Ruby block/builder style:

```ruby
part = solid do
  box 10, 20, 30
  fillet 2, edges: :vertical
  cut do
    cylinder r: 5, h: 40, at: [5, 10, 0]
  end
end
part.export("part.step")
preview part
```
