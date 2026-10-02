# rrcad samples

Each file is a self-contained Ruby script demonstrating a specific feature set.
Scripts are numbered in order of increasing complexity.

| File | What it shows | Requires |
|------|---------------|----------|
| `01_hello_box.rb` | Create a box and export as STEP | Phase 1 |
| `02_boolean_ops.rb` | `fuse`, `cut`, `common` | Phase 1 |
| `03_transforms.rb` | `translate`, `rotate`, `scale` | Phase 1 |
| `04_bracket.rb` | Realistic L-bracket with holes | Phase 1–2 |
| `05_export_formats.rb` | STEP, 3MF, STL, and glTF from one part | Phase 1 |
| `06_live_preview.rb` | Live browser viewer with `preview` | Phase 1, 3 |
| `07_teapot.rb` | Utah Teapot from 28 Newell Bézier patches (`bezier_patch`, `sew`) | Phase 6 |
| `08_parametric_box.rb` | Parametric box with `param` DSL; drive with `--param` or `--design-table` | Phase 5 |
| `09_fastener_stack.rb` | Washer, nut, and clearance-hole assembly demo | Phase 10 |
| `10_sketch_slot.rb` | Diagonal `slot_between` sketch and extrusion demo | Phase 10 |
| `11_sheet_metal_tray.rb` | Folded sheet-metal tray with bend relief; prints the bend table and writes the flat blank as DXF | Phase 11 |
| `12_propeller.rb` | Two-blade propeller: NACA `airfoil` sections swept with per-station `twist:`/`scale:`, engraved `text` size marking on the hub | Phase 12 |
| `pen_schmidt.rb` | Ball pen body (4 parts): barrel, tip, front cap, tail cap — L-tenon/mortise joint, spring-relief snap tabs, Schmidt refill compatible | Phase 1, 3 |
| `split_tkl_keyboard.rb` | 86-key split TKL mechanical keyboard (Cherry MX, 19.05 mm spacing): compact layout with left Fn row aligned with number row for tighter width, right half (≈20.7 cm) with single nav column and inverted-T arrows; flat-bottom case (glue on custom tenting feet at any preferred angle); right bottom row: Space (1 U) + RAlt/Fn/RCtrl (1.25 U each) + arrow cluster + PgDn; RShift (2 U) correctly positioned flush against `/` and Up ↑; M2.5 heat-set insert corner + mid-edge screw bosses with counterbored plate vias; 4 screw-less support pillars per side (placed at diagonal midpoints between adjacent key centres to clear switch bodies and Pico footprints); M2.5 Pico mounting standoffs (4 per side); USB-C inter-half connectors + wall slots for USB-C adapter boards; bottom-face lead-in step on switch cutouts for easy Cherry MX clip insertion | Phase 1, 2 |
| `hitbox_mx_controller.rb` | Leverless (Hitbox-style) game controller with Cherry MX switches and a Raspberry Pi Pico: 18 keys (PC-style inverted-T arrow cluster tilted 10° for a straight wrist and isolated by a ≥ 10 mm gap, 8 finger-staggered attack keys, 2 spare thumb keys, 4 function keys), 206 × 150 mm two-part case that fits a 220 × 220 mm bed, 5 mm top plate with 1.5 mm MX clip ledge, M3 flat-head screws into heat-set inserts, M2 Pico standoffs, micro-USB back-wall opening, rubber-foot recesses; layout and printability checks before building; exploded preview + `hitbox_exploded.glb` (`--param explode=0` for the assembled view), with the XDA caps from `xda_keycaps.rb` | Phase 1, 2 |
| `xda_keycaps.rb` | 18 XDA-profile 1u keycaps for the leverless controller: tapered shell with uniform 1.5 mm walls, spherical dish, MX cross stem (4 mm blind socket, FDM tolerance), laid out 6 × 3 as one supportless print job; single-cap export for a stem-fit test. Geometry lives in `lib/xda_keycap.rb`, which `hitbox_mx_controller.rb` also loads to preview the real caps | Phase 1, 2 |

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
