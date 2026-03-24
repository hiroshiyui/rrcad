# Ball Pen Body Parts — Schmidt Refill Compatible
# Four parts laid out on the X-Y plane (Z is the pen axis, upward):
#
#   [BARREL] ── [TIP] ── [FRONT CAP] ── [TAIL CAP]
#
# Refill:  Schmidt P8126, body ø5.8 mm, length ≈ 67 mm
# Joint:   L-shaped tenon & mortise (bayonet / quarter-turn lock)
#            Insert tip, push home, rotate 90° clockwise to lock.
#            Spring-relief slots on each tab give tactile snap on seating.
#
# Usage:
#   cargo run -- samples/pen_schmidt.rb
#   cargo run -- --preview samples/pen_schmidt.rb

# ─── Parameters ───────────────────────────────────────────────────────────────

# Barrel
BARREL_OD  = 11.0   # outer diameter (ergonomic grip)
BARREL_ID  =  6.4   # bore: Schmidt ø5.8 + 0.6 running clearance
BARREL_LEN = 115.0  # body length

# L-joint pin / socket
PIN_OD  = 10.0  # pin outer diameter
PIN_LEN =  8.0  # pin / socket depth

# Tab geometry
TAB_RADIAL = 1.5  # radial height above pin surface
TAB_WIDTH  = 3.0  # circumferential width
TAB_HEIGHT = 2.0  # axial height
TAB_CLR    = 0.2  # sliding clearance (applied to slot dimensions)

# Derived slot dimensions
SLOT_RADIAL = TAB_RADIAL + TAB_CLR   # 1.7 mm — radial depth of entry slot
SLOT_WIDTH  = TAB_WIDTH  + TAB_CLR   # 3.2 mm — circumferential slot width
SLOT_AXIAL  = PIN_LEN - TAB_HEIGHT   # 6.0 mm — axial travel before locking
LOCK_AXIAL  = TAB_HEIGHT + TAB_CLR   # 2.2 mm — locking channel height
SOCKET_R    = PIN_OD / 2.0 + 0.1    # 5.1 mm — socket bore radius (pin + 0.1 clearance)

# 90° locking channel arc (approximated as a tangential box centred at 45°)
R_SC    = SOCKET_R + SLOT_RADIAL / 2.0  # 5.95 mm — radial centre of slot
ARC_LEN = Math::PI / 2.0 * R_SC         # ≈ 9.33 mm

# Tip nose
TIP_TOTAL    = 28.0  # overall tip length
TIP_NOSE_LEN = 20.0  # conical nose length  (TIP_TOTAL − PIN_LEN)
TIP_END_OD   =  4.0  # OD at writing end
TIP_BORE_OD  =  6.0  # refill body bore
TIP_EXIT_OD  =  1.8  # writing-tip exit bore
TIP_EXIT_LEN =  5.0  # length of narrow exit section

# Spring-relief slots (make tabs flexible for tactile snap-fit)
RELIEF_W = 1.0                              # slot width (circumferential)
RELIEF_X = PIN_OD / 2.0 + TAB_RADIAL + 0.5 # 7.0 mm — clears tab tip radially

# Front cap
FC_OD       = 13.0  # outer diameter (barrel OD 11 + 2 × 1 mm wall)
FC_ID       = 11.2  # inner diameter (barrel OD + 0.2 clearance)
FC_LEN      = 47.0  # total length (covers 28 mm tip + 17 mm grip on barrel)
FC_WALL_END =  2.0  # closed-end wall thickness
FC_RIB_ID   = 10.6  # retention rib ID (0.3 mm interference snap)
FC_RIB_LEN  =  1.0  # retention rib axial length

# Tail cap
TC_FLANGE_OD =  13.0  # flange OD (same as front cap for visual symmetry)
TC_FLANGE_H  =   3.0  # flange thickness
TC_PLUG_OD   =   6.2  # plug OD (barrel bore 6.4 − 0.2 → light press fit)
TC_PLUG_H    =  10.0  # plug insertion depth

# Layout
GAP = 20.0  # spacing between parts

# ─── Part 1: Barrel ───────────────────────────────────────────────────────────
# z=0: tip end (socket here)   z=BARREL_LEN: open cap end
#
# Cross-section (at socket end):
#   outer wall OD 11 ──> bore ID 6.4 ──> socket ID 10.2 (first 8 mm)
#
# Two L-mortise slots (at θ=0° and θ=180°):
#   Axial entry leg:  6 mm deep, SLOT_WIDTH wide, SLOT_RADIAL into wall
#   Locking arc leg:  90° sweep at z=6..8, approximated as tangential box at 45°

barrel = solid do; cylinder BARREL_OD / 2, BARREL_LEN; end
           .cut(solid do; cylinder BARREL_ID / 2, BARREL_LEN; end)
           .cut(solid do; cylinder SOCKET_R,       PIN_LEN;    end)

slot_e1 = solid do; box SLOT_RADIAL, SLOT_WIDTH, SLOT_AXIAL; end
            .translate(SOCKET_R, -SLOT_WIDTH / 2, 0)
slot_e2 = solid do; box SLOT_RADIAL, SLOT_WIDTH, SLOT_AXIAL; end
            .translate(-(SOCKET_R + SLOT_RADIAL), -SLOT_WIDTH / 2, 0)

lock_a1 = solid do; box ARC_LEN, SLOT_RADIAL, LOCK_AXIAL; end
            .translate(-ARC_LEN / 2, -SLOT_RADIAL / 2, -LOCK_AXIAL / 2)
            .translate(R_SC, 0, 0)
            .rotate(0, 0, 1, 45)
            .translate(0, 0, SLOT_AXIAL + LOCK_AXIAL / 2)
lock_a2 = solid do; box ARC_LEN, SLOT_RADIAL, LOCK_AXIAL; end
            .translate(-ARC_LEN / 2, -SLOT_RADIAL / 2, -LOCK_AXIAL / 2)
            .translate(R_SC, 0, 0)
            .rotate(0, 0, 1, 225)
            .translate(0, 0, SLOT_AXIAL + LOCK_AXIAL / 2)

barrel = barrel.cut(slot_e1).cut(slot_e2).cut(lock_a1).cut(lock_a2)

# ─── Part 2: Tip ──────────────────────────────────────────────────────────────
# z=0: writing end   z=TIP_NOSE_LEN: shoulder (flush with barrel face)
# z=TIP_TOTAL:       top of pin (seats against barrel)
#
# Profile:  conical nose ø4→ø11 (20 mm) + smooth pin ø10 (8 mm)
# Bore:     ø6 refill body → ø1.8 exit (last 5 mm)
# Tabs:     two radial tenons at z=26..28 (barrel z=6..8 when assembled)
# Springs:  axial relief slots flank each tab — cantilever arms flex on snap-in

z_tab = TIP_NOSE_LEN + SLOT_AXIAL  # 26 — tab base in tip frame

tip_nose = solid do; cone TIP_END_OD / 2, BARREL_OD / 2, TIP_NOSE_LEN; end
tip_pin  = solid do; cylinder PIN_OD / 2, PIN_LEN; end

tab_1 = solid do; box TAB_RADIAL, TAB_WIDTH, TAB_HEIGHT; end
          .translate(PIN_OD / 2, -TAB_WIDTH / 2, z_tab)
tab_2 = solid do; box TAB_RADIAL, TAB_WIDTH, TAB_HEIGHT; end
          .translate(-(PIN_OD / 2 + TAB_RADIAL), -TAB_WIDTH / 2, z_tab)

tip_body = tip_nose
             .fuse(tip_pin.translate(0, 0, TIP_NOSE_LEN))
             .fuse(tab_1)
             .fuse(tab_2)

refill_bore = solid do; cylinder TIP_BORE_OD / 2, TIP_TOTAL - TIP_EXIT_LEN; end
exit_bore   = solid do; cylinder TIP_EXIT_OD / 2, TIP_EXIT_LEN + 0.1;        end
tip = tip_body
        .cut(refill_bore.translate(0, 0, TIP_EXIT_LEN))
        .cut(exit_bore)

# Spring-relief slots: two axial slots per tab isolate each tab as a cantilever.
# Slots run the full pin length from the shoulder outward.
# Tab_1 (+X side): slots cut through +X half of pin wall
spring_1a = solid do; box RELIEF_X, RELIEF_W, PIN_LEN; end
              .translate(0, TAB_WIDTH / 2, TIP_NOSE_LEN)
spring_1b = solid do; box RELIEF_X, RELIEF_W, PIN_LEN; end
              .translate(0, -(TAB_WIDTH / 2 + RELIEF_W), TIP_NOSE_LEN)
# Tab_2 (−X side): slots cut through −X half of pin wall
spring_2a = solid do; box RELIEF_X, RELIEF_W, PIN_LEN; end
              .translate(-RELIEF_X, TAB_WIDTH / 2, TIP_NOSE_LEN)
spring_2b = solid do; box RELIEF_X, RELIEF_W, PIN_LEN; end
              .translate(-RELIEF_X, -(TAB_WIDTH / 2 + RELIEF_W), TIP_NOSE_LEN)

tip = tip.cut(spring_1a).cut(spring_1b).cut(spring_2a).cut(spring_2b)

# ─── Part 3: Front Cap ────────────────────────────────────────────────────────
# Slides over the barrel tip end when the pen is not in use.
# z=0: open end (fits over barrel)   z=FC_LEN: closed end
#
# Retention rib at the open end (ID 10.6 mm, 1 mm band) grips the barrel
# with 0.3 mm interference — friction hold, no tools needed.

fc_outer = solid do; cylinder FC_OD    / 2, FC_LEN;            end
fc_bore  = solid do; cylinder FC_ID    / 2, FC_LEN - FC_WALL_END; end
fc_rib   = solid do; cylinder FC_RIB_ID / 2, FC_RIB_LEN;       end

front_cap = fc_outer
              .cut(fc_bore.translate(0, 0, FC_RIB_LEN))
              .cut(fc_rib)

# ─── Part 4: Tail Cap ─────────────────────────────────────────────────────────
# Seals the back of the barrel and retains the refill.
# Flange (ø13 × 3 mm) sits flush on barrel top; plug (ø6.2 × 10 mm) press-fits
# into the barrel bore.

tc_flange = solid do; cylinder TC_FLANGE_OD / 2, TC_FLANGE_H; end
tc_plug   = solid do; cylinder TC_PLUG_OD   / 2, TC_PLUG_H;   end
tail_cap  = tc_flange.fuse(tc_plug.translate(0, 0, TC_FLANGE_H))

# ─── Layout: side by side on X-Y plane ────────────────────────────────────────
tip_x = BARREL_OD / 2 + GAP + BARREL_OD / 2        # 31
fc_x  = tip_x + BARREL_OD / 2 + GAP + FC_OD / 2    # 63
tc_x  = fc_x  + FC_OD / 2     + GAP + TC_FLANGE_OD / 2  # 96

scene = barrel
          .fuse(tip.translate(tip_x, 0, 0))
          .fuse(front_cap.translate(fc_x, 0, 0))
          .fuse(tail_cap.translate(tc_x, 0, 0))

preview scene
scene.export "pen_schmidt.step"
scene.export "pen_schmidt.stl"
