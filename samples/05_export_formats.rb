# 05_export_formats.rb — export the same part to STEP, 3MF, STL, and glTF.
#
# Requires: Phase 1

part = box(20, 20, 20).cut(sphere(12)).color(0.2, 0.6, 0.9)

part.export("part.step")   # STEP  — preferred for CAD interchange
part.export("part.3mf")    # 3MF   — for slicers: carries mm, colour, and bodies
part.export("part.stl")    # STL   — binary; when the tool only reads STL
                           #         pass ascii: true for the text encoding
part.export("part.glb")    # glTF  — for web viewers and game engines
