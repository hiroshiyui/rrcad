# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rrcad** is a Ruby DSL-driven 3D CAD language. Users write `.rb` scripts; mRuby executes them; Rust binds mRuby to OCCT (the geometry kernel). See `doc/TODOs.md` for the full phased roadmap.

## Build & Run

```sh
cargo build
cargo run                          # start REPL
cargo run -- script.rb             # run a script
cargo run -- --preview script.rb   # live browser preview (auto-reloads on save)
cargo run -- --mcp                 # MCP server over stdio (Claude Desktop / Claude Code)
cargo test
cargo test <test_name>   # run a single test by name substring
cargo clippy
```

## Architecture

```
Ruby DSL (.rb script)
      │ mRuby VM
Rust binding layer          (src/ruby/)
  • glue.c: C shim hiding mrb_value from Rust
  • native.rs: extern "C" entry points
  • Shape: Box<occt::Shape> raw pointer in mRuby RData void*
  • dfree callback drops shape on GC
      │ cxx bridge (C++ ABI)
OCCT geometry kernel        (src/occt/)
  • BRep modeling, splines, tessellation
  • STEP / STL / glTF (text) / GLB (binary) export
      │
Live preview               (src/preview/)
  • export_glb → /tmp/rrcad_preview.glb
  • axum HTTP: GET / (Three.js HTML), GET /model.glb, GET /ws (WebSocket)
  • notify watches .rb script → re-eval → GLB → WS "reload"
```

**Memory model:** Each native `Shape` is a heap-allocated `Box<occt::Shape>`. The raw pointer is stored directly in the mRuby `RData void*` slot — no SlotMap. The `dfree` GC callback drops the Box. No cross-language reference counting.

## Key Technology Choices

- **mRuby FFI** — use raw C FFI (chosen; not `mruby-sys` or `mrusty`). Vendored at `vendor/mruby`; glue shim in `src/ruby/glue.c` hides `mrb_value` from Rust. Wire Ruby classes to Rust via `mrb_define_class` / `mrb_define_method`.
- **OCCT bindings** — use the `cxx` crate with a hand-written C++ bridge. Bind only what is needed incrementally; do not attempt full OCCT coverage. Header: `src/occt/bridge.h`, implementation: `src/occt/bridge.cpp`.
- **Preview** — `axum` HTTP server + WebSocket + Three.js. OCCT tessellates to binary GLB via `RWGltf_CafWriter` (isBinary=true); `notify` watches the `.rb` script; `preview(shape)` writes the GLB and fires a WebSocket reload. Activated with `rrcad --preview <script.rb>`. `preview(shape)` is a no-op outside this mode. The web-based preview is the long-term approach; a native egui/wgpu viewer is not planned.
- **MCP server** — `rmcp` crate (stdio transport). All logic in `src/mcp/mod.rs`. A fresh `MrubyVm` is created per tool call (no shared state). Security prelude strips dangerous Kernel methods at runtime; `tokio::time::timeout` enforces 30 s limit; CWD is changed to `/tmp/rrcad_mcp/` at startup so export paths satisfy `safe_path()`. Do not share a VM across requests.

## While Coding

- When coding, provide sufficient comments to help other developers understand the logic.
- Follow Rust conventions by writing tests in the same file as the source code.
- Implement mRuby-specific tests under the `tests/` directory.

## Testing Notes

- mRuby is not thread-safe. `.cargo/config.toml` sets `RUST_TEST_THREADS=1` so `cargo test` runs all
  test binaries single-threaded. Do not remove this — parallel mRuby VMs will SIGSEGV.

## Code Style

Formatting is enforced automatically by hooks in `.claude/settings.json` — no manual step needed.

- **Rust** — `rustfmt` runs automatically on every `*.rs` file after each write or edit. `cargo clippy` must also pass clean.
- **C++** — `clang-format -i` runs automatically on every `*.h` / `*.cpp` file after each write or edit.
  Config is in `.clang-format` (LLVM base, 100-col, 4-space indent).

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
