# ============================================================
# XDA-profile 1u keycap for Cherry MX switches — library
# ============================================================
# Definitions only; nothing is built or exported on load. Pull it in with
#   require_relative "lib/xda_keycap"
# and call xda_keycap to get one cap (see samples/xda_keycaps.rb).
#
# XDA is a uniform profile: every row has the same height and a shallow
# spherical dish, so one cap fits every key of a leverless controller.
# Dimensions follow common XDA measurements; manufacturers differ by a few
# tenths of a millimetre.
#
# Cap frame: centred on the Z axis, skirt bottom at Z = 0, top up (+Z) — the
# print orientation. Printed upright, nothing needs supports: the walls lean
# in only ~11°, the stem grows from the bed, and the ceiling is a ~15 mm
# bridge between the walls and the stem.
# ============================================================

XDA_BASE      = 18.0   # skirt footprint, square
XDA_TOP       = 14.5   # top face, square
XDA_HEIGHT    =  9.1   # skirt bottom → top face edge
XDA_BASE_R    =  1.5   # plan-view corner radius at the skirt
XDA_TOP_R     =  2.0   # plan-view corner radius at the top
XDA_DISH_R    = 40.0   # spherical dish radius
XDA_DISH_D    =  0.8   # dish depth at the centre, below the top-face plane
XDA_WALL      =  1.5   # side wall thickness (measured horizontally)
XDA_TOP_T     =  1.5   # material under the deepest point of the dish

# MX stem. The cross is a blind socket ~4 mm deep so the cap rides on the
# switch slider rather than sinking until its skirt hits the housing.
# FDM tolerance: real MX stems are ~4.0 × 1.17 mm; printed sockets need a
# little more room. Tighten toward 4.0 × 1.25 for resin or a snug printer.
MX_STEM_OD      = 5.5
MX_CROSS_L      = 4.1
MX_CROSS_W      = 1.35
MX_CROSS_DEPTH  = 4.0
# Height of a resting (unpressed) MX stem top above the plate's top face —
# where a cap's socket floor sits once installed.
MX_STEM_TOP     = 11.6

# Rounded square of side +w+ and corner radius +r+, centred on the Z axis
# at height +z+.
def xda_rounded_square(w, r, z)
  rect(w, w).fillet_wire(r).translate(-w / 2.0, -w / 2.0, z)
end

# One XDA keycap, in the cap frame described above.
def xda_keycap
  outer = loft([xda_rounded_square(XDA_BASE, XDA_BASE_R, 0.0),
                xda_rounded_square(XDA_TOP, XDA_TOP_R, XDA_HEIGHT)])

  # Hollow it: an inner loft parallel to the outer walls, starting just
  # below Z = 0 so the cut opens the bottom cleanly. Its ceiling sits
  # XDA_TOP_T below the dish's deepest point.
  ceiling = XDA_HEIGHT - XDA_DISH_D - XDA_TOP_T
  taper   = (XDA_BASE - XDA_TOP) / XDA_HEIGHT   # width lost per mm of height
  cavity  = loft([xda_rounded_square(XDA_BASE - 2 * XDA_WALL, XDA_BASE_R - 0.9, -0.01),
                  xda_rounded_square(XDA_BASE - 2 * XDA_WALL - taper * ceiling,
                                     XDA_TOP_R - 0.9, ceiling)])
  cap = outer.cut(cavity)

  # Spherical dish: the sphere's lowest point sits XDA_DISH_D below the top.
  dish = sphere(XDA_DISH_R).translate(0, 0, XDA_HEIGHT - XDA_DISH_D + XDA_DISH_R)
  cap  = cap.cut(dish)

  # Stem: a solid post from the bed up into the ceiling, with the MX cross
  # socket cut into its lower end.
  stem  = cylinder(MX_STEM_OD / 2.0, ceiling + 0.2)
  cross = box(MX_CROSS_L, MX_CROSS_W, MX_CROSS_DEPTH + 1.0)
            .translate(-MX_CROSS_L / 2.0, -MX_CROSS_W / 2.0, -1.0)
            .fuse(box(MX_CROSS_W, MX_CROSS_L, MX_CROSS_DEPTH + 1.0)
                    .translate(-MX_CROSS_W / 2.0, -MX_CROSS_L / 2.0, -1.0))
  cap.fuse(stem.cut(cross))
end
