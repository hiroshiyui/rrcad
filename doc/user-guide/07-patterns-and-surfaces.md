# 7. Patterns and Surfaces

Repeating geometry — bolt circles, vent grids, screw rows — is what
patterns are for. Surfaces fill in the gaps when you need a non-prismatic
skin (a ruled blend between two wires, a fairing patch). This chapter
covers both, plus the `slice` helper for cross-sections.

## Patterns

| Function | Description |
|----------|-------------|
| `linear_pattern(shape, n, dx, dy, dz)` | `n` copies, each translated by `(dx, dy, dz)` from the previous |
| `polar_pattern(shape, n, angle_deg)` | `n` copies rotated evenly around the Z axis |
| `grid_pattern(shape, nx, ny, dx, dy)` | `nx × ny` copies in a 2D grid |

**Typical use** — a row of bolts, a ring of bolts, a stud grid:

```ruby
bolt_row  = linear_pattern(cylinder(3, 20), 5, 15, 0, 0)
bolt_ring = polar_pattern(cylinder(3, 15).translate(30, 0, 0), 6, 360)
stud_grid = grid_pattern(cylinder(2, 5), 4, 3, 10, 10)
```

Patterns return a compound shape you can subtract from a body in a single
`.cut` call, which is much faster than cutting holes one at a time.

## Surfaces

| Function / Method | Description |
|-------------------|-------------|
| `ruled_surface(wire_a, wire_b)` | Ruled surface between two wires |
| `fill_surface(boundary_wire)` | Smooth NURBS patch filling a closed boundary |
| `.slice(plane: :xy, z: d)` | Cross-section at an axis-aligned plane |

**Typical use** — a faired blend between two profile wires:

```ruby
wire_top    = circle(10).translate(0, 0, 20)
wire_bottom = rect(30, 30)
skin = ruled_surface(wire_top, wire_bottom)
```

**Typical use** — section a part at Z=10 to inspect a wall thickness:

```ruby
section = part.slice(plane: :xy, z: 10)
section.export("section.svg")
```

## Worked example: vented disc

A round cover with a polar pattern of slotted vents — the kind of part you
might 3D-print as a fan grille:

```ruby
# vented_disc.rb
disc = cylinder(40, 4)

# One slot, then 12 of them around the disc.
slot = sketch do
  a = point(15, 0)
  b = point(35, 0)
  slot_between a, b, 2
end.extrude(6).translate(0, 0, -1)

slots = polar_pattern(slot, 12, 360)
disc  = disc.cut(slots).fillet(0.5, :horizontal)

disc.export("vented_disc.step")
preview disc
```

---

[← Previous: Topology and Selectors](06-topology-and-selectors.md) · [Index](../user-guide.md) · [Next: Assemblies →](08-assemblies.md)
