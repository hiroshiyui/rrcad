# 01_hello_box.rb — the simplest possible rrcad script.
#
# Creates a 10 × 20 × 30 mm rectangular box and exports it as a STEP file.
#
# Requires: Phase 1

b = box(10, 20, 30)
b.export("hello_box.step")
