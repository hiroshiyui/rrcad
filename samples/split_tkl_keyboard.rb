# ============================================================
# Split TKL Mechanical Keyboard — Complete Assembly
# ============================================================
# Specs:
#   86 keys total (36 left + 51 right)
#   Cherry MX switches, 19.05 mm key spacing (ANSI layout)
#   Hand-wired, Raspberry Pi Pico per side
#   Ethernet interconnect (RJ-45, both back walls)
#   USB micro on left back wall → host computer
#   5° forward pitch (wrist→finger), solid wedge base
#   M2 corner screws, chamfered case + plate edges
# ============================================================

U       = 19.05   # 1 key unit (mm)
SW      = 14.0    # Cherry MX plate cutout
PT      = 3.0     # Plate thickness
WT      = 3.0     # Case wall / floor thickness
CH      = 18.0    # Case interior height (wiring clearance)
MG      = 8.0     # Margin: switch body edge → plate edge
SCREW_D = 5.0     # M2 boss/via centre distance from plate edge
POST_R  = 2.5     # Boss outer radius
M2_R    = 1.2     # M2 clearance / tap hole radius
CHAMFER_CASE  = 1.5
CHAMFER_PLATE = 1.0
PITCH   = 5.0     # Forward tilt in degrees

# Raspberry Pi Pico board dimensions
PICO_W = 21.0; PICO_L = 51.0
# Pico mounting hole offsets from board corner (Raspberry Pi Pico datasheet):
# four M2.1 holes at (2, 2), (2, 19), (49, 2), (49, 19) mm in board coordinates.
PICO_HOLES = [[2.0, 2.0], [2.0, 19.0], [49.0, 2.0], [49.0, 19.0]]
# M2.5 copper heat-set insert standoffs
M25_BOSS_R   = 3.5   # standoff outer radius (7 mm OD; 2.3 mm wall around insert)
M25_BOSS_H   = 4.0   # standoff height = insert pocket depth (insert length: 4 mm)
M25_INSERT_R = 1.6   # press-fit hole radius (3.2 mm Ø for M2.5 knurled insert)
USB_W  =  8.0; USB_H  =  3.5
ETH_W  = 16.0; ETH_H  = 14.0

# Row Y-centres (Y-up: fn row at top = highest Y)
R0 = 5.5 * U   # Fn row
R1 = 4.5 * U   # Number row
R2 = 3.5 * U   # QWERTY row
R3 = 2.5 * U   # Home row
R4 = 1.5 * U   # Shift row
R5 = 0.5 * U   # Bottom row

# ── Corner M2 screw positions in plate coordinates ───────────
def screw_pts(pw, ph, d)
  [[d, d], [pw-d, d], [d, ph-d], [pw-d, ph-d]]
end

# ── Build switch plate with cutouts + M2 via holes ───────────
def build_plate(key_centres)
  sw = SW; pt = PT; mg = MG; sd = SCREW_D; m2r = M2_R
  xs = key_centres.map { |p| p[0] }
  ys = key_centres.map { |p| p[1] }
  ox = sw/2.0 + mg - xs.min
  oy = sw/2.0 + mg - ys.min
  shifted = key_centres.map { |p| [p[0]+ox, p[1]+oy] }
  pw = xs.max - xs.min + sw + 2.0*mg
  ph = ys.max - ys.min + sw + 2.0*mg
  plate = box(pw, ph, pt)
  shifted.each do |key|
    cx = key[0]; cy = key[1]
    plate = plate.cut(box(sw, sw, pt+2.0).translate(cx-sw/2.0, cy-sw/2.0, -1.0))
  end
  screw_pts(pw, ph, sd).each do |sx, sy|
    plate = plate.cut(cylinder(m2r, pt+2.0).translate(sx, sy, -1.0))
  end
  plate = plate.chamfer(CHAMFER_PLATE)
  [plate, pw, ph]
end

# ── Build case shell with chamfer + corner screw bosses ───────
def build_case(pw, ph)
  wt = WT; ch = CH; sd = SCREW_D; pr = POST_R; m2r = M2_R
  outer  = box(pw+2.0*wt, ph+2.0*wt, ch+wt).chamfer(CHAMFER_CASE)
  cshape = outer.cut(box(pw, ph, ch).translate(wt, wt, wt))
  post_h = ch - PT - 0.5
  screw_pts(pw, ph, sd).each do |sx, sy|
    bx = wt+sx; by = wt+sy
    cshape = cshape
               .fuse(cylinder(pr, post_h).translate(bx, by, wt))
               .cut(cylinder(m2r, post_h+2.0).translate(bx, by, wt-1.0))
  end
  cshape
end

# ── Extra mid-edge M2 boss+via pairs for plate–case rigidity ──
# pts: [[x, y], …] in plate coordinates.
# Returns [plate, cshape] with via holes and matching screw bosses added.
def add_mid_bosses(plate, cshape, pts)
  post_h = CH - PT - 0.5   # boss height: reaches just below plate underside
  pts.each do |sx, sy|
    plate  = plate.cut(cylinder(M2_R, PT+2.0).translate(sx, sy, -1.0))
    cshape = cshape
               .fuse(cylinder(POST_R, post_h).translate(WT+sx, WT+sy, WT))
               .cut(cylinder(M2_R, post_h+2.0).translate(WT+sx, WT+sy, WT-1.0))
  end
  [plate, cshape]
end

# ── Solid wedge base for forward pitch tenting ────────────────
# Fills the triangular gap under the tilted case so the bottom
# is flat and the assembly sits flush on a table.
def solid_tent(half, total_w, total_d, pitch)
  rad    = pitch * Math::PI / 180.0
  max_wh = total_d * Math.sin(rad) + 2.0
  cutter = box(total_w+10, total_d+10, max_wh+50)
             .translate(-5, -5, 0)
             .rotate(1, 0, 0, pitch)
  box(total_w, total_d, max_wh).cut(cutter)
    .fuse(half.rotate(1, 0, 0, pitch))
end

# ════════════════════════════════════════════════════════════
# LEFT SIDE — 36 keys
#   Fn:     Esc + F1–F6
#   Num:    ` 1–6
#   QWERTY: Tab + Q–T
#   Home:   CapsLock + A–G
#   Shift:  LShift + Z–B
#   Bottom: LCtrl + Win + LAlt + Space
# ════════════════════════════════════════════════════════════
lk = []
lk << [0.5*U, R0]
(1..6).each { |i| lk << [(1.5+i)*U, R0] }        # F1–F6
(0..6).each { |i| lk << [(0.5+i)*U, R1] }        # ` 1–6
lk << [0.75*U, R2]                                # Tab (1.5 U)
(0..4).each { |i| lk << [(2.0+i)*U, R2] }        # Q W E R T
lk << [0.875*U, R3]                               # CapsLock (1.75 U)
(0..4).each { |i| lk << [(2.25+i)*U, R3] }       # A S D F G
lk << [1.125*U, R4]                               # LShift (2.25 U)
(0..4).each { |i| lk << [(2.75+i)*U, R4] }       # Z X C V B
lk << [0.75*U, R5]; lk << [2.0*U,  R5]           # LCtrl, Win
lk << [3.25*U, R5]; lk << [5.5*U,  R5]           # LAlt, Space

lplate, lpw, lph = build_plate(lk)
lcase = build_case(lpw, lph)

# USB micro — back wall at 1/4 width (outer/left side)
# Z raised to WT+M25_BOSS_H so the cutout aligns with the Pico PCB level.
lcase = lcase.cut(
  box(USB_W, WT+2.0, USB_H)
    .translate(WT + lpw/4.0 - USB_W/2.0, WT+lph-1.0, WT + M25_BOSS_H)
)
# RJ-45 — back wall at 3/4 width (inner/right side)
lcase = lcase.cut(
  box(ETH_W, WT+2.0, ETH_H)
    .translate(WT + 3.0*lpw/4.0 - ETH_W/2.0, WT+lph-1.0, WT)
)
# M2.5 heat-set insert standoffs for Pico (left side)
# Pico rotated 90°: 21 mm along case X, USB end facing back wall (51 mm along case Y).
# Board SWD-end corner placed at (lpico_x, lpico_y); USB end sits flush at back wall.
# This centres the USB port on the existing cutout at x = WT + lpw/4.
lpico_x = WT + lpw/4.0 - PICO_W/2.0
lpico_y = WT + lph - PICO_L
# Datasheet holes (X_b, Y_b) map to case offsets (+Y_b, PICO_L−X_b) after 90° rotation:
# (2,2)→(+2,+49)  (2,19)→(+19,+49)  (49,2)→(+2,+2)  (49,19)→(+19,+2)
[[2.0, 49.0], [19.0, 49.0], [2.0, 2.0], [19.0, 2.0]].each do |hx, hy|
  bx = lpico_x + hx
  by = lpico_y + hy
  lcase = lcase.fuse(cylinder(M25_BOSS_R, M25_BOSS_H).translate(bx, by, WT))
  lcase = lcase.cut(cylinder(M25_INSERT_R, M25_BOSS_H + 1.0).translate(bx, by, WT - 0.5))
end
# 3 extra M2 bosses for left plate–case rigidity:
#   ox = SW/2 + MG − 0.5·U = 5.475 mm (plate coordinate origin offset)
#   1. plate centre — clear of Pico board (X 30–51, Y 74–125) on both axes
#   2. bottom edge — midpoint of the 2.25U LAlt–Space gap (x = 4.375U + ox)
#   3. right edge — mid-height
_lox = SW/2.0 + MG - 0.5*U
lplate, lcase = add_mid_bosses(lplate, lcase, [
  [lpw / 2.0,       lph / 2.0    ],   # plate centre ← avoids Pico footprint
  [4.375*U  + _lox, SCREW_D      ],   # bottom edge, LAlt–Space gap
  [lpw - SCREW_D,   lph / 2.0    ],   # right edge, mid-height
])

# ════════════════════════════════════════════════════════════
# RIGHT SIDE — 51 keys  (compact; ≈ 20.7 cm case width)
#   Fn:     F7–F12 + PrtSc/ScrLk/Pause + Ins        ← nav col
#   Num:    7–= + BS + Home                          ← nav col
#   QWERTY: Y–] + \ + PgUp                          ← nav col
#   Home:   H–' + Enter + Del                       ← nav col
#   Shift:  N–/ + RShift(2U) + Up↑ + End            ← nav col
#   Bottom: Space + RAlt/Fn/RCtrl + Left←/Down↓/Right→ + PgDn  ← nav col
#
# Nav column at X = 9.5 U (single column, all 6 nav keys)
# Direction inverted-T: Up(R4 @7.5U), Left/Down/Right(R5 @6.5/7.5/8.5U)
# ════════════════════════════════════════════════════════════
rk = []
(0..5).each { |i| rk << [(0.5+i)*U, R0] }        # F7–F12
rk << [6.5*U,R0]; rk << [7.5*U,R0]; rk << [8.5*U,R0]   # PrtSc ScrLk Pause
rk << [9.5*U, R0]                                  # Ins          ← nav col

(0..5).each { |i| rk << [(0.5+i)*U, R1] }        # 7 8 9 0 - =
rk << [7.0*U,  R1]                                 # BS (2 U)
rk << [9.5*U,  R1]                                 # Home         ← nav col

(0..6).each { |i| rk << [(0.5+i)*U, R2] }        # Y U I O P [ ]
rk << [7.75*U, R2]                                 # \ (1.5 U)
rk << [9.5*U,  R2]                                 # PgUp         ← nav col

(0..5).each { |i| rk << [(0.5+i)*U, R3] }        # H J K L ; '
rk << [7.125*U, R3]                                # Enter (2.25 U)
rk << [9.5*U,   R3]                                # Del          ← nav col

(0..4).each { |i| rk << [(0.5+i)*U, R4] }        # N M , . /
rk << [5.5*U,   R4]                                # RShift (2 U)
rk << [7.5*U,   R4]                                # Up ↑         ← direction cluster
rk << [9.5*U,   R4]                                # End          ← nav col

rk << [1.5*U,  R5]                                 # Space (inner)
rk << [3.5*U,  R5]; rk << [4.5*U,  R5]            # RAlt, Fn
rk << [5.5*U,  R5]                                 # RCtrl
rk << [6.5*U,  R5]; rk << [7.5*U,  R5]            # Left ←, Down ↓  ← direction cluster
rk << [8.5*U,  R5]                                 # Right →       ← direction cluster
rk << [9.5*U,  R5]                                 # PgDn         ← nav col

rplate, rpw, rph = build_plate(rk)
rcase = build_case(rpw, rph)

# RJ-45 — back wall, centred
rcase = rcase.cut(
  box(ETH_W, WT+2.0, ETH_H)
    .translate(WT+rpw/2.0-ETH_W/2.0, WT+rph-1.0, WT)
)
# M2.5 heat-set insert standoffs for Pico (right side — Pico near inner/left edge)
# Board corner origin in case coordinates: x = WT+6, y = WT+rph/2-PICO_W/2
PICO_HOLES.each do |hx, hy|
  bx = WT + 6.0 + hx
  by = WT + rph/2.0 - PICO_W/2.0 + hy
  rcase = rcase.fuse(cylinder(M25_BOSS_R, M25_BOSS_H).translate(bx, by, WT))
  rcase = rcase.cut(cylinder(M25_INSERT_R, M25_BOSS_H + 1.0).translate(bx, by, WT - 0.5))
end

# Extra M2 mid-edge bosses — right side (3 additional plate↔case attachment points)
# Left-edge boss omitted: it would overlap the Pico board (case X 9–60, Y 55–76).
# Centre boss replaces it and also satisfies the central-region strength requirement.
_rox = SW/2.0 + MG - 0.5*U
rplate, rcase = add_mid_bosses(rplate, rcase, [
  [rpw / 2.0,      rph / 2.0  ],   # plate centre ← avoids Pico (X 6–57) and adds central support
  [2.5*U + _rox,   SCREW_D    ],   # bottom edge, Space–RAlt gap
  [rpw - SCREW_D,  rph / 2.0  ],   # right edge, mid-height
])

# ════════════════════════════════════════════════════════════
# PRINTABLE PARTS (4 separate pieces)
# ════════════════════════════════════════════════════════════

# Left case: case shell with solid wedge base (5° pitch, flat bottom)
left_case_part  = solid_tent(lcase, lpw+2.0*WT, lph+2.0*WT, PITCH)
# Left plate: flat 3 mm plate with switch cutouts + M2 via holes
left_plate_part = lplate

# Right case: case shell with solid wedge base
right_case_part  = solid_tent(rcase, rpw+2.0*WT, rph+2.0*WT, PITCH)
# Right plate: flat 3 mm plate with switch cutouts + M2 via holes
right_plate_part = rplate

# ── Individual exports ────────────────────────────────────────
left_case_part.export("left_case.step")
left_plate_part.export("left_plate.step")
right_case_part.export("right_case.step")
right_plate_part.export("right_plate.step")

# ── 2×2 preview layout (cases top row, plates bottom row) ────
gap = 40.0
col_x = lpw + 2.0*WT + gap
row_y = lph + 2.0*WT + gap
scene = left_case_part
          .fuse(right_case_part.translate(col_x, 0, 0))
          .fuse(left_plate_part.translate(0, row_y, 0))
          .fuse(right_plate_part.translate(col_x, row_y, 0))
preview scene
