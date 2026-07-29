# 11_sheet_metal_tray.rb — a folded sheet-metal tray and its flat blank.
#
# A sheet-metal part is one sheet of constant thickness, folded, so the folded
# solid and the flat blank the laser cuts are two views of the same thing. The
# builder records the bends, which is what makes the blank derivable: unfolding
# needs to know where each bend line ran and how tight it is.
#
#   cargo run -- samples/11_sheet_metal_tray.rb
#   cargo run -- --preview samples/11_sheet_metal_tray.rb

THICKNESS = param :thickness, default: 1.5, range: 0.5..4.0
WALL      = param :wall, default: 25.0, range: 5.0..60.0

tray = sheet_metal(thickness: THICKNESS, radius: THICKNESS, k_factor: 0.44) do |s|
  s.base 120, 80

  # Each flange is inset 6 mm from the corners. Two flanges that both ran into
  # a shared corner would meet there at a point with nothing joining them, and
  # the blank would pinch to nothing — so the builder refuses that outright.
  # The inset also gives the automatic bend relief somewhere to sit.
  [:xmin, :xmax].each { |side| s.flange side, length: WALL, from: 6, to: 74 }
  [:ymin, :ymax].each { |side| s.flange side, length: WALL, from: 6, to: 114 }
end

# The blank is longer than the plate by the bend allowance — the arc length of
# the neutral axis — plus the straight leg, on every fold.
w, h = tray.flat_size
puts "blank: #{w.round(1)} x #{h.round(1)} mm"

tray.bends.each do |b|
  puts "  #{b[:side]}  #{b[:angle]} deg  leg #{b[:length]}  " \
       "allowance #{b[:allowance].round(3)}  relief #{b[:relief]}"
end

tray.export("sheet_metal_tray.step")   # folded, for the model
tray.export_flat("sheet_metal_tray.dxf") # blank, 1:1, for the cutter

preview tray.to_shape
