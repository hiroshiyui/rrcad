# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
