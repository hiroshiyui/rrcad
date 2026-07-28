# 11. Parametric Design and Batch Export

Real CAD parts almost always come in *families*: the same bracket in
three plate thicknesses, the same enclosure in five sizes. rrcad supports
this with the `param()` DSL plus two ways to feed values in: per-run CLI
flags, and CSV design tables.

## Declaring parameters

| Function | Description |
|----------|-------------|
| `param(name, default:, range: nil)` | Declare a parameter; CLI `--param name=value` overrides the default |
| `preview(shape)` | Push geometry to the live browser viewer (no-op outside `--preview` mode) |

`range:` is optional and used for validation; the CLI rejects values
outside the declared range.

**Typical use** — width-driven box:

```ruby
width = param :width, default: 50, range: 1..500
part  = box(width, 30, 20).fillet(2)
part.export("part_#{width}.step")
preview part
```

## Single override from the CLI

Pass one or more `--param name=value` flags. Each flag overrides one
`param()` declaration in the script.

```ruby
# design.rb
w = param :width,  default: 50
h = param :height, default: 20
box(w, 30, h).export("design_#{w}x#{h}.step")
```

```bash
cargo run -- --param width=80 --param height=35 design.rb
```

You can also set defaults in [`rrcad.toml`](01-getting-started.md#project-configuration);
CLI flags still win.

## Batch export from a CSV design table

Drop a CSV next to the script, list each variant on its own row, and
rrcad runs the script once per row. Column names map to `param()`
declarations.

```csv
name,width,height
small,30,15
medium,50,25
large,80,40
```

```bash
cargo run -- --design-table sizes.csv design.rb
```

If the CSV has a `name` column, that value becomes the output filename
stem so each row produces its own export.

## Worked example: countersunk plate family

A small parametric plate. Run it once with defaults; run it through a CSV
to produce a family of plates.

```ruby
# bolt_plate.rb
d     = param :bolt_dia,  default: 5.0
depth = param :thickness, default: 10.0

plate = box(80, 60, depth)
tool  = csink(d: d, csink_d: d * 2, csink_angle: 90, depth: depth)
        .translate(20, 20, depth)
plate.cut(tool).export("plate.step")
```

```csv
bolt_dia,thickness
3,8
5,10
8,15
```

```bash
cargo run -- --design-table plates.csv bolt_plate.rb
# writes plate.step three times — once per row
```

For a fully parametric box, see
[`samples/08_parametric_box.rb`](../../samples/08_parametric_box.rb) and
its companion [`samples/08_box_sizes.csv`](../../samples/08_box_sizes.csv).

## Splitting a project across files

A project of any size outgrows one script. `require_relative` pulls in another
file, exactly as in Ruby:

```ruby
# frame/params.rb
PLATE_T      = 2.0
BOLT_SPACING = 16.0

# frame/arm.rb
require_relative "params"          # frame/params.rb, not ./params.rb
def arm
  box(60, 8, PLATE_T)
end

# quad.rb
require_relative "frame/arm"
asm = assembly("quad") { |a| a.place arm }
```

```bash
cargo run -- quad.rb
```

The path resolves against **the file doing the requiring**, not the working
directory — so `frame/arm.rb` finds its own `params.rb` neighbour, and the
project runs the same from anywhere. The `.rb` suffix is optional.

Everything a required file defines — constants, methods, classes — is visible
to the file that required it. Sharing a parameter table this way is the point:
a plate thickness defined once cannot drift out of sync between the arm and
the motor mount.

Each file is evaluated **once**. Requiring it again returns `false` instead of
re-running it, so two files can both require a shared dependency, and a
require cycle terminates instead of recursing:

```ruby
first  = require_relative "params"   # => true, evaluated
second = require_relative "params"   # => false, already loaded
```

`param` declarations work in a required file, and `rrcad.toml` is still found
by walking up from the entry script, so one config covers the whole project.

Under `--preview`, every required file is watched, not just the entry script —
editing `frame/arm.rb` reloads the model. Adding or removing a
`require_relative` updates the watch set on the next reload.

Two things to know:

- There is no load path and no gem system, so plain `require` does not exist;
  it raises and points you at `require_relative`.
- `require_relative` is unavailable in [MCP mode](13-mcp-server.md), where
  scripts arrive as strings with no file to be relative to, and reading
  arbitrary files is not something a sandboxed script should do.

---

[← Previous: Import and Export](10-import-export.md) · [Index](../user-guide.md) · [Next: Live Preview and REPL →](12-live-preview-and-repl.md)
