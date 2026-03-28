# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Preview: studio photography look** (`src/preview/viewer.html`): transformed the
  live viewer into a dark product-photography studio — vertical gradient background
  (cool dark top → faintly warm bottom), a large polished studio floor plane that
  catches shadows and shows faint model reflection, multi-light rig (key/fill/rim/bounce),
  40° telephoto FOV (less distortion than 45°), CSS vignette overlay for cinematic
  corners, per-model shadow frustum fitted to bounding box for crisp shadows, orbit
  floor clamp so the camera cannot go below the studio floor, axes helper (press A to
  toggle), and ACES filmic tone mapping at 1.2× exposure.
- **Preview: tighter camera fit** (`src/preview/viewer.html`): `fitCamera` multipliers
  reduced from 2.5 × 1.4 to 1.15 × 1.0, so the model fills the frame closely on every
  load rather than sitting in a wide empty viewport.
- **Split TKL keyboard — flat-bottom case** (`samples/split_tkl_keyboard.rb`): removed
  the `solid_tent` wedge base entirely.  The case now has a flat bottom; users glue on
  custom-printed tenting feet at whatever angle they prefer.
- **Split TKL keyboard — right bottom-row modifiers widened** (`samples/split_tkl_keyboard.rb`):
  RAlt, Fn, and RCtrl widened from 1 U to 1.25 U each; Space repositioned to 1 U at the
  left edge of the row (flush with the case wall) so all three modifiers pack tightly
  against the spacebar.  Leaves a natural 1.25 U gap before the arrow cluster.

### Fixed

- **Split TKL keyboard — RShift position corrected** (`samples/split_tkl_keyboard.rb`):
  RShift switch centre moved from 5.5 U to 6.0 U so the 2 U keycap sits flush against
  `/` on the left (both edges at 5.0 U) and flush against Up ↑ on the right (both edges
  at 7.0 U).  The previous 5.5 U position caused the RShift keycap to overlap `/` by
  0.5 U.
- **Preview: zoom limits removed** (`src/preview/viewer.html`): `minDistance` and
  `maxDistance` constraints on `OrbitControls` were removed so the camera can orbit
  right up to the model surface for close-up detail inspection.
- **CLI: script paths outside working directory** (`src/main.rs`): the `safe_path`
  CWD restriction on the input script argument was overly strict for CLI use — users
  should be able to run `rrcad --preview ../mykb/script.rb` or any absolute path.
  Removed the guard from `run_script`, `run_preview`, and `run_table`; export paths
  produced *inside* scripts remain confined by `safe_path` in `native.rs`.
- **Split TKL keyboard sample** (`samples/split_tkl_keyboard.rb`): replaced
  fraction-based pillar positions with diagonal midpoints between 4 adjacent
  key centres, giving ~13.5 mm clearance from every switch body edge
  (previously the positions fell directly under Cherry MX switch bodies and
  inside the Raspberry Pi Pico board footprints).  Both halves now have
  4 pillars each.

---

## [0.1.5] - 2026-03-26

### Fixed

- **MCP server: SIGSEGV from concurrent mRuby VMs** — `spawn_blocking` runs closures on a
  thread pool, so a timed-out tool call could leave a mRuby VM alive on one thread while a
  second call spawned a new VM on another. mRuby is not thread-safe; concurrent VMs caused
  SIGSEGV crashes. A new `MRUBY_EVAL_LOCK` (`std::sync::Mutex<()>`) is now acquired at the
  top of every `spawn_blocking` closure, serialising all mRuby/OCCT work regardless of how
  many concurrent calls arrive or how many timed-out threads are still running.
- **MCP server: TOCTOU port-binding race in `cad_preview`** — the old code bound a listener
  to discover a free port, dropped it, then had axum try to rebind the same port. Another
  process could steal the port in that window, causing a `panic!` inside the spawned task.
  The `tokio::net::TcpListener` is now kept alive and passed directly to a new
  `serve_with_listener()` on the axum server, eliminating both the race and a 200 ms sleep.
- **Split TKL keyboard sample** (`samples/split_tkl_keyboard.rb`): added M2.5 button-head
  counterbores (Ø4.8 mm, 1.5 mm deep) to all plate screw vias so screw heads sit flush with
  the plate top face.

### Changed

- **Split TKL keyboard — connectors, bosses, and manufacturing details** (`samples/split_tkl_keyboard.rb`):
  - Replaced RJ-45 inter-half connector with USB-C (safer for hot-plug; no VCC on interconnect
    cable). Left half: USB Micro host port at back wall ¼-width + USB-C interconnect at ¾-width.
    Right half: USB-C interconnect at ¼-width + USB-C host port at ¾-width.
  - Added wall slots for USB-C adapter boards (12×4.2 mm PCB, open-top insertion pocket,
    9×3.5 mm USB-C port opening in back wall).
  - Upgraded corner and mid-edge screw bosses to M2.5 heat-set copper insert compatibility
    (POST_R 2.5 → 3.2 mm, M2_R 1.2 → 1.6 mm; 3D print FIT_TOL 0.2 mm per side).
  - Added bottom-face lead-in step (0.5 mm box-cut method) on all switch cutouts to ease
    Cherry MX switch clip insertion from below.
  - Added `CHAMFER_CASE` chamfer to the solid tent wedge base (applied before fusing with the
    tilted case half to avoid BRepFilletAPI_MakeChamfer failures on complex fused geometry).
  - Preview changed from 2×2 parts layout to a fully assembled side-by-side view (plates
    seated in cases, left and right halves with a 20 mm gap).

### Added

- **MCP stress tests** (`tests/mcp_stress.rs`): 10 new tests covering sequential VM churn,
  error recovery, boundary inputs, deep boolean chains, geometry validation after operations,
  security prelude persistence, and a two-thread lock-serialisation proof.

---

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
