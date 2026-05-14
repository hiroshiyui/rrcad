# 09_fastener_stack.rb — washer/nut assembly demo
#
# Shows the new hardware body helpers in a simple exploded assembly:
#   - a plate with a 1/4-20 clearance hole
#   - a washer positioned above the plate
#   - a flange nut positioned below the plate
#
# Usage:
#   cargo run -- samples/09_fastener_stack.rb
#   cargo run -- --preview samples/09_fastener_stack.rb

plate = box(40, 40, 6).cut(
  clearance_hole(:"1/4-20", depth: 8).translate(20, 20, -1)
)

washer_part = washer(:"1/4-20", thickness: 1.6).translate(20, 20, 7.2)
nut_part    = nut(:"1/4-20", thickness: 5.0, style: :flange).translate(20, 20, -5.0)

asm = assembly("fastener_stack") do |a|
  a.place plate
  a.place washer_part
  a.place nut_part
end

preview asm.to_shape
asm.export("fastener_stack.step")
