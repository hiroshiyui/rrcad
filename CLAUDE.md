# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rrcad** is a Ruby DSL-driven 3D CAD language. Users write `.rb` scripts; mRuby executes them; Rust binds mRuby to OCCT (the geometry kernel). See `doc/ROADMAP.md` for the full phased roadmap.

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

**Clean build** (required after changing `build.rs` or `mruby_configs/`):

```sh
./scripts/clean-build.sh
```

`cargo build` skips the mruby `rake` step when `vendor/mruby/build/host/lib/libmruby.a`
exists. The clean-build script deletes it first, mirroring what CI does on every run.
A `pre-push` git hook runs this automatically when either file is in the outgoing commits.

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
  • Shape logic split across focused Rust modules under src/occt/
  • STEP / STL / glTF (text) / GLB (binary) / OBJ export
  • 3MF export: C++ emits the model XML, Rust (src/occt/threemf.rs) writes the OPC ZIP
  • SVG / DXF 2-D drawings via HLR, incl. sheets, GD&T frames, section views
      │
Live preview               (src/preview/)
  • export_glb → <temp-dir>/rrcad_preview_<uuid>.glb (CLI --preview)
                 or /tmp/rrcad_mcp/preview.glb (MCP cad_preview)
  • axum HTTP: GET / (Three.js HTML), GET /model.glb, GET /metadata.json, GET /logo.png, GET /ws (WebSocket)
  • notify watches .rb script → re-eval → GLB → WS "reload"
```

**Multi-file scripts:** `require_relative` (`src/ruby/loader.rs` + `native_loader.rs`)
resolves against the requiring file's directory, evaluates each file once, and
records the load set so `--preview` can watch the whole project. It is a
file-read primitive: it refuses to run until a base directory is set (the CLI
sets one, MCP never does) *and* is undefined by the MCP security prelude. Both
guards are deliberate — keep both. Because mRuby propagates exceptions with
`longjmp`, `glue.c` catches them with `MRB_TRY`/`MRB_CATCH` so the include
stack unwinds with the C stack.

**Memory model:** Each native `Shape` is a heap-allocated `Box<occt::Shape>`. The raw pointer is stored directly in the mRuby `RData void*` slot — no SlotMap. The `dfree` GC callback drops the Box. No cross-language reference counting.

**Bridge invariants:** Keep OCCT bridge and mRuby lifetime changes one-way: Rust owns the `Shape` box, mRuby only holds the opaque pointer, `dfree` remains the only drop path, and live `MrubyVm` / `mrb_state` values must never be shared across threads.

## Key Technology Choices

- **mRuby FFI** — use raw C FFI (chosen; not `mruby-sys` or `mrusty`). Vendored at `vendor/mruby`; glue shim in `src/ruby/glue.c` hides `mrb_value` from Rust. Wire Ruby classes to Rust via `mrb_define_class` / `mrb_define_method`.
- **OCCT bindings** — use the `cxx` crate with a hand-written C++ bridge. Bind only what is needed incrementally; do not attempt full OCCT coverage. Header: `src/occt/bridge.h`, implementation: `src/occt/bridge.cpp`. Multi-shape bridge calls use a builder pattern (`ThruSectionsBuilder`, `FragmentBuilder`, `ShellOpenBuilder`, `StepAssemblyWriter`) because cxx cannot pass slices of opaque types. `text()` links `TKService` + `TKV3d` (`Font_BRepFont` aliases `StdPrs_BRepFont`, which lives in TKV3d despite the package name) and needs a system font at runtime — CI's slim container installs `libocct-visualization-dev` and `fonts-dejavu-core` for it.
- **Preview** — `axum` HTTP server + WebSocket + Three.js. OCCT tessellates to binary GLB via `RWGltf_CafWriter` (isBinary=true); `notify` watches the `.rb` script; `preview(shape)` writes the GLB and fires a WebSocket reload. Activated with `rrcad --preview <script.rb>`. `preview(shape)` is a no-op outside this mode. The web-based preview is the long-term approach; a native egui/wgpu viewer is not planned. `preview::start` must create the tokio runtime *before* binding the port — `bind_listener` ends in `TcpListener::from_std`, which panics without a reactor, and a `#[tokio::test]` cannot catch that because it supplies the missing runtime. A `metadata.json` sidecar written beside the GLB carries what the mesh cannot (validity, volume, named refs, feature graph) and drives the viewer's Model and Features panels.
- **Project config** — optional `rrcad.toml` files are loaded from the script directory or current working directory, walking up parent directories. Use `preview_port` for `--preview` defaults and `[params]` for default `param()` overrides; CLI flags still override the file.
- **Config template** — [`rrcad.toml.example`](rrcad.toml.example) is the repo template for standalone CAD projects. Copy it to `rrcad.toml` in your project root if you want to set default `preview_port` or `[params]` values.
- **MCP server** — `rmcp` crate (stdio transport). Public wiring lives in `src/mcp/mod.rs`; implementation is split across focused helpers in `src/mcp/`. A fresh `MrubyVm` is created per tool call (no shared state). Security prelude strips dangerous Kernel methods at runtime; each tool call runs in a one-shot worker child process (`src/mcp/worker.rs`) that the parent kills after 30 s and that caps itself at 2 GB via `setrlimit(RLIMIT_AS)`; CWD is changed to `/tmp/rrcad_mcp/` at startup so export paths satisfy `safe_path()`. Do not share a VM across requests.

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
