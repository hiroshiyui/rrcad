# 8. Assemblies

An assembly groups several solids into a single named compound. rrcad has
two styles: an ad-hoc `place` API for quick layouts, and a declarative
constraint solver for assemblies that grow more than two or three parts
deep.

## The basic form

```ruby
asm = assembly("frame") do |a|
  a.place base
  a.place post.mate(post.faces(:bottom).first, base.faces(:top).first)
end
asm.export("frame.step")
```

`a.place(shape)` adds a part to the assembly verbatim. Use the
`.mate(from_face, to_face)` method on the shape to align it before placing.

## Declarative solver

For assemblies with chained dependencies — part B sits on part A, part C
sits on part B — use the named form. The first `ground` part is fixed;
later `part` blocks describe how each new shape attaches.

```ruby
asm = assembly("rig") do |a|
  a.ground :base, base
  a.part :post, post do
    mate from: :bottom, to: face(:base, :top)
  end
  a.part :cap, cap do
    mate from: :bottom, to: face(:post, :top), offset: 2.0
  end
end
asm.to_shape
```

## `.mate` alignment

```ruby
# Align two faces flush.
placed = part.mate(from_face, to_face)

# Align with a gap.
placed = part.mate(from_face, to_face, offset: 2.0)
```

## Constraint helpers

| Helper | Purpose |
|--------|---------|
| `a.distance_mate(part, from:, to:, distance:)` | Named air-gap variant of mate |
| `a.axis_align(part, from: [p1, p2], to: [q1, q2])` | Coaxial / concentric alignment by two source / target axis points |
| `a.angle_mate(part, from:, to:, angle:, pivot:, axis_dir:)` | Mate + lock the leftover rotational DOF by rotating about a chosen pivot |

```ruby
asm = assembly("rig") do |a|
  a.place base

  # Air gap between two faces.
  a.distance_mate post, from: post.faces(:bottom).first,
                        to:   base.faces(:top).first, distance: 5

  # Concentric alignment of a shaft's centreline to a hole's centreline.
  a.axis_align shaft_part,
    from: [[0, 0, 0], [0, 0, 10]],
    to:   [[hole_x, hole_y, 0], [hole_x, hole_y, 10]]

  # Mate + rotate about a pivot to lock the orientation.
  a.angle_mate cover, from: cover.faces(:bottom).first,
                      to:   base.faces(:top).first,
                      angle: 30, pivot: [10, 10, 5], axis_dir: [0, 0, 1]
end
```

`Shape#rotate_about(point, axis_dir, angle_deg)` is the underlying
primitive when you need explicit control outside of an assembly.

## Checking for clashes

An assembly that solves cleanly can still be wrong: two parts may occupy the
same space. `interferences` intersects every pair of components and reports
the ones that actually overlap.

```ruby
asm.interferences
# => [{a: :plate, b: :bracket, type: :interference,
#      volume: 192.0, centroid: [54.0, 34.0, 3.5]}]

asm.clash?   # => true
```

A flush `mate` is *not* a clash — two boxes sharing a face overlap in zero
volume. Findings come back worst-first, by descending overlap volume, and
the `centroid` tells you where to look.

Pass `clearance:` to also require an air gap between every pair, which is how
you check a fit tolerance or room for a tool:

```ruby
asm.interferences(clearance: 0.5)
# => [{a: :shaft, b: :housing, type: :clearance,
#      distance: 0.2, minimum: 0.5}]
```

Parts that touch are skipped by the clearance check — a mate puts faces
together on purpose, and every mated pair would otherwise be reported. When
nothing in the assembly is meant to touch, pass `ignore_contact: false`.

Each check is an O(n²) sweep of boolean intersections, so run it deliberately
rather than on every rebuild.

## Bill of materials

Tag components as you place them, and `bom` rolls them up by quantity:

```ruby
screw = cylinder(1.5, 10)

asm = assembly("panel") do |a|
  a.place plate, name: :plate, material: "aluminium"
  4.times do |i|
    a.place screw.translate(10 + i * 12, 20, 5),
            name: :"screw_#{i}", component: :m3_screw, material: "stainless"
  end
end

puts asm.bom_text
```

```
Item  Component  Qty  Material   Volume (mm3)  Mass (g)
----  ---------  ---  ---------  ------------  --------
1     m3_screw     4  stainless       282.743     2.262
2     plate        1  aluminium     12000.000    32.400
----  ---------  ---  ---------  ------------  --------
      TOTAL        5                12282.743    34.662
```

`component:` is the grouping key; `name:` stays unique per part. Parts sharing
a component key are treated as interchangeable, and grouping two parts of
different volume raises rather than quietly averaging them. Use `asm.bom` for
the same data as an Array of Hashes.

`material:` is free text, but a recognised name supplies a density, so masses
come out right without you stating one. An explicit `density:` (g/cm³) always
wins, and an unknown material falls back to the default rather than failing —
every row reports the density it used.

## Parts you buy rather than model

Density only helps for parts you actually draw. A motor, a battery, or a
flight controller has a mass on its datasheet and a shape you are modelling
only as an envelope — computing `volume × density` for those gives a number
with no relation to reality.

State the mass instead:

```ruby
a.place battery_envelope, name: :battery, mass: 182      # grams
a.place motor_step, name: :motor_fl, component: :motor_2207, mass: 32.5
```

The envelope is still real geometry, so it keeps taking part in clash and
clearance checks — that is exactly why you keep it. Only the mass comes from
elsewhere. `mass:` and `density:` together raises, since they are two answers
to the same question, and every row reports `mass_source: :stated` or
`:density` so an overridden figure is never mistaken for a computed one.

A stated mass on a shape with no volume — a `datum_plane`, say — behaves as a
point mass at that location, which is a convenient way to account for wiring,
tape, or glue:

```ruby
a.place datum_plane(origin: [0, 0, 12], normal: [0, 0, 1], x_dir: [1, 0, 0]),
        name: :wiring, mass: 15
```

## Mass rollup

```ruby
mp = asm.mass_properties
mp[:volume]          # => 18000.0        mm³
mp[:mass]            # => 58.9           grams
mp[:center_of_mass]  # => [20.0, 20.0, 8.998]
mp[:parts]           # per-part volume, mass, density, centroid, mass_source
```

The centre of mass is weighted by mass, not volume, so a steel post on an
aluminium base pulls it upward correctly. Parts that interpenetrate count
their shared volume twice — check `interferences` first if you are unsure.

## Inertia

`mass_properties` also returns the inertia tensor, in g·mm², taken about the
assembly's centre of mass:

```ruby
mp[:inertia]        # => {ixx:, iyy:, izz:, ixy:, ixz:, iyz:}
mp[:inertia_about]  # => the reference point used

asm.mass_properties(about: :origin)[:inertia]
asm.mass_properties(about: [0, 0, 50])[:inertia]
```

Each part contributes its own tensor plus a parallel-axis transfer to the
common reference. Multiply by 1e−9 for kg·m², which is what most simulators
expect.

The off-diagonal terms are worth reading: they measure how much rotation
about one axis couples into another. On a symmetric design they should
vanish, so a non-zero one usually means something is off-centre.

Two limits to keep in mind. A stated `mass:` rescales the envelope's inertia
distribution to match the declared weight, which assumes the part's density is
uniform across that envelope — fair for a battery, rougher for a motor with a
heavy stator and a hollow bell. And as with the mass rollup, overlapping parts
count their shared volume twice.

## Worked example: a quadcopter airframe

Symmetric arms, four bought motors, and a battery — the two heaviest items
carrying stated masses rather than computed ones:

```ruby
arm   = box(60, 8, 4).translate(20, -4, -2)
plate = box(60, 60, 2).translate(-30, -30, -1)
motor = cylinder(14, 25)
batt  = box(105, 35, 26).translate(-52.5, -17.5, -30)

asm = assembly("quad") do |a|
  a.place plate, name: :plate, material: "carbon", density: 1.55
  4.times do |i|
    ang = 45 + i * 90
    rad = ang * Math::PI / 180
    a.place arm.rotate(0, 0, 1, ang), name: :"arm_#{i}",
            component: :arm, material: "carbon", density: 1.55
    a.place motor.translate(56 * Math.cos(rad), 56 * Math.sin(rad), 2),
            name: :"motor_#{i}", component: :motor_2207, mass: 32.5
  end
  a.place batt, name: :battery, mass: 182.0
end
```

```
Item  Component   Qty  Material  Volume (mm3)  Mass (g)
----  ----------  ---  --------  ------------  --------
1     arm           4  carbon        7680.000    11.904
2     motor_2207    4  -            61575.216   130.000
3     battery       1  -            95550.000   182.000
4     plate         1  carbon        7200.000    11.160
----  ----------  ---  --------  ------------  --------
      TOTAL        10              172005.216   335.064

AUW            = 335.06 g
centre of mass = [0.0, 0.0, -3.61]
inertia about the CoM, g*mm^2:
  ixx 341445.7   iyy 490079.0   izz 646302.4
  ixy 0.0   ixz 0.0   iyz 0.0
```

The centre of mass sits dead-centre in x and y and 3.6 mm *below* the plate,
pulled down by the battery slung underneath. All three coupling terms are
exactly zero, as they should be for a symmetric airframe. `ixx` is lower than
`iyy` because the battery is long in x and narrow in y, so the airframe rolls
more readily than it pitches — the kind of asymmetry that is invisible in the
model and obvious in the numbers.

Every report calls `solve` first, so the figures describe the assembly in its
final positions; none of them consume or modify it.

## Color

Per-part color is written to glTF, GLB, and OBJ exports (with companion
`.mtl`). It is ignored by STEP and STL.

```ruby
part.color(0.8, 0.3, 0.1)   # sRGB
```

## Exporting an assembly

`Assembly#export` fuses the components and writes the result, taking the same
options `Shape#export` does:

```ruby
asm.export("rig.step")                                    # solid, for CAM
asm.export("rig.svg", view: :sheet, title_block: true)    # 3-view sheet
asm.export("rig.svg", view: :front, section: :xz)         # section view
asm.export("rig.svg", view: :top, dimensions: true, ordinate: true)
```

Everything in [chapter 10](10-import-export.md#2-d-drawings-svg--dxf) —
`view:`, `scale:`, `section:`, `dimensions:`, `ordinate:`, `detail:`,
`hidden:`, `title_block:` — is forwarded untouched.

To hand the design to another CAD system with the parts kept separate,
`structured: true` (STEP only) writes each component as its own named
product under one root assembly instead of fusing:

```ruby
asm.export("drone.step", structured: true)   # FreeCAD/Fusion see every part
```

Component names come from `name:` on `place`/`part` (unnamed parts become
`part_1`, `part_2`, …), and a component's `.color` travels with it.

The export is of the **fused** geometry, so it is a picture of the assembled
product. Two extra options put the component data back on the page.

### Parts lists and balloons

```ruby
asm.export("panel.svg", view: :sheet, bom: true, balloons: true,
                        title_block: true)
```

`bom: true` draws the bill of materials as a table below the drawing:

```
Item  Component  Qty  Material   Mass (g)
1     m6_screw   4    stainless  10.86
2     plate      1    aluminium  155.52
3     post       1    steel      80.38
```

`balloons: true` adds a numbered circle for each row, with a leader landing on
that component — **balloon 3 and table row 3 name the same part**. There is one
balloon per *component*, not per part: the four screws share one balloon,
because the table already says there are four.

Balloons ring the geometry so they never sit on top of it, and are ordered by
where their part actually is, so the leaders fan out without crossing. On a
three-view sheet they attach to the top view, which is the one plan every part
appears somewhere on.

The table's item numbers, quantities, and masses come from
[`bom`](#bill-of-materials), so a component key that groups parts of different
volume raises there rather than producing a misleading drawing.

Both options are drawing-only — `asm.export("panel.step", bom: true)` simply
ignores them.

Output details:

- **SVG** — a `bom` group for the table, a `balloons` group for the callouts.
- **DXF** — `BOM` and `BALLOON` layers, so either can be switched off without
  touching geometry.

## Worked example: coloured assembly

```ruby
# assembly.rb
base = box(100, 80, 10).color(0.6, 0.6, 0.7)
post = box(15, 15, 60).color(0.9, 0.4, 0.1).translate(10, 10, 10)
asm  = fuse_all([base, post])
asm.export("assembly.glb")
preview asm
```

For a fastener-stack example using `washer()` and `nut()` plus a real mate
chain, see [`samples/09_fastener_stack.rb`](../../samples/09_fastener_stack.rb).

---

[← Previous: Patterns and Surfaces](07-patterns-and-surfaces.md) · [Index](../user-guide.md) · [Next: Inspection and CAM Checks →](09-inspection-and-cam.md)
