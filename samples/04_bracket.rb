# 04_bracket.rb — a simple L-shaped mounting bracket.
#
# Demonstrates a realistic multi-step part: base plate with two mounting holes
# and a vertical flange with one central hole.
#
# Requires: Phase 1 (primitives, booleans)
#           Phase 2 (fillet on vertical edges)

THICKNESS  = 4.0
BASE_W     = 60.0
BASE_D     = 40.0
FLANGE_H   = 30.0
HOLE_R     =  3.5   # M7 clearance
FILLET_R   =  2.0

# --- base plate ---
base = box(BASE_W, BASE_D, THICKNESS)

# Mounting holes (countersunk not modelled yet — plain through holes).
hole_bl = cylinder(HOLE_R, THICKNESS + 2).translate(10,        10,        -1)
hole_br = cylinder(HOLE_R, THICKNESS + 2).translate(BASE_W-10, 10,        -1)
base    = base.cut(hole_bl).cut(hole_br)

# --- vertical flange ---
flange     = box(BASE_W, THICKNESS, FLANGE_H).translate(0, BASE_D - THICKNESS, THICKNESS)
flange_hole = cylinder(HOLE_R, THICKNESS + 2)
              .translate(BASE_W / 2, BASE_D - THICKNESS - 1, THICKNESS + FLANGE_H / 2)
flange     = flange.cut(flange_hole)

# --- assemble ---
bracket = base.fuse(flange)

# Fillet all vertical edges for a production look (Phase 2).
# bracket = bracket.fillet(FILLET_R)

bracket.export("bracket.step")
