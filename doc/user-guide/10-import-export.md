# 10. Import and Export

Every part eventually leaves rrcad — for a slicer, a CAM tool, a drawing
package, or a web viewer. This chapter covers the supported formats and
the per-format options.

## Import

| Method | Description |
|--------|-------------|
| `Shape::import_step("file.step")` | Import a STEP file |
| `Shape::import_stl("file.stl")` | Import an STL file |

Imported shapes are normal `Shape` objects: you can transform, fillet,
boolean, or pad onto them just like geometry built in rrcad.

## Export

```ruby
shape.export("file.step")    # format determined by file extension
```

Supported extensions: `.step`, `.stl`, `.glb`, `.gltf`, `.obj`, `.svg`, `.dxf`.

| Extension | Format | Best for |
|-----------|--------|----------|
| `.step` | STEP AP203 | CAD interchange, manufacturing, CNC/CAM |
| `.stl` | ASCII STL | 3D printing slicers |
| `.glb` | Binary glTF 2.0 | Web visualization, game engines, live preview |
| `.gltf` | Text glTF 2.0 | Human-readable; separate `.bin` companion |
| `.obj` | Wavefront OBJ | 3D modeling software; companion `.mtl` created |
| `.svg` | SVG (2D) | Technical drawings (uses HLR projection) |
| `.dxf` | DXF R12 (2D) | CAD software 2D drawing exchange |

## Where files may be read and written

Every path a script passes to an import or export method must resolve to a
location **inside the current working directory**. This is deliberate: a
script — including one written by an AI assistant through the MCP server —
should not be able to read or overwrite arbitrary files on your machine.

```ruby
part.export("out/bracket.step")   # ✓ relative to the working directory
part.export("/tmp/bracket.step")  # ✗ rejected: outside the working directory
part.export("../bracket.step")    # ✗ rejected: escapes via ..
```

Details worth knowing:

- Paths are canonicalised before the check, so a **symlink** that lives
  inside the working directory but points outside it is rejected too.
- The **target directory must already exist**. rrcad never creates
  directories on a script's behalf, so `out/bracket.step` fails until `out/`
  exists. Create it beforehand, or export into a directory you know is
  present.
- The rule applies to **imports as well as exports**, and to every format.
- The working directory is the one you launched `rrcad` from — not the
  directory containing the script. If exports land somewhere unexpected,
  check where you ran the command from.

A rejected path raises an error naming the offending path, for example:
`path '/tmp/bracket.step' is outside the working directory (path traversal
rejected)`.

In MCP mode the working directory is fixed to `/tmp/rrcad_mcp/`, so exported
files always land there; see
[13. MCP Server](13-mcp-server.md) for the full sandbox description.

## SVG / DXF view options

The 2D exporters support multiple view angles, sheet layouts, hidden
lines, center marks, callouts, and dimension labels. Examples:

```ruby
face = part.faces(:top).first

part.export("drawing.svg")                                        # top view (default)
part.export("drawing.svg", view: :front)                          # front view
part.export("drawing.svg", view: :side, scale: 2.0)               # side view at 2:1
part.export("drawing.svg", view: :sheet)                          # 3-view sheet
part.export("drawing.svg", view: :sheet, title_block: true)       # sheet with title block
part.export("drawing.svg", view: :sheet, dimensions: true)        # sheet with X/Y/Z labels
part.export("drawing.svg", view: :sheet, dimensions: true, tolerance: 0.1)
part.export("drawing.svg", view: :sheet, dimensions: true,
            tolerance: { plus: 0.2, minus: 0.05 })                # asymmetric tolerances
part.export("drawing.svg",
            datum: { label: :A, face: face },
            feature_control: { text: "⌀0.1", datums: [:A, :B] })  # GD&T frame
part.export("drawing.svg",
            datum: { label: :A, selector: :top },
            feature_control: { frame: "⌀0.1", datums: [:A, :B] }) # selector / frame aliases
part.export("drawing.svg", hidden: true)                          # dashed hidden lines
part.export("drawing.svg", center_marks: true)                    # cylinder centres
part.export("drawing.svg", callouts: true)                        # cylinder diameter callouts
part.export("drawing.svg", dimensions: true)                      # overall width/height labels

part.export("drawing.dxf", scale: 0.5, hidden: true)              # DXF at 1:2 with HIDDEN layer
part.export("drawing.dxf", view: :sheet)                          # 3-view sheet
part.export("drawing.dxf", view: :sheet, title_block: true)
part.export("drawing.dxf", view: :sheet, dimensions: true)
part.export("drawing.dxf", view: :sheet, dimensions: true, tolerance: 0.1)
part.export("drawing.dxf", view: :sheet, dimensions: true,
            tolerance: { plus: 0.2, minus: 0.05 })
part.export("drawing.dxf",
            datum: { label: :A, face: face },
            feature_control: { text: "⌀0.1", datums: [:A, :B] })
part.export("drawing.dxf", center_marks: true)
part.export("drawing.dxf", callouts: true)
part.export("drawing.dxf", dimensions: true)
```

See [chapter 6](06-topology-and-selectors.md#structured-gdt) for the
`gdt do |g| … end` form that attaches GD&T frames to the model itself,
so you don't have to repeat the export options.

## Section views

`section:` turns a drawing into a section view: the solid is cut with a
plane, the material in front of the plane is removed, and the exposed cut
face is drawn with standard 45° hatching.

```ruby
part.export("part.svg", view: :front, section: :xz)
part.export("part.svg", view: :front, section: { plane: :xz, offset: 15.mm })
```

The plane is `:xy`, `:xz`, or `:yz`. **Without an explicit `offset:` the cut
goes through the middle of the part** (the centre of its bounding box along
that axis), which is what a section view usually wants. Give `offset:` to cut
somewhere else — it is a coordinate on the cut axis, not a distance from the
centre.

Output details:

- **SVG** — a `hatch` group of thin lines, then a `section` group holding the
  cut outline at normal visible-edge weight.
- **DXF** — hatch lines on a dedicated `HATCH` layer; the cut outline on
  layer `0`, so it keeps the visible-edge weight.

Interior holes stay unhatched, so a sectioned part with a bore reads
correctly. Geometry behind the cutting plane is still projected normally.

Only `.svg` and `.dxf` use `section:`; other formats ignore it. In `:sheet`
mode the same plane applies to all three views, so a plane parallel to a
view's projection direction shows that view's cut face edge-on.

Errors are reported rather than producing an empty drawing: a plane that
misses the part reports the part's extent along that axis, a non-solid input
is rejected (there is no material to cut), and an unknown plane name is
caught before any geometry work.

## Detail views

Some features are too small to dimension at the drawing's scale. `detail:`
magnifies one circular region and draws it beside the parent view, with a thin
circle on the parent marking what was blown up.

```ruby
bracket = box(80, 50, 6).fillet(4, :vertical)
[[12, 12], [68, 12], [12, 38], [68, 38]].each do |x, y|
  bracket = bracket.cut(cylinder(2.6, 12).translate(x, y, -1))
end

bracket.export("bracket.svg", view: :top, dimensions: true,
               detail: { at: [68, 38], radius: 8, scale: 4, label: "A" })
```

That marks an 8 mm circle around the corner hole and draws it again at 4:1,
captioned `DETAIL A (4:1)`. The canvas grows to hold both views — 194 mm wide
here, against 118 mm for the same drawing without the detail.

**`at:` is stated on the view's own drawing plane**, in model units:

| view | `at:` means |
|---|---|
| `:top` (default) | `[X, Y]` |
| `:front` | `[X, Z]` |
| `:side` | `[Y, Z]` |

So you use the same numbers you modelled with, and `scale:` on the export does
not change them. `radius:` is likewise in model units — it sizes the region on
the part, not the bubble on the page.

Options:

- **`at:`** — `[x, y]` centre of the region. Required.
- **`radius:`** (or `r:`) — region radius, in model units. Required.
- **`scale:`** — magnification relative to the parent view. Default `2`.
  Whole numbers caption as `4:1`, others as `2.5:1`.
- **`label:`** — the letter on the bubble. Default `"A"`.

Geometry is cut exactly on the region boundary rather than at the nearest
tessellation vertex, so an edge crossing the circle stops on it cleanly. Hidden
lines, section outlines, and hatching inside the region are all magnified along
with the visible edges.

Output details:

- **SVG** — a `detail` group holding the marker circle, the border circle, the
  label, and the caption.
- **DXF** — all of that on a dedicated `DETAIL` layer, so a shop can turn the
  annotation off without losing geometry.

Two limits worth knowing:

- The close-up carries **no dimensions of its own** — `dimensions: true`
  annotates the parent view only. Overall width and height labels on a
  magnified fragment would repeat the parent's extents at the wrong scale.
- Centre marks and diameter callouts are **not** carried into the detail. They
  are positioned against the parent's geometry, and re-deriving them for the
  clipped region is not implemented.

A region containing no geometry is an error rather than a blank bubble — most
often it means the centre was given in the wrong pair of axes. Detail views
need a single view; `view: :sheet` is refused, since there is no one parent to
magnify. Export the detail separately.

## GLB / glTF / OBJ tessellation quality

Mesh formats require a tessellation step. The `linear_deflection` option
controls how closely the mesh follows the BRep — smaller values produce a
finer mesh and a larger file.

```ruby
part.export("model.glb", linear_deflection: 0.1)   # fine (production)
part.export("model.glb", linear_deflection: 0.5)   # coarse (quick preview)
```

## Worked example: STEP + STL + drawing in one script

```ruby
# export_all.rb
part = box(50, 30, 10).fillet(2)

part.export("part.step")                                          # for CNC
part.export("part.stl",   linear_deflection: 0.1)                 # for printing
part.export("part.glb",   linear_deflection: 0.2)                 # for web preview
part.export("drawing.svg", view: :sheet, title_block: true,
                           dimensions: true, hidden: true)        # for the shop
```

See [`samples/05_export_formats.rb`](../../samples/05_export_formats.rb)
for a more elaborate format showcase.

## Cut files for laser and CNC

`export("plate.dxf")` writes a *drawing*: an HLR projection of the whole solid,
carrying whatever else is visible from that direction. That is the right output
for a shop drawing and the wrong one for a cutter.

`export_outline` writes a **cut file** — the closed loops of one flat face, at
1:1, and nothing else:

```ruby
plate = box(60, 40, 2).fillet(5, :vertical)
[[10, 10], [50, 10], [10, 30], [50, 30]].each do |x, y|
  plate = plate.cut(cylinder(1.7, 10).translate(x, y, -1))
end

plate.faces(:top).first.export_outline("plate.dxf")
```

That plate comes out as 4 `CIRCLE`, 4 `ARC`, and 4 `LINE` entities — the holes
are real circles and the rounded corners real arcs, not strings of short
chords. A controller cuts them exactly, and the file stays small. Only
free-form curves (splines) have to be approximated; `deflection:` bounds their
chord error in millimetres:

```ruby
sk.export_outline("cam.dxf", deflection: 0.01)   # default 0.05
```

The receiver must be a **planar face**, or a shape holding exactly one face —
so a sketch profile exports directly:

```ruby
circle(12).export_outline("washer.dxf")
```

A solid has many faces, and rather than guess, `export_outline` asks you to
pick one. A curved face is refused outright: a cut file needs a flat one.

Three things the output does for you:

- **Shifted to the origin.** The outline's bounding box starts at (0, 0),
  ready to nest on a sheet.
- **True size, whatever the orientation.** The outline is taken in the face's
  own plane, so a face tilted in space still measures its real dimensions
  rather than a foreshortened projection.
- **Holes on their own layer.** DXF `HOLES` and `PROFILE` layers (SVG
  `class="holes"` / `class="profile"`), because inside cuts normally run
  before the outside profile.

DXF declares millimetres via `$INSUNITS`, so the controller need not guess the
scale. `.svg` is accepted too, sized in millimetres, for cutters that take it.

---

[← Previous: Inspection and CAM Checks](09-inspection-and-cam.md) · [Index](../user-guide.md) · [Next: Parametric Design and Batch Export →](11-parametric-and-batch.md)
