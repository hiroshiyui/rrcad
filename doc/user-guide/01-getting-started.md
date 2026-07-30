# 1. Getting Started

You write CAD scripts in plain Ruby `.rb` files. rrcad runs them, produces
exact BRep solids in OCCT, and either previews them in a browser or exports
to STEP / STL / 3MF / glTF / GLB / OBJ / SVG / DXF.

This chapter walks you from a fresh checkout to a running example.

## Prerequisites

You need a working Rust toolchain, Ruby and rake (for the mRuby build), and
the OpenCASCADE 7.7+ development headers.

**Ubuntu / Debian:**

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build tools and mRuby dependencies
sudo apt-get install -y ruby rake clang-format

# OCCT geometry kernel (7.7+)
sudo apt-get install -y \
  libocct-foundation-dev \
  libocct-modeling-data-dev \
  libocct-modeling-algorithms-dev \
  libocct-data-exchange-dev \
  libocct-ocaf-dev \
  libocct-visualization-dev \
  fontconfig fonts-dejavu-core
```

## Build

```bash
cargo build
```

The first build compiles mRuby from source (~1 minute). Subsequent builds
are fast.

## Your first script

Save the following as `hello.rb`:

```ruby
# hello.rb
b = box(10, 20, 30)
b.export("hello.step")
preview b
```

Run it:

```bash
cargo run -- hello.rb
```

This creates a 10 × 20 × 30 mm rectangular solid, writes it as STEP, and (if
you also passed `--preview`) shows it in your browser.

## CLI modes

rrcad has five run modes, all from a single binary:

```bash
cargo run                                            # interactive REPL
cargo run -- script.rb                               # run a script
cargo run -- --preview script.rb                     # live browser preview (auto-selects a free port)
cargo run -- --preview --preview-port 3000 script.rb # use a fixed preview port
cargo run -- --param width=80 script.rb              # run with parameter override
cargo run -- --design-table table.csv script.rb      # batch export from CSV
cargo run -- --mcp                                   # MCP server (stdio JSON-RPC)
```

| Flag | Description |
|------|-------------|
| `--preview` | Starts a local HTTP server and browser viewer; auto-reloads on script save |
| `--preview-port <n>` | Override the preview port when using `--preview` |
| `--param name=value` | Override a `param` declaration in the script (can repeat) |
| `--design-table <csv>` | Run the script once per CSV row, substituting named columns as params |
| `--mcp` | Serve the CAD engine over the Model Context Protocol |

**Typical use** — iterate on a part with live reload:

```bash
cargo run -- --preview hello.rb
# edit hello.rb in your editor; the browser refreshes on every save
```

## Project configuration

rrcad reads an optional `rrcad.toml` from the script directory or any parent
directory. It is intended for standalone CAD projects that want a small local
config file checked into the project root. The repository ships
[`rrcad.toml.example`](../../rrcad.toml.example) as a copy-then-edit template.

```toml
# preview_port = 3000

[params]
width = 50
label = "bracket"
```

`preview_port` sets the default browser preview port for `--preview` when
present; if you leave it out, preview auto-selects a free local port. The
`[params]` table provides default `param()` overrides. Command-line
`--param` and `--preview-port` flags still win over the file.

## Worked example: a labelled coupon

A small bracket you can print and hand around — sized in millimetres, with a
chamfer, written to STEP and STL in one shot.

```ruby
# coupon.rb
coupon = box(40, 20, 4)
  .chamfer(0.5)
  .translate(-20, -10, 0)

coupon.export("coupon.step")
coupon.export("coupon.stl")
preview coupon
```

Run with `cargo run -- --preview coupon.rb`, then open the printed
`http://localhost:<port>` URL. Save the file in your editor and the viewer
will reload automatically.

---

[Index](../user-guide.md) · [Next: Modeling Basics →](02-modeling-basics.md)
