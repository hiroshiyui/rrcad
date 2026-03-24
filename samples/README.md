# rrcad samples

Each file is a self-contained Ruby script demonstrating a specific feature set.
Scripts are numbered in order of increasing complexity.

| File | What it shows | Requires |
|------|---------------|----------|
| `01_hello_box.rb` | Create a box and export as STEP | Phase 1 |
| `02_boolean_ops.rb` | `fuse`, `cut`, `common` | Phase 1 |
| `03_transforms.rb` | `translate`, `rotate`, `scale` | Phase 1 |
| `04_bracket.rb` | Realistic L-bracket with holes | Phase 1–2 |
| `05_export_formats.rb` | STEP, STL, and glTF from one part | Phase 1 |
| `06_live_preview.rb` | Live browser viewer with `preview` | Phase 1, 3 |
| `07_teapot.rb` | Utah Teapot from 28 Newell Bézier patches (`bezier_patch`, `sew`) | Phase 6 |
| `08_parametric_box.rb` | Parametric box with `param` DSL; drive with `--param` or `--design-table` | Phase 5 |
| `pen_schmidt.rb` | Ball pen body (4 parts): barrel, tip, front cap, tail cap — L-tenon/mortise joint, spring-relief snap tabs, Schmidt refill compatible | Phase 1, 3 |
| `split_tkl_keyboard.rb` | 86-key split TKL mechanical keyboard (Cherry MX, 19.05 mm spacing): compact layout with left Fn row aligned with number row for tighter width, right half (≈20.7 cm) with single nav column and inverted-T arrows; 5° pitch wedge base; M2 corner + edge screw bosses + central screw-less pillar per side; M2.5 heat-set insert standoffs (4 per side) for Raspberry Pi Pico; symmetrical micro-USB (left 1/4) + RJ-45 (right 1/4) wall cutouts. Preview: [`doc/split_tkl_keyboard.stl`](../doc/split_tkl_keyboard.stl) | Phase 1, 2 |

`08_box_sizes.csv` is the design-table CSV for `08_parametric_box.rb`.

## Running a script

```sh
cargo run -- samples/01_hello_box.rb
```

With live browser preview (Phase 3):

```sh
cargo run -- --preview samples/06_live_preview.rb
```

With a parameter override (Phase 5):

```sh
cargo run -- --param width=80 samples/08_parametric_box.rb
```

Batch export via design table (Phase 5):

```sh
cargo run -- --design-table samples/08_box_sizes.csv samples/08_parametric_box.rb
```

Via MCP (Phase 9) — any of these scripts can be run by an AI client using the
`cad_eval` or `cad_export` tools once `rrcad --mcp` is active:

```sh
cargo run -- --mcp   # start MCP server; connect Claude Desktop or Claude Code
```
