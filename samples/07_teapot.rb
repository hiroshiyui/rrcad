# Utah Teapot — samples/07_teapot.rb
#
# Rebuilt from the Newell triangle mesh (doc/images/utah_teapot.obj).
# All coordinates are OBJ-native values scaled ×3.0 to rrcad working units.
#
# OBJ coordinate system (already Z-up):
#   Z = height (body 0.0 → 2.2 obj = 0.0 → 6.6 ours)
#   X = spout (+) / handle (−) direction
#   Y = depth (symmetric ±2.0 obj)
#
# Body profile derived from OBJ cylindrical cross-sections in the ±60° Y-axis
# band (avoids contamination from spout/handle vertices):
#   z_obj=0.0  r=1.52 → z=0.00 r=4.56
#   z_obj=0.2  r=1.75 → z=0.60 r=5.25  (widest in lower third)
#   z_obj=0.8  r=2.00 → z=2.40 r=6.00  (maximum girth)
#   z_obj=1.5  r=1.84 → z=4.50 r=5.53  (shoulder)
#   z_obj=2.0  r=1.64 → z=6.00 r=4.92  (neck)
#   z_obj=2.2  r=1.55 → z=6.60 r=4.64  (rim)
#
# Handle tube: OBJ ry=0.225 × 3 = 0.68 (→ r=0.70); centerline apex at x=−8.54 z=4.80.
# Spout tip:   OBJ center x=3.08, r=0.19 × 3 → x=9.23, r=0.56.
# Lid:         OBJ dome z=6.60→7.80; knob peaks at z=8.70 r≈1.09.

# ============================================================
# Parametric glaze colour
# ============================================================
glaze_r = param(:glaze_r, default: 0.96)  # cream white
glaze_g = param(:glaze_g, default: 0.92)
glaze_b = param(:glaze_b, default: 0.84)

# ============================================================
# Step 1 — Body loft (OBJ-derived profile, 8 cross-sections)
# ============================================================
# The OBJ body is more uniform than Newell parametric data suggests:
# widest girth (r=6.00) is at Z=2.40 (36% of height), not Z=2.0.
# The body tapers gently from rim (r=4.64) down to base (r=4.56),
# with a modest belly — significantly less extreme than the Newell approximation.
body = loft([
  circle(4.56).translate(0, 0, 0.00),  # base            (obj z=0.0 r=1.52)
  circle(5.25).translate(0, 0, 0.60),  # lower           (obj z=0.2 r=1.75)
  circle(5.65).translate(0, 0, 1.20),  # lower-mid       (obj z=0.4 r=1.88)
  circle(5.94).translate(0, 0, 1.80),  # upper-mid       (obj z=0.6 r=1.98)
  circle(6.00).translate(0, 0, 2.40),  # widest          (obj z=0.8 r=2.00)
  circle(5.53).translate(0, 0, 4.50),  # shoulder        (obj z=1.5 r=1.84)
  circle(4.92).translate(0, 0, 6.00),  # neck            (obj z=2.0 r=1.64)
  circle(4.64).translate(0, 0, 6.60),  # rim             (obj z=2.2 r=1.55)
])

# ============================================================
# Step 2 — Handle sweep (OBJ-derived C-arc, ear style)
# ============================================================
# Centerline traced directly from OBJ mesh cross-sections (x < −2.0 filter).
# Apex at x=−8.54 z=4.80 (obj: cx=−2.85, z=1.60).
# Tube radius 0.70 matches OBJ ry=0.225 × 3.0 = 0.68 (rounded to 0.70).
#
# Endpoints extend to x=−4.0 (inside body by ≈1.8 units at both attachment
# heights) to ensure solid volume overlap for a clean boolean fuse.
# The sweep tube crosses the body wall on the way out and back in, creating
# two neat circular openings — the classic ear-handle look.
handle_path = spline_3d([
  [-4.00, 0.0, 1.50],  # inside body — bottom attachment (body r=5.86 here)
  [-7.00, 0.0, 2.40],  # outside body — lower outer curve
  [-8.30, 0.0, 3.60],  # outside body — outer arc rising
  [-8.54, 0.0, 4.80],  # C-arc apex   (obj cx=−8.54, z=4.80)
  [-8.10, 0.0, 5.40],  # outside body — outer arc descending
  [-7.00, 0.0, 6.00],  # outside body — upper outer curve
  [-3.50, 0.0, 6.30],  # inside body — top attachment    (body r=4.78 here)
])
handle = circle(0.70).sweep(handle_path)
body_handle = body.fuse(handle)

# ============================================================
# Step 3 — Tapered spout loft (OBJ-derived centerline + radii)
# ============================================================
# Spout centerline runs from inside the body (base at x=4.50, inside body
# surface at r=5.86) curving up-and-outward to the pour tip at x=9.23 z=6.90.
# Radii at perpendicular cross-sections:
#   base ≈1.40  (estimated at body junction, OBJ angle-distorted at low z)
#   tip  ≈0.56  (OBJ: ry=0.187 × 3 at z_obj=2.3, near-perpendicular cut)
spout = loft([
  circle(1.40).translate(4.50, 0.0, 1.50),  # base — inside body
  circle(1.10).translate(6.50, 0.0, 2.80),  # lower arc
  circle(0.80).translate(7.80, 0.0, 4.50),  # mid arc
  circle(0.65).translate(8.10, 0.0, 5.70),  # upper arc
  circle(0.56).translate(9.23, 0.0, 6.90),  # pour tip
])
body_handle_spout = body_handle.fuse(spout)

# ============================================================
# Step 4 — Lid + knob subassembly (OBJ-derived dome profile)
# ============================================================
# OBJ lid profile (Y-axis band cross-sections × 3):
#   z=6.60 r=4.64 (rim, same as body top)
#   z=6.90 r=4.46
#   z=7.20 r=3.26 (dome narrows sharply)
#   z=7.50 r=1.37
#   z=7.80 r=0.60 (dome apex)
# Lid rim starts at z=6.50, r=4.80 — 0.10 below and 0.16 wider than body rim —
# ensuring volume overlap for a clean boolean fuse with the body.
lid = loft([
  circle(4.80).translate(0, 0, 6.50),  # rim — wider+lower than body for fuse
  circle(4.46).translate(0, 0, 6.90),  # below flare
  circle(3.26).translate(0, 0, 7.20),  # dome narrowing
  circle(1.37).translate(0, 0, 7.50),  # near apex
  circle(0.30).translate(0, 0, 7.80),  # dome apex (capped by knob)
])
# Knob: OBJ peaks at r≈1.09 at z=8.70 (obj: r_max=0.362 × 3).
knob = sphere(0.90).translate(0, 0, 8.40)
lid_assy = lid.fuse(knob)

# ============================================================
# Step 5 — Final assembly and export
# ============================================================
teapot = body_handle_spout.fuse(lid_assy).color(glaze_r, glaze_g, glaze_b)

teapot.export("07_teapot.step")
teapot.export("07_teapot.glb")
preview teapot
