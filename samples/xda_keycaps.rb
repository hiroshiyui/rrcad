# ============================================================
# XDA-profile keycaps — print set for the leverless controller
# ============================================================
# 18 identical 1u XDA caps for samples/hitbox_mx_controller.rb: 4 arrows,
# 8 attack keys, 2 thumb keys, 4 function keys. XDA is uniform, so every
# position takes the same cap — including the 10°-tilted arrow cluster.
#
# Exports:
#   xda_keycap.step / .stl      — one cap: print this first to test the
#                                  stem fit on a real switch
#   xda_keycaps_x18.stl         — the full set laid out as one print job
#
# Print upright (as exported), no supports. The cap geometry and the MX
# stem tolerances live in lib/xda_keycap.rb.
# ============================================================
require_relative "lib/xda_keycap"

COUNT    = 18
COLS     = 6
GAP      = 4.0                    # bare bed between neighbouring skirts
PITCH    = XDA_BASE + GAP

cap = xda_keycap

# ── Pre-flight checks on the single cap ─────────────────────
v = cap.validate
raise "keycap: invalid BRep #{v.inspect}" unless v.to_s == "ok"
bb = cap.bounding_box
unless (bb[:dx] - XDA_BASE).abs < 0.01 && (bb[:dz] - XDA_HEIGHT).abs < 0.01
  raise "keycap: unexpected size #{bb.inspect}"
end

# ── Print set: COUNT copies on a grid ───────────────────────
rows  = (COUNT + COLS - 1) / COLS
caps  = (0...COUNT).map do |i|
  cap.translate((i % COLS) * PITCH, (i / COLS) * PITCH, 0)
end
set = assembly("xda_keycaps_x#{COUNT}") { |a| caps.each { |c| a.place c } }.to_shape

fit = print_volume_check(set, x: 220, y: 220, z: 220)
raise "keycap set exceeds the 220 × 220 bed" unless fit[:fits]
puts "keycap: #{XDA_BASE} × #{XDA_BASE} × #{XDA_HEIGHT} mm, " \
     "#{mass_estimate(cap).round(2)} g PLA each"
puts "set:    #{COUNT} caps, #{COLS} × #{rows} grid, " \
     "#{fit[:dx].round(1)} × #{fit[:dy].round(1)} mm on the bed, " \
     "#{(mass_estimate(cap) * COUNT).round(1)} g PLA total"

cap.export("xda_keycap.step")
cap.export("xda_keycap.stl")
set.export("xda_keycaps_x#{COUNT}.stl")

preview set
