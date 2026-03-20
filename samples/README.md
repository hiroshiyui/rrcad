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

## Running a script

```sh
cargo run -- samples/01_hello_box.rb
```

With live browser preview (Phase 3):

```sh
cargo run -- --preview samples/06_live_preview.rb
```

> **Note:** Phase 1 is not yet complete. Running these scripts today will
> raise `NotImplementedError` for the geometry methods.  The DSL classes
> (`Shape`, `box`, `cylinder`, …) are already defined in the prelude, so
> syntax errors will not appear — only the unimplemented primitives will fail.
