---
name: code-review
description: Perform a project-wide code review covering security, correctness, code quality, documentation, UI/UX, and style.
---

When performing a project-wide code review, always follow these steps:

1. **Survey recent changes** — Run `git log --oneline -20` and skim the corresponding diffs to understand the scope of work before examining individual files.

2. **Security audit** — Apply the `security-audit` skill. Give particular attention to the following areas:
   - *MCP server:* Confirm that `safe_path()` is enforced on every export path, the 30-second `tokio::time::timeout` is in place, and no VM state is shared between tool calls.
   - *Ruby DSL evaluation:* Verify that the security prelude removes all dangerous `Kernel` methods (`system`, `exec`, `` ` ``, `open`, `eval`, and their equivalents) before any user script runs.
   - *FFI boundary:* Confirm that no raw pointer escapes the `dfree` / `RData` ownership contract, and that the Rust↔mRuby bridge is free from double-free and use-after-free bugs.

3. **Correctness and logic** — Review the Rust and C++ implementation for:
   - *Memory safety:* Every `unsafe` block must include a comment that states the invariant it relies on.
   - *OCCT bridge* (`bridge.h` / `bridge.cpp`): Verify that all OCCT calls are wrapped in proper exception handling, since unhandled C++ exceptions crossing the `cxx` boundary cause aborts.
   - *mRuby GC hazards:* Confirm that any `mrb_value` retained across a potential allocation is either protected with `mrb_gc_protect` or converted to a stable pointer before the allocation occurs.

4. **Code smells** — Flag any of the following:
   - Duplicated logic that should be extracted into a shared helper.
   - Functions exceeding roughly 60 lines without clear justification.
   - Magic numbers or hard-coded paths (e.g., `/tmp/rrcad_preview.glb` appearing in multiple places without a named constant).
   - `unwrap()` or `expect()` in non-test code where a meaningful error could be propagated instead.
   - Dead code or stale commented-out blocks.

5. **Test coverage** — Verify that:
   - New Rust logic has unit tests co-located in the same file, per project convention.
   - New DSL features have integration tests under `tests/`.
   - No test spawns threads that create additional mRuby VMs; the `RUST_TEST_THREADS=1` setting in `.cargo/config.toml` enforces single-threaded execution, and tests must not circumvent it.

6. **Documentation quality** — Confirm that:
   - Public Rust items that form the DSL API are covered by doc comments (`///`).
   - Non-obvious C++ bridge functions carry inline comments explaining the relevant OCCT behavior.
   - `CLAUDE.md`, `doc/user-guide.md`, and `doc/TODOs.md` are updated to reflect any new architectural or behavioral changes.

7. **UI/UX (preview server)** — Review the Three.js HTML and WebSocket reload flow for:
   - *Control usability:* orbit, zoom, wireframe toggle, flat-line view, and scene background controls should behave intuitively.
   - *Error states:* The browser should display a clear, user-facing message when the WebSocket disconnects or the GLB file fails to load.
   - *Console hygiene:* Normal use should produce no uncaught JavaScript errors.

8. **Code style** — Confirm that formatting rules are observed throughout:
   - *Rust:* Code must be `rustfmt`-clean. Hooks enforce this automatically, but verify that no suppression comments (`#[rustfmt::skip]`) were quietly added.
   - *C++:* Code must be `clang-format`-clean according to `.clang-format` (LLVM base, 100-column limit, 4-space indent).
   - Any `#[allow(clippy::...)]` suppression must be accompanied by a comment explaining why the lint is a false positive in that context.

9. **Report findings** — Present all identified issues grouped by category: Security, Correctness, Code Smell, Tests, Documentation, UI/UX, and Style. Assign each a severity of **Critical**, **High**, **Medium**, or **Low**. For every finding, include the file path and line number, a clear description of the problem, and a concrete recommendation for how to fix it.
