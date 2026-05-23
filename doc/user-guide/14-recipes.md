# 14. Recipes

This chapter collects short, cross-cutting recipes you can copy as
starting points. Each recipe touches two or three DSL areas — booleans
with patterns, sketches with parameters, surfaces with sweeps — and links
to the chapter that documents each piece in depth.

## Box with a through-hole

A 50 × 50 × 20 plate with a centred 10 mm bore.

```ruby
base = box(50, 50, 20)
hole = cylinder(5, 25).translate(25, 25, -2)
part = base.cut(hole)
part.export("part.step")
```

→ [Modeling Basics](02-modeling-basics.md), [Part Design](05-part-design.md)
for the `clearance_hole` shortcut.

## Circular bolt pattern

Six holes equally spaced around a plate's centre.

```ruby
hole_template = cylinder(3, 25).translate(30, 0, 0)
holes = polar_pattern(hole_template, 6, 360)
plate = box(80, 80, 10).translate(-40, -40, 0).cut(holes)
plate.export("plate.step")
```

→ [Patterns and Surfaces](07-patterns-and-surfaces.md).

## Parametric part family

Three-axis box driven by `param()`; pair with a CSV for a family run.

```ruby
w = param :width,  default: 50, range: 10..200
d = param :depth,  default: 30, range: 10..200
h = param :height, default: 20, range: 10..100

box(w, d, h).fillet(2).export("box_#{w}x#{d}x#{h}.step")
```

→ [Parametric Design and Batch Export](11-parametric-and-batch.md).

## Extruded profile with rounded corners

Round the 2D corners *before* extruding for a clean rounded prism.

```ruby
profile = rect(30, 20).fillet_wire(3)
solid   = profile.extrude(15).fillet(1)
solid.export("extrusion.step")
```

→ [Sketches and Profiles](03-sketches-and-profiles.md),
[Features and Modifiers](04-features-and-modifiers.md).

## Revolved vase

Spline profile in the XZ plane, revolved 360°.

```ruby
profile = spline_2d([[2, 0], [4, 5], [3, 10], [5, 15]])
body    = profile.revolve(360)
body.export("vase.step")
preview body
```

→ [Features and Modifiers](04-features-and-modifiers.md).

## Sweep along a 3D path

Drive a circular section along a smooth 3D spline.

```ruby
path    = spline_3d([[0,0,0], [10,5,10], [20,0,20]])
section = circle(3)
pipe    = section.sweep(path)
pipe.export("pipe.step")
```

→ [Features and Modifiers](04-features-and-modifiers.md).

## Coloured assembly for web preview

Two parts, two colours, exported as a single GLB.

```ruby
base = box(100, 80, 10).color(0.6, 0.6, 0.7)
post = box(15, 15, 60).color(0.9, 0.4, 0.1).translate(10, 10, 10)
asm  = fuse_all([base, post])
asm.export("assembly.glb")
```

→ [Assemblies](08-assemblies.md).

## Countersinks via a design table

Same script, multiple bolt sizes — driven from a CSV.

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

→ [Parametric Design and Batch Export](11-parametric-and-batch.md),
[Part Design](05-part-design.md).

---

[← Previous: MCP Server](13-mcp-server.md) · [Index](../user-guide.md) · [Next: Troubleshooting →](15-troubleshooting.md)
