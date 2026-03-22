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
