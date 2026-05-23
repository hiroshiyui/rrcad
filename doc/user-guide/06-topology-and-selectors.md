# 6. Topology and Selectors

To apply a feature to a specific face — say, fillet only the top edge, or
counterbore only the mounting face — you need a way to *name* sub-shapes
without hard-coding indexes. rrcad's selector system gives you symbolic
names (`:top`, `:vertical`) plus direction strings (`">Z"`, `"<X"`), and
lets you bind your own labels via the named-topology API.

## Selecting faces, edges, vertices

| Method | Description |
|--------|-------------|
| `.faces(:all)` / `:top` / `:bottom` / `:side` | Select faces by orientation |
| `.faces(">Z")` / `"<Z"` / `">X"` / `"<X"` / `">Y"` / `"<Y"` | Select face by outward-normal direction |
| `.edges(:all)` / `:vertical` / `:horizontal` | Select edges |
| `.vertices(:all)` | All vertices |

These return an `Array` of Shape objects.

**Typical use** — pick the top face for a `pad`, the vertical edges for a
chamfer:

```ruby
top_face  = part.faces(:top).first
side_edge = part.edges(:vertical).first
```

## Named topology

Register named selectors for recurring faces and edges, or attach a datum
shape for later reuse.

| Method | Description |
|--------|-------------|
| `name_face(name, selector)` | Name a face selector such as `:top` or `">Z"` |
| `name_edge(name, selector)` | Name an edge selector such as `:vertical` |
| `datum(name, shape)` | Store a reference shape such as a datum plane |
| `ref(name)` | Resolve a named face, edge, or datum |

**Typical use** — give a face a meaningful name and reuse it later:

```ruby
part = box(10, 20, 30)
part.name_face(:mounting_face, :top)
part.name_edge(:boss_edges, :vertical)
part.datum(:fixture_plane,
           datum_plane(origin: [0, 0, 0], normal: [0, 0, 1], x_dir: [1, 0, 0]))

part.faces(:mounting_face).first
part.edges(:boss_edges).length
part.ref(:fixture_plane)
```

## Structured GD&T

Use `Shape#gdt` when you want a drawing annotation to live with the model
instead of passing `datum:` / `feature_control:` ad hoc at export time.

```ruby
part = box(20, 10, 5)
face = part.faces(:top).first

part.gdt(standard: :asme) do |g|
  g.datum :A, face: face
  g.feature_control text: "⌀0.1", face: face, datums: [:A, :B]
end

part.export("drawing.svg")

part.gdt(standard: :iso) do |g|
  g.datum :A, face: face
  g.feature_control text: "⌀0.1", face: face, datums: [:A, :B]
end
part.export("drawing.svg")
```

See [chapter 10](10-import-export.md) for the equivalent inline `datum:` /
`feature_control:` export form.

## Worked example: corner-only fillet on a labelled face

```ruby
# labelled.rb
plate = box(60, 40, 8)
plate.name_face(:logo_face, :top)
plate.name_edge(:rim, :vertical)

plate = plate.fillet(1.0)                                 # break every edge
plate = plate.pad(:logo_face, height: 1) do               # raised logo pad
  centered_rectangle point(:c, 30, 20), 20.mm, 6.mm
end

plate.export("labelled.step")
preview plate
```

---

[← Previous: Part Design](05-part-design.md) · [Index](../user-guide.md) · [Next: Patterns and Surfaces →](07-patterns-and-surfaces.md)
