# rrcad User Guide

rrcad is a Ruby DSL for 3D parametric CAD. You write `.rb` scripts; an embedded mRuby VM executes them; Rust bindings call OpenCASCADE (OCCT) for exact BRep geometry. The result is industrial-grade solids exportable to STEP, STL, glTF, OBJ, SVG, and DXF.

---

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [CLI Modes](#cli-modes)
4. [DSL Reference](#dsl-reference)
   - [Primitives](#primitives-3d-solids)
   - [2D Sketch Faces](#2d-sketch-faces)
   - [Transforms](#transforms)
   - [Boolean Operations](#boolean-operations)
   - [Modifiers](#modifiers)
   - [Sub-shape Selectors](#sub-shape-selectors)
   - [Patterns](#patterns)
   - [Surface Modeling](#surface-modeling)
   - [Part Design](#part-design)
   - [Validation & Introspection](#validation--introspection)
   - [Import / Export](#import--export)
   - [Parametric Design](#parametric-design)
   - [Assembly](#assembly)
5. [Export Formats](#export-formats)
6. [Live Preview](#live-preview)
7. [Parametric Design & Batch Export](#parametric-design--batch-export)
8. [REPL](#repl)
9. [MCP Server](#mcp-server)
10. [Common Patterns](#common-patterns)
11. [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites

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
  libocct-ocaf-dev
```

### Build

```bash
cargo build
```

The first build compiles mRuby from source (~1 minute). Subsequent builds are fast.

---

## Quick Start

```ruby
# hello.rb
b = box(10, 20, 30)
b.export("hello.step")
preview b
```

```bash
cargo run -- hello.rb
```

That creates a 10 × 20 × 30 mm box, exports it as STEP, and (in `--preview` mode) shows it in the browser.

---

## CLI Modes

```bash
cargo run                                     # interactive REPL
cargo run -- script.rb                        # run script
cargo run -- --preview script.rb              # live browser preview
cargo run -- --param width=80 script.rb       # run with parameter override
cargo run -- --design-table table.csv script.rb  # batch export from CSV
cargo run -- --mcp                            # MCP server (stdio JSON-RPC)
```

| Flag | Description |
|------|-------------|
| `--preview` | Starts a local HTTP server and browser viewer; auto-reloads on script save |
| `--param name=value` | Override a `param` declaration in the script (can repeat) |
| `--design-table <csv>` | Run the script once per CSV row, substituting named columns as params |
| `--mcp` | Serve the CAD engine over the Model Context Protocol |

---

## DSL Reference

### Primitives (3D Solids)

| Function | Description |
|----------|-------------|
| `box(dx, dy, dz)` | Axis-aligned rectangular solid with one corner at the origin |
| `cylinder(r, h)` | Cylinder along the Z axis |
| `sphere(r)` | Sphere centred at the origin |
| `cone(r_base, r_top, h)` | Cone or frustum; `r_base` at Z=0, `r_top` at Z=`h` |
| `torus(r_major, r_tube)` | Torus in the XY plane |
| `wedge(dx, dy, dz, ltx)` | Wedge with base `dx × dz`, height `dy`, top-face X-width `ltx` |

```ruby
b = box(10, 20, 30)
c = cylinder(5, 15)
s = sphere(8)
k = cone(10, 4, 20)
t = torus(20, 3)
w = wedge(10, 8, 6, 4)
```

### 2D Sketch Faces

Used as input for `.extrude` and `.revolve`.

| Function | Description |
|----------|-------------|
| `rect(w, h)` | Rectangular face in the XY plane |
| `circle(r)` | Circular face in the XY plane |
| `ellipse(rx, ry)` | Elliptical face; axes are swapped automatically if `rx < ry` |
| `polygon([[x,y], ...])` | Closed polygon face; at least 3 points |
| `arc(r, start_deg, end_deg)` | Circular arc Wire (counterclockwise) |
| `spline_2d(pts, tangents: nil)` | Closed profile in the XZ plane for `.revolve` |
| `spline_3d(pts, tangents: nil)` | 3D Wire path for `.sweep` |

```ruby
face = rect(15, 10)
solid = face.extrude(20)

profile = spline_2d([[0, 0], [5, 3], [8, 5]], tangents: [[1, 0], [1, 0]])
body = profile.revolve(360)
```

### Transforms

All transforms return a new Shape; the original is unchanged.

| Method | Description |
|--------|-------------|
| `.translate(dx, dy, dz)` | Move by vector |
| `.rotate(ax, ay, az, angle_deg)` | Rotate around axis `(ax, ay, az)` by `angle_deg` degrees |
| `.scale(factor)` | Uniform scale |
| `.scale(sx, sy, sz)` | Non-uniform scale per axis |
| `.mirror("xy")` / `"xz"` / `"yz"` | Mirror about a coordinate plane |

```ruby
part = box(10, 10, 10)
  .translate(5, 0, 0)
  .rotate(0, 0, 1, 45)
  .scale(2)
```

### Boolean Operations

| Method | Description |
|--------|-------------|
| `.fuse(other)` | Union (A ∪ B) |
| `.cut(other)` | Difference (A − B) |
| `.common(other)` | Intersection (A ∩ B) |

```ruby
blob   = box(20, 20, 20).fuse(sphere(14).translate(20, 20, 20))
holed  = box(40, 40, 20).cut(cylinder(8, 30).translate(20, 20, -5))
cross  = box(60, 10, 10).common(box(10, 60, 10))
```

**Batch booleans:**

```ruby
merged = fuse_all([box(10,10,10), sphere(8), cylinder(5,20)])
result = cut_all(box(100,100,20), [cyl1, cyl2, cyl3])
```

### Modifiers

| Method | Description |
|--------|-------------|
| `.extrude(h)` | Extrude face/wire upward by `h` |
| `.extrude(h, draft: angle)` | Extrude with draft angle (tapers the walls) |
| `.revolve(angle_deg = 360)` | Revolve profile around the Z axis |
| `.sweep(path)` | Sweep profile along a 3D Wire path |
| `.fillet(r)` | Round all edges with radius `r` |
| `.fillet(r, :vertical)` | Round only vertical edges |
| `.fillet(r, :horizontal)` | Round only horizontal edges |
| `.chamfer(d)` | Bevel all edges symmetrically by distance `d` |
| `.chamfer(d, :vertical)` | Bevel only vertical edges |
| `.chamfer_asym(d1, d2)` | Asymmetric bevel (different distances on each side) |
| `.shell(thickness)` | Hollow the solid (negative = inward offset) |
| `.offset(distance)` | Offset the solid volume |
| `.offset_2d(distance)` | Offset a 2D Wire or Face in its own plane |
| `.simplify(min_feature_size)` | Remove features smaller than the threshold |

```ruby
part = rect(20, 10)
  .fillet_wire(2)      # round 2D corners first
  .extrude(15)
  .fillet(1)
  .chamfer(0.5, :vertical)
```

### Sub-shape Selectors

Select sub-shapes for operations or inspection.

| Method | Description |
|--------|-------------|
| `.faces(:all)` / `.faces(:top)` / `.faces(:bottom)` / `.faces(:side)` | Select faces by orientation |
| `.faces(">Z")` / `"<Z"` / `">X"` / `"<X"` / `">Y"` / `"<Y"` | Select face by outward-normal direction |
| `.edges(:all)` / `.edges(:vertical)` / `.edges(:horizontal)` | Select edges |
| `.vertices(:all)` | All vertices |

These return an `Array` of Shape objects.

```ruby
top_face  = part.faces(:top).first
side_edge = part.edges(:vertical).first
```

### Patterns

| Function | Description |
|----------|-------------|
| `linear_pattern(shape, n, dx, dy, dz)` | `n` copies, each translated by `(dx, dy, dz)` from the previous |
| `polar_pattern(shape, n, angle_deg)` | `n` copies rotated evenly around the Z axis |
| `grid_pattern(shape, nx, ny, dx, dy)` | `nx × ny` copies in a 2D grid |

```ruby
bolt_row   = linear_pattern(cylinder(3, 20), 5, 15, 0, 0)
bolt_ring  = polar_pattern(cylinder(3, 15).translate(30, 0, 0), 6, 360)
stud_grid  = grid_pattern(cylinder(2, 5), 4, 3, 10, 10)
```

### Surface Modeling

| Function / Method | Description |
|-------------------|-------------|
| `ruled_surface(wire_a, wire_b)` | Ruled surface between two wires |
| `fill_surface(boundary_wire)` | Smooth NURBS patch filling a closed boundary |
| `.slice(plane: :xy, z: d)` | Cross-section at an axis-aligned plane |

### Part Design

| Function / Method | Description |
|-------------------|-------------|
| `.pad(face_sel, height: h) { sketch }` | Extrude a sketch onto a face and fuse |
| `.pocket(face_sel, depth: d) { sketch }` | Extrude a sketch into a face and cut |
| `.fillet_wire(r)` | Round all corners of a 2D Wire/Face profile |
| `datum_plane(origin:, normal:, x_dir:)` | Create a reference plane |
| `helix(radius:, pitch:, height:)` | Helical Wire path (for thread sweeps) |
| `thread(solid, face_sel, pitch:, depth:)` | Cut a helical thread groove |
| `cbore(d:, cbore_d:, cbore_h:, depth:)` | Counterbore hole tool (use with `.cut`) |
| `csink(d:, csink_d:, csink_angle:, depth:)` | Countersink hole tool (use with `.cut`) |

### Validation & Introspection

| Method | Returns | Description |
|--------|---------|-------------|
| `.shape_type` | Symbol | `:solid`, `:shell`, `:face`, `:wire`, `:edge`, `:vertex`, `:compound`, `:compsolid` |
| `.validate` | `"ok"` or Array | List of topology errors, or `"ok"` |
| `.closed?` | Boolean | True if every edge has ≥ 2 adjacent faces |
| `.manifold?` | Boolean | True if every edge has exactly 2 adjacent faces |
| `.centroid` | `[x, y, z]` | Centre of mass |
| `.inertia` | Hash | `{ixx:, iyy:, izz:, ixy:, ixz:, iyz:}` inertia tensor |
| `.min_thickness` | Float | Minimum wall thickness |
| `.distance_to(other)` | Float | Minimum distance between two shapes (0 if touching) |

### Import / Export

| Method | Description |
|--------|-------------|
| `shape.export("file.step")` | Export; format determined by file extension |
| `Shape::import_step("file.step")` | Import a STEP file |
| `Shape::import_stl("file.stl")` | Import an STL file |

Supported extensions: `.step`, `.stl`, `.glb`, `.gltf`, `.obj`, `.svg`, `.dxf`.

### Parametric Design

| Function | Description |
|----------|-------------|
| `param(name, default:, range: nil)` | Declare a parameter; CLI `--param name=value` overrides the default |
| `preview(shape)` | Push geometry to the live browser viewer (no-op outside `--preview` mode) |

```ruby
width = param :width, default: 50, range: 1..500
part  = box(width, 30, 20).fillet(2)
part.export("part_#{width}.step")
preview part
```

### Assembly

```ruby
asm = assembly("frame") do |a|
  a.place base
  a.place post.mate(post.faces(:bottom).first, base.faces(:top).first)
end
asm.export("frame.step")
```

**`.mate` alignment:**

```ruby
# Align two faces flush
placed = part.mate(from_face, to_face)

# Align with a gap
placed = part.mate(from_face, to_face, offset: 2.0)
```

**Color:**

```ruby
part.color(0.8, 0.3, 0.1)   # sRGB — written to glTF/GLB/OBJ
```

---

## Export Formats

| Extension | Format | Best for |
|-----------|--------|----------|
| `.step` | STEP AP203 | CAD interchange, manufacturing, CNC/CAM |
| `.stl` | ASCII STL | 3D printing slicers |
| `.glb` | Binary glTF 2.0 | Web visualization, game engines, live preview |
| `.gltf` | Text glTF 2.0 | Human-readable; separate `.bin` companion |
| `.obj` | Wavefront OBJ | 3D modeling software; companion `.mtl` created |
| `.svg` | SVG (2D) | Technical drawings (uses HLR projection) |
| `.dxf` | DXF R12 (2D) | CAD software 2D drawing exchange |

**SVG / DXF view options:**

```ruby
part.export("drawing.svg")                # top view (default)
part.export("drawing.svg", view: :front)  # front view
part.export("drawing.svg", view: :side)   # side view
```

**GLB / glTF / OBJ tessellation quality:**

```ruby
part.export("model.glb", linear_deflection: 0.1)   # fine (production)
part.export("model.glb", linear_deflection: 0.5)   # coarse (quick preview)
```

---

## Live Preview

```bash
cargo run -- --preview script.rb
```

1. Launches an HTTP server (default `http://localhost:7878`)
2. Opens your browser to a Three.js 3D viewer
3. Watches the script file; on every save, re-evaluates the script and pushes the new geometry over WebSocket
4. Call `preview(shape)` in your script to specify which shape to display

```ruby
part = box(50, 40, 20).fillet(3)
preview part          # send to browser
part.export("part.step")
```

Press Ctrl-C to stop the server.

---

## Parametric Design & Batch Export

### Single override

```ruby
# design.rb
w = param :width,  default: 50
h = param :height, default: 20

box(w, 30, h).export("design_#{w}x#{h}.step")
```

```bash
cargo run -- --param width=80 --param height=35 design.rb
```

### Batch export from a CSV design table

```csv
name,width,height
small,30,15
medium,50,25
large,80,40
```

```bash
cargo run -- --design-table sizes.csv design.rb
```

The script runs once per row; `name` becomes the output filename stem if present.

---

## REPL

```bash
cargo run
```

| Feature | Description |
|---------|-------------|
| Tab completion | Press Tab after `.` to autocomplete Shape methods |
| History | Up/down arrows recall previous lines |
| `help` | Print the full DSL reference |
| `exit` / `quit` / Ctrl-D | Exit |

```
rrcad> b = box(10, 20, 30)
=> #<Shape:Solid>
rrcad> b.fillet(2).translate(5, 0, 0).export("out.step")
```

---

## MCP Server

```bash
cargo run -- --mcp
```

Starts a stdio JSON-RPC server compatible with Claude Desktop and Claude Code. AI clients can call rrcad tools directly without shell syntax knowledge.

### Tools

| Tool | Input | Output |
|------|-------|--------|
| `cad_eval` | `{ "code": "..." }` | Shape type, volume, surface area, bounding box, validity |
| `cad_export` | `{ "code": "...", "format": "step" }` | `{ "path": "/tmp/rrcad_mcp/shape.step" }` |
| `cad_preview` | `{ "code": "..." }` | `{ "url": "http://localhost:7878" }` |
| `cad_validate` | `{ "code": "..." }` | `{ "status": "ok" }` or `{ "errors": [...] }` |

### Resources

| URI | Content |
|-----|---------|
| `rrcad://api` | Full API reference (`doc/api.md`) |
| `rrcad://examples` | All scripts from `samples/` |

### Security

- 30-second execution timeout
- 512 MB address-space limit (Linux)
- Fresh mRuby VM per call (no shared state)
- Dangerous Kernel methods (`system`, `exec`, `fork`, …) are removed at startup
- All exports confined to `/tmp/rrcad_mcp/` (mode 0700)
- 64 KB input size cap; null-byte filtering

---

## Common Patterns

### Box with a through-hole

```ruby
base = box(50, 50, 20)
hole = cylinder(5, 25).translate(25, 25, -2)
part = base.cut(hole)
part.export("part.step")
```

### Circular bolt pattern

```ruby
hole_template = cylinder(3, 25).translate(30, 0, 0)
holes = polar_pattern(hole_template, 6, 360)
plate = box(80, 80, 10).translate(-40, -40, 0).cut(holes)
plate.export("plate.step")
```

### Parametric part family

```ruby
w = param :width,  default: 50, range: 10..200
d = param :depth,  default: 30, range: 10..200
h = param :height, default: 20, range: 10..100

box(w, d, h).fillet(2).export("box_#{w}x#{d}x#{h}.step")
```

### Extruded profile with rounded corners

```ruby
profile = rect(30, 20).fillet_wire(3)
solid = profile.extrude(15).fillet(1)
solid.export("extrusion.step")
```

### Revolve a profile

```ruby
# Profile in XZ plane (X = radius, Z = height)
profile = spline_2d([[2, 0], [4, 5], [3, 10], [5, 15]])
body = profile.revolve(360)
body.export("vase.step")
preview body
```

### Sweep along a path

```ruby
path = spline_3d([[0,0,0], [10,5,10], [20,0,20]])
section = circle(3)
pipe = section.sweep(path)
pipe.export("pipe.step")
```

### Colored assembly

```ruby
base = box(100, 80, 10).color(0.6, 0.6, 0.7)
post = box(15, 15, 60).color(0.9, 0.4, 0.1).translate(10, 10, 10)
asm  = fuse_all([base, post])
asm.export("assembly.glb")
```

### Countersink holes with a design table

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

---

## Troubleshooting

### Build failures

**`rake: command not found`**
```bash
gem install rake
cargo clean && cargo build
```

**`BRepPrimAPI_MakeBox.hxx: No such file`**
```bash
sudo apt-get install libocct-modeling-data-dev
```

### Fillet / chamfer failures

OCCT fillets can fail on degenerate topology produced by booleans:

- Try a smaller radius
- Apply fillets before booleans where possible
- Check the shape first: `puts part.validate`

### Boolean failures

- Ensure the two shapes actually overlap (translate if necessary)
- Check both shapes are valid before the operation

### General shape issues

```ruby
puts part.validate      # "ok" or list of errors
puts part.shape_type    # :solid, :shell, :face, …
puts part.closed?
puts part.manifold?
```

### Export failures

- Ensure the target directory exists
- Use ASCII-only paths on some platforms

---

## See Also

- `doc/api.md` — Rust/C++ API reference for contributors
- `doc/development.md` — Architecture and contributor guide
- `doc/troubleshooting.md` — Extended troubleshooting reference
- `samples/` — Eight annotated example scripts
- `doc/TODOs.md` — Roadmap and phase status
