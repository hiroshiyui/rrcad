# 12. Live Preview and REPL

Two interactive modes shorten the iteration loop: a browser-based **live
preview** that re-renders on every save, and a **REPL** for poking at the
DSL one line at a time.

## Live preview

```bash
cargo run -- --preview script.rb
```

1. Launches an HTTP server (default `http://localhost:3000`, or a free
   port if `3000` is taken or `preview_port` is set in `rrcad.toml`).
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
| Show | Axes on/off |
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
