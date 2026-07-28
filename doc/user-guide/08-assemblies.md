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

## Mass rollup

```ruby
mp = asm.mass_properties
mp[:volume]          # => 18000.0        mm³
mp[:mass]            # => 58.9           grams
mp[:center_of_mass]  # => [20.0, 20.0, 8.998]
mp[:parts]           # per-part volume, mass, density, centroid
```

The centre of mass is weighted by mass, not volume, so a steel post on an
aluminium base pulls it upward correctly. Parts that interpenetrate count
their shared volume twice — check `interferences` first if you are unsure.

Every report calls `solve` first, so the figures describe the assembly in its
final positions; none of them consume or modify it.

## Color

Per-part color is written to glTF, GLB, and OBJ exports (with companion
`.mtl`). It is ignored by STEP and STL.

```ruby
part.color(0.8, 0.3, 0.1)   # sRGB
```

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
