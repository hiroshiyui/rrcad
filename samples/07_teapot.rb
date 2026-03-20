# Utah Teapot — rrcad DSL example
#
# Approximation using spline profiles swept/revolved into solids.
# Run with:  cargo run -- samples/07_teapot.rb
# Preview:   cargo run -- --preview samples/07_teapot.rb

# --- Body (surface of revolution) ---
body_profile = spline_2d([
  [0.0, 0.0], [2.8, 0.3], [3.5, 1.2], [4.2, 3.5],
  [3.8, 6.0], [3.2, 7.2], [2.8, 7.8], [0.0, 7.8],
])
body = body_profile.revolve(360)

# --- Lid ---
lid_profile = spline_2d([
  [0.0, 0.0], [2.8, 0.1], [2.2, 0.6], [1.5, 1.3], [0.5, 1.7], [0.0, 2.0],
])
lid = lid_profile.revolve(360).translate(0.0, 0.0, 7.8)

# --- Knob on top of lid ---
knob = sphere(0.5).translate(0.0, 0.0, 10.3)

# --- Spout (circular cross-section swept along 3D curve) ---
spout_path = spline_3d([
  [4.0, 0.0, 2.5], [5.5, 0.0, 3.5], [7.5, 0.0, 5.5], [8.5, 0.0, 7.0],
])
spout = circle(0.7).sweep(spout_path)

# --- Handle ---
handle_path = spline_3d([
  [-4.0, 0.0, 2.0], [-7.0, 0.0, 3.0], [-7.5, 0.0, 5.5],
  [-7.0, 0.0, 8.0], [-4.0, 0.0, 8.5],
])
handle = circle(0.6).sweep(handle_path)

# --- Assemble ---
teapot = solid do
  body.fuse(lid).fuse(knob).fuse(spout).fuse(handle)
end

teapot.export("teapot.step")
puts "Exported teapot.step"
preview teapot
