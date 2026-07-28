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

---

[← Previous: Inspection and CAM Checks](09-inspection-and-cam.md) · [Index](../user-guide.md) · [Next: Parametric Design and Batch Export →](11-parametric-and-batch.md)
