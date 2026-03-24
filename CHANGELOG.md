# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] - 2026-03-24

### Changed

- **Split TKL keyboard layout refinements** (`samples/split_tkl_keyboard.rb`):
  - Left Fn row (F1–F6) aligned with number row (` 1–6) while maintaining proper Tab/Home/Shift row key widths (1.5U, 1.75U, 2.25U respectively)
  - RJ-45 Ethernet ports repositioned to symmetrical 1/4-width positions on both left and right cases (mirrors USB positioning)

---

## [0.1.3] - 2026-03-24

### Added

- **Split TKL keyboard sample** (`samples/split_tkl_keyboard.rb`): complete
  86-key split TKL mechanical keyboard (Cherry MX, 19.05 mm pitch) with a
  compact right half (≈20.7 cm) that fits a 22 cm print bed. Layout features:
  single nav column, inverted-T arrow cluster on the bottom row,
  PrtSc/ScrLk/Pause on the Fn row alongside F7–F12.
- **M2.5 heat-set insert standoffs** for Raspberry Pi Pico mounting (4 bosses
  per side, 4 mm tall, 3.2 mm Ø press-fit holes). Left Pico rotated 90° so
  the micro-USB port faces the back wall; USB cutout Z-offset raised to align
  with the Pico PCB level.
- **Mid-edge M2 screw bosses** (2 per side) at verified switch-cutout-clear
  edge positions to improve plate–case rigidity beyond the four corner screws.
- **Central screw-less support pillar** (1 per side) at the plate midpoint —
  a solid post rising to 0.2 mm below the plate underside to resist flex under
  typing load without requiring a via hole through the plate.
- `doc/split_tkl_keyboard.stl` — 2×2 preview layout (cases + plates) for
  interactive 3D viewing on GitHub.

---

## [0.1.2] - 2026-03-24

### Added

- **Schmidt ball pen sample** (`samples/pen_schmidt.rb`): four-part pen body
  (barrel, tip, front cap, tail cap) demonstrating `cone`, `rotate`,
  boolean ops, and multi-part layout. Tip-to-barrel joint uses an L-shaped
  tenon & mortise (quarter-turn bayonet) with spring-relief cantilever tabs
  for tactile snap-fit installation. Exports STEP and STL.

### Changed

- **Preview render quality** (`src/preview/viewer.html`): replaced the ad-hoc
  three-directional-light rig (including a blue fill that caused unnatural
  colour casts) with ACES filmic tone mapping, a `RoomEnvironment` PBR
  ambient map, a `HemisphereLight` (cool sky / warm ground), and a single
  clean key light. Shadows upgraded to `PCFSoftShadowMap` at 2048 × 2048
  with bias to eliminate shadow acne.

### Fixed

- **`set_params()` backslash injection** (`src/ruby/vm.rs`): only
  double-quotes were escaped when building the `$_rrcad_params` Ruby hash
  literal from `--param` CLI values. A backslash in a value (e.g.
  `--param path=C:\dir`) produced an unterminated string literal. Backslashes
  are now escaped before double-quotes.
- **Memory-limit doc discrepancy** (`src/mcp/mod.rs`): module-level table
  said `512 MB` but the constant is `2 GB`. Updated to match actual value and
  clarified that the limit is applied once in `start()`, not per-call.

---

## [0.1.1] - 2026-03-23

### Fixed

- **`MRUBY_CONFIG` path doubling**: mruby's Rakefile already prepends
  `build_config/` when resolving `MRUBY_CONFIG`, so passing
  `build_config/rrcad` produced `build_config/build_config/rrcad.rb` and
  broke every CI build from scratch. Fixed by passing the bare name `rrcad`.
  The bug was masked locally by the cached `libmruby.a`.
- **Missing `mruby-compiler` in `mcp_safe` gembox**: `mrb_load_string()` —
  used by `glue.c` to evaluate DSL strings — is implemented in the
  `mruby-compiler` core gem, not in mruby's base C library. Omitting it
  caused a linker error on all clean builds. `mruby-compiler` (C-level
  parser) is distinct from `mruby-eval` (Ruby-level `Kernel#eval`), which
  remains excluded.

### Changed

- Added `scripts/clean-build.sh` and a `pre-push` git hook that
  automatically runs a clean mruby build when `build.rs` or
  `mruby_configs/` are in the outgoing commits, catching build-plumbing
  bugs before they reach CI.

---

## [0.1.0] - 2026-03-23

### Added

- **MCP server** (`rrcad --mcp`): exposes four tools over stdio JSON-RPC —
  `cad_eval`, `cad_export`, `cad_preview`, `cad_validate` — and two resources
  (`rrcad://api`, `rrcad://examples`). Compatible with Claude Desktop and
  Claude Code out of the box.
- MCP server configuration template (`.mcp.json.example`) for easy client setup.
- User guide (`doc/user-guide.md`) covering all run modes including the MCP server.

### Fixed

- **MCP stability**: `setrlimit(RLIMIT_AS)` was called inside every
  `spawn_blocking` closure, permanently capping the entire server process's
  virtual address space to 512 MB after the first tool call. Moved to a single
  call in `start()` and raised the limit to 2 GB so OCCT boolean operations no
  longer crash the server.

### Changed

- rrcad-specific mruby build configs (`rrcad.rb`, `mcp_safe.gembox`) moved from
  `vendor/mruby/build_config/` into `mruby_configs/` in the rrcad repo.
  `build.rs` now copies them into the submodule before invoking `rake`, keeping
  the vendored mruby tree pristine at tag `3.4.0`.

---

## [0.0.1] - 2026-03-23

Initial public release of **rrcad** — a Ruby DSL-driven 3D CAD language backed
by mRuby and OpenCASCADE Technology (OCCT).

### Added

**Core language & runtime**
- mRuby 3.4.0 vendored as a submodule; manual C FFI (`glue.c`) hides `mrb_value` from Rust
- OCCT geometry kernel bound via a hand-written `cxx` bridge (`bridge.h` / `bridge.cpp`)
- Interactive REPL with tab-completion and inline `help` reference
- Script execution mode (`rrcad script.rb`) and live browser preview (`rrcad --preview script.rb`)
- DSL prelude auto-loaded on VM startup

**Primitives**
- 3D solids: `box`, `cylinder`, `sphere`, `cone`, `torus`, `wedge`
- 2D sketch profiles: `rect`, `circle`, `polygon`, `ellipse`, `arc`
- Splines: `spline_2d`, `spline_3d` (with optional `tangents:` constraint)

**Transforms**
- `translate`, `rotate`, `scale` (uniform and non-uniform), `mirror`

**Modifiers**
- `fillet` (constant and variable-radius via Range syntax), `chamfer`, `chamfer_asym`
- `extrude` (with optional `draft:` angle and `twist_deg:`/`scale:` for twisted forms)
- `revolve`, `sweep`, `sweep` with `guide:` auxiliary spine

**Boolean & multi-shape operations**
- `fuse`, `cut`, `common`; `fuse_all`, `cut_all`
- `fragment` — partition overlapping solids into non-overlapping pieces

**Surface modeling**
- `loft`, `sweep_sections` — multi-section sweeps
- `ruled_surface`, `fill_surface` — NURBS surface generation
- `bezier_patch` + `sew` — Bézier patch assembly (Utah Teapot sample)
- `slice` — cross-section extraction

**Part design**
- `pad`, `pocket` — sketch-on-face feature operations
- `fillet_wire` — 2D wire/face corner rounding
- `datum_plane` — reference plane construction
- `helix`, `thread` — helical wire and thread groove
- `cbore`, `csink` — counterbore and countersink hole tools

**Patterns**
- `linear_pattern`, `polar_pattern`, `grid_pattern`
- `path_pattern` — arc-length-spaced copies along a wire path

**Assembly**
- `color(r, g, b)` — sRGB material tagging (written into GLB/glTF/OBJ)
- `mate` — face-based assembly mating with optional gap offset
- `assembly` builder with `place` and keyword `mate`

**Query & introspection**
- `shape_type`, `centroid`, `bounding_box`, `volume`, `surface_area`
- `closed?`, `manifold?`, `validate`
- `faces`, `edges`, `vertices` selectors (symbolic and CadQuery-style direction)
- `distance_to`, `inertia`, `min_thickness`
- `convex_hull`, `simplify`
- `offset`, `offset_2d`

**Export / Import**
- Export: STEP, STL, GLB (binary), glTF (text), OBJ, SVG (HLR projection), DXF R12
- Import: STEP, STL
- SVG/DXF support `:top`, `:front`, `:side` view selectors

**Parametric & batch**
- `param` DSL declaration with default and optional `range:` validation
- `--param name=value` CLI override
- `--design-table table.csv script.rb` batch export

**Security**
- `safe_path` guard on all file I/O — rejects path traversal and paths outside the working directory
- Randomised preview GLB path to prevent symlink attacks
- Integer overflow guard on mRuby→C integer casts

**Developer experience**
- 343 tests across 20 test files (unit, integration, and end-to-end)
- `rustfmt` and `clang-format` enforced automatically via Claude Code hooks
- `CLAUDE.md` with architecture, build instructions, and coding conventions
