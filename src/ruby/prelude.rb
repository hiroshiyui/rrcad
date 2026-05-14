# rrcad DSL prelude — loaded automatically into every interpreter session.
#
# This file is embedded in the binary (include_str!) and evaluated during
# MrubyVm::new().  Users never need to write `require` or `require_relative`.
#
# After the prelude runs, MrubyVm::new() calls rrcad_register_shape_class()
# which registers native implementations for Shape and all DSL methods.
# Native methods shadow the Ruby stubs below.

# ---------------------------------------------------------------------------
# Units — all model length values are millimetres, and all angular APIs use
# degrees.  These helpers are plain numeric conversions so they work anywhere a
# number is accepted: primitives, transforms, params, sketches, and patterns.
# ---------------------------------------------------------------------------
class Numeric
  def mm
    self
  end

  alias millimeter mm
  alias millimeters mm

  def cm
    self * 10.0
  end

  alias centimeter cm
  alias centimeters cm

  def m
    self * 1000.0
  end

  alias meter m
  alias meters m

  def inch
    self * 25.4
  end

  alias inches inch

  def deg
    self
  end

  alias degree deg
  alias degrees deg

  def rad
    self * 180.0 / Math::PI
  end

  alias radian rad
  alias radians rad
end

# ---------------------------------------------------------------------------
# Constraint sketching — Phase 10 MVP
# ---------------------------------------------------------------------------
class SketchPoint
  attr_accessor :x, :y

  def initialize(x = nil, y = nil)
    @x = x
    @y = y
  end

  def resolved?
    !@x.nil? && !@y.nil?
  end

  def to_a
    raise RuntimeError, "sketch point is under-constrained" unless resolved?
    [@x, @y]
  end
end

class SketchBuilder
  def initialize
    @points = []
    @lines = []
    @constraints = []
    @named = {}
    @profile = nil
  end

  def point(x_or_name = nil, y = nil, maybe_y = nil)
    if x_or_name.is_a?(Symbol) || x_or_name.is_a?(String)
      p = SketchPoint.new(y, maybe_y)
      @named[x_or_name.to_s] = p
    else
      p = SketchPoint.new(x_or_name, y)
    end
    @points << p
    p
  end

  def construction_point(name, x = nil, y = nil)
    point(name, x, y)
  end

  def ref(name)
    point = @named[name.to_s]
    raise KeyError, "unknown sketch reference: #{name}" if point.nil?
    point
  end

  def [](name)
    ref(name)
  end

  def midpoint(name_or_a, a_or_b, maybe_b = nil)
    if name_or_a.is_a?(Symbol) || name_or_a.is_a?(String)
      name = name_or_a
      a = a_or_b
      b = maybe_b
      p = point(name, nil, nil)
    else
      a = name_or_a
      b = a_or_b
      p = point(nil, nil)
    end

    require_points!(a, b, "midpoint")
    @constraints << [:midpoint, p, a, b]
    p
  end

  def circle_at(center, radius)
    require_point!(center, "circle_at")
    require_positive_number!(radius, "circle_at radius")
    @profile = [:circle_at, center, radius]
    nil
  end

  def arc_at(center, radius, start_deg, end_deg)
    require_point!(center, "arc_at")
    require_positive_number!(radius, "arc_at radius")
    @profile = [:arc_at, center, radius, start_deg, end_deg]
    nil
  end

  def slot_between(a, b, radius)
    require_points!(a, b, "slot_between")
    require_positive_number!(radius, "slot_between radius")
    @profile = [:slot_between, a, b, radius]
    nil
  end

  def line(a, b)
    unless a.is_a?(SketchPoint) && b.is_a?(SketchPoint)
      raise TypeError, "line endpoints must be sketch points"
    end
    @lines << [a, b]
    [a, b]
  end

  def construction_line(a, b)
    require_points!(a, b, "construction_line")
    [a, b]
  end

  # polar_point([:name,] center, radius, angle_deg) — register a construction
  # point at polar coordinates around +center+. Once +center+ resolves, the
  # new point's coordinates are computed as
  #   x = center.x + radius * cos(θ)
  #   y = center.y + radius * sin(θ)
  # +radius+ must be positive and +angle_deg+ is measured CCW from +X.
  # Useful for bolt circles, fan patterns, and other polar layouts.
  def polar_point(name_or_center, center_or_radius = nil, radius_or_angle = nil, angle_deg = nil)
    if name_or_center.is_a?(Symbol) || name_or_center.is_a?(String)
      name = name_or_center
      center = center_or_radius
      radius = radius_or_angle
      angle = angle_deg
      p = point(name, nil, nil)
    else
      center = name_or_center
      radius = center_or_radius
      angle = radius_or_angle
      p = point(nil, nil)
    end

    require_point!(center, "polar_point")
    require_positive_number!(radius, "polar_point radius")
    unless angle.is_a?(Numeric)
      raise ArgumentError, "polar_point angle must be a number"
    end

    @constraints << [:polar_point, p, center, radius, angle]
    p
  end

  def rectangle(origin, width, height)
    require_point!(origin, "rectangle")
    require_positive_number!(width, "rectangle width")
    require_positive_number!(height, "rectangle height")

    right = point(nil, nil)
    top_right = point(nil, nil)
    top_left = point(nil, nil)

    horizontal origin, right
    vertical right, top_right
    horizontal top_right, top_left
    vertical top_left, origin
    dimension origin, right, width
    dimension right, top_right, height

    line origin, right
    line right, top_right
    line top_right, top_left
    line top_left, origin

    [origin, right, top_right, top_left]
  end

  def centered_rectangle(center, width, height)
    require_point!(center, "centered_rectangle")
    require_positive_number!(width, "centered_rectangle width")
    require_positive_number!(height, "centered_rectangle height")

    bottom_left = point(nil, nil)
    bottom_right = point(nil, nil)
    top_right = point(nil, nil)
    top_left = point(nil, nil)

    horizontal bottom_left, bottom_right
    horizontal top_right, top_left
    vertical bottom_left, top_left
    vertical bottom_right, top_right
    @constraints << [:centered_dimension, center, bottom_left, bottom_right, :x, width]
    @constraints << [:centered_dimension, center, bottom_left, top_left, :y, height]

    line bottom_left, bottom_right
    line bottom_right, top_right
    line top_right, top_left
    line top_left, bottom_left

    [bottom_left, bottom_right, top_right, top_left]
  end

  def fixed(point, x = point.x, y = point.y)
    require_point!(point, "fixed")
    @constraints << [:fixed, point, x, y]
    point
  end

  def horizontal(a, b)
    require_points!(a, b, "horizontal")
    @constraints << [:horizontal, a, b]
    [a, b]
  end

  def vertical(a, b)
    require_points!(a, b, "vertical")
    @constraints << [:vertical, a, b]
    [a, b]
  end

  def coincident(a, b)
    require_points!(a, b, "coincident")
    @constraints << [:coincident, a, b]
    [a, b]
  end

  def dimension(a, b, length)
    require_points!(a, b, "dimension")
    require_positive_number!(length, "dimension length")
    @constraints << [:dimension, a, b, length]
    [a, b]
  end

  def equal_length(a, b, c, d)
    require_points!(a, b, "equal_length")
    require_points!(c, d, "equal_length")
    @constraints << [:equal_length, a, b, c, d]
    [a, b, c, d]
  end

  def parallel(a, b, c, d)
    require_points!(a, b, "parallel")
    require_points!(c, d, "parallel")
    @constraints << [:parallel, a, b, c, d]
    [a, b, c, d]
  end

  def perpendicular(a, b, c, d)
    require_points!(a, b, "perpendicular")
    require_points!(c, d, "perpendicular")
    @constraints << [:perpendicular, a, b, c, d]
    [a, b, c, d]
  end

  def symmetric(a, b, center)
    require_points!(a, b, "symmetric")
    require_point!(center, "symmetric")
    @constraints << [:symmetric, a, b, center]
    [a, b, center]
  end

  def mirror_x(source, target, axis_y = 0.0)
    require_points!(source, target, "mirror_x")
    @constraints << [:mirror_x, source, target, axis_y]
    [source, target]
  end

  def mirror_y(source, target, axis_x = 0.0)
    require_points!(source, target, "mirror_y")
    @constraints << [:mirror_y, source, target, axis_x]
    [source, target]
  end

  # tangent(a, b, center, radius, side: nil)
  #
  # Constrain that the line segment a→b is tangent to the circle of given
  # center and radius. The constraint propagates and/or verifies depending on
  # what is already known:
  #
  #   • Horizontal line + circle center known/unknown along Y: with `side:`
  #     of `:above` (line above center) or `:below`, the unknown Y coordinate
  #     is solved to satisfy |line.y − center.y| == radius.
  #   • Vertical line + circle center known/unknown along X: with `side:`
  #     of `:left` or `:right`, the unknown X coordinate is solved.
  #   • Fully resolved line and center: verifies |distance(center, line)| ==
  #     radius and raises on conflict (no propagation needed).
  #
  # When the line is at a non-axis-aligned angle the constraint acts as a
  # verifier only; users should supply a separate angle constraint first.
  def tangent(a, b, center, radius, side: nil)
    require_points!(a, b, "tangent")
    require_point!(center, "tangent")
    require_positive_number!(radius, "tangent radius")
    unless [nil, :above, :below, :left, :right].include?(side)
      raise ArgumentError, "tangent: side: must be :above, :below, :left, :right, or nil"
    end
    @constraints << [:tangent, a, b, center, radius, side]
    [a, b, center]
  end

  def to_profile
    if @profile
      solve_constraints
      return profile_shape
    end

    raise RuntimeError, "sketch requires at least 3 line segments" if @lines.length < 3
    solve_constraints

    pts = []
    @lines.each_with_index do |(a, b), i|
      unless a.resolved? && b.resolved?
        point = a.resolved? ? b : a
        raise RuntimeError, "sketch is under-constrained: #{point_label(point)} missing #{missing_coords(point)}"
      end

      if i > 0 && !same_point?(@lines[i - 1][1], a)
        raise RuntimeError, "sketch lines must form one closed loop"
      end

      pts << a.to_a
    end

    unless same_point?(@lines[-1][1], @lines[0][0])
      raise RuntimeError, "sketch lines must form one closed loop"
    end

    polygon(pts)
  end

  private

  def profile_shape
    type = @profile[0]
    case type
    when :circle_at
      _type, center, radius = @profile
      unless center.resolved?
        raise RuntimeError, "sketch is under-constrained: #{point_label(center)} missing #{missing_coords(center)}"
      end
      circle(radius).translate(center.x, center.y, 0)
    when :arc_at
      _type, center, radius, start_deg, end_deg = @profile
      unless center.resolved?
        raise RuntimeError, "sketch is under-constrained: #{point_label(center)} missing #{missing_coords(center)}"
      end
      arc(radius, start_deg, end_deg).translate(center.x, center.y, 0)
    when :slot_between
      _type, a, b, radius = @profile
      unless a.resolved? && b.resolved?
        point = a.resolved? ? b : a
        raise RuntimeError, "sketch is under-constrained: #{point_label(point)} missing #{missing_coords(point)}"
      end
      slot_shape(a, b, radius)
    else
      raise RuntimeError, "unknown sketch profile"
    end
  end

  def slot_shape(a, b, radius)
    if (a.y - b.y).abs <= 1.0e-9
      left, right = a.x <= b.x ? [a, b] : [b, a]
      pts = arc_points(right.x, right.y, radius, -90.0, 90.0, 12)
      pts += arc_points(left.x, left.y, radius, 90.0, 270.0, 12)
    elsif (a.x - b.x).abs <= 1.0e-9
      bottom, top = a.y <= b.y ? [a, b] : [b, a]
      pts = arc_points(top.x, top.y, radius, 0.0, 180.0, 12)
      pts += arc_points(bottom.x, bottom.y, radius, 180.0, 360.0, 12)
    else
      raise RuntimeError, "slot_between currently requires horizontal or vertical points"
    end

    polygon(pts)
  end

  def arc_points(cx, cy, radius, start_deg, end_deg, segments)
    pts = []
    i = 0
    while i <= segments
      t = start_deg + (end_deg - start_deg) * i / segments
      rad = t * Math::PI / 180.0
      pts << [cx + radius * Math.cos(rad), cy + radius * Math.sin(rad)]
      i += 1
    end
    pts
  end

  def solve_constraints
    32.times do
      changed = false
      @constraints.each do |constraint|
        changed = apply_constraint(constraint) || changed
      end
      return unless changed
    end

    unresolved = @points.reject(&:resolved?).map do |pt|
      "#{point_label(pt)} (missing #{missing_coords(pt)})"
    end
    suffix = unresolved.empty? ? "" : "; unresolved: #{unresolved.join(', ')}"
    raise RuntimeError, "sketch constraints did not converge#{suffix}"
  end

  def apply_constraint(constraint)
    type = constraint[0]
    case type
    when :fixed
      _type, p, x, y = constraint
      changed = assign_coord(p, :x, x, "fixed") if !x.nil?
      changed = assign_coord(p, :y, y, "fixed") || changed if !y.nil?
      changed
    when :horizontal
      _type, a, b = constraint
      unify_coord(a, b, :y, "horizontal")
    when :vertical
      _type, a, b = constraint
      unify_coord(a, b, :x, "vertical")
    when :coincident
      _type, a, b = constraint
      changed = unify_coord(a, b, :x, "coincident")
      unify_coord(a, b, :y, "coincident") || changed
    when :dimension
      _type, a, b, length = constraint
      apply_dimension(a, b, length)
    when :equal_length
      _type, a, b, c, d = constraint
      apply_equal_length(a, b, c, d)
    when :parallel
      _type, a, b, c, d = constraint
      apply_parallel(a, b, c, d)
    when :perpendicular
      _type, a, b, c, d = constraint
      apply_perpendicular(a, b, c, d)
    when :midpoint
      _type, p, a, b = constraint
      apply_midpoint(p, a, b)
    when :centered_dimension
      _type, center, a, b, attr, length = constraint
      apply_centered_dimension(center, a, b, attr, length)
    when :symmetric
      _type, a, b, center = constraint
      apply_symmetric(a, b, center)
    when :mirror_x
      _type, source, target, axis_y = constraint
      apply_mirror_x(source, target, axis_y)
    when :mirror_y
      _type, source, target, axis_x = constraint
      apply_mirror_y(source, target, axis_x)
    when :tangent
      _type, a, b, center, radius, side = constraint
      apply_tangent(a, b, center, radius, side)
    when :polar_point
      _type, p, center, radius, angle_deg = constraint
      apply_polar_point(p, center, radius, angle_deg)
    else
      false
    end
  end

  def apply_tangent(a, b, center, radius, side)
    orient = axis_orientation(a, b) || infer_axis_from_endpoints(a, b)

    if orient == :horizontal
      apply_tangent_axis(a, b, center, radius, side, :y, :above, :below)
    elsif orient == :vertical
      apply_tangent_axis(a, b, center, radius, side, :x, :right, :left)
    elsif a.resolved? && b.resolved? && center.resolved?
      d = point_line_distance(center, a, b)
      if (d - radius).abs > 1.0e-6
        raise RuntimeError,
              "conflicting tangent constraint: distance from #{point_label(center)} " \
              "to line (#{point_label(a)}, #{point_label(b)}) is " \
              "#{format_num(d)}, expected radius #{format_num(radius)}"
      end
      false
    else
      false
    end
  end

  def apply_polar_point(p, center, radius, angle_deg)
    return false unless center.resolved?

    rad = angle_deg * Math::PI / 180.0
    target_x = center.x + radius * Math.cos(rad)
    target_y = center.y + radius * Math.sin(rad)

    changed = assign_coord(p, :x, target_x, "polar_point")
    assign_coord(p, :y, target_y, "polar_point") || changed
  end

  # Infer line orientation from known endpoint coordinates alone (independent
  # of explicit horizontal/vertical constraints). If both endpoints share a
  # known X but differ in Y, the line must be vertical (and vice versa).
  def infer_axis_from_endpoints(a, b)
    ax = coord_get(a, :x)
    bx = coord_get(b, :x)
    ay = coord_get(a, :y)
    by = coord_get(b, :y)

    if ax && bx && (ax - bx).abs > 1.0e-9
      :horizontal
    elsif ay && by && (ay - by).abs > 1.0e-9
      :vertical
    end
  end

  # Propagate a tangent constraint when the line is axis-aligned.
  # `attr` is the perpendicular axis (:y for a horizontal line, :x for vertical).
  # `pos_side` and `neg_side` map the `side:` keyword onto the +/-radius sign.
  def apply_tangent_axis(a, b, center, radius, side, attr, pos_side, neg_side)
    line_v = coord_get(a, attr) # axis-aligned ⇒ a.attr == b.attr
    cv = coord_get(center, attr)

    if line_v && cv
      if ((line_v - cv).abs - radius).abs > 1.0e-6
        actual = (line_v - cv).abs
        raise RuntimeError,
              "conflicting tangent constraint: line #{attr}=#{format_num(line_v)}, " \
              "#{point_label(center)} #{attr}=#{format_num(cv)}, |Δ|=" \
              "#{format_num(actual)}, expected radius #{format_num(radius)}"
      end
      false
    elsif line_v.nil? && cv
      return false if side.nil?
      delta = (side == pos_side) ? radius : (side == neg_side ? -radius : nil)
      return false if delta.nil?
      changed = assign_coord(a, attr, cv + delta, "tangent")
      assign_coord(b, attr, cv + delta, "tangent") || changed
    elsif line_v && cv.nil?
      return false if side.nil?
      delta = (side == pos_side) ? -radius : (side == neg_side ? radius : nil)
      return false if delta.nil?
      assign_coord(center, attr, line_v + delta, "tangent")
    else
      false
    end
  end

  # 2D distance from point p to the infinite line through a and b.
  def point_line_distance(p, a, b)
    abx = b.x - a.x
    aby = b.y - a.y
    apx = p.x - a.x
    apy = p.y - a.y
    num = (abx * apy - aby * apx).abs
    den = Math.sqrt(abx * abx + aby * aby)
    num / den
  end

  def apply_mirror_x(source, target, axis_y)
    changed = unify_coord(source, target, :x, "mirror_x")
    apply_mirror_axis(source, target, :y, axis_y, "mirror_x") || changed
  end

  def apply_mirror_y(source, target, axis_x)
    changed = unify_coord(source, target, :y, "mirror_y")
    apply_mirror_axis(source, target, :x, axis_x, "mirror_y") || changed
  end

  def apply_mirror_axis(source, target, attr, axis_value, name)
    sv = coord_get(source, attr)
    tv = coord_get(target, attr)

    if sv && tv.nil?
      coord_set(target, attr, 2.0 * axis_value - sv)
      true
    elsif tv && sv.nil?
      coord_set(source, attr, 2.0 * axis_value - tv)
      true
    elsif sv && tv && ((sv + tv) / 2.0 - axis_value).abs > 1.0e-6
      raise RuntimeError, "conflicting #{name} constraint"
    else
      false
    end
  end

  def apply_symmetric(a, b, center)
    changed = apply_symmetric_axis(a, b, center, :x)
    apply_symmetric_axis(a, b, center, :y) || changed
  end

  def apply_symmetric_axis(a, b, center, attr)
    av = coord_get(a, attr)
    bv = coord_get(b, attr)
    cv = coord_get(center, attr)

    if av && bv && cv.nil?
      assign_coord(center, attr, (av + bv) / 2.0, "symmetric")
    elsif av && cv && bv.nil?
      assign_coord(b, attr, 2.0 * cv - av, "symmetric")
    elsif bv && cv && av.nil?
      assign_coord(a, attr, 2.0 * cv - bv, "symmetric")
    elsif av && bv && cv && ((av + bv) / 2.0 - cv).abs > 1.0e-6
      raise RuntimeError, "conflicting symmetric constraint"
    else
      false
    end
  end

  def apply_centered_dimension(center, a, b, attr, length)
    cv = coord_get(center, attr)
    av = coord_get(a, attr)
    bv = coord_get(b, attr)

    if cv && av.nil? && bv.nil?
      changed = assign_coord(a, attr, cv - length / 2.0, "centered_dimension")
      assign_coord(b, attr, cv + length / 2.0, "centered_dimension") || changed
    elsif av && bv && cv.nil?
      assign_coord(center, attr, (av + bv) / 2.0, "centered_dimension")
    elsif cv && av && bv
      expected_a = cv - length / 2.0
      expected_b = cv + length / 2.0
      if (av - expected_a).abs > 1.0e-6 || (bv - expected_b).abs > 1.0e-6
        raise RuntimeError, "conflicting centered_dimension constraint"
      end
      false
    elsif cv && av
      assign_coord(b, attr, cv + length / 2.0, "centered_dimension")
    elsif cv && bv
      assign_coord(a, attr, cv - length / 2.0, "centered_dimension")
    else
      false
    end
  end

  def apply_midpoint(p, a, b)
    changed = apply_midpoint_axis(p, a, b, :x)
    apply_midpoint_axis(p, a, b, :y) || changed
  end

  def apply_midpoint_axis(p, a, b, attr)
    pv = coord_get(p, attr)
    av = coord_get(a, attr)
    bv = coord_get(b, attr)

    if av && bv
      assign_coord(p, attr, (av + bv) / 2.0, "midpoint")
    elsif pv && av
      assign_coord(b, attr, 2.0 * pv - av, "midpoint")
    elsif pv && bv
      assign_coord(a, attr, 2.0 * pv - bv, "midpoint")
    else
      false
    end
  end

  def apply_parallel(a, b, c, d)
    first = axis_orientation(a, b)
    second = axis_orientation(c, d)

    if first && second
      raise RuntimeError, "conflicting parallel constraint" unless first == second
      false
    elsif first == :horizontal
      unify_coord(c, d, :y, "parallel")
    elsif first == :vertical
      unify_coord(c, d, :x, "parallel")
    elsif second == :horizontal
      unify_coord(a, b, :y, "parallel")
    elsif second == :vertical
      unify_coord(a, b, :x, "parallel")
    else
      false
    end
  end

  def apply_perpendicular(a, b, c, d)
    first = axis_orientation(a, b)
    second = axis_orientation(c, d)

    if first && second
      raise RuntimeError, "conflicting perpendicular constraint" if first == second
      false
    elsif first == :horizontal
      unify_coord(c, d, :x, "perpendicular")
    elsif first == :vertical
      unify_coord(c, d, :y, "perpendicular")
    elsif second == :horizontal
      unify_coord(a, b, :x, "perpendicular")
    elsif second == :vertical
      unify_coord(a, b, :y, "perpendicular")
    else
      false
    end
  end

  def axis_orientation(a, b)
    if same_known_coord?(a, b, :y)
      :horizontal
    elsif same_known_coord?(a, b, :x)
      :vertical
    else
      nil
    end
  end

  def apply_dimension(a, b, length)
    if same_known_coord?(a, b, :y)
      constrain_axis_distance(a, b, :x, length, "dimension")
    elsif same_known_coord?(a, b, :x)
      constrain_axis_distance(a, b, :y, length, "dimension")
    elsif a.resolved? && b.resolved?
      actual = distance(a, b)
      if (actual - length).abs > 1.0e-6
        raise RuntimeError,
              "conflicting dimension constraint: #{point_label(a)}→#{point_label(b)} " \
              "length=#{format_num(actual)}, expected #{format_num(length)}"
      end
      false
    else
      false
    end
  end

  def apply_equal_length(a, b, c, d)
    ab = segment_length(a, b)
    cd = segment_length(c, d)

    if ab && cd
      if (ab - cd).abs > 1.0e-6
        raise RuntimeError,
              "conflicting equal_length constraint: #{point_label(a)}→#{point_label(b)} " \
              "length=#{format_num(ab)}, #{point_label(c)}→#{point_label(d)} " \
              "length=#{format_num(cd)}"
      end
      false
    elsif ab
      apply_dimension(c, d, ab)
    elsif cd
      apply_dimension(a, b, cd)
    else
      false
    end
  end

  def same_known_coord?(a, b, attr)
    av = coord_get(a, attr)
    bv = coord_get(b, attr)
    !av.nil? && !bv.nil? && (av - bv).abs <= 1.0e-9
  end

  def constrain_axis_distance(a, b, attr, length, name)
    av = coord_get(a, attr)
    bv = coord_get(b, attr)

    if av.nil? && bv.nil?
      false
    elsif av.nil?
      coord_set(a, attr, bv - length)
      true
    elsif bv.nil?
      coord_set(b, attr, av + length)
      true
    elsif ((bv - av).abs - length).abs > 1.0e-6
      actual = (bv - av).abs
      raise RuntimeError,
            "conflicting #{name} constraint: #{point_label(a)}→#{point_label(b)} " \
            "|Δ#{attr}|=#{format_num(actual)}, expected #{format_num(length)}"
    else
      false
    end
  end

  def segment_length(a, b)
    return nil unless a.resolved? && b.resolved?
    distance(a, b)
  end

  def distance(a, b)
    dx = b.x - a.x
    dy = b.y - a.y
    Math.sqrt(dx * dx + dy * dy)
  end

  def assign_coord(point, attr, value, name)
    current = coord_get(point, attr)
    if current.nil?
      coord_set(point, attr, value)
      true
    elsif (current - value).abs > 1.0e-9
      raise RuntimeError,
            "conflicting #{name} constraint: #{point_label(point)} #{attr}=" \
            "#{format_num(current)}, expected #{format_num(value)}"
    else
      false
    end
  end

  def unify_coord(a, b, attr, name)
    av = coord_get(a, attr)
    bv = coord_get(b, attr)

    if av.nil? && bv.nil?
      false
    elsif av.nil?
      coord_set(a, attr, bv)
      true
    elsif bv.nil?
      coord_set(b, attr, av)
      true
    elsif (av - bv).abs > 1.0e-9
      raise RuntimeError,
            "conflicting #{name} constraint: #{point_label(a)} #{attr}=" \
            "#{format_num(av)}, #{point_label(b)} #{attr}=#{format_num(bv)}"
    else
      false
    end
  end

  def format_num(v)
    return v.to_s unless v.is_a?(Numeric)
    rounded = (v * 1.0e6).round / 1.0e6
    rounded.to_s
  end

  def coord_get(point, attr)
    attr == :x ? point.x : point.y
  end

  def coord_set(point, attr, value)
    if attr == :x
      point.x = value
    else
      point.y = value
    end
  end

  def require_point!(point, name)
    raise TypeError, "#{name} constraint expects sketch points" unless point.is_a?(SketchPoint)
  end

  def require_points!(a, b, name)
    require_point!(a, name)
    require_point!(b, name)
  end

  def require_positive_number!(value, name)
    unless value.is_a?(Numeric) && value > 0
      raise ArgumentError, "#{name} must be > 0"
    end
  end

  def same_point?(a, b)
    a.resolved? && b.resolved? && (a.x - b.x).abs < 1.0e-9 && (a.y - b.y).abs < 1.0e-9
  end

  def point_label(point)
    @named.each do |name, candidate|
      return ":#{name}" if candidate.equal?(point)
    end

    index = @points.index(point)
    index.nil? ? "point" : "point #{index + 1}"
  end

  def missing_coords(point)
    missing = []
    missing << "x" if point.x.nil?
    missing << "y" if point.y.nil?
    missing.join("/")
  end
end

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

  # rotate_about(point, axis_dir, angle_deg) — rotate this shape by angle_deg
  # degrees around an axis through `point` (a 3-element [x, y, z]) pointing in
  # `axis_dir` (also 3-element). Implemented via translate(-p) → rotate →
  # translate(+p) so the pivot point stays fixed.
  def rotate_about(point, axis_dir, angle_deg)
    unless point.is_a?(Array) && point.length == 3 && point.all? { |v| v.is_a?(Numeric) }
      raise ArgumentError, "rotate_about: point must be a 3-element numeric array"
    end
    unless axis_dir.is_a?(Array) && axis_dir.length == 3 && axis_dir.all? { |v| v.is_a?(Numeric) }
      raise ArgumentError, "rotate_about: axis_dir must be a 3-element numeric array"
    end
    unless angle_deg.is_a?(Numeric)
      raise ArgumentError, "rotate_about: angle_deg must be a number"
    end
    px, py, pz = point
    ax, ay, az = axis_dir
    mag = Math.sqrt(ax * ax + ay * ay + az * az)
    raise ArgumentError, "rotate_about: axis_dir must be non-zero" if mag < 1.0e-12

    translate(-px, -py, -pz).rotate(ax, ay, az, angle_deg).translate(px, py, pz)
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

  # --- Core Part Design — Phase 8 Tier 1 -----------------------------------

  # Extrude the sketch returned by the block onto +face_sel+ and fuse with self.
  # +face_sel+ may be a Symbol (:top, :bottom, ...) or a Shape (from .faces).
  # +height:+ controls the extrusion distance.
  #
  #   body.pad(:top, height: 5) { rect(4, 4) }
  #
  # Overridden by the native implementation after the prelude runs.
  def pad(_face_sel, height: 1.0, &_block)
    raise NotImplementedError, "Shape#pad is not yet implemented (Phase 8 Tier 1)"
  end

  # Cut a pocket into +self+ using the sketch returned by the block.
  # +face_sel+ selects the face to start from; the sketch is extruded
  # along -normal by +depth:.
  #
  #   body.pocket(:top, depth: 3) { circle(2) }
  #
  # Overridden by the native implementation after the prelude runs.
  def pocket(_face_sel, depth: 1.0, &_block)
    raise NotImplementedError, "Shape#pocket is not yet implemented (Phase 8 Tier 1)"
  end

  # --- Inspection & clearance — Phase 8 Tier 3 -----------------------------------

  # Return the minimum distance between +self+ and +other+ (Float, ≥ 0).
  # Returns 0.0 when the shapes overlap or touch.
  # Uses BRepExtrema_DistShapeShape.
  def distance_to(_other)
    raise NotImplementedError, "Shape#distance_to is not yet implemented (Phase 8 Tier 3)"
  end

  # Return the inertia tensor about the centre of mass as a Hash:
  #   {ixx:, iyy:, izz:, ixy:, ixz:, iyz:}  (world frame, units = mass × length²).
  # Uses BRepGProp::VolumeProperties → GProp_GProps::MatrixOfInertia.
  def inertia
    raise NotImplementedError, "Shape#inertia is not yet implemented (Phase 8 Tier 3)"
  end

  # Return the minimum wall thickness of a solid or shell (Float).
  # Offsets the outer shell inward and measures the gap with BRepExtrema_DistShapeShape.
  def min_thickness
    raise NotImplementedError, "Shape#min_thickness is not yet implemented (Phase 8 Tier 3)"
  end

  # Return the outward unit normal of a planar face as [nx, ny, nz].
  # The shape must be a Face; sampled at the middle of the face's parameter
  # space. Flipped when the face's orientation is REVERSED so the vector
  # points out of the parent solid. Raises if the shape isn't a face or the
  # normal is undefined.
  def normal
    raise NotImplementedError, "Shape#normal is not yet implemented (Phase 10)"
  end

  # Fillet all corner vertices of a 2D Wire or Face profile.
  # Uses BRepFilletAPI_MakeFillet2d; non-corner vertices are silently skipped.
  #
  #   rect(10, 10).fillet_wire(2.0)   # rounded rectangle
  #
  # Overridden by the native implementation after the prelude runs.
  def fillet_wire(_radius)
    raise NotImplementedError, "Shape#fillet_wire is not yet implemented (Phase 8 Tier 1)"
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

  # distance_mate(shape, from:, to:, distance:) — same as mate, but expresses
  # the gap explicitly. `distance:` must be positive (a gap, not contact);
  # use the plain `mate(... offset: 0)` for flush contact.
  def distance_mate(shape, from:, to:, distance:)
    unless distance.is_a?(Numeric) && distance > 0
      raise ArgumentError, "distance_mate distance must be > 0 (use mate for flush contact)"
    end
    positioned = shape.mate(from, to, distance)
    @shapes << positioned
    positioned
  end

  # axis_align(shape, from: [p1, p2], to: [q1, q2]) — rotate and translate
  # +shape+ so that the source axis (p1 → p2, in the shape's current frame)
  # maps to the target axis (q1 → q2 in world coordinates). p1 becomes
  # coincident with q1, and direction (p2−p1) is rotated to (q2−q1). Useful
  # for concentric / coaxial alignment of cylindrical features when you can
  # name two points on each axis.
  def axis_align(shape, from:, to:)
    p1, p2 = validate_axis_pair!(from, "axis_align from:")
    q1, q2 = validate_axis_pair!(to, "axis_align to:")

    u = vec_normalize(vec_sub(p2, p1), "axis_align from:")
    v = vec_normalize(vec_sub(q2, q1), "axis_align to:")

    # First translate p1 to the origin, then rotate u → v about the origin,
    # then translate the origin to q1.
    intermediate = shape.translate(-p1[0], -p1[1], -p1[2])
    intermediate = apply_axis_rotation(intermediate, u, v)
    positioned = intermediate.translate(q1[0], q1[1], q1[2])
    @shapes << positioned
    positioned
  end

  # angle_mate(shape, from:, to:, angle:, pivot:, axis_dir:, offset: 0)
  # — mate +from+ face flush onto +to+ face (optionally with `offset:` gap),
  # then rotate the placed shape by +angle+ degrees about an axis through
  # +pivot+ in direction +axis_dir+ (both 3-element world-space vectors).
  # Useful for locking the rotational degree of freedom left over after a
  # planar mate.
  def angle_mate(shape, from:, to:, angle:, pivot:, axis_dir:, offset: 0.0)
    unless angle.is_a?(Numeric)
      raise ArgumentError, "angle_mate angle must be a number"
    end
    mated = shape.mate(from, to, offset)
    positioned = mated.rotate_about(pivot, axis_dir, angle)
    @shapes << positioned
    positioned
  end

  def validate_axis_pair!(pair, label)
    unless pair.is_a?(Array) && pair.length == 2
      raise ArgumentError, "#{label} must be a [point_a, point_b] pair"
    end
    pair.each do |pt|
      unless pt.is_a?(Array) && pt.length == 3 && pt.all? { |v| v.is_a?(Numeric) }
        raise ArgumentError, "#{label} entries must be 3-element numeric arrays"
      end
    end
    pair
  end

  def vec_sub(a, b)
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
  end

  def vec_normalize(v, label = "axis")
    mag = Math.sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
    raise ArgumentError, "#{label} axis points must be distinct" if mag < 1.0e-12
    [v[0] / mag, v[1] / mag, v[2] / mag]
  end

  # Apply the rotation that takes unit vector u to unit vector v, pivoting
  # about the origin.  Returns a new shape.
  def apply_axis_rotation(shape, u, v)
    # k = u × v (rotation axis); |k| = sin θ; u·v = cos θ.
    kx = u[1] * v[2] - u[2] * v[1]
    ky = u[2] * v[0] - u[0] * v[2]
    kz = u[0] * v[1] - u[1] * v[0]
    sin_t = Math.sqrt(kx * kx + ky * ky + kz * kz)
    cos_t = u[0] * v[0] + u[1] * v[1] + u[2] * v[2]

    if sin_t < 1.0e-12
      if cos_t > 0.0
        # Already aligned — no rotation needed.
        shape
      else
        # Antiparallel: rotate 180° about any axis perpendicular to u.
        perp = if u[0].abs < 0.9 then [1.0, 0.0, 0.0] else [0.0, 1.0, 0.0] end
        px = u[1] * perp[2] - u[2] * perp[1]
        py = u[2] * perp[0] - u[0] * perp[2]
        pz = u[0] * perp[1] - u[1] * perp[0]
        shape.rotate(px, py, pz, 180.0)
      end
    else
      angle_deg = Math.atan2(sin_t, cos_t) * 180.0 / Math::PI
      shape.rotate(kx / sin_t, ky / sin_t, kz / sin_t, angle_deg)
    end
  end

  private :validate_axis_pair!, :vec_sub, :vec_normalize, :apply_axis_rotation

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

  # datum_plane — Phase 8 Tier 1 reference plane.
  # Constructs a finite planar Face from origin, normal, and X direction.
  #
  #   datum_plane(origin: [0, 0, 5], normal: [0, 0, 1], x_dir: [1, 0, 0])
  #
  # Overridden by the native implementation after the prelude runs.
  def datum_plane(origin:, normal:, x_dir:)
    raise NotImplementedError, "datum_plane() is not yet implemented (Phase 8 Tier 1)"
  end

  # helix(radius:, pitch:, height:) — Phase 8 Tier 2 helical Wire path.
  # Returns a Wire approximated by 32 samples per turn via GeomAPI_Interpolate.
  #   path = helix(radius: 5, pitch: 1.5, height: 12)
  # Overridden by the native implementation after the prelude runs.
  def helix(radius:, pitch:, height:)
    raise NotImplementedError, "helix() is not yet implemented (Phase 8 Tier 2)"
  end

  # thread(solid, face_sym, pitch:, depth:) — Phase 8 Tier 2 compound feature.
  # Cuts a helical thread groove into +solid+ by sweeping a triangular profile
  # along a helix derived from the solid's bounding box and removing the result.
  #
  # Conventions:
  #   face_sym  — ignored for geometry (reserved for future face-local thread);
  #               pass :side for ISO-style external threads.
  #   pitch:    — thread pitch in mm (distance between crests).
  #   depth:    — radial groove depth in mm (how far the triangle cuts in).
  #
  #   bolt = cylinder(5, 20)
  #   bolt = thread(bolt, :side, pitch: 1.0, depth: 0.6)
  def thread(solid, _face_sym = :side, pitch:, depth:)
    bb = solid.bounding_box
    height  = bb[:dz]
    # Infer radius from bounding box (assumes shape roughly centred on Z axis).
    radius  = [bb[:dx], bb[:dy]].min / 2.0
    n_turns = (height / pitch).to_i
    return solid if n_turns < 1

    actual_h = n_turns * pitch

    # Helical path starting at the surface of the cylinder.
    path = helix(radius: radius, pitch: pitch, height: actual_h)

    # Isosceles triangle profile: base width = pitch, height = depth.
    # Positioned at world origin; sweep will carry it along the helix.
    hp = pitch / 2.0
    profile = polygon([[0.0, 0.0], [-depth, hp], [0.0, pitch]])

    thread_tool = profile.sweep(path)
    solid.cut(thread_tool)
  end

  # clearance_hole(size, depth:) — standard ISO close-clearance hole tool.
  # +size+ may be a Symbol/String (`:m2`, `:m2_5`, `:m3`, `:m4`, `:m5`) or a
  # numeric diameter in millimetres. Returns a cylindrical solid suitable for
  # subtracting with `.cut`.
  def clearance_hole(size, depth:)
    d = hardware_diameter(size, {
      "m2" => 2.4,
      "m2_5" => 2.9,
      "m25" => 2.9,
      "m3" => 3.4,
      "m4" => 4.5,
      "m5" => 5.5,
    }, "clearance_hole")
    validate_positive_dimension(depth, "clearance_hole depth")
    cylinder(d / 2.0, depth)
  end

  # tap_drill(size, depth:) — standard metric coarse tap-drill hole tool.
  # +size+ may be a Symbol/String (`:m2`, `:m2_5`, `:m3`, `:m4`, `:m5`) or a
  # numeric drill diameter in millimetres.
  def tap_drill(size, depth:)
    d = hardware_diameter(size, {
      "m2" => 1.6,
      "m2_5" => 2.05,
      "m25" => 2.05,
      "m3" => 2.5,
      "m4" => 3.3,
      "m5" => 4.2,
    }, "tap_drill")
    validate_positive_dimension(depth, "tap_drill depth")
    cylinder(d / 2.0, depth)
  end

  # heat_set_insert(size, depth:) — pilot-hole tool for common heat-set inserts.
  # Diameters are conservative starter values and can be overridden by passing
  # a numeric diameter in millimetres.
  def heat_set_insert(size, depth:)
    d = hardware_diameter(size, {
      "m2" => 3.2,
      "m2_5" => 3.8,
      "m25" => 3.8,
      "m3" => 4.6,
    }, "heat_set_insert")
    validate_positive_dimension(depth, "heat_set_insert depth")
    cylinder(d / 2.0, depth)
  end

  # socket_head_cbore(size, depth:, head_depth:) — counterbore tool sized for
  # common metric socket-head screws. Use `.cut` after positioning the tool.
  def socket_head_cbore(size, depth:, head_depth:)
    spec = hardware_spec(size, {
      "m2" => [2.4, 4.0],
      "m2_5" => [2.9, 5.0],
      "m25" => [2.9, 5.0],
      "m3" => [3.4, 6.0],
      "m4" => [4.5, 8.0],
      "m5" => [5.5, 10.0],
    }, "socket_head_cbore")
    validate_positive_dimension(depth, "socket_head_cbore depth")
    validate_positive_dimension(head_depth, "socket_head_cbore head_depth")
    cbore(d: spec[0], cbore_d: spec[1], cbore_h: head_depth, depth: depth)
  end

  # flat_head_csink(size, depth:, angle: 45.0) — countersink tool sized for
  # common metric flat-head screws. +angle+ is the cone half-angle in degrees.
  def flat_head_csink(size, depth:, angle: 45.0)
    spec = hardware_spec(size, {
      "m2" => [2.4, 4.4],
      "m2_5" => [2.9, 5.5],
      "m25" => [2.9, 5.5],
      "m3" => [3.4, 6.3],
      "m4" => [4.5, 9.4],
      "m5" => [5.5, 10.4],
    }, "flat_head_csink")
    validate_positive_dimension(depth, "flat_head_csink depth")
    validate_positive_dimension(angle, "flat_head_csink angle")
    csink(d: spec[0], csink_d: spec[1], csink_angle: angle, depth: depth)
  end

  # bearing_bore(size, depth:, fit: :press) — bore tool sized for the outer
  # diameter of common deep-groove ball bearings. +size+ may be a Symbol/String
  # naming a bearing (`:b608`, `:b623`, `:b624`, `:b625`, `:b626`, `:b688`,
  # `:b695`, `:b6000`, `:b6001`) or a numeric outer diameter in millimetres.
  # +fit+ is `:press` (slight interference, default) or `:slip` (light
  # clearance). Returns a cylindrical solid suitable for `.cut`.
  def bearing_bore(size, depth:, fit: :press)
    d = hardware_diameter(size, {
      "b608" => 22.0,
      "b623" => 10.0,
      "b624" => 13.0,
      "b625" => 16.0,
      "b626" => 19.0,
      "b688" => 16.0,
      "b695" => 13.0,
      "b6000" => 26.0,
      "b6001" => 28.0,
    }, "bearing_bore")
    validate_positive_dimension(depth, "bearing_bore depth")
    adjust = case fit
             when :press then -0.01
             when :slip  then  0.05
             else
               raise ArgumentError, "bearing_bore: unsupported fit #{fit.inspect}"
             end
    cylinder((d + adjust) / 2.0, depth)
  end

  # shaft(diameter, length:, fit: :nominal) — solid cylinder shaft sized for a
  # nominal hole of +diameter+ millimetres, with the diameter adjusted to match
  # a standard fit class. +fit+ may be `:nominal`, `:press` (interference),
  # `:slip` (light clearance), or `:running` (running clearance). Returns a
  # solid cylinder oriented along +Z, suitable for `.fuse` or assembly.
  def shaft(diameter, length:, fit: :nominal)
    validate_positive_dimension(diameter, "shaft diameter")
    validate_positive_dimension(length, "shaft length")
    adjust = case fit
             when :nominal then  0.0
             when :press   then  0.02
             when :slip    then -0.02
             when :running then -0.05
             else
               raise ArgumentError, "shaft: unsupported fit #{fit.inspect}"
             end
    cylinder((diameter + adjust) / 2.0, length)
  end

  # screw(size, length:, style: :socket) — solid fastener body for assemblies.
  # +size+ may be `:m2`, `:m2_5`, `:m3`, `:m4`, or `:m5`. +length+ is the shank
  # length below the head, in millimetres. +style+ may be `:socket` (ISO 4762
  # cylindrical socket-head cap screw), `:button` (ISO 7380 low dome head), or
  # `:flat` (ISO 10642 90° countersunk flat head).
  #
  # Geometry: shank along +Z from z=0 to z=length; head sits above z=length.
  # For `:flat` the head is a conical frustum widening from shank_d at z=length
  # to head_d at z=length+head_h, suitable for sitting flush in a 90°
  # countersink.
  def screw(size, length:, style: :socket)
    spec = hardware_spec(size, {
      # [shaft_d, shcs_head_d, shcs_head_h, bhcs_head_d, bhcs_head_h, fhcs_head_d]
      "m2"   => [2.0, 3.8, 2.0, 3.5, 1.3, 3.8],
      "m2_5" => [2.5, 4.5, 2.5, 4.7, 1.5, 4.7],
      "m25"  => [2.5, 4.5, 2.5, 4.7, 1.5, 4.7],
      "m3"   => [3.0, 5.5, 3.0, 5.7, 1.65, 6.0],
      "m4"   => [4.0, 7.0, 4.0, 7.6, 2.2, 8.0],
      "m5"   => [5.0, 8.5, 5.0, 9.5, 2.75, 10.0],
    }, "screw")
    shaft_d, shcs_d, shcs_h, bhcs_d, bhcs_h, fhcs_d = spec
    validate_positive_dimension(length, "screw length")

    shank = cylinder(shaft_d / 2.0, length)
    head = case style
           when :socket
             cylinder(shcs_d / 2.0, shcs_h).translate(0, 0, length)
           when :button
             cylinder(bhcs_d / 2.0, bhcs_h).translate(0, 0, length)
           when :flat
             # 90° included angle ⇒ head_h = (head_d − shaft_d) / 2.
             head_h = (fhcs_d - shaft_d) / 2.0
             cone(shaft_d / 2.0, fhcs_d / 2.0, head_h).translate(0, 0, length)
           else
             raise ArgumentError, "screw: unsupported style #{style.inspect}"
           end
    shank.fuse(head)
  end

  # mass_estimate(part, density: 1.24) — rough mass estimate in grams.
  # Volume is the OCCT solid volume in mm³; density is in g/cm³ (1 cm³ =
  # 1000 mm³). Typical 3-D printing material densities: PLA 1.24, ABS 1.04,
  # PETG 1.27, standard resin 1.10. Useful for material/cost estimates;
  # ignores infill and walls (treats the part as a solid block).
  def mass_estimate(part, density: 1.24)
    unless density.is_a?(Numeric) && density > 0
      raise ArgumentError, "mass_estimate density must be > 0 (g/cm³)"
    end
    part.volume * density / 1000.0
  end

  # overhang_faces(part, max_angle_deg: 45.0) — list faces of +part+ whose
  # outward normal tips downward more than +max_angle_deg+ from horizontal.
  # Assumes the part is oriented +Z-up (build direction).  A horizontal
  # downward-facing face (normal ≈ −Z) yields an angle of 90°.  Returns an
  # Array of Face shapes; an empty array means no critical overhangs at the
  # given threshold.
  #
  # The angle is measured between the build plane (XY) and the line from
  # the face up to its support — i.e. asin(−normal.z) for normal.z ≤ 0.
  # Vertical walls (normal.z = 0) yield 0° (no overhang); a fully
  # downward-facing face yields 90° (worst case).
  def overhang_faces(part, max_angle_deg: 45.0)
    unless max_angle_deg.is_a?(Numeric) && max_angle_deg >= 0 && max_angle_deg <= 90
      raise ArgumentError, "overhang_faces max_angle_deg must be in [0, 90]"
    end
    sin_limit = Math.sin(max_angle_deg * Math::PI / 180.0)
    part.faces("all").select do |face|
      nz = face.normal[2]
      nz < -sin_limit
    end
  end

  # draft_faces(part, axis: [0, 0, 1], min_draft_deg: 1.0) — list faces with
  # insufficient draft for the given pull direction.  A face's draft angle is
  # `asin(|normal · axis|)`: 0° when the face is parallel to the pull axis (a
  # vertical wall when pulling along Z — sticks in the mould), 90° when the
  # face is perpendicular to the axis (top/bottom — releases cleanly).
  # Returns faces whose draft is strictly less than +min_draft_deg+, i.e. the
  # walls that need a taper applied before injection moulding or casting.
  # Top/bottom faces are naturally excluded (their draft is 90°).
  def draft_faces(part, axis: [0, 0, 1], min_draft_deg: 1.0)
    unless axis.is_a?(Array) && axis.length == 3 && axis.all? { |v| v.is_a?(Numeric) }
      raise ArgumentError, "draft_faces axis must be a 3-element numeric array"
    end
    mag = Math.sqrt(axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2])
    raise ArgumentError, "draft_faces axis must be non-zero" if mag < 1.0e-12
    unless min_draft_deg.is_a?(Numeric) && min_draft_deg >= 0 && min_draft_deg <= 90
      raise ArgumentError, "draft_faces min_draft_deg must be in [0, 90]"
    end

    ux = axis[0] / mag
    uy = axis[1] / mag
    uz = axis[2] / mag
    sin_limit = Math.sin(min_draft_deg * Math::PI / 180.0)

    part.faces("all").select do |face|
      n = face.normal
      dot_abs = (n[0] * ux + n[1] * uy + n[2] * uz).abs
      dot_abs < sin_limit
    end
  end

  # print_volume_check(part, x:, y:, z:) — verify that +part+ fits within
  # a rectangular build volume of +x+ × +y+ × +z+ millimetres.
  # Compares bounding-box extents (orientation-insensitive); the part is
  # assumed to be pre-oriented by the caller.
  # Returns a Hash:
  #   fits        → Boolean
  #   dx, dy, dz  → part extents in mm
  #   overflow_x, overflow_y, overflow_z → mm by which the part exceeds the
  #                                        build volume on each axis (0 if it fits)
  def print_volume_check(part, x:, y:, z:)
    [[x, "x"], [y, "y"], [z, "z"]].each do |v, label|
      unless v.is_a?(Numeric) && v > 0
        raise ArgumentError, "print_volume_check #{label} must be > 0"
      end
    end
    bb = part.bounding_box
    dx, dy, dz = bb[:dx], bb[:dy], bb[:dz]
    ox = [dx - x, 0.0].max
    oy = [dy - y, 0.0].max
    oz = [dz - z, 0.0].max
    {
      fits: ox.zero? && oy.zero? && oz.zero?,
      dx: dx, dy: dy, dz: dz,
      overflow_x: ox, overflow_y: oy, overflow_z: oz
    }
  end

  def hardware_diameter(size, table, label)
    if size.is_a?(Numeric)
      validate_positive_dimension(size, "#{label} diameter")
      size
    else
      key = size.to_s.downcase.gsub("-", "_")
      d = table[key]
      raise ArgumentError, "#{label}: unsupported size #{size.inspect}" if d.nil?
      d
    end
  end

  def hardware_spec(size, table, label)
    key = size.to_s.downcase.gsub("-", "_")
    spec = table[key]
    raise ArgumentError, "#{label}: unsupported size #{size.inspect}" if spec.nil?
    spec
  end

  def validate_positive_dimension(value, label)
    unless value.is_a?(Numeric) && value > 0
      raise ArgumentError, "#{label} must be > 0"
    end
  end

  # cbore(d:, cbore_d:, cbore_h:, depth:) — Phase 8 Tier 2 counterbore tool.
  # Returns a 3-D solid hole tool.  Subtract it from a plate with `.cut` to
  # leave a stepped counterbore hole: a large-diameter shallow bore over a
  # narrower through-hole.  Position the tool before cutting.
  #
  # All dimensions are diameters (not radii).
  #   d:       — clearance hole diameter (the narrow through-hole).
  #   cbore_d: — counterbore diameter (must be > d).
  #   cbore_h: — counterbore depth (shallower than depth).
  #   depth:   — total depth of the hole.
  #
  # Example — centred counterbore on a 50×50×20 plate:
  #   plate = box(50, 50, 20)
  #   hole  = cbore(d: 5, cbore_d: 9, cbore_h: 4, depth: 20)
  #   result = plate.cut(hole)
  def cbore(d:, cbore_d:, cbore_h:, depth:)
    clearance   = circle(d / 2.0).extrude(depth)
    counterbore = circle(cbore_d / 2.0).extrude(cbore_h)
    counterbore.fuse(clearance)
  end

  # csink(d:, csink_d:, csink_angle:, depth:) — Phase 8 Tier 2 countersink tool.
  # Returns a 3-D solid hole tool.  Subtract it from a plate with `.cut` to
  # leave a conical countersink over a clearance hole.  Position the tool before cutting.
  #
  # All diameters are in mm; csink_angle is the cone half-angle in degrees
  # (45° gives a 90° included angle — standard for flat-head screws).
  #   d:           — clearance hole diameter.
  #   csink_d:     — countersink opening diameter at the surface (must be > d).
  #   csink_angle: — half-angle of the cone in degrees (e.g. 45 for 90° included).
  #   depth:       — total depth of the clearance hole below the countersink.
  #
  # Example:
  #   plate = box(50, 50, 20)
  #   hole  = csink(d: 4, csink_d: 8, csink_angle: 45, depth: 20)
  #   result = plate.cut(hole)
  def csink(d:, csink_d:, csink_angle:, depth:)
    clearance = circle(d / 2.0).extrude(depth)
    # Cone height from the difference in radii and the half-angle.
    csink_h = (csink_d - d) / 2.0 / Math.tan(csink_angle * Math::PI / 180.0)
    # cone(r_base, r_top, h): wide end at Z=0, narrows upward.
    conical = cone(csink_d / 2.0, d / 2.0, csink_h)
    conical.fuse(clearance)
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

  # `sketch do ... end` — builds a constrained 2-D profile and returns a Shape.
  # The Phase 10 MVP supports closed polygon loops. Constraint methods are
  # added incrementally on SketchBuilder.
  def sketch(&block)
    builder = SketchBuilder.new
    result = if block.arity == 1
      block.call(builder)
    else
      builder.instance_eval(&block)
    end
    return result if result.is_a?(Shape)
    builder.to_profile
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
