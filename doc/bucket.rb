# Solid box — 60×60×80 mm
bucket = box(60, 60, 80)

# Pocket the top face 70 mm deep, leaving 5 mm walls and a 10 mm base
bucket = bucket.pocket(:top, depth: 70) { rect(50, 50) }

bucket.export("doc/bucket.stl")
