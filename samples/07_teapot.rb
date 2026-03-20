# Utah Teapot — rrcad DSL example
#
# Control points derived from the reference STL (Wikipedia solid teapot).
# Key measurements from the reference:
#   body max radius ≈ 7.0  (at z ≈ 4.0–6.0)
#   body height     ≈ 8.5  → width:height ≈ 1.65 : 1
#   sharp shoulder  at z ≈ 6.0→6.5  (r drops 7.0 → 4.2)
#   spout exits body at  x ≈ 6.5, z ≈ 2.5
#   handle spans z ≈ 1.9 – 6.1, max x ≈ -8.2
#
# Run with:  cargo run -- samples/07_teapot.rb
# Preview:   cargo run -- --preview samples/07_teapot.rb

# --- Body (surface of revolution, XZ plane, [radius, z]) ---
#
# The wide nearly-cylindrical mid-section (z=3–6) and the abrupt shoulder
# at z=6→6.5 are the two defining features of the teapot silhouette.
body_profile = spline_2d([
  [0.0, 0.0],   # bottom centre (closes the solid base)
  [4.0, 0.3],   # foot ring — jump quickly to base radius
  [5.5, 1.5],   # lower belly
  [6.6, 2.5],   # lower equator
  [7.0, 4.0],   # widest — cylindrical section begins
  [7.0, 5.5],   # still wide (body is nearly cylindrical here)
  [7.0, 6.0],   # top of wide section (two close pts tighten the shoulder curve)
  [4.2, 6.5],   # shoulder — abrupt narrowing
  [3.8, 7.0],   # neck
  [0.0, 7.5],   # top centre (lid opening)
])
body = body_profile.revolve(360)

# --- Lid (sits on rim at z = 7.5) ---
#
# The lid base radius (3.8) matches the neck; height ≈ 1.0 (shallow dome).
lid_profile = spline_2d([
  [0.0, 0.0],
  [3.6, 0.1],   # base — fits just inside the neck
  [3.2, 0.4],   # dome flare
  [2.0, 0.7],
  [0.6, 0.9],
  [0.0, 1.0],   # apex
])
lid = lid_profile.revolve(360).translate(0.0, 0.0, 7.5)

# --- Knob ---
knob = sphere(0.5).translate(0.0, 0.0, 9.0)

# --- Spout ---
#
# Starts at x=5.5 (inside the body, whose radius at z=2.5 is ≈6.6) so the
# fuse creates a clean junction.  The path curves quickly to x=7.5 at z=3.5,
# which is clearly outside the body, producing a visible protrusion.
# Tip matches reference: (9.4, 0, 6.7).
spout_path = spline_3d([
  [5.5, 0.0, 2.5],
  [7.5, 0.0, 3.5],
  [9.0, 0.0, 5.5],
  [9.4, 0.0, 6.7],
])
spout = circle(0.85).sweep(spout_path)

# --- Handle ---
#
# D-shaped arc on the −X side.  Attachment z-range (2.0–6.1) matches the
# reference.  Max extension x = −8.2 also matches.
handle_path = spline_3d([
  [-6.0, 0.0, 2.0],
  [-8.0, 0.0, 3.0],
  [-8.2, 0.0, 4.5],
  [-8.0, 0.0, 5.5],
  [-6.0, 0.0, 6.1],
])
handle = circle(0.7).sweep(handle_path)

# --- Assemble ---
teapot = solid do
  body.fuse(lid).fuse(knob).fuse(spout).fuse(handle)
end

teapot.export("teapot.step")
puts "Exported teapot.step"
preview teapot
