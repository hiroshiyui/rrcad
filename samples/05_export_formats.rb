# 05_export_formats.rb — export the same part to STEP, STL, and glTF.
#
# Requires: Phase 1

part = box(20, 20, 20).cut(sphere(12))

part.export("part.step")   # STEP  — preferred for CAD interchange
part.export("part.stl")    # STL   — for slicers and mesh tools
part.export("part.glb")    # glTF  — for web viewers and game engines
