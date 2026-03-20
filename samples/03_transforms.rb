# 03_transforms.rb — translate, rotate, and scale.
#
# Requires: Phase 1

base = box(30, 10, 5)

# Translate: move a copy 40 mm along X.
shelf = base.translate(40, 0, 0)

# Rotate: tilt a cylinder 45° around the Z axis.
peg = cylinder(3, 20).rotate(0, 0, 1, 45)

# Scale: shrink a sphere to half size.
small = sphere(10).scale(0.5)

# Combine everything and export.
part = base.fuse(shelf).fuse(peg).fuse(small)
part.export("transforms.step")
