# ============================================================
# Leverless ("Hitbox"-style) Game Controller — Cherry MX
# ============================================================
# Specs:
#   14 play keys + 4 function keys, all Cherry MX (or MX-compatible)
#     Left hand : inverted-T arrow cluster (PC keyboard style, 19.05 mm pitch),
#                 tilted 10° clockwise (ARROW_ANGLE) to follow the forearm:
#                 Up above Down; Left / Down / Right under ring / middle / index
#     Right hand: 8 attack keys in two finger-staggered rows of 4
#     Thumbs    : 2 spare keys (e.g. L3 / R3), centred below the hands
#     Function  : Select / Start / Home / Capture, top-left, above the arrows
#   Every non-arrow keycap keeps ≥ ARROW_GAP of bare plate from the arrow
#   cluster (vs ~1 mm between ordinary neighbours) to avoid mis-hits.
#   Raspberry Pi Pico (RP2040) on M2 heat-set standoffs, micro-USB out the back
#   Two printed parts that fit a 220 × 220 mm bed:
#     case  — 206 × 150 × 19 mm tray, print floor-down, no supports
#     plate — 206 × 150 × 5 mm top plate, print TOP-FACE-DOWN, no supports
#   Plate screws to the case with 10 × M3 flat-head screws into heat-set inserts
#
# Coordinates: case corner at the origin, X to the right, Y away from the
# player (Y = 0 is the palm edge, Y = D is the back wall with the USB port).
# Key layout positions are given relative to LAYOUT_ORIGIN, Y-up.
# ============================================================

# ── Overall case ────────────────────────────────────────────
W      = 206.0   # case width  (X) — leaves 14 mm on a 220 mm bed for brim/skirt
D      = 150.0   # case depth  (Y) — includes a ~40 mm palm rest below the thumbs
WT     = 3.0     # wall thickness
FT     = 3.0     # floor thickness
CH     = 16.0    # cavity height: floor top → plate underside
PT     = 5.0     # top plate thickness
CORNER_R      = 8.0   # plan-view corner radius of case and plate
CHAMFER_TOP   = 1.0   # break on the plate's top outer edge
CHAMFER_BOT   = 1.0   # break on the case's bottom outer edge (elephant-foot relief)

# ── Cherry MX plate geometry ────────────────────────────────
# MX clips need a 1.5 mm ledge; the plate is 5 mm thick for stiffness, so a
# wider relief pocket is cut from the underside, leaving a 1.5 mm ledge on top.
SW_CUT    = 14.0   # MX square cutout (top 1.5 mm of the plate)
SW_LEDGE  = 1.5    # MX clip-in plate thickness
SW_RELIEF = 15.0   # underside relief pocket so the clips can snap open
CAP       = 18.2   # keycap footprint allowance for clearance checks (≥ XDA_BASE)

# ── Raspberry Pi Pico ───────────────────────────────────────
PICO_W = 21.0; PICO_L = 51.0; PICO_T = 1.0
# Mounting holes (datasheet): 2.0 mm in from each short edge, 11.4 mm apart
# across the width → 4.8 mm in from each long edge.
PICO_HOLES   = [[4.8, 2.0], [16.2, 2.0], [4.8, 49.0], [16.2, 49.0]]  # [x across, y along]
PICO_USB_OH  = 1.3     # micro-USB receptacle overhangs the board edge by ~1.3 mm
STANDOFF_H   = 4.0     # board underside height above the floor
STANDOFF_R   = 2.6     # boss radius around an M2 insert
USB_OPEN_W   = 12.0    # back-wall opening: fits a typical micro-USB overmold
USB_OPEN_H   = 8.0

# ── Plate fasteners: M3 flat head into heat-set inserts ─────
BOSS_R       = 4.5     # boss radius (corner/edge bosses merge into the wall)
BOSS_INSET   = 7.0     # boss centre from the case's outer edge
INSERT_DEPTH = 5.7     # M3 × 5.7 heat-set insert
PLATE_GAP    = 0.0     # plate rests directly on boss tops / wall top

# ── Rubber feet recesses (10 mm self-adhesive bumpers) ──────
FOOT_R = 5.4; FOOT_DEPTH = 1.0; FOOT_INSET = 16.0

# Keycaps: XDA profile, printed separately by samples/xda_keycaps.rb. Loaded
# here so the preview shows the real caps and the clearance checks can
# confirm they fit the CAP allowance.
require_relative "lib/xda_keycap"
raise "XDA keycap (#{XDA_BASE} mm) exceeds the CAP allowance" if XDA_BASE > CAP

# ════════════════════════════════════════════════════════════
# Key layout (mm, relative to LAYOUT_ORIGIN)
# ════════════════════════════════════════════════════════════
# Right hand: columns are 19.5 mm apart (≥ 19.05 MX pitch, a hair extra for
# fat fingers); the vertical stagger follows finger length, middle highest.
COL   = [20.0, 39.5, 59.0, 78.5]   # right-hand column X: index, middle, ring, pinky
STAG  = [18.0, 24.0, 22.0, 16.0]   # top-row Y stagger for the same fingers
ROW_DY = 20.0                      # top → bottom row spacing on the right hand

# Left hand: inverted-T arrows at exact keyboard pitch, so PC muscle memory
# carries over. ARROW_DOWN is the Down key; the others hang off it.
ARROW_PITCH = 19.05
ARROW_DOWN  = [-53.0, 5.0]         # Down key centre; tuned so the rotated cluster
                                   # clears Start (above) and THUMB_L (below)
ARROW_GAP   = 10.0                 # min bare plate, arrow cap ↔ any other cap
# Whole cluster rotated about the Down key, in degrees (CCW positive). The left
# forearm reaches in from the left toward the controller's centre, so turning
# the cluster clockwise lines its finger columns up with the forearm and keeps
# the wrist straight: Up leans toward the centre, and the Left–Down–Right row
# slopes down to the right. Use +10 for the opposite tilt, 0 for a square T.
ARROW_ANGLE = -10.0

LAYOUT_ORIGIN = [W / 2.0, 76.0]    # layout (0,0) in case coordinates

# Each key is [x, y, angle_deg]: its centre, and its rotation about that centre
# (the switch cutout and keycap turn with it; MX cutouts are square, so any
# angle works).
KEYS = {}
# Right hand: top row (punches) and bottom row (kicks).
%w[P1 P2 P3 P4].each_with_index { |k, i| KEYS[k] = [COL[i], STAG[i], 0.0] }
%w[K1 K2 K3 K4].each_with_index { |k, i| KEYS[k] = [COL[i], STAG[i] - ROW_DY, 0.0] }
# Left hand: inverted-T arrow cluster, laid out square around the Down key and
# then rotated as a unit by ARROW_ANGLE about Down's centre.
ARROWS = %w[LEFT DOWN RIGHT UP]
ca = Math.cos(ARROW_ANGLE * Math::PI / 180.0)
sa = Math.sin(ARROW_ANGLE * Math::PI / 180.0)
{ "LEFT" => [-ARROW_PITCH, 0.0], "DOWN" => [0.0, 0.0],
  "RIGHT" => [ARROW_PITCH, 0.0], "UP" => [0.0, ARROW_PITCH] }.each do |name, (ox, oy)|
  KEYS[name] = [ARROW_DOWN[0] + ox * ca - oy * sa,
                ARROW_DOWN[1] + ox * sa + oy * ca, ARROW_ANGLE]
end
# Thumbs: centred spare pair, 21 mm apart, dropped low enough to clear the
# arrow cluster by ARROW_GAP.
KEYS["THUMB_L"] = [-10.5, -27.0, 0.0]
KEYS["THUMB_R"] = [ 10.5, -27.0, 0.0]
# Function row, top-left — above the arrows with an ARROW_GAP margin, and
# well clear of the Pico at the back centre.
FUNC_Y = 54.0
%w[SELECT START HOME CAPTURE].each_with_index do |k, i|
  KEYS[k] = [-78.5 + i * 19.5, FUNC_Y, 0.0]
end

# Key poses [x, y, angle] in case coordinates, by name.
KEY_POSES = {}
KEYS.each { |k, (x, y, a)| KEY_POSES[k] = [LAYOUT_ORIGIN[0] + x, LAYOUT_ORIGIN[1] + y, a] }

# ── Pico placement: back-centre, USB facing the back wall ───
# The board's long axis runs along Y; its USB end sits PICO_USB_OH short of the
# inner back wall so the receptacle face lands flush with the wall's inner face.
PICO_X0 = W / 2.0 - PICO_W / 2.0
PICO_Y1 = D - WT - PICO_USB_OH           # USB-end board edge
PICO_Y0 = PICO_Y1 - PICO_L
PICO_STANDOFFS = PICO_HOLES.map { |hx, hy| [PICO_X0 + hx, PICO_Y0 + hy] }
USB_Z = FT + STANDOFF_H + PICO_T + 1.5   # receptacle centre height (~1.5 mm above PCB)

# ── Plate screw positions (case coordinates) ────────────────
SCREWS = [
  [BOSS_INSET, BOSS_INSET], [W - BOSS_INSET, BOSS_INSET],           # front corners
  [BOSS_INSET, D - BOSS_INSET], [W - BOSS_INSET, D - BOSS_INSET],   # back corners
  [W / 2.0, BOSS_INSET],                                            # front middle
  [W / 2.0 - 25.0, D - BOSS_INSET], [W / 2.0 + 25.0, D - BOSS_INSET], # back, flanking the Pico
  [BOSS_INSET, D / 2.0], [W - BOSS_INSET, D / 2.0],                 # side middles
  [LAYOUT_ORIGIN[0], LAYOUT_ORIGIN[1]],                             # centre, between the hands
]

# ── Sanity checks: run before any geometry is built ─────────
# Switch bodies (relief pocket) must not overlap each other, the bosses, or
# the Pico footprint; keycaps must stay inside the plate. Keys can be rotated,
# so footprints are handled as convex polygons and gaps are true distances.

# Corners of a square of half-width +half+ centred at (cx, cy), turned by +ang+°.
def square_pts(cx, cy, half, ang)
  c = Math.cos(ang * Math::PI / 180.0); s = Math.sin(ang * Math::PI / 180.0)
  [[-half, -half], [half, -half], [half, half], [-half, half]].map do |u, v|
    [cx + u * c - v * s, cy + u * s + v * c]
  end
end

# Distance from point p to segment a–b.
def seg_dist(p, a, b)
  abx = b[0] - a[0]; aby = b[1] - a[1]
  t = ((p[0] - a[0]) * abx + (p[1] - a[1]) * aby) / (abx * abx + aby * aby)
  t = [[t, 0.0].max, 1.0].min
  Math.hypot(p[0] - (a[0] + t * abx), p[1] - (a[1] + t * aby))
end

# Separating-axis test: true when two convex polygons overlap.
def polys_overlap?(p, q)
  [p, q].each do |poly|
    poly.each_with_index do |a, i|
      b = poly[(i + 1) % poly.length]
      nx = a[1] - b[1]; ny = b[0] - a[0]   # edge normal
      pp = p.map { |v| v[0] * nx + v[1] * ny }
      qq = q.map { |v| v[0] * nx + v[1] * ny }
      return false if pp.max <= qq.min || qq.max <= pp.min
    end
  end
  true
end

# Bare-plate gap between two convex polygons (negative when they overlap).
def poly_gap(p, q)
  return -1.0 if polys_overlap?(p, q)
  edges = ->(poly) { poly.each_with_index.map { |a, i| [a, poly[(i + 1) % poly.length]] } }
  d1 = p.map { |v| edges.(q).map { |a, b| seg_dist(v, a, b) }.min }.min
  d2 = q.map { |v| edges.(p).map { |a, b| seg_dist(v, a, b) }.min }.min
  [d1, d2].min
end

# Gap between a convex polygon and a circle (negative when they overlap).
def circle_gap(poly, cx, cy, r)
  centre = [cx, cy]
  inside = poly.each_with_index.all? do |a, i|
    b = poly[(i + 1) % poly.length]
    (b[0] - a[0]) * (cy - a[1]) - (b[1] - a[1]) * (cx - a[0]) >= 0   # CCW corners
  end
  return -r if inside
  poly.each_with_index.map { |a, i| seg_dist(centre, a, poly[(i + 1) % poly.length]) }.min - r
end

cap_poly    = ->((x, y, a)) { square_pts(x, y, CAP / 2, a) }
relief_poly = ->((x, y, a)) { square_pts(x, y, SW_RELIEF / 2, a) }
pico_poly   = square_pts(0, 0, 1, 0).map { |u, v| [W / 2.0 + u * PICO_W / 2, (PICO_Y0 + PICO_Y1) / 2 + v * PICO_L / 2] }

KEY_POSES.to_a.combination(2).each do |(na, a), (nb, b)|
  g = poly_gap(cap_poly.(a), cap_poly.(b))
  raise "keycaps #{na} / #{nb} collide (gap #{g.round(2)} mm)" if g < 0.5
end
# Arrow isolation: every other keycap keeps ARROW_GAP of bare plate from every
# arrow keycap, measured as true distance between the (rotated) caps.
KEY_POSES.each do |name, pose|
  next if ARROWS.include?(name)
  ARROWS.each do |arrow|
    g = poly_gap(cap_poly.(pose), cap_poly.(KEY_POSES[arrow]))
    raise "#{name} is only #{g.round(1)} mm from #{arrow} (need #{ARROW_GAP})" if g < ARROW_GAP
  end
end
KEY_POSES.each do |name, pose|
  SCREWS.each do |sx, sy|
    g = circle_gap(relief_poly.(pose), sx, sy, BOSS_R)
    raise "boss #{[sx, sy].inspect} hits switch #{name}" if g < 1.0
  end
  raise "switch #{name} sits over the Pico" if poly_gap(relief_poly.(pose), pico_poly) < 1.0
  cap_poly.(pose).each do |vx, vy|
    edge = [vx, W - vx, vy, D - vy].min
    raise "keycap #{name} overhangs the plate edge" if edge < WT + 2.0
  end
end

# ════════════════════════════════════════════════════════════
# Geometry
# ════════════════════════════════════════════════════════════

# Rounded-rectangle prism with its corner at the origin.
def rounded_slab(w, d, h, r)
  rect(w, d).fillet_wire(r).extrude(h)
end

# Rounded-rectangle prism whose top outer edge is bevelled 45° by +c+: a
# straight slab of height h − c topped by a 45° drafted (inward-tapering)
# extrude of height c.
def bevelled_slab(w, d, h, r, c)
  base = rounded_slab(w, d, h - c, r)
  cap  = rect(w, d).fillet_wire(r).extrude(c, draft: 45.deg).translate(0, 0, h - c)
  base.fuse(cap)
end

# ── Case (tray) ─────────────────────────────────────────────
def build_case
  # Bevel only the bottom outer edge: hides elephant's foot and feels nicer in
  # the lap. Built upside down as a bevelled slab, then flipped back.
  outer  = bevelled_slab(W, D, FT + CH, CORNER_R, CHAMFER_BOT).mirror("xy").translate(0, 0, FT + CH)
  cavity = rounded_slab(W - 2 * WT, D - 2 * WT, CH + 1.0, CORNER_R - WT)
             .translate(WT, WT, FT)
  body = outer.cut(cavity)

  # Plate bosses: full cavity height so the plate rests on them, with an M3
  # heat-set insert pocket from the top.
  bosses = SCREWS.map { |x, y| cylinder(BOSS_R, CH).translate(x, y, FT - 0.01) }
  body = fuse_all([body] + bosses)
  inserts = SCREWS.map do |x, y|
    heat_set_insert(:m3, depth: INSERT_DEPTH + 0.5).translate(x, y, FT + CH - INSERT_DEPTH)
  end

  # Pico standoffs with M2 heat-set insert pockets.
  standoffs = PICO_STANDOFFS.map { |x, y| cylinder(STANDOFF_R, STANDOFF_H).translate(x, y, FT - 0.01) }
  body = fuse_all([body] + standoffs)
  pico_inserts = PICO_STANDOFFS.map do |x, y|
    heat_set_insert(:m2, depth: STANDOFF_H).translate(x, y, FT + 0.5)
  end

  # Back-wall USB opening, centred on the Pico's receptacle. The bottom is
  # kept ≥ 2 mm above the floor so the wall prints as a clean bridge.
  usb = box(USB_OPEN_W, WT + 2.0, USB_OPEN_H)
          .translate(W / 2.0 - USB_OPEN_W / 2.0, D - WT - 1.0, USB_Z - USB_OPEN_H / 2.0)

  # Rubber-foot recesses in the underside.
  feet = [[FOOT_INSET, FOOT_INSET], [W - FOOT_INSET, FOOT_INSET],
          [FOOT_INSET, D - FOOT_INSET], [W - FOOT_INSET, D - FOOT_INSET]]
           .map { |x, y| cylinder(FOOT_R, FOOT_DEPTH + 0.5).translate(x, y, -0.5) }

  cut_all(body, inserts + pico_inserts + [usb] + feet)
end

# Place a shape modelled centred on the Z axis at a key pose: turn it about
# its own centre first, then move it to the key.
def at_key(shape, x, y, a)
  shape = shape.rotate(0, 0, 1, a) unless a == 0
  shape.translate(x, y, 0)
end

# ── Top plate (switch plate) ────────────────────────────────
def build_plate
  plate = bevelled_slab(W, D, PT, CORNER_R, CHAMFER_TOP)

  tools = []
  KEY_POSES.each_value do |x, y, a|
    # Through-cut: 14 × 14 MX square.
    tools << at_key(box(SW_CUT, SW_CUT, PT + 2.0).translate(-SW_CUT / 2, -SW_CUT / 2, -1.0), x, y, a)
    # Underside relief: leaves a 1.5 mm ledge at the top for the MX clips.
    tools << at_key(box(SW_RELIEF, SW_RELIEF, PT - SW_LEDGE + 1.0)
                      .translate(-SW_RELIEF / 2, -SW_RELIEF / 2, -1.0), x, y, a)
  end
  SCREWS.each do |x, y|
    # flat_head_csink is wide at Z=0 and narrows upward; flip it so the cone
    # opens at the plate's top face and the shank runs down through the plate.
    tools << flat_head_csink(:m3, depth: PT + 2.0).mirror("xy").translate(x, y, PT)
  end
  cut_all(plate, tools)
end

case_part  = build_case
plate_part = build_plate

# ── Pre-flight checks ───────────────────────────────────────
[["case", case_part], ["plate", plate_part]].each do |name, part|
  v = part.validate
  raise "#{name}: invalid BRep #{v.inspect}" unless v.to_s == "ok"
  fit = print_volume_check(part, x: 220, y: 220, z: 220)
  raise "#{name}: exceeds the 220 × 220 bed" unless fit[:fits]
  puts "#{name}: #{fit[:dx].round(1)} × #{fit[:dy].round(1)} × #{fit[:dz].round(1)} mm, " \
       "#{mass_estimate(part).round(1)} g PLA"
end

# ── Exports ─────────────────────────────────────────────────
case_part.export("hitbox_case.step")
case_part.export("hitbox_case.stl")
# Plate is exported in print orientation: flipped top-face-down so the switch
# relief pockets and countersinks need no supports and the top face is smooth.
plate_print = plate_part.mirror("xy").translate(0, 0, PT)
plate_part.export("hitbox_plate.step")
plate_print.export("hitbox_plate_print.stl")

# ── Exploded preview: every part lifted apart along Z ──────
# Each layer rises by EXPLODE mm above where it sits when assembled, so the
# parts can be inspected one by one while staying aligned in X/Y: standoffs
# line up under the Pico holes, bosses under the plate's countersinks, and
# switches over their cutouts. `--param explode=0` shows the assembled build.
EXPLODE = param :explode, default: 25, range: 0..200

plate_z = FT + CH + PLATE_GAP
sw_z    = plate_z + PT - SW_LEDGE + 1.5   # MX top housing seat height (= plate top)
# A resting MX stem tops out 11.6 mm above the plate; the cap's cross socket
# floor (MX_CROSS_DEPTH up from its skirt) sits on it.
cap_z   = plate_z + PT + MX_STEM_TOP - MX_CROSS_DEPTH

# Stand-in hardware for checking fit (not printed parts).
pico = box(PICO_W, PICO_L, PICO_T).translate(PICO_X0, PICO_Y0, FT + STANDOFF_H)
switches = KEY_POSES.values.map do |x, y, a|
  # MX top housing (15.6 × 15.6 × 6.6) plus the 14 × 14 lower body that hangs
  # below the clip ledge into the plate's relief pocket.
  top = box(15.6, 15.6, 6.6).translate(-7.8, -7.8, sw_z)
  low = box(SW_CUT, SW_CUT, 5.0).translate(-SW_CUT / 2, -SW_CUT / 2, sw_z - 5.0)
  at_key(top.fuse(low), x, y, a)
end
# Build the XDA cap once; each key gets a moved (and, for arrows, turned) copy.
xda = xda_keycap.translate(0, 0, cap_z)
caps = KEY_POSES.values.map { |x, y, a| at_key(xda, x, y, a) }

# Layers, bottom to top, with their explode step (0 = stays on the case).
LAYERS = [
  [[case_part],  0, [0.2, 0.22, 0.26]],   # printed: case tray
  [[pico],       1, [0.1, 0.5, 0.2]],     # Raspberry Pi Pico
  [[plate_part], 2, [0.75, 0.15, 0.2]],   # printed: top plate
  [switches,     3, [0.15, 0.15, 0.15]],  # Cherry MX switches
  [caps,         4, [0.85, 0.85, 0.88]],  # keycaps
]

exploded = assembly("hitbox_exploded") do |a|
  LAYERS.each do |shapes, step, rgb|
    shapes.each do |s|
      a.place s.translate(0, 0, step * EXPLODE).color(rgb[0], rgb[1], rgb[2])
    end
  end
end
scene = exploded.to_shape
scene.export("hitbox_exploded.glb")   # open in any glTF viewer for a 3D check
preview scene
