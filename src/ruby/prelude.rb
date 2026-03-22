# rrcad DSL prelude — loaded automatically into every interpreter session.
#
# This file is embedded in the binary (include_str!) and evaluated during
# MrubyVm::new().  Users never need to write `require` or `require_relative`.
#
# After the prelude runs, MrubyVm::new() calls rrcad_register_shape_class()
# which registers native implementations for Shape and all DSL methods.
# Native methods shadow the Ruby stubs below.

# ---------------------------------------------------------------------------
# Shape — backing class for all solid geometry objects.
#
# Native instances (created via box/cylinder/sphere/rect/circle) are mRuby
# RData objects holding a raw pointer to a heap-allocated Rust Shape.
#
# Stub instances (created via Shape.new for Phase 3+ stub tests) carry
# @kind / @args instance variables; their inspect uses the prelude definition
# until the native override runs.
# ---------------------------------------------------------------------------
class Shape
  def initialize(kind = nil, *args)
    @kind = kind
    @args = args
  end

  def to_s
    "#<Shape:#{@kind}(#{@args.map(&:inspect).join(', ')})>"
  end

  alias inspect to_s

  # --- Stubs overridden by native after prelude runs -----------------------

  def export(_path)
    raise NotImplementedError, "Shape#export is not yet implemented (Phase 1)"
  end

  def fuse(_other)
    raise NotImplementedError, "Shape#fuse is not yet implemented (Phase 1)"
  end

  def cut(_other)
    raise NotImplementedError, "Shape#cut is not yet implemented (Phase 1)"
  end

  def common(_other)
    raise NotImplementedError, "Shape#common is not yet implemented (Phase 1)"
  end

  def translate(_x, _y, _z)
    raise NotImplementedError, "Shape#translate is not yet implemented (Phase 2)"
  end

  def rotate(_ax, _ay, _az, _angle)
    raise NotImplementedError, "Shape#rotate is not yet implemented (Phase 2)"
  end

  def scale(_factor)
    raise NotImplementedError, "Shape#scale is not yet implemented (Phase 2)"
  end

  def fillet(_radius)
    raise NotImplementedError, "Shape#fillet is not yet implemented (Phase 2)"
  end

  def chamfer(_dist)
    raise NotImplementedError, "Shape#chamfer is not yet implemented (Phase 2)"
  end

  def mirror(_plane)
    raise NotImplementedError, "Shape#mirror is not yet implemented (Phase 2)"
  end

  def extrude(_height, _opts = {})
    raise NotImplementedError, "Shape#extrude is not yet implemented (Phase 2)"
  end

  # --- 3-D operations — Phase 4 -------------------------------------------

  def shell(_thickness)
    raise NotImplementedError, "Shape#shell is not yet implemented (Phase 4)"
  end

  def offset(_distance)
    raise NotImplementedError, "Shape#offset is not yet implemented (Phase 4)"
  end

  # Offset a 2D Wire or Face inward (negative) or outward (positive) in its
  # own plane.  Uses BRepOffsetAPI_MakeOffset.  Phase 7 Tier 1.
  def offset_2d(_distance)
    raise NotImplementedError, "Shape#offset_2d is not yet implemented (Phase 7 Tier 1)"
  end

  # Asymmetric chamfer: d1 and d2 are the two bevel distances on each side
  # of the edge.  An optional selector restricts which edges are chamfered.
  #   part.chamfer_asym(3, 1)           # all edges
  #   part.chamfer_asym(3, 1, :vertical) # only vertical edges
  def chamfer_asym(_d1, _d2, _sel = nil)
    raise NotImplementedError, "Shape#chamfer_asym is not yet implemented (Phase 7 Tier 1)"
  end

  # Remove small holes and fillets for simplified simulation meshes.
  # Faces with surface area smaller than min_feature_size² are treated as
  # belonging to small features and are removed via BRepAlgoAPI_Defeaturing.
  # Returns the shape unchanged if no faces qualify.
  #
  #   part.simplify(1.0)   # remove features smaller than ~1 mm²
  #
  # Overridden by the native implementation after the prelude runs.
  def simplify(_min_feature_size)
    raise NotImplementedError, "Shape#simplify is not yet implemented (Tier 4)"
  end

  def revolve(_angle = 360.0)
    raise NotImplementedError, "Shape#revolve is not yet implemented (Phase 2)"
  end

  # --- Sweep (pipe) — Phase 3 ---------------------------------------------

  def sweep(_path)
    raise NotImplementedError, "Shape#sweep is not yet implemented (Phase 3)"
  end

  # --- Face/edge selectors — Phase 3+ -------------------------------------

  def faces(_selector)
    raise NotImplementedError, "Shape#faces is not yet implemented (Phase 3)"
  end

  def edges(_selector)
    raise NotImplementedError, "Shape#edges is not yet implemented (Phase 3)"
  end

  # --- Color — Phase 5 ------------------------------------------------------

  # Attach an sRGB surface color to this shape.  Returns a new Shape with the
  # same geometry and the color tag stored; the original is unchanged.
  # r, g, b must each be in [0.0, 1.0].  The color is written into the XDE
  # document during GLB / glTF / OBJ export and is visible in the live preview.
  #
  #   body  = box(10, 20, 30).color(0.8, 0.5, 0.2)   # warm orange
  #   knob  = sphere(2).color(0.2, 0.6, 0.9)          # sky blue
  #
  # Overridden by the native implementation after the prelude runs.
  def color(_r, _g, _b)
    raise NotImplementedError, "Shape#color is not yet implemented (Phase 5)"
  end

  # --- Mate — Phase 5 -------------------------------------------------------

  # Return a copy of this shape rigidly repositioned so that +from_face+
  # (a planar face of this shape) lies flush against +to_face+ (a fixed
  # reference face on another shape).
  #
  # The transform aligns the face centroids and makes the outward normals
  # antiparallel (contact orientation, not overlap).
  #
  # +offset+ (default 0.0) shifts the mated shape along to_face's outward
  # normal: positive = gap, negative = interference.
  #
  #   base = box(100, 80, 10)
  #   post = box(20, 20, 50)
  #   post_placed = post.mate(post.faces(:bottom).first,
  #                           base.faces(:top).first)
  #   post_placed = post.mate(post.faces(:bottom).first,
  #                           base.faces(:top).first, 2.0)   # 2 mm gap
  #
  # Overridden by the native implementation after the prelude runs.
  def mate(_from_face, _to_face, _offset = 0.0)
    raise NotImplementedError, "Shape#mate is not yet implemented (Phase 5)"
  end

  # --- Validation & introspection — Phase 7 Tier 2 --------------------------

  # Return a Symbol naming the topological shape type:
  #   :compound, :compsolid, :solid, :shell, :face, :wire, :edge, :vertex
  def shape_type
    raise NotImplementedError, "Shape#shape_type is not yet implemented (Phase 7 Tier 2)"
  end

  # Return the centroid of the shape as [x, y, z].
  # Uses volume properties for solids, surface properties for shells/faces,
  # and linear properties for wires/edges.
  def centroid
    raise NotImplementedError, "Shape#centroid is not yet implemented (Phase 7 Tier 2)"
  end

  # Return true if every edge is shared by at least 2 faces (no open boundary).
  def closed?
    raise NotImplementedError, "Shape#closed? is not yet implemented (Phase 7 Tier 2)"
  end

  # Return true if every edge is shared by exactly 2 faces (manifold mesh).
  def manifold?
    raise NotImplementedError, "Shape#manifold? is not yet implemented (Phase 7 Tier 2)"
  end

  # Run BRepCheck_Analyzer on this shape.
  # Returns :ok if the shape is valid, or an Array of error description strings.
  def validate
    raise NotImplementedError, "Shape#validate is not yet implemented (Phase 7 Tier 2)"
  end

  # --- Surface modeling — Phase 7 Tier 3 -----------------------------------

  # Cross-section of this shape by an axis-aligned plane.
  # Returns a compound of the section edges/wires.
  #
  #   solid.slice(plane: :xy, z: 5.0)   # XY plane at z=5
  #   solid.slice(plane: :xz, y: 2.0)   # XZ plane at y=2
  #   solid.slice(plane: :yz, x: 1.0)   # YZ plane at x=1
  def slice(**_kwargs)
    raise NotImplementedError, "Shape#slice is not yet implemented (Phase 7 Tier 3)"
  end
end

# ---------------------------------------------------------------------------
# Assembly — groups named shapes; supports place; mate is Phase 5.
# ---------------------------------------------------------------------------
class Assembly
  def initialize(name)
    @name = name
    @shapes = []
  end

  def place(shape)
    @shapes << shape
    shape
  end

  # Reposition +shape+ so that +from:+ face aligns with +to:+ face, then add
  # it to the assembly.  Returns the repositioned shape.
  #
  #   assembly("bracket") do |a|
  #     a.place base
  #     a.mate post, from: post.faces(:bottom).first,
  #                  to:   base.faces(:top).first
  #     a.mate post2, from: post2.faces(:bottom).first,
  #                   to:   base.faces(:top).first, offset: 5.0
  #   end
  def mate(shape, from:, to:, offset: 0.0)
    positioned = shape.mate(from, to, offset)
    @shapes << positioned
    positioned
  end

  def to_shape
    raise RuntimeError, "Assembly '#{@name}' contains no shapes" if @shapes.empty?
    @shapes.inject { |acc, s| acc.fuse(s) }
  end

  def export(path)
    to_shape.export(path)
  end

  def inspect
    "#<Assembly:#{@name} (#{@shapes.length} shapes)>"
  end

  alias to_s inspect
end

# ---------------------------------------------------------------------------
# Top-level DSL methods
# ---------------------------------------------------------------------------
module Kernel
  # Primitives — overridden natively after prelude runs.
  def box(_x, _y, _z)
    raise NotImplementedError, "box() is not yet implemented (Phase 1)"
  end

  def cylinder(_r, _h)
    raise NotImplementedError, "cylinder() is not yet implemented (Phase 1)"
  end

  def sphere(_r)
    raise NotImplementedError, "sphere() is not yet implemented (Phase 1)"
  end

  # 2D sketch faces — overridden natively after prelude runs.
  def rect(_w, _h)
    raise NotImplementedError, "rect() is not yet implemented (Phase 2)"
  end

  def circle(_r)
    raise NotImplementedError, "circle() is not yet implemented (Phase 2)"
  end

  # Loft — overridden natively after prelude runs.
  def loft(_profiles, _opts = {})
    raise NotImplementedError, "loft() is not yet implemented (Phase 4)"
  end

  # grid_pattern(shape, nx, ny, dx, dy) — nx × ny copies in a 2-D grid.
  # Copy (i, j) is at (i*dx, j*dy, 0).  Phase 7 Tier 1.
  def grid_pattern(_shape, _nx, _ny, _dx, _dy)
    raise NotImplementedError, "grid_pattern() is not yet implemented (Phase 7 Tier 1)"
  end

  # fuse_all([a, b, c]) — fold-left union of 2+ shapes.  Phase 7 Tier 1.
  def fuse_all(_shapes)
    raise NotImplementedError, "fuse_all() is not yet implemented (Phase 7 Tier 1)"
  end

  # cut_all(base, [t1, t2]) — subtract each tool from base in sequence.  Phase 7 Tier 1.
  def cut_all(_base, _tools)
    raise NotImplementedError, "cut_all() is not yet implemented (Phase 7 Tier 1)"
  end

  # ruled_surface(wire_a, wire_b) — ruled surface between two wires.  Phase 7 Tier 3.
  def ruled_surface(_wire_a, _wire_b)
    raise NotImplementedError, "ruled_surface() is not yet implemented (Phase 7 Tier 3)"
  end

  # fill_surface(boundary_wire) — smooth surface filling a closed wire boundary.  Phase 7 Tier 3.
  def fill_surface(_boundary_wire)
    raise NotImplementedError, "fill_surface() is not yet implemented (Phase 7 Tier 3)"
  end

  # Spline profiles — overridden natively after prelude runs.
  #
  # Optional +tangents:+ keyword suppresses natural-boundary oscillation at
  # the endpoints of short splines.  Pass exactly two tangent vectors:
  #
  #   spline_2d([[0,0],[5,10],[10,5]], tangents: [[1,0],[1,0]])
  #   spline_3d([[0,0,0],[5,5,5],[10,0,0]], tangents: [[1,0,0],[1,0,0]])
  #
  # 2D tangents live in the XZ plane: [x, z].
  # 3D tangents are full vectors: [x, y, z].
  # Vector magnitude is ignored; only direction matters.
  def spline_2d(_points, tangents: nil)
    raise NotImplementedError, "spline_2d() is not yet implemented (Phase 3)"
  end

  def spline_3d(_points, tangents: nil)
    raise NotImplementedError, "spline_3d() is not yet implemented (Phase 3)"
  end

  # `solid do ... end` — evaluates block, returns its result.
  def solid
    yield
  end

  # `assembly "name" do |asm| ... end` — creates an Assembly.
  def assembly(name)
    asm = Assembly.new(name)
    yield asm if block_given?
    asm
  end

  # Tessellate shape and push it to the live browser preview — Phase 3.
  # Overridden natively after prelude runs; no-op when not in --preview mode.
  def preview(_shape)
    raise NotImplementedError, "preview() is not yet implemented (Phase 3)"
  end

  # ---------------------------------------------------------------------------
  # param — Phase 5 parametric DSL
  # ---------------------------------------------------------------------------
  # Declare a named script parameter with a default value and an optional
  # range constraint.  Returns the effective value, giving precedence to any
  # CLI override supplied via --param key=value.
  #
  # CLI values arrive as strings; they are coerced to the same Ruby type as
  # +default+ (Integer, Float, or String).
  #
  #   width  = param :width,  default: 50,  range: 1..200
  #   scale  = param :scale,  default: 1.0, range: 0.1..10.0
  #   label  = param :label,  default: "part"
  #
  # $_rrcad_params is populated by the Rust CLI layer before the user script
  # is evaluated.  Keys are strings.
  $_rrcad_params ||= {}

  def param(name, default:, range: nil)
    key = name.to_s
    raw = $_rrcad_params.key?(key) ? $_rrcad_params[key] : default

    # Coerce CLI string values to the declared default's type.
    val = if raw.is_a?(String)
      case default
      when Integer then raw.to_i
      when Float   then raw.to_f
      when TrueClass, FalseClass then raw == "true"
      else raw
      end
    else
      raw
    end

    if range && !range.include?(val)
      raise ArgumentError,
            "param :#{name} value #{val.inspect} is outside range #{range.inspect}"
    end

    val
  end
end
