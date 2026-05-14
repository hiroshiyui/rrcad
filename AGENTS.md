# Repository Guidelines

## Project Structure & Module Organization

`rrcad` is a Rust 2024 CAD language runtime with embedded mRuby and OCCT bindings. Core Rust code lives in `src/`: `main.rs` handles CLI modes, `lib.rs` exports modules, `ruby/` embeds mRuby and the DSL prelude, `occt/` contains the `cxx` bridge plus C++ geometry code, `preview/` serves the browser viewer, and `mcp/` implements MCP tools. Integration tests are in `tests/`. Example Ruby CAD scripts are in `samples/`, documentation is in `doc/`, mRuby build configuration is in `mruby_configs/`, and the pinned mRuby submodule is under `vendor/mruby/`.

## Build, Test, and Development Commands

Use `/bin/zsh` when invoking shell commands for this workspace.

- `cargo build`: builds Rust, the C shim, OCCT bridge, and `vendor/mruby` if needed.
- `cargo run`: starts the interactive REPL.
- `cargo run -- samples/01_hello_box.rb`: runs a DSL script.
- `cargo run -- --preview samples/06_live_preview.rb`: starts live browser preview.
- `cargo run -- --mcp`: starts the MCP server over stdio.
- `cargo test`: runs all integration and unit tests.
- `cargo test --test phase8_tier1`: runs one integration test file.
- `cargo clippy`: runs Rust lints.
- `./scripts/clean-build.sh`: removes cached `libmruby.a` and verifies a clean build.

## Coding Style & Naming Conventions

Use `rustfmt` formatting for Rust and keep `cargo clippy` clean before submitting. Follow existing Rust naming: `snake_case` for functions/modules, `PascalCase` for types, and descriptive integration test filenames such as `phase7_tier2.rs`. Keep C++ bridge declarations in `src/occt/bridge.h` synchronized with implementations in `bridge.cpp` and Rust declarations in `src/occt/mod.rs`. Prefer small, focused DSL additions in `src/ruby/prelude.rb`, `glue.c`, and `native.rs`.

## Testing Guidelines

Add or update integration tests in `tests/` for every new DSL or geometry operation. Use targeted runs while developing, then run `cargo test` before handoff. For build-system or mRuby configuration changes, also run `./scripts/clean-build.sh`.

## Commit & Pull Request Guidelines

Recent history uses Conventional Commit-style messages, for example `fix(preview): ...`, `test(cut_all): ...`, `refactor: ...`, and `chore: release v0.2.1`. Keep commits scoped and imperative. Pull requests should describe behavior changes, list test commands run, link relevant issues, and include screenshots or generated model notes for preview/UI-visible changes.

## Security & Configuration Tips

Do not bypass MCP sandbox constraints or export path confinement. Avoid committing generated CAD outputs unless they are intentional fixtures or documentation assets.

## Agent-Specific Instructions

At session start, load user-defined skills from `.claude/skills/` and apply any relevant `SKILL.md` guidance before making changes.
