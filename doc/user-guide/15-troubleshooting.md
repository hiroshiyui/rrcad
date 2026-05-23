# 15. Troubleshooting

Most rrcad errors fall into one of four categories: build / install,
sketch solver failures, OCCT geometry failures (fillets, booleans), and
export failures. This chapter shows how to read the error messages and
the most common fixes.

## Reading error messages

OCCT errors raised from boolean, fillet / chamfer, extrude / sweep /
loft, Part Design, and import / export operations lead with the call
site and operand kind, and the most common failures carry a one-line
`hint:` suffix. For example:

```
fillet(r=10) on solid failed: BRepFilletAPI_MakeFillet: ...
  hint: radius likely exceeds the smallest adjacent face/edge; try a
        smaller value or use fillet_sel with an edge selector
```

If a sketch fails to solve, the message names the involved points and
the actual vs expected values, for example:

```
conflicting tangent constraint: distance from :c to line (:p1, :p2) is 2.0,
expected radius 3.0
```

A non-convergence error lists every unresolved point with its missing
coordinates so the offending free variable is easy to spot.

For deeper geometry failures, set `RRCAD_DEBUG_EXPORTS=1` to emit STEP
debug artifacts under `RRCAD_DEBUG_EXPORTS_DIR` or the system temp
directory. The error message includes the debug directory path so you
can open the failing operands directly.

## Build failures

**`rake: command not found`**

```bash
gem install rake
cargo clean && cargo build
```

**`BRepPrimAPI_MakeBox.hxx: No such file`** — missing OCCT headers:

```bash
sudo apt-get install libocct-modeling-data-dev
```

See [chapter 1](01-getting-started.md#prerequisites) for the full list of
OCCT packages.

## Fillet / chamfer failures

OCCT fillets can fail on degenerate topology produced by booleans:

- Try a smaller radius.
- Apply fillets *before* booleans where possible.
- Check the shape first: `puts part.validate`.
- Restrict the fillet to a known-good edge set with `fillet_sel` plus an
  edge selector ([chapter 6](06-topology-and-selectors.md)).

## Boolean failures

- Ensure the two shapes actually overlap (translate one slightly if
  necessary).
- Check both shapes are valid before the operation
  (`puts a.validate; puts b.validate`).
- For many operands, prefer `fuse_all` / `cut_all` over chained calls —
  they handle degenerate cases more gracefully.

## General shape issues

Run the four-line health check from [chapter 9](09-inspection-and-cam.md):

```ruby
puts part.validate      # "ok" or list of errors
puts part.shape_type    # :solid, :shell, :face, …
puts part.closed?
puts part.manifold?
```

A non-manifold solid usually means a boolean produced a degenerate edge —
try translating the operands by a small amount and retrying.

## Export failures

- Ensure the target directory exists.
- Use ASCII-only paths on platforms where non-ASCII filesystem support
  is unreliable.
- For STL / GLB / OBJ failures, try a smaller `linear_deflection` if
  tessellation diverges.

For the long-form troubleshooting reference, see
[`doc/troubleshooting.md`](../troubleshooting.md).

---

[← Previous: Recipes](14-recipes.md) · [Index](../user-guide.md)
