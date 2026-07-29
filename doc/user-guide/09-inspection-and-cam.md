# 9. Inspection and CAM Checks

Before you export a part for machining or 3D printing, you want answers
to practical questions: is the BRep valid? what does it weigh? are there
unsupported overhangs? rrcad ships a small toolbox for these checks.

## Validation and introspection

| Method | Returns | Description |
|--------|---------|-------------|
| `.shape_type` | Symbol | `:solid`, `:shell`, `:face`, `:wire`, `:edge`, `:vertex`, `:compound`, `:compsolid` |
| `.validate` | `"ok"` or Array | List of topology errors, or `"ok"` |
| `.history` | Array of Strings | Provenance chain of modeling operations that produced the shape |
| `.feature_graph` | Array of Hashes | Dependency tree with stable node IDs, labels, parent IDs, and history entries |
| `.rebuild` | Shape | Replays the stored feature tree from its recorded parents |
| `.closed?` | Boolean | True if every edge has ≥ 2 adjacent faces |
| `.manifold?` | Boolean | True if every edge has exactly 2 adjacent faces |
| `.volume` | Float | Enclosed material in mm³; `0.0` for an open surface |
| `.surface_area` | Float | Total area of every face, in mm² |
| `.centroid` | `[x, y, z]` | Centre of mass |
| `.inertia` | Hash | `{ixx:, iyy:, izz:, ixy:, ixz:, iyz:}` inertia tensor |
| `.min_thickness` | Float | Minimum wall thickness |
| `.distance_to(other)` | Float | Minimum distance between two shapes (0 if touching) |
| `.normal` (on a Face) | `[nx, ny, nz]` | Outward unit normal of a planar face; raises if the shape isn't a face |
| `.cylinder_axis` (on a Face) | Hash | `{origin:, axis:, radius:}` for a cylindrical face; raises if the face isn't cylindrical |

**Typical use** — quick sanity check before exporting:

```ruby
puts part.validate      # "ok" or list of errors
puts part.shape_type    # :solid, :shell, :face, …
puts part.closed?
puts part.manifold?
```

`closed?` and `manifold?` are *topological* tests — they count how many faces
each edge is shared by. That is not the same question as "is this watertight",
and the difference surprises people: a `sphere` reports `closed? == false`
because its seam edge belongs to one face, and an imported STL reports `false`
because its triangles are never sewn together. Both are perfectly good solids.
Treat a `false` as "worth a look", not as a defect.

`volume` does not rely on that test. It measures the enclosed material and
returns `0.0` for an **open** surface — a Shell with a free boundary, such as a
`ruled_surface` with no end caps. Such a surface encloses nothing, so there is
no volume to report and no mass either:

```ruby
skin = ruled_surface(bottom_loop, top_loop)
skin.volume            # 0.0 — a surface, not a part
mass_estimate(skin)    # 0.0

skin.thicken(1.5).volume   # a real number: it has walls now
```

Spheres, boolean results, and imported meshes all measure normally.

**Typical use** — inspect the modeling chain:

```ruby
part = box(10, 20, 30).translate(5, 0, 0).scale(2)
puts part.history.join(" -> ")

graph = part.feature_graph
puts graph.map { |node| "#{node[:id]}: #{node[:label]} <- #{node[:parents].inspect}" }
```

The same graph is drawn as a **Features** panel in the live preview — see
[Live Preview and REPL](12-live-preview-and-repl.md#the-feature-tree). The node
numbers match, so a row in the browser can be tied back to a node here.

## CAM and 3D-printing checks

Lightweight manufacturability helpers built on top of the inspection
methods above.

| Function | Description |
|----------|-------------|
| `mass_estimate(part, density: 1.24)` | Rough mass in grams from `part.volume × density / 1000` (mm³ × g/cm³). Default density is PLA; pass ABS 1.04, PETG 1.27, steel 7.85, etc. |
| `print_volume_check(part, x:, y:, z:)` | Returns `{fits:, dx:, dy:, dz:, overflow_x:, overflow_y:, overflow_z:}` against a rectangular build volume |
| `overhang_faces(part, max_angle_deg: 45)` | Faces whose outward normal tips downward more than the threshold (assumes the part is +Z-up) |
| `draft_faces(part, axis: [0, 0, 1], min_draft_deg: 1.0)` | Faces with insufficient mould draft along the pull axis |
| `hole_axes(part, orientation: nil, tolerance_deg: 5.0)` | Enumerate cylindrical faces. Filter with `orientation:` `:vertical` (axis ‖ Z) or `:horizontal` (axis ⊥ Z) |
| `unsupported_islands(part, layer_height: 0.2, axis: :z, min_area: 0.0, tolerance: 0.05)` | Slice the part by layers and report disconnected footprints that do not overlap the previous layer |

**Typical use** — print check before slicing:

```ruby
report = unsupported_islands(part, layer_height: 0.2, axis: :z)
report.each do |layer|
  puts "#{layer[:offset]}: #{layer[:unsupported].length} unsupported islands"
end

vol = print_volume_check(part, x: 220, y: 220, z: 250)
puts vol[:fits] ? "fits on the printer" : "exceeds bed by #{vol[:overflow_x]} x #{vol[:overflow_y]}"
```

## Worked example: printable lid pre-flight

A small "do my checks pass" preamble you can drop into any printable
script:

```ruby
# lid_preflight.rb
lid = box(80, 50, 4).fillet(2).shell(-2)
preview lid

raise "invalid BRep: #{lid.validate.inspect}" unless lid.validate == "ok"

puts "mass:        #{mass_estimate(lid, density: 1.24).round(1)} g (PLA)"
puts "bed fit:     #{print_volume_check(lid, x: 220, y: 220, z: 250)[:fits]}"
puts "min wall:    #{lid.min_thickness.round(2)} mm"

overhangs = overhang_faces(lid, max_angle_deg: 45)
puts "overhangs:   #{overhangs.length} faces beyond 45°"

lid.export("lid.stl")
```

---

[← Previous: Assemblies](08-assemblies.md) · [Index](../user-guide.md) · [Next: Import and Export →](10-import-export.md)
