# 12_propeller.rb — two-blade quadcopter propeller (Phase 12 showcase).
#
# Brings together the Phase 12 additions in one part:
#   airfoil          — NACA 2412 blade sections
#   sweep_sections   — one call per blade, chord shrinking and pitch
#                      unwinding toward the tip via scale: / twist:
#   text             — engraved size marking on the hub
#
# Run:  cargo run -- samples/12_propeller.rb

# --- Blade -------------------------------------------------------------
# The blade grows along Z (sweep_sections places origin-centred profiles on
# the spine), then lies down along +X to meet the hub. Twist is the pitch
# angle at each station: steep at the root, shallow at the tip.
blade_length = 50
section = airfoil(naca: "2412", chord: 12)
spine = spline_3d([[0, 0, 0], [0, 0, blade_length / 2.0], [0, 0, blade_length]])

blade = sweep_sections(spine, [section, section, section],
                       twist: [38, 24, 14],
                       scale: [1.0, 0.8, 0.5])
blade = blade.rotate(0, 1, 0, 90) # lay the blade along +X

# --- Hub ---------------------------------------------------------------
hub = cylinder(8, 8)
hub = hub.cut(cylinder(2.5, 8))                       # 5 mm shaft bore
hub = hub.fillet(1, :horizontal)

# Blades meet the hub at mid-height, one each side.
z_mid = 4
prop = hub.fuse(blade.translate(6, 0, z_mid))
prop = prop.fuse(blade.rotate(0, 0, 1, 180).translate(-6, 0, z_mid))

# --- Marking -----------------------------------------------------------
# Engrave the size on top of the hub: 100 mm diameter, 38° root pitch.
mark = text("100x38", size: 3).extrude(0.6)
prop = prop.cut(mark.translate(-6.5, -1.5, 7.4))

puts "propeller volume: #{prop.volume.round(1)} mm^3"
puts "validate:         #{prop.validate}"

prop.export("propeller.stl")
prop.export("propeller.step")
preview prop
