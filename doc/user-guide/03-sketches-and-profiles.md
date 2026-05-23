# 3. Sketches and Profiles

Most real parts are built by drawing a 2D profile and then extruding,
revolving, or sweeping it into 3D. This chapter covers the three ways
rrcad builds profiles:

- **Direct 2D primitives** — `rect`, `circle`, `polygon`, splines.
- **Wire paths** — `arc`, `spline_3d` for sweep paths.
- **Constraint sketches** — `sketch do … end` for parametric, constraint-driven
  profiles.

## 2D sketch faces

These functions return a Face you can pass to `.extrude` or `.revolve`.

| Function | Description |
|----------|-------------|
| `rect(w, h)` | Rectangular face in the XY plane |
| `circle(r)` | Circular face in the XY plane |
| `ellipse(rx, ry)` | Elliptical face; axes are swapped automatically if `rx < ry` |
| `polygon([[x,y], ...])` | Closed polygon face; at least 3 points |
| `arc(r, start_deg, end_deg)` | Circular arc Wire (counterclockwise) |
| `spline_2d(pts, tangents: nil)` | Closed profile in the XZ plane for `.revolve` |
| `spline_3d(pts, tangents: nil)` | 3D Wire path for `.sweep` |

**Typical use** — extrude a rectangle into a slab:

```ruby
face  = rect(15, 10)
solid = face.extrude(20)
```

**Typical use** — revolve a 2D spline into a body of revolution:

```ruby
profile = spline_2d([[0, 0], [5, 3], [8, 5]], tangents: [[1, 0], [1, 0]])
body    = profile.revolve(360)
```

## Constraint sketches

`sketch do … end` builds a 2D profile from points, line segments, and
simple constraints. The current MVP returns a closed polygon face, so it
works anywhere `polygon`, `rect`, or `circle` profiles work — including
`.extrude`, `.pad`, and `.pocket`.

If the block returns an existing `Shape` profile, `sketch` returns that
shape directly. This keeps exact profiles such as `circle(5)` available
through the same entry point.

| Method | Description |
|--------|-------------|
| `point(x = nil, y = nil)` | Create a sketch point; coordinates may be left unknown |
| `point(:name, x = nil, y = nil)` | Create and name a sketch point |
| `construction_point(:name, x = nil, y = nil)` | Alias for a named reference point |
| `ref(:name)` / `self[:name]` | Look up a named sketch point |
| `midpoint(a, b)` / `midpoint(:name, a, b)` | Construction point halfway between two points |
| `polar_point(center, radius, angle_deg)` | Construction point at polar coordinates around `center` (CCW from +X); handy for bolt circles and fan patterns |
| `circle_at(center, radius)` | Build an exact circular profile at a resolved sketch point |
| `arc_at(center, radius, start_deg, end_deg)` | Build an arc wire at a resolved sketch point |
| `slot_between(a, b, radius)` | Build a rounded slot between two resolved points |
| `line(a, b)` | Add a line segment between two sketch points |
| `construction_line(a, b)` | Reference line for constraints; does not add profile edges |
| `rectangle(origin, width, height)` | Add a constrained rectangular line loop from an origin point |
| `centered_rectangle(center, width, height)` | Add a constrained rectangle around a center point |
| `fixed(point, x = point.x, y = point.y)` | Lock a point coordinate |
| `horizontal(a, b)` | Force two points to share Y |
| `vertical(a, b)` | Force two points to share X |
| `coincident(a, b)` | Force two points to share X and Y |
| `dimension(a, b, length)` | Set or validate an axis-aligned segment length |
| `equal_length(a, b, c, d)` | Make two line segments the same length |
| `parallel(a, b, c, d)` | Make two axis-aligned segments share orientation |
| `perpendicular(a, b, c, d)` | Make two axis-aligned segments perpendicular |
| `symmetric(a, b, center)` | Keep two points opposite each other around a center point |
| `mirror_x(source, target, axis_y = 0)` | Mirror a point across a horizontal axis |
| `mirror_y(source, target, axis_x = 0)` | Mirror a point across a vertical axis |
| `tangent(a, b, center, radius, side: nil)` | Constrain line segment `a→b` tangent to a circle. With `side:` of `:above`/`:below` (horizontal lines) or `:left`/`:right` (vertical lines), solves the unknown perpendicular coordinate; otherwise verifies distance |

**Typical use** — a constrained rectangle with implicit corners:

```ruby
profile = sketch do
  p1 = point(0, 0)
  p2 = point(nil, nil)
  p3 = point(nil, nil)
  p4 = point(nil, nil)

  horizontal p1, p2
  vertical   p2, p3
  horizontal p3, p4
  vertical   p4, p1

  dimension p1, p2, 40.mm
  dimension p2, p3, 20.mm

  line p1, p2
  line p2, p3
  line p3, p4
  line p4, p1
end

part = profile.extrude(5.mm)
```

**Typical use** — built-in rectangle and slot shortcuts:

```ruby
plate = sketch do
  origin = point(:origin, 0, 0)
  rectangle origin, 40.mm, 20.mm
end.extrude(4.mm)

boss_plate = sketch do
  center = point(:center, 0, 0)
  centered_rectangle center, 30.mm, 12.mm
end.extrude(3.mm)

round_part = sketch { circle(8.mm) }.extrude(3.mm)

boss = sketch do
  c = point(:center, 20.mm, 10.mm)
  circle_at c, 4.mm
end.extrude(6.mm)

slot = sketch do
  a = point(0, 0)
  b = point(24.mm, 0)
  slot_between a, b, 3.mm
end
```

For a diagonal example that uses the generalized `slot_between()` path, see
[`samples/10_sketch_slot.rb`](../../samples/10_sketch_slot.rb).

### Diagnostics

When a sketch fails to solve, the error names the constraint type, the
involved point labels, and the actual vs expected values (for example
`conflicting dimension constraint: :a→:b length=10.0, expected 5.0`), and a
non-convergence message lists every unresolved point with its missing
coordinates so the offending free variable is easy to spot.

For solved sketches, `sketch(diagnostics: true)` attaches a structured
`shape.sketch_diagnostics` hash with connected components and any redundant
constraints. `sketch(strict: true)` raises instead of returning a profile
when redundant constraints are present. `SketchBuilder#diagnostics` returns
the same report from the builder object before solving.

## Worked example: nameplate with a chamfered border

```ruby
# nameplate.rb
border = sketch do
  o = point(:origin, 0, 0)
  rectangle o, 60.mm, 25.mm
end.extrude(2.mm).chamfer(0.4)

# Inner recess where a label sticker goes.
recess = sketch do
  c = point(:center, 30.mm, 12.5.mm)
  centered_rectangle c, 50.mm, 15.mm
end.extrude(1.mm).translate(0, 0, 1)

plate = border.cut(recess)
plate.export("nameplate.step")
preview plate
```

---

[← Previous: Modeling Basics](02-modeling-basics.md) · [Index](../user-guide.md) · [Next: Features and Modifiers →](04-features-and-modifiers.md)
