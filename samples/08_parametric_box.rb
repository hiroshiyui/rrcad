# samples/08_parametric_box.rb — Phase 5 parametric DSL demo
#
# Demonstrates param() with CLI overrides and design-table batch export.
#
# Run once:
#   rrcad samples/08_parametric_box.rb
#   rrcad --param width=80 --param depth=40 samples/08_parametric_box.rb
#
# Batch export with a design table:
#   rrcad --design-table samples/08_box_sizes.csv samples/08_parametric_box.rb

name  = param :name,  default: "box_part"
width = param :width, default: 50, range: 1..500
depth = param :depth, default: 30, range: 1..500
height = param :height, default: 20, range: 1..500
fillet_r = param :fillet_r, default: 2.0, range: 0.0..10.0

part = box(width, depth, height)
part = part.fillet(fillet_r) if fillet_r > 0.0

part.export("#{name}.step")
preview part
