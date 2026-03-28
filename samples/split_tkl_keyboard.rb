# ============================================================
# Split TKL Mechanical Keyboard — Complete Assembly
# ============================================================
# Specs:
#   86 keys total (36 left + 51 right)
#   Cherry MX switches, 19.05 mm key spacing (ANSI layout)
#   Hand-wired, Raspberry Pi Pico per side
#   USB-C interconnect (both back walls, inner sides) — UART TX/RX + GND only, no VCC on cable
#   USB micro on left back wall → host computer; right Pico gets its own USB to host
#   5° forward pitch (wrist→finger), solid wedge base
#   M2.5 corner screws (heat-set inserts), chamfered case + plate edges
# ============================================================

U       = 19.05   # 1 key unit (mm)
SW      = 14.0    # Cherry MX plate cutout
PT      = 3.0     # Plate thickness
WT      = 3.0     # Case wall / floor thickness
CH      = 18.0    # Case interior height (wiring clearance)
MG      = 8.0     # Margin: switch body edge → plate edge
SCREW_D = 5.0     # M2.5 boss/via centre distance from plate edge
POST_R  = 3.2     # Boss outer radius (6.4 mm OD; 1.6 mm wall around M2.5 insert)
M2_R    = 1.6     # M2.5 heat-set insert hole radius (3.2 mm Ø for M2.5 knurled insert)
CHAMFER_CASE  = 1.5
CHAMFER_PLATE = 1.0   # outer plate edge chamfer (applied to blank box before cuts)
CHAMFER_CUTS  = 0.5   # bottom-face lead-in step on switch cutouts (per-side overshoot, depth = same)
# Per-side clearance between plate and case cavity.
# FDM (PLA/PETG): 0.2  — ABS: 0.3  — Resin (SLA/MSLA): 0.1
FIT_TOL = 0.2

# Raspberry Pi Pico board dimensions
PICO_W = 21.0; PICO_L = 51.0
# Pico mounting hole offsets from board corner (Raspberry Pi Pico datasheet):
# four M2.1 holes at (2, 2), (2, 19), (49, 2), (49, 19) mm in board coordinates.
PICO_HOLES = [[2.0, 2.0], [2.0, 19.0], [49.0, 2.0], [49.0, 19.0]]
# M2.5 copper heat-set insert standoffs
M25_BOSS_R   = 3.5   # standoff outer radius (7 mm OD; 2.3 mm wall around insert)
M25_BOSS_H   = 4.0   # standoff height = insert pocket depth (insert length: 4 mm)
M25_INSERT_R = 1.6   # press-fit hole radius (3.2 mm Ø for M2.5 knurled insert)
USB_W        =  8.0; USB_H    = 3.5   # USB Micro cutout (Pico built-in port)
USBC_W       =  9.0; USBC_H   = 3.5   # USB-C connector opening (outer wall, 9 × 3.5 mm)
USBC_BOARD_W = 12.0               # adapter board width
USBC_BOARD_H =  4.2               # adapter board total height (PCB + connector)
SLOT_TOL     =  0.15              # per-side slot clearance for snug sliding fit
# M2.5 button-head screw counterbore (head Ø4.5mm → r=2.4mm with 0.15mm clearance).
# Depth 1.5mm leaves 1.5mm of plate below — head sits flush with plate surface.
# Gap from counterbore edge to nearest switch cutout = 8-(5+2.4) = 0.6mm — printable.
SCREW_CBR    =  2.4               # counterbore radius
SCREW_CBH    =  1.5               # counterbore depth

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
  # Chamfer outer plate edges first, before switch/via cuts, so the chamfer
  # does not conflict with the tight gap between counterbore rims and switch cutouts.
  # Step 1: chamfer outer plate edges on the blank box.
  plate = box(pw, ph, pt).chamfer(CHAMFER_PLATE)
  # Step 2: cut switch holes with a bottom-face lead-in step.
  # A box oversized by CHAMFER_CUTS on all sides is subtracted at the bottom, giving a
  # stepped lead-in that guides switch insertion without requiring OCCT chamfer calls
  # (which fail due to topology interactions with the pre-chamfered outer edges).
  c = CHAMFER_CUTS
  shifted.each do |key|
    cx = key[0]; cy = key[1]
    plate = plate.cut(box(sw, sw, pt+2.0).translate(cx-sw/2.0, cy-sw/2.0, -1.0))
    plate = plate.cut(box(sw+2.0*c, sw+2.0*c, c+1.0).translate(cx-(sw+2.0*c)/2.0, cy-(sw+2.0*c)/2.0, -1.0))
  end
  # Step 3: cut via shaft + counterbore.
  screw_pts(pw, ph, sd).each do |sx, sy|
    plate = plate.cut(cylinder(m2r, pt+2.0).translate(sx, sy, -1.0))
    # Counterbore on top face: M2.5 button head sits flush with plate surface.
    plate = plate.cut(cylinder(SCREW_CBR, SCREW_CBH).translate(sx, sy, pt - SCREW_CBH))
  end
  [plate, pw, ph]
end

# ── Build case shell with chamfer + corner screw bosses ───────
def build_case(pw, ph)
  wt = WT; ch = CH; sd = SCREW_D; pr = POST_R; m2r = M2_R
  outer  = box(pw+2.0*wt, ph+2.0*wt, ch+wt).chamfer(CHAMFER_CASE)
  # Cavity is expanded by FIT_TOL on each side so the plate slides in without binding.
  tol = FIT_TOL
  cshape = outer.cut(box(pw + 2.0*tol, ph + 2.0*tol, ch).translate(wt - tol, wt - tol, wt))
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
    plate  = plate.cut(cylinder(SCREW_CBR, SCREW_CBH).translate(sx, sy, PT - SCREW_CBH))
    cshape = cshape
               .fuse(cylinder(POST_R, post_h).translate(WT+sx, WT+sy, WT))
               .cut(cylinder(M2_R, post_h+2.0).translate(WT+sx, WT+sy, WT-1.0))
  end
  [plate, cshape]
end

# ── Screw-less support pillars (no plate via holes) ───────────
# Use for central positions where switch-cutout avoidance would
# complicate via placement.  Pillars rise to 0.2 mm below the
# plate underside so the plate rests on the corner screws while
# the pillars resist flex under typing load.
def add_pillars(cshape, pts)
  post_h = CH - PT - 0.2
  pts.each do |sx, sy|
    cshape = cshape.fuse(cylinder(POST_R, post_h).translate(WT+sx, WT+sy, WT))
  end
  cshape
end

# ── Wall slot for USB-C adapter board ────────────────────────
# Cuts a counterbore slot in the back wall inner face so an adapter board
# (USBC_BOARD_W × USBC_BOARD_H) can slide in from the top of the open case.
# The board face registers on the 2 mm deep pocket; the connector protrudes
# through the 9 × 3.5 mm outer opening.
#
# cx:     X centre of the slot in case coordinates
# wall_y: Y coordinate of the back wall inner face (= WT + ph)
def add_usbc_slot(cshape, cx, wall_y)
  slot_cz = WT + CH / 2.0                  # vertical centre of the port
  sw = USBC_BOARD_W + 2.0 * SLOT_TOL       # pocket width  (12.3 mm)
  sh = USBC_BOARD_H + 2.0 * SLOT_TOL       # pocket height (4.5 mm)
  sz = slot_cz - sh / 2.0                  # pocket floor Z

  # Inner guide channel: board-width pocket, 2 mm deep into wall, open at top
  # so the board can be dropped in before the plate is installed.
  cshape = cshape.cut(
    box(sw, 3.0, (WT + CH + 1.0) - sz)
      .translate(cx - sw / 2.0, wall_y - 1.0, sz)
  )
  # Outer connector opening: 9 × 3.5 mm through the full wall thickness.
  cshape = cshape.cut(
    box(USBC_W, WT + 2.0, USBC_H)
      .translate(cx - USBC_W / 2.0, wall_y - 1.0, slot_cz - USBC_H / 2.0)
  )
  cshape
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
(1..6).each { |i| lk << [(0.5+i)*U, R0] }        # F1–F6 (aligned with 1–6 below)
(0..6).each { |i| lk << [(0.5+i)*U, R1] }        # ` 1–6 (7 keys)
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
# USB-C interconnect slot — back wall at 3/4 width (inner/right side).
# Adapter board slides in from the top; connector protrudes out the back.
lcase = add_usbc_slot(lcase, WT + 3.0*lpw/4.0, WT + lph)
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
# Extra M2 screw bosses — left side (2 edge positions, via holes through plate)
#   ox = SW/2 + MG − 0.5·U = 5.475 mm (plate coordinate origin offset)
_lox = SW/2.0 + MG - 0.5*U
lplate, lcase = add_mid_bosses(lplate, lcase, [
  [4.375*U  + _lox, SCREW_D      ],   # bottom edge, LAlt–Space gap
  [lpw - SCREW_D,   lph / 2.0    ],   # right edge, mid-height
])
# Support pillars — screw-less columns that rise to 0.2 mm below the plate
# underside, resisting deflection under keystroke loads without via holes.
#
# All positions are diagonal midpoints between 4 adjacent key centres, giving
# ~13.5 mm clearance from every switch body edge (need > SW/2 + POST_R = 10.2 mm).
# Coordinates use the same per-axis offset _lox = SW/2 + MG − 0.5·U = 5.475 mm.
# Left Pico footprint in plate coords: X ≈ 26.8–47.8 mm, Y ≈ 74.3–125.3 mm —
# no pillar falls inside that rectangle.
lcase = add_pillars(lcase, [
  [1.4375*U + _lox, 3.0*U + _lox],  # mid R2–R3, left gap  (between CapsLock & Q cols)
  [4.0*U    + _lox, 5.0*U + _lox],  # mid R0–R1, centre    (between F3–F4 / 3–4 cols)
  [6.0*U    + _lox, 5.0*U + _lox],  # mid R0–R1, right     (between F5–F6 / 5–6 cols)
  [6.25*U   + _lox, 1.0*U + _lox],  # mid R4–R5, lower-right (B–last shift col, open below)
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
rk << [6.0*U,   R4]                                # RShift (2 U): left edge flush with / right edge (5.0 U), right edge flush with Up left edge (7.0 U)
rk << [7.5*U,   R4]                                # Up ↑         ← direction cluster
rk << [9.5*U,   R4]                                # End          ← nav col

rk << [0.5*U,   R5]                                # Space (1 U)
rk << [1.625*U, R5]                                # RAlt  (1.25 U): left flush with Space right (1.0 U)
rk << [2.875*U, R5]                                # Fn    (1.25 U)
rk << [4.125*U, R5]                                # RCtrl (1.25 U): right edge 4.75 U; 1.25 U gap before Left←
rk << [6.5*U,  R5]; rk << [7.5*U,  R5]            # Left ←, Down ↓  ← direction cluster
rk << [8.5*U,  R5]                                 # Right →       ← direction cluster
rk << [9.5*U,  R5]                                 # PgDn         ← nav col

rplate, rpw, rph = build_plate(rk)
rcase = build_case(rpw, rph)

# USB-C interconnect slot — back wall at 1/4 width (inner/left side, mirrors left half).
rcase = add_usbc_slot(rcase, WT + rpw/4.0, WT + rph)
# USB-C host slot — back wall at 3/4 width (outer/right side); right Pico → host computer.
rcase = add_usbc_slot(rcase, WT + 3.0*rpw/4.0, WT + rph)
# M2.5 heat-set insert standoffs for Pico (right side — Pico near inner/left edge)
# Board corner origin in case coordinates: x = WT+6, y = WT+rph/2-PICO_W/2
PICO_HOLES.each do |hx, hy|
  bx = WT + 6.0 + hx
  by = WT + rph/2.0 - PICO_W/2.0 + hy
  rcase = rcase.fuse(cylinder(M25_BOSS_R, M25_BOSS_H).translate(bx, by, WT))
  rcase = rcase.cut(cylinder(M25_INSERT_R, M25_BOSS_H + 1.0).translate(bx, by, WT - 0.5))
end

# Extra M2 screw bosses — right side (2 edge positions, via holes through plate)
_rox = SW/2.0 + MG - 0.5*U
rplate, rcase = add_mid_bosses(rplate, rcase, [
  [2.5*U + _rox,   SCREW_D    ],   # bottom edge, Space–RAlt gap
  [rpw - SCREW_D,  rph / 2.0  ],   # right edge, mid-height
])
# Support pillars (right plate ≈ 201 × 125 mm).
#
# All positions are diagonal midpoints between 4 adjacent key centres, giving
# ~13.5 mm clearance from every switch body edge.
# Right Pico footprint in plate coords: X ≈ 6–57 mm, Y ≈ 52–73 mm —
# no pillar falls inside that rectangle.
rcase = add_pillars(rcase, [
  [2.0*U  + _rox, 5.0*U + _rox],   # mid R0–R1, left    (between F8–F9 / 8–9 cols)
  [5.0*U  + _rox, 5.0*U + _rox],   # mid R0–R1, centre  (between F11–F12 / 0–= cols)
  [8.5*U  + _rox, 3.0*U + _rox],   # mid R2–R3, nav area (right of \, left of nav col)
  [6.0*U  + _rox, 1.0*U + _rox],   # mid R4–R5, centre  (between RCtrl & Left-arrow cols)
])

# ════════════════════════════════════════════════════════════
# PRINTABLE PARTS (4 separate pieces)
# ════════════════════════════════════════════════════════════

# Left case: flat-bottom shell (glue on custom tenting feet as preferred)
left_case_part  = lcase
# Left plate: flat 3 mm plate with switch cutouts + M2 via holes
left_plate_part = lplate

# Right case: flat-bottom shell
right_case_part  = rcase
# Right plate: flat 3 mm plate with switch cutouts + M2 via holes
right_plate_part = rplate

# ── Individual exports ────────────────────────────────────────
left_case_part.export("left_case.step")
left_plate_part.export("left_plate.step")
right_case_part.export("right_case.step")
right_plate_part.export("right_plate.step")

# ── Assembled preview: plates seated in cases, both halves side by side ──
# Plate bottom sits on screw post tops at Z = WT + (CH - PT - 0.5).
plate_z = WT + CH - PT - 0.5
left_assembled  = lcase.fuse(lplate.translate(WT, WT, plate_z))
right_assembled = rcase.fuse(rplate.translate(WT, WT, plate_z))
scene = left_assembled.fuse(right_assembled.translate(lpw + 2.0*WT + 20.0, 0, 0))
preview scene
