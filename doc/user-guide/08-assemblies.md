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
