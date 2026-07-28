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
| `spline(a, b, through: [...])` | Add a curved segment through the given interior points ([below](#spline-segments)) |
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
| `fillet(point, radius)` | Round a corner of the profile with a tangent arc ([below](#corner-fillets-and-chamfers)) |
| `chamfer(point, distance)` | Bevel a corner, setting back `distance` along both adjacent segments |
| `trim(a, b, by:)` / `trim(a, b, to:)` | Shorten a segment, by a distance or up to an intersection ([below](#trimming-and-extending-segments)) |
| `extend(a, b, by:)` / `extend(a, b, to:)` | Lengthen a segment; same arguments as `trim` |
| `offset(distance)` | Grow or shrink the finished profile in its own plane ([below](#offsetting-the-profile)) |
| `linear_pattern(count:, dx:, dy:)` | Repeat the finished profile along a row ([below](#patterning-the-profile)) |
| `polar_pattern(count:, center:, angle:)` | Repeat it around a centre point |
| `grid_pattern(nx:, ny:, dx:, dy:)` | Repeat it across a 2-D grid |

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

### Corner fillets and chamfers

`fillet` and `chamfer` round or bevel an individual corner of a line-based
sketch. They shape the 2-D profile itself, so the rounding exists before the
profile is extruded — unlike `Shape#fillet`, which rounds edges of a solid
that already exists.

```ruby
bracket = sketch do
  a = point(:a, 0, 0)
  b = point(:b, 40.mm, 0)
  c = point(:c, 40.mm, 20.mm)
  d = point(:d, 0, 20.mm)
  line a, b
  line b, c
  line c, d
  line d, a

  fillet a, 5.mm      # rounded corner
  chamfer c, 3.mm     # 45° bevel, 3 mm back along each edge
end.extrude(6.mm)
```

`fillet` takes a radius; `chamfer` takes the setback distance along each
adjacent segment. Both accept unit values (`5.mm`) and both take the corner
point itself, so a named point can be modified from anywhere in the sketch.

Use the sketch-level version when the rounding is part of the profile's
definition — a filleted plate outline stays filleted through every later pad,
pocket, or boolean. Use `Shape#fillet` when you want to soften edges of the
finished solid, including edges that no sketch created.

Errors are reported before any geometry is built:

- A modifier larger than its adjacent segments (`corner modifier at :a is too
  large: it needs 30.0 but the segment is 20.0`).
- Two modifiers whose setbacks overlap on a shared segment.
- More than one modifier on the same corner.
- A modifier on a point that is not a corner of the closed loop.

### Trimming and extending segments

`trim` and `extend` shorten or lengthen an individual segment of a line-based
sketch. One endpoint slides along the segment's own direction while the other
anchors it; because adjacent segments share their corner point, the corner
moves with it and the loop stays closed.

Give the distance with `by:`, or an intersection with `to:`:

```ruby
plate = sketch do
  a = point(:a, 0, 0)
  b = point(:b, 40.mm, 0)
  c = point(:c, 40.mm, 20.mm)
  d = point(:d, 0, 20.mm)
  bottom = line a, b
  line b, c
  line c, d
  line d, a

  trim c, d, by: 10.mm     # pull :d in to x = 10, making a trapezoid
  extend bottom, by: 5.mm  # push :b out to x = 45
end.extrude(6.mm)
```

With `to:`, the moved endpoint lands where the segment's *infinite* line
crosses the reference segment's infinite line — so the reference need not
physically reach the segment, which is what makes `extend` useful:

```ruby
sketch do
  # …
  rail_a = construction_point(:rail_a, 60.mm, 0)
  rail_b = construction_point(:rail_b, 40.mm, 20.mm)
  extend a, b, to: construction_line(rail_a, rail_b)
end
```

Details:

- The segment can be given as two points (`trim c, d`) or as the array `line`
  and `construction_line` return (`trim top`), and either argument order
  identifies it.
- The argument order chooses the direction: the second point is the one that
  moves. `at: :start` moves the first point instead, and `at:` also accepts
  either endpoint directly.
- `to:` takes a reference segment in the same two forms. It may be a drawn
  `line` or a `construction_line` that exists only to aim at.
- Edits apply in declaration order after the constraint solver runs, so a
  later edit sees the result of an earlier one, and corner `fillet` /
  `chamfer` modifiers act on the moved corner's final position.
- Distances accept unit values (`10.mm`) and must be positive; use `extend`
  rather than a negative `trim`.

Errors are reported before any geometry is built:

- An edit that would collapse the segment (`trim at :b leaves no segment: it
  would take 40.0 down to -10.0`).
- A `to:` reference parallel to the segment, which never meets it.
- A `to:` intersection on the wrong side, which names the operation you
  wanted (`trim at :b would lengthen the segment from 40.0 to 50.0; use
  extend instead`).
- A point pair that no `line` connects.
- Passing both `to:` and `by:`, or neither.

### Offsetting the profile

`offset` grows (positive) or shrinks (negative) the finished profile in its
own plane, keeping every edge parallel to where it started:

```ruby
seal = sketch do
  a = point(:a, 0, 0)
  b = point(:b, 40.mm, 0)
  c = point(:c, 40.mm, 20.mm)
  d = point(:d, 0, 20.mm)
  line a, b
  line b, c
  line c, d
  line d, a

  offset 3.mm       # 46 x 26 outline with r = 3 corners
end.extrude(2.mm)
```

The offset is the last step of building the profile, so it applies to whatever
the sketch produced — a constrained polygon including its corner fillets and
`trim` / `extend` edits, or a `circle_at`, `arc_at`, or `slot_between` profile.
Growing rounds the corners it opens up; shrinking leaves interior corners
square.

A sketch takes at most one `offset`, and the distance must be non-zero — use a
negative distance to shrink rather than a second call. An inward offset larger
than the profile raises rather than returning an empty shape.

`Shape#offset_2d(distance)` is the same operation on an already-built profile,
including profiles with holes; see
[Features and Modifiers](04-features-and-modifiers.md).

### Spline segments

`spline` draws a curved segment of the outline. It takes the two endpoints and
the interior points the curve must pass through:

```ruby
scoop = sketch do
  a = point(:a, 0, 0)
  b = point(:b, 10.mm, 0)
  c = point(:c, 10.mm, 10.mm)
  d = point(:d, 0, 10.mm)
  line a, b
  line b, c
  spline c, d, through: [[5.mm, 14.mm]]   # top edge bows up
  line d, a
end.extrude(2.mm)
```

The curve reaches the profile as a real interpolated BSpline edge, not a
polyline standing in for one, so it stays smooth through export and every
later pad, pocket, or boolean. A four-segment sketch gives a four-edge
profile whether or not one of those segments is curved.

Interior points may be sketch points as well as literal pairs, so the solver
can place them:

```ruby
sketch do
  # …
  mid  = midpoint(:mid, c, d)
  peak = point(:peak, nil, 14.mm)
  vertical mid, peak            # peak.x follows the midpoint's
  spline c, d, through: [peak]
end
```

Because a curve can close a loop on its own, a sketch with a spline needs only
two segments rather than the usual three — one curve and one straight edge is
a valid outline.

Two limits follow from what the other operations measure:

- `fillet` and `chamfer` set back along a straight run, so a corner where a
  spline meets another segment cannot be modified (`cannot fillet :d: it joins
  a spline segment`). Corners between two straight segments are unaffected,
  even in a sketch that has curves elsewhere.
- `trim` and `extend` slide an endpoint along a straight segment, so they
  reject spline segments.

`offset` and the patterns work on the finished profile and apply to curved
sketches unchanged.

### Patterning the profile

`linear_pattern`, `polar_pattern`, and `grid_pattern` repeat the finished
profile. The result is a single profile holding every copy, so one `extrude`,
`pad`, or `pocket` applies to all of them — six bolt holes become one pocket
rather than six:

```ruby
plate = box(60.mm, 60.mm, 10.mm).translate(-30.mm, -30.mm, 0)

plate = plate.pocket(:top, depth: 3.mm) do
  sketch do
    c = point(:c, 20.mm, 0)
    circle_at c, 3.mm
    polar_pattern count: 6           # 6 holes on a 40 mm bolt circle
  end
end
```

| Pattern | Arguments |
|---------|-----------|
| `linear_pattern` | `count:` and at least one non-zero `dx:` / `dy:`; copy *i* sits at *i* × (`dx`, `dy`) |
| `polar_pattern` | `count:`, optional `center:` (a sketch point, an `[x, y]` pair, or the origin) and `angle:` (default `360`); copy *i* sits at *i* × (`angle` / `count`) |
| `grid_pattern` | `nx:`, `ny:`, `dx:`, `dy:` — the two axes combined |

Patterning is the last step of building the profile, after corner modifiers,
`trim` / `extend`, and `offset`, so every copy carries that shaping. A
`center:` that names a corner the sketch moved uses the corner's final
position.

```ruby
rail = sketch do
  a = point(:a, 0, 0)
  b = point(:b, 8.mm, 0)
  c = point(:c, 8.mm, 8.mm)
  d = point(:d, 0, 8.mm)
  line a, b
  line b, c
  line c, d
  line d, a

  fillet a, 2.mm                     # every copy is filleted
  linear_pattern count: 5, dx: 20.mm
end.extrude(3.mm)
```

A sketch takes one pattern, `count:` must be a positive Integer, and a count
of 1 is a no-op so a parametric script can drive it down to a single copy. A
linear or grid pattern with no spacing along an axis it repeats on is
rejected, since every copy would land on the original.

These three names also exist as top-level functions taking a shape —
`polar_pattern(circle(3).translate(20, 0, 0), 6, 360)`. That form still works
inside a sketch block, so a block can build compound geometry itself and
return it directly.

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
