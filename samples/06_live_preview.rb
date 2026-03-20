# 06_live_preview.rb — open the live browser viewer while editing.
#
# Run with:
#   cargo run -- --preview samples/06_live_preview.rb
#
# rrcad tessellates the model, starts an axum HTTP server, and opens a
# Three.js viewer in the default browser.  Saving this file triggers an
# automatic re-evaluation and mesh push over WebSocket.
#
# Requires: Phase 1 (primitives), Phase 3 (preview / WebSocket server)

part = solid do
  box 50, 30, 10
  fillet 3, edges: :vertical
  cut do
    cylinder r: 6, h: 12, at: [10, 15, -1]
    cylinder r: 6, h: 12, at: [40, 15, -1]
  end
end

# Export for downstream tools as well.
part.export("preview_part.step")

# Open (or refresh) the browser viewer.
preview part
