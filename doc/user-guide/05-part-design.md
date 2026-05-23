# 5. Part Design

The part-design DSL captures the workflow CAD packages call "sketch on a
face, then pad or pocket": add material on an existing face, or cut into
it. This chapter also covers reference geometry (datum planes), helical
features (threads), and ready-made fastener bodies for assemblies.

## Pad and pocket

`pad` and `pocket` take a face selector and a sketch block. The sketch is
drawn in the face's local frame, then either fused (pad) or cut (pocket).

| Method | Description |
|--------|-------------|
| `.pad(face_sel, height: h) { sketch }` | Extrude a sketch onto a face and fuse |
| `.pocket(face_sel, depth: d) { sketch }` | Extrude a sketch into a face and cut |

**Typical use** — add a circular boss to the top face of a plate:

```ruby
plate = box(60, 40, 6)
plate = plate.pad(:top, height: 4) do
  circle(5)
end
```

## Reference geometry

| Function | Description |
|----------|-------------|
| `datum_plane(origin:, normal:, x_dir:)` | Create a reference plane (store it with `name_face` / `datum`) |
| `helix(radius:, pitch:, height:)` | Helical Wire path, e.g. as a sweep path for threads |

```ruby
fixture = datum_plane(origin: [0, 0, 30], normal: [0, 0, 1], x_dir: [1, 0, 0])
spiral  = helix(radius: 10, pitch: 2, height: 20)
```

## Threads

| Function | Description |
|----------|-------------|
| `thread(solid, face_sel, pitch:, depth:)` | Cut a helical thread groove into a cylindrical face |

```ruby
post = cylinder(5, 30)
post = thread(post, :side, pitch: 1.0, depth: 0.5)
```

## Hole tools

These return sub-shapes designed to be subtracted from a part with
`.cut`. They share a consistent `size:`/`depth:` interface; size accepts a
symbol (`:m2`, `:m2_5`, `:m3`, `:m4`, `:m5`) or a numeric diameter.

| Function | Description |
|----------|-------------|
| `clearance_hole(size, depth:)` | Standard clearance hole |
| `tap_drill(size, depth:)` | Metric coarse tap-drill hole |
| `heat_set_insert(size, depth:)` | Pilot hole for a heat-set insert (`:m2`, `:m2_5`, `:m3`, or numeric) |
| `socket_head_cbore(size, depth:, head_depth:)` | Counterbore for a socket-head screw |
| `flat_head_csink(size, depth:, angle: 45)` | Countersink for a flat-head screw |
| `bearing_bore(size, depth:, fit: :press)` | Outer-diameter bore for deep-groove ball bearings (`:b608`, `:b623`, …, or numeric OD); `fit:` is `:press` or `:slip` |
| `cbore(d:, cbore_d:, cbore_h:, depth:)` | Generic counterbore tool |
| `csink(d:, csink_d:, csink_angle:, depth:)` | Generic countersink tool |

**Typical use** — drill four M3 clearance holes into a plate corner pattern:

```ruby
plate = box(60, 40, 6)
hole  = clearance_hole(:m3, depth: 6)
[[5, 5], [55, 5], [5, 35], [55, 35]].each do |x, y|
  plate = plate.cut(hole.translate(x, y, 0))
end
```

## Mating bodies (fasteners, shafts, bearings)

When you need a solid body for visualization or interference checking — not
a hole — use these helpers. They are also handy in assemblies.

| Function | Description |
|----------|-------------|
| `shaft(diameter, length:, fit: :nominal)` | Mating shaft cylinder; `fit:` is `:nominal`, `:press`, `:slip`, or `:running` |
| `screw(size, length:, style: :socket)` | Fastener body for `:m2`–`:m5`; `style:` is `:socket` (ISO 4762), `:button` (ISO 7380), or `:flat` (ISO 10642 90° conical) |
| `washer(size, thickness:)` | Plain washer body |
| `nut(size, thickness:, style: :hex)` | Nut body with a centered through hole; `style:` supports `:hex`, `:jam`, `:square`, `:flange`, `:nyloc` |

See [`samples/09_fastener_stack.rb`](../../samples/09_fastener_stack.rb)
for a small hardware-body example using `washer()` and `nut()`.

## Worked example: bracket with mounting holes and a boss

```ruby
# bracket.rb
bracket = box(60, 30, 6)
  .fillet(2, :vertical)
  .pad(:top, height: 8) { circle(6) }                 # central boss
  .pocket(:top, depth: 5) { circle(3) }               # bore through the boss

# Four M3 clearance holes near the corners.
hole = clearance_hole(:m3, depth: 6)
[[5, 5], [55, 5], [5, 25], [55, 25]].each do |x, y|
  bracket = bracket.cut(hole.translate(x, y, 0))
end

bracket.export("bracket.step")
preview bracket
```

For a worked sketch-driven bracket, see
[`samples/04_bracket.rb`](../../samples/04_bracket.rb).

---

[← Previous: Features and Modifiers](04-features-and-modifiers.md) · [Index](../user-guide.md) · [Next: Topology and Selectors →](06-topology-and-selectors.md)
