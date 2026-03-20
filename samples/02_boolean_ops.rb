# 02_boolean_ops.rb — boolean union, difference, and intersection.
#
# Demonstrates fuse / cut / common on primitive shapes.
#
# Requires: Phase 1

# Union: merge a box and a sphere that overlap at one corner.
blob = box(20, 20, 20).fuse(sphere(14).translate(20, 20, 20))
blob.export("fuse.step")

# Difference: drill a cylindrical hole through the centre of a box.
drilled = box(40, 40, 20).cut(cylinder(8, 30).translate(20, 20, -5))
drilled.export("drilled.step")

# Intersection: keep only the volume common to two overlapping boxes.
cross = box(60, 10, 10).common(box(10, 60, 10))
cross.export("cross.step")
