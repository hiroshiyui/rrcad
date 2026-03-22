# Outer cylinder — radius 30 mm, height 80 mm
bucket = cylinder(30, 80)

# Pocket the top face 70 mm deep, leaving 5 mm walls and a 10 mm base
bucket = bucket.pocket(:top, depth: 70) { circle(25) }

bucket.export("doc/bucket.stl")
