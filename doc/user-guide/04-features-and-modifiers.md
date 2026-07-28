# 4. Features and Modifiers

Once you have a profile or a primitive, *modifiers* turn it into the part
you actually want: extrude it into a solid, fillet its edges, hollow it
into a shell, offset its faces. Every modifier returns a new shape — the
input is left untouched, which makes chained operations safe to read top to
bottom.

## Modifier reference

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
| `.offset_2d(distance)` | Offset a 2D profile in its own plane; returns a profile you can extrude |
| `.simplify(min_feature_size)` | Remove features smaller than the threshold |
| `.fillet_wire(r)` | Round all corners of a 2D Wire/Face profile (before extruding) |

## Extrude

The bread-and-butter operation: take a 2D face and pull it into a 3D solid.

**Typical use** — extrude with a draft angle for moulded parts:

```ruby
boss = rect(10, 10).extrude(20, draft: 3.deg)
```

## Revolve

Make bodies of revolution (bushings, knobs, vases) from a 2D profile in
the XZ plane (X = radius, Z = height).

```ruby
profile = spline_2d([[2, 0], [4, 5], [3, 10], [5, 15]])
body    = profile.revolve(360)
body.export("vase.step")
```

For a partial revolve, pass an angle less than 360.

## Sweep

Drive a profile along a 3D path:

```ruby
path    = spline_3d([[0, 0, 0], [10, 5, 10], [20, 0, 20]])
section = circle(3)
pipe    = section.sweep(path)
```

For a helical sweep (threads, springs), pair a `helix(...)` path from
[chapter 5](05-part-design.md) with a small circular section.

## Fillet and chamfer

Round or bevel edges. Both accept an optional `:vertical` or `:horizontal`
selector to limit which edges are touched.

```ruby
slab = box(40, 20, 8)
  .fillet(2, :vertical)         # round the four vertical corners
  .chamfer(0.5)                 # break every other edge
```

If a fillet fails ("BRepFilletAPI_MakeFillet …"), the radius likely
exceeds the smallest adjacent face. Try a smaller radius, or fillet a
selected edge with [`fillet_sel`](06-topology-and-selectors.md).

## Shell and offset

`.shell` hollows a solid (positive thickness offsets outward, negative
inward); `.offset` grows or shrinks the whole volume.

```ruby
case_body = box(80, 50, 30).fillet(4).shell(-2)   # hollow with 2 mm walls
```

`.offset_2d(distance)` grows (positive) or shrinks (negative) a 2-D profile
in its own plane, keeping every edge parallel to where it started. A Face in
gives a Face out, so the offset profile extrudes, pads, and pockets like any
other:

```ruby
gasket = rect(40, 20).offset_2d(3).extrude(2)   # 46 x 26 with r = 3 corners
```

Growing rounds the corners the offset opens up (radius = the distance);
shrinking leaves interior corners square. Profiles with holes work in both
directions — growing the material closes the holes by the same distance:

```ruby
plate = rect(40, 20).cut(circle(5).translate(20, 10, 0))
plate.offset_2d(2)    # outline out to 44 x 24, hole in from r = 5 to r = 3
```

An inward offset larger than the profile raises rather than returning an
empty shape. Inside a sketch, `offset` does the same thing as part of the
profile's definition — see
[Sketches and Profiles](03-sketches-and-profiles.md#offsetting-the-profile).

## Worked example: filleted enclosure with rounded profile

```ruby
# enclosure.rb
profile = rect(80, 50).fillet_wire(6)    # round the 2D corners first
body    = profile.extrude(30)
        .fillet(2, :vertical)
        .chamfer(0.4, :horizontal)
        .shell(-2)                       # hollow it out

body.export("enclosure.step")
preview body
```

For a body of revolution worked example, see
[`samples/07_teapot.rb`](../../samples/07_teapot.rb).

---

[← Previous: Sketches and Profiles](03-sketches-and-profiles.md) · [Index](../user-guide.md) · [Next: Part Design →](05-part-design.md)
