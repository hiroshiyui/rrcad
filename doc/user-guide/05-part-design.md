# 5. Part Design

The part-design DSL captures the workflow CAD packages call "sketch on a
face, then pad or pocket": add material on an existing face, or cut into
it. This chapter also covers reference geometry (datum planes), helical
features (threads), ready-made fastener bodies for assemblies, and sheet
metal — folded parts, which are built from a recipe of bends so that the
folded solid and the flat blank can both be derived from it.

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
| `nut_pocket(size, depth:, style: :hex, clearance: 0.2, slot: nil)` | Hex (or `:square`) recess for a captive nut; `slot:` opens a slide-in channel along +Y |
| `standoff_pocket(size, depth:, clearance: 0.2)` | Hex recess that keeps a threaded standoff from spinning |
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

## Labels — emboss and engrave

`text` renders a string as glyph outline faces; extrude gives it depth, then
fuse raises it off the surface and cut sinks it in. On a quad frame the
CW/CCW motor markings are genuinely functional, not decoration.

```ruby
plate = box(40, 12, 3)
label = text("X-450 V2", size: 6).extrude(0.6)
embossed = plate.fuse(label.translate(4, 3, 3))    # raised off the top
engraved = plate.cut(label.translate(4, 3, 2.4))   # sunk into the top
```

`font:` picks a family by name or takes a `.ttf`/`.otf` path; without it the
system sans-serif is used.

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

## Sheet metal

A sheet-metal part is one sheet of constant thickness, folded. That constraint
is what earns it its own builder: the folded solid and the flat blank the
laser cuts are two views of the same part, and the blank cannot be recovered
from finished geometry. Unfolding has to know where each bend line ran and how
tight the bend is — only the recipe knows that, so the recipe is what you
write.

```ruby
part = sheet_metal(thickness: 2, radius: 2, k_factor: 0.44) do |s|
  s.base 100, 60
  s.flange :xmax, length: 25
  s.flange :xmin, length: 15, angle: 45
end

part.export("bracket.step")      # folded
part.export_flat("bracket.dxf")  # blank, 1:1, ready to cut
```

`base` is the flat plate. It lies in the XY plane with its lower-left corner
at the origin and its thickness running up in +z; flanges fold upward off one
of its four sides, named `:xmin`, `:xmax`, `:ymin`, `:ymax`.

Two dimensions are easy to mistake for each other:

- `radius:` is the **inner** bend radius — the one the tool leaves on the
  inside of the fold, not the part's outside corner. It defaults to the
  thickness, which is the usual shop starting point.
- `length:` is the straight leg **past** the bend, not the overall height. A
  90° flange therefore stands `thickness + radius + length` tall, and opening
  the radius does not silently shorten the wall.

`angle:` is how far the material turns: 90 gives a wall square to the base,
180 a hem folded back over it. Anything over 180 is refused.

### The blank

`flat` develops the part, and `export_flat` writes it as a 1:1 cut file —
`.dxf` or `.svg`, through the same writer as
[`export_outline`](10-import-export.md#cut-files-for-laser-and-cnc), so
circular edges come out as true `CIRCLE` / `ARC` entities.

The blank is longer than the plate by the **bend allowance**: the arc length
of the neutral axis, the line through the thickness that neither stretches nor
compresses. That is `angle × (radius + k × thickness)`, and `k_factor:` is
where that axis sits, as a fraction of the thickness. 0.44 is a fair default
for mild steel; get the real figure from whoever bends the part, because it is
the difference between a part that fits and one that is a millimetre short on
every fold.

```ruby
part.flat_size.map { |v| v.round(2) }   # => [146.79, 60.0]
```

For the bracket above: 100 mm of plate, plus 4.52 + 25 for the square fold,
plus 2.26 + 15 for the 45° one. Neither the leg alone (140) nor the outside
girth (154) is the right answer.

`bends` reports what each fold consumed, for a press-brake setup sheet:

```ruby
part.bends.each do |b|
  puts "#{b[:side]}  #{b[:angle]} deg  r#{b[:radius]}  leg #{b[:length]}  " \
       "allowance #{b[:allowance].round(3)}"
end
```

```
xmax  90.0 deg  r2.0  leg 25.0  allowance 4.524
xmin  45.0 deg  r2.0  leg 15.0  allowance 2.262
```

Each row also carries `from:`, `to:`, and the `relief:` style that was used.

### Partial flanges and bend relief

`from:` / `to:` narrow a flange to part of a side, measured along the global
axis that side runs in — x for `:ymin` / `:ymax`, y for `:xmin` / `:xmax`.

A flange that stops short of the corner tears the plate where the fold ends,
so a narrowed flange gets **bend relief** automatically: a notch at each end of
the bend line, one thickness wide and `radius + thickness` deep by default.

```ruby
s.flange :ymin, length: 15, from: 10, to: 50                      # relieved
s.flange :ymin, length: 15, from: 10, to: 50, relief: :obround    # rounded end
s.flange :ymin, length: 15, from: 10, to: 50, relief: :none       # no notch
```

`:obround` ends the notch in a half-round, which is what you want if the part
is going to be cycled — a square internal corner is where a crack starts. It
reaches the cut file as a real arc, not a chord approximation. `relief_width:`
and `relief_depth:` override the defaults; the depth includes the round end
rather than adding to it, so switching styles does not move the clearance.

A notch is skipped at any end where the flange already runs to the corner, and
a flange that spans its whole side cannot be relieved at all — there are no
bend ends to relieve, so asking for it raises rather than quietly doing
nothing.

### A tray

Four flanges, each inset from the corners so the reliefs have room:

```ruby
tray = sheet_metal(thickness: 1.5, radius: 1.5) do |s|
  s.base 120, 80
  [:xmin, :xmax].each { |e| s.flange e, length: 25, from: 6, to: 74 }
  [:ymin, :ymax].each { |e| s.flange e, length: 25, from: 6, to: 114 }
end

tray.flat_size          # => [176.8, 136.8]  — the sheet it has to nest on
tray.export("tray.step")
tray.export_flat("tray.dxf")
preview tray.to_shape
```

Two flanges on neighbouring sides that both run into their shared corner are
refused. They would meet there at a single point with no material joining
them, and the blank would pinch to nothing — a folded solid that looks
entirely plausible right up until someone tries to cut it. Inset one of them
and let its relief notch carry the corner.

### What this does not do

- **Holes are not developed.** `flat` gives the outline. A hole in a bend zone
  moves and distorts as the metal wraps, and guessing where it lands is worse
  than not guessing. Cut holes into `flat` yourself where they sit on flat
  runs: `tray.flat.cut(circle(2).translate(30, 20, 0)).export_outline("x.dxf")`.
- **One flange per side, folding from the base only.** There is no flange on a
  flange, and no non-rectangular base.
- `to_shape` gives the folded `Shape`, so anything in this guide that takes a
  shape — booleans, `mass_properties`, drawings, `preview` — works on it.

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
