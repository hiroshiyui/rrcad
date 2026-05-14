# 10_sketch_slot.rb — diagonal slot_between sketch demo
#
# Shows the generalized rounded-slot helper between two non-axis-aligned
# sketch points, then extrudes the profile into a simple solid.
#
# Usage:
#   cargo run -- samples/10_sketch_slot.rb
#   cargo run -- --preview samples/10_sketch_slot.rb

profile = sketch do
  a = point(0, 0)
  b = point(20, 5)
  slot_between a, b, 3
end

part = profile.extrude(4)

preview part
part.export("sketch_slot.step")
