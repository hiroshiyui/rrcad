# 2. Modeling Basics

Most CAD models start with a few primitive solids placed in space and
combined with boolean operations. This chapter covers the four foundations:
**units**, **primitives**, **transforms**, and **booleans**.

## Units

Model lengths are millimetres by default, and angular APIs use degrees.
Numeric unit helpers return typed unit values, so arithmetic keeps track of
whether you are working with a length or an angle before values are handed
to constructors, sketches, transforms, params, and patterns.

| Helper | Meaning |
|--------|---------|
| `.mm`, `.millimeter`, `.millimeters` | Typed millimetre length |
| `.cm`, `.centimeter`, `.centimeters` | Typed millimetre length from centimetres |
| `.m`, `.meter`, `.meters` | Typed millimetre length from metres |
| `.inch`, `.inches` | Typed millimetre length from inches |
| `.deg`, `.degree`, `.degrees` | Typed degree angle |
| `.rad`, `.radian`, `.radians` | Typed degree angle from radians |

**Typical use** — mix metric and imperial in one part:

```ruby
plate = box(2.inch, 30.mm, 0.25.inch)
hole  = cylinder(3.mm, 10.mm).translate(25.4.mm, 15.mm, -1.mm)
part  = plate.cut(hole).rotate(0, 0, 1, Math::PI.rad)
```

Typed values still behave like numbers in the common CAD cases, but they
reject obvious unit mixups in Ruby arithmetic.

## Primitives (3D solids)

| Function | Description |
|----------|-------------|
| `box(dx, dy, dz)` | Axis-aligned rectangular solid with one corner at the origin |
| `cylinder(r, h)` | Cylinder along the Z axis |
| `sphere(r)` | Sphere centred at the origin |
| `cone(r_base, r_top, h)` | Cone or frustum; `r_base` at Z=0, `r_top` at Z=`h` |
| `torus(r_major, r_tube)` | Torus in the XY plane |
| `wedge(dx, dy, dz, ltx)` | Wedge with base `dx × dz`, height `dy`, top-face X-width `ltx` |

**Typical use** — quick six-shape sampler:

```ruby
b = box(10, 20, 30)
c = cylinder(5, 15)
s = sphere(8)
k = cone(10, 4, 20)
t = torus(20, 3)
w = wedge(10, 8, 6, 4)
```

## Transforms

All transforms return a *new* Shape; the original is unchanged.

| Method | Description |
|--------|-------------|
| `.translate(dx, dy, dz)` | Move by vector |
| `.rotate(ax, ay, az, angle_deg)` | Rotate around axis `(ax, ay, az)` by `angle_deg` degrees |
| `.scale(factor)` | Uniform scale |
| `.scale(sx, sy, sz)` | Non-uniform scale per axis |
| `.mirror("xy")` / `"xz"` / `"yz"` | Mirror about a coordinate plane |

**Typical use** — chain transforms; each call returns a new shape:

```ruby
part = box(10, 10, 10)
  .translate(5, 0, 0)
  .rotate(0, 0, 1, 45)
  .scale(2)
```

`Shape#rotate_about(point, axis_dir, angle_deg)` is the lower-level
primitive when you need to rotate about an arbitrary point and axis instead
of an axis through the origin.

## Boolean operations

OCCT booleans are exact: they produce a new BRep with proper edges, not a
mesh approximation.

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

For many operands, use the batch variants — they are more efficient and
clearer to read than chained `.fuse` / `.cut` calls:

```ruby
merged = fuse_all([box(10, 10, 10), sphere(8), cylinder(5, 20)])
result = cut_all(box(100, 100, 20), [cyl1, cyl2, cyl3])
```

For booleans to succeed, the operands must actually overlap. If a fuse or
cut fails, translate one operand by a small amount and try again, or run
`puts shape.validate` to check the inputs.

## Worked example: a domed paperweight

Combines primitives, a transform, and a boolean cut to make a printable
desk object:

```ruby
# paperweight.rb
base = box(60, 60, 8).translate(-30, -30, 0)
dome = sphere(24).translate(0, 0, 8)

# Cut a finger recess so it picks up easily.
recess = cylinder(10, 12).translate(0, 0, 20)

body = base.fuse(dome).cut(recess).fillet(2)
body.export("paperweight.step")
preview body
```

See [`samples/02_boolean_ops.rb`](../../samples/02_boolean_ops.rb) for more
worked boolean examples.

---

[← Previous: Getting Started](01-getting-started.md) · [Index](../user-guide.md) · [Next: Sketches and Profiles →](03-sketches-and-profiles.md)
