# Utah Teapot — samples/07_teapot.rb
#
# Approximation of the original Newell teapot using rrcad's Phase 4 DSL.
# Built incrementally: body → handle → spout → lid+knob.
#
# Coordinate system (Z-up, scaled from Newell units):
#   height : Newell-Y × 3.333 → OCCT-Z   (total body height = 7.5 units)
#   radius : Newell-X/Z × 3.5  → OCCT-X/Y (max radius     = 7.0)
#
# Key reference heights (Newell → OCCT):
#   foot ring  Y=0.15 r=1.50  →  Z=0.50 r=5.25
#   widest     Y=0.90 r=2.00  →  Z=2.00 r=7.00  (lowered for squat belly)
#   shoulder   Y=1.35 r=1.75  →  Z=4.50 r=6.13
#   neck       Y=1.65 r=1.40  →  Z=5.50 r=4.90
#   rim        Y=2.25 r=1.40  →  Z=7.50 r=4.90

# ============================================================
# Step 1 — Body loft
# ============================================================
# Flat base at r=3.00 (real teapots rest on a foot ring).
# Widest point at Z=2.00 (27 % of height) gives a squat, round belly.
# Section at Z=1.00 keeps the loft surface simple at the spout junction
# (Z=1.50) for robust boolean fuse.
body = loft([
  circle(3.00).translate(0, 0, 0.00),  # flat base
  circle(5.25).translate(0, 0, 0.50),  # foot ring     (Newell Y=0.15)
  circle(5.80).translate(0, 0, 1.00),  # lower belly   (below spout junction Z=1.5)
  circle(7.00).translate(0, 0, 2.00),  # widest        (lowered from Z=3.00)
  circle(6.80).translate(0, 0, 3.00),  # upper belly
  circle(6.13).translate(0, 0, 4.50),  # shoulder      (Newell Y=1.35)
  circle(4.90).translate(0, 0, 5.50),  # neck          (Newell Y=1.65)
  circle(4.90).translate(0, 0, 7.50),  # rim           (Newell Y=2.25)
])

# ============================================================
# Step 2 — Handle sweep, fused into body
# ============================================================
# Both endpoints inside the body for clean attachment.
handle_path = spline_3d([
  [-3.50,  0.0, 1.50],   # inside body — bottom attachment
  [-7.00,  0.0, 2.00],   # lower handle arc
  [-10.50, 0.0, 4.50],   # outer apex
  [-7.00,  0.0, 6.80],   # upper handle arc
  [-3.50,  0.0, 7.00],   # inside body — top attachment
])
handle = circle(1.00).sweep(handle_path)
body_handle = body.fuse(handle)

# ============================================================
# Step 3 — Spout sweep, fused into body+handle
# ============================================================
# Path starts inside the body so the fuse produces a gap-free junction.
spout_path = spline_3d([
  [ 4.00, 0.0, 1.50],   # inside body at spout-junction height
  [ 6.50, 0.0, 2.80],   # exit body surface
  [ 9.50, 0.0, 4.50],   # mid-spout arc
  [12.00, 0.0, 5.80],   # upper arc
  [14.00, 0.0, 6.50],   # pour tip — further out, less high, so opening points forward
])
spout = circle(1.80).sweep(spout_path)
body_handle_spout = body_handle.fuse(spout)

# ============================================================
# Step 4 — Lid + knob subassembly
# ============================================================
# Lid built as loft of circles (proper solid, not spline_2d.revolve shell).
# Rim at r=5.00 > body r=4.90, Z=7.40 < body top Z=7.50 — clear overlap,
# no near-coincident faces, robust boolean fuse.
lid = loft([
  circle(0.30).translate(0, 0, 8.70),  # near-apex (covered by knob)
  circle(1.50).translate(0, 0, 8.50),  # upper dome
  circle(3.00).translate(0, 0, 8.10),  # mid dome
  circle(4.00).translate(0, 0, 7.70),  # lower dome shoulder
  circle(5.00).translate(0, 0, 7.40),  # rim — wider than body, 0.10 below body top
])
knob = sphere(1.20).translate(0, 0, 9.10)
lid_assy = lid.fuse(knob)

# ============================================================
# Step 5 — Final assembly
# ============================================================
teapot = body_handle_spout.fuse(lid_assy)

teapot.export("07_teapot.step")
preview teapot
