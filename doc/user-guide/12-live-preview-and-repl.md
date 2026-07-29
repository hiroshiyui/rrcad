# 12. Live Preview and REPL

Two interactive modes shorten the iteration loop: a browser-based **live
preview** that re-renders on every save, and a **REPL** for poking at the
DSL one line at a time.

## Live preview

```bash
cargo run -- --preview script.rb
```

1. Launches an HTTP server. The port auto-selects a free port by default;
   `--preview-port` or `preview_port` in `rrcad.toml` pin a fixed one. The
   exact URL is printed at startup.
2. Opens your browser to a Three.js 3D viewer.
3. Watches the script file; on every save, re-evaluates the script and
   pushes the new geometry over WebSocket.
4. Call `preview(shape)` in your script to specify which shape to display.

```ruby
part = box(50, 40, 20).fillet(3)
preview part            # send to browser
part.export("part.step")
```

Press Ctrl-C to stop the server.

### Viewer controls

The browser viewer has a hamburger menu (top-right corner) and keyboard
shortcuts:

| Control | Action |
|---------|--------|
| Left-drag | Orbit |
| Right-drag / two-finger drag | Pan |
| Scroll / pinch | Zoom |
| **F** | Toggle flat-line view (white flat-shaded surfaces + gray edge lines) |
| **A** | Toggle axes helper |
| **M** | Toggle measurement mode |
| **Esc** | Clear the active measurement; press again to exit measurement mode |

The hamburger menu exposes the same toggles plus a **Scene** selector:

| Menu section | Options |
|-------------|---------|
| View | Normal (PBR studio material) · Flat-line (technical illustration style) |
| Scene | Showroom (dark studio, default) · White (bright neutral background) |
| Show | Axes on/off · Measure on/off · Explode on/off |
| Section | Off, X/Y/Z clipping planes, and offset slider |

The top-left model panel updates with each `preview(shape)` call and
reports the shape type, BRep validation status, bounding-box size,
volume, and surface area for quick inspection while iterating.

Hovering the model shows a best-fit face or edge label, clicking prints
its selector into the panel, and the **Explode** menu option separates
top-level parts for inspection. If a preview update fails, the panel
shows the error message instead of silently keeping the old state. In
measurement mode, click two model points to draw a cyan segment and
report their 3D distance in millimetres.

### The feature tree

Below the model panel, the **Features** panel lists the modelling history the
shape carries — the same data `shape.feature_graph` returns, laid out the way a
CAD tree is. The chain that leads to the previewed shape runs flush left; a
branch feeding into it — the tool body of a boolean, say — is indented beneath
the step that consumes it, and the merge is marked with the node it pulls in:

```
box(dx=80, dy=50, dz=30)
fillet(radius=4)
  cylinder(radius=3, height=40)
  translate(dx=20, dy=25, dz=-5)
cut()                                  +#6
```

Click a row to see its full recorded entry, which carries detail the short
label leaves out — a boolean's operand kinds, for instance. Node numbers are the
stable IDs from `feature_graph`, so a row can be matched to the corresponding
entry in a script that inspects the graph. Click the **Features** heading to
collapse the panel.

The panel is read-only: it shows how the shape was built, but the script
remains the place where the model is edited. Changing a parameter there and
saving re-runs the whole script, and the tree redraws with it.

For a preview-friendly starter script, see
[`samples/06_live_preview.rb`](../../samples/06_live_preview.rb).

## REPL

```bash
cargo run
```

| Feature | Description |
|---------|-------------|
| Tab completion | Press Tab after `.` to autocomplete Shape methods |
| History | Up/down arrows recall previous lines |
| `help` | Print the full DSL reference |
| `exit` / `quit` / Ctrl-D | Exit |

**Typical use** — interactively build and export a shape:

```
rrcad> b = box(10, 20, 30)
=> #<Shape:Solid>
rrcad> b.fillet(2).translate(5, 0, 0).export("out.step")
```

## Worked example: iterative preview loop

```ruby
# scratch.rb
w = param :width, default: 40
part = box(w, 30, 8).fillet(2).chamfer(0.4, :vertical)
preview part
```

```bash
cargo run -- --preview scratch.rb
# In your editor, change `box(w, 30, 8)` to `box(w, 30, 12)` and save.
# The browser shows the new geometry instantly.
```

---

[← Previous: Parametric Design and Batch Export](11-parametric-and-batch.md) · [Index](../user-guide.md) · [Next: MCP Server →](13-mcp-server.md)
