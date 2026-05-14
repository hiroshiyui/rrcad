# rrcad DSL prelude — loaded automatically into every interpreter session.
#
# This file is embedded in the binary (include_str!) and evaluated during
# MrubyVm::new().  Users never need to write `require` or `require_relative`.
#
# After the prelude runs, MrubyVm::new() calls rrcad_register_shape_class()
# which registers native implementations for Shape and all DSL methods.
# Native methods shadow the Ruby stubs below.

# ---------------------------------------------------------------------------
# Units — model lengths are mm, angles are degrees.
#
# Numeric helpers now return lightweight typed value objects so CAD scripts can
# keep unit information through arithmetic while still interoping with the
# existing numeric API surface.  Plain numerics are still accepted everywhere
# the DSL already expected them.
# ---------------------------------------------------------------------------
module RRCADUnits
  def self.scalar(value)
    value.respond_to?(:to_f) ? value.to_f : value
  end

  def self.format_num(value)
    rounded = (scalar(value) * 1.0e6).round / 1.0e6
    rounded.to_s
  end

  class UnitValue < Numeric
    include Comparable

    attr_reader :value

    def initialize(value)
      @value = RRCADUnits.scalar(value)
    end

    def to_f
      @value.to_f
    end

    def to_i
      @value.to_i
    end

    def to_int
      to_i
    end

    def round(*args)
      @value.round(*args)
    end

    def floor
      @value.floor
    end

    def ceil
      @value.ceil
    end

    def abs
      self.class.new(@value.abs)
    end

    def -@
      self.class.new(-@value)
    end

    def <=>(other)
      if other.is_a?(UnitValue)
        return nil unless other.class == self.class
        @value <=> other.value
      elsif other.is_a?(Numeric)
        @value <=> other.to_f
      else
        nil
      end
    end

    def ==(other)
      if other.is_a?(UnitValue)
        other.class == self.class && @value == other.value
      elsif other.is_a?(Numeric)
        @value == other.to_f
      else
        false
      end
    end

    alias eql? ==

    def hash
      [self.class, @value].hash
    end

    def coerce(other)
      if other.is_a?(UnitValue)
        raise TypeError, "incompatible units: #{other.class} and #{self.class}" unless other.class == self.class
        [other, self]
      elsif other.is_a?(Numeric)
        [UnitScalar.new(other), self]
      else
        raise TypeError, "#{other.class} cannot be coerced into #{self.class}"
      end
    end

    def +(other)
      if other.is_a?(UnitValue)
        raise TypeError, "incompatible units: #{other.class} and #{self.class}" unless other.class == self.class
        self.class.new(@value + other.value)
      elsif other.is_a?(Numeric)
        self.class.new(@value + other.to_f)
      else
        raise TypeError, "#{other.class} cannot be added to #{self.class}"
      end
    end

    def -(other)
      if other.is_a?(UnitValue)
        raise TypeError, "incompatible units: #{other.class} and #{self.class}" unless other.class == self.class
        self.class.new(@value - other.value)
      elsif other.is_a?(Numeric)
        self.class.new(@value - other.to_f)
      else
        raise TypeError, "#{other.class} cannot be subtracted from #{self.class}"
      end
    end

    def *(other)
      if other.is_a?(UnitValue)
        raise TypeError, "cannot multiply #{self.class} by #{other.class}"
      elsif other.is_a?(Numeric)
        self.class.new(@value * other.to_f)
      else
        raise TypeError, "#{other.class} cannot be multiplied with #{self.class}"
      end
    end

    def /(other)
      if other.is_a?(UnitValue)
        raise TypeError, "cannot divide #{self.class} by #{other.class}"
      elsif other.is_a?(Numeric)
        self.class.new(@value / other.to_f)
      else
        raise TypeError, "#{other.class} cannot divide #{self.class}"
      end
    end

    def inspect
      "#{RRCADUnits.format_num(@value)}#{suffix}"
    end

    alias to_s inspect

    protected

    def suffix
      ""
    end
  end

  class UnitScalar < UnitValue
    def coerce(other)
      if other.is_a?(Numeric)
        [UnitScalar.new(other), self]
      else
        raise TypeError, "#{other.class} cannot be coerced into UnitScalar"
      end
    end

    def +(other)
      if other.is_a?(UnitValue)
        other.class.new(@value + other.value)
      elsif other.is_a?(Numeric)
        @value + other.to_f
      else
        raise TypeError, "#{other.class} cannot be added to UnitScalar"
      end
    end

    def -(other)
      if other.is_a?(UnitValue)
        other.class.new(@value - other.value)
      elsif other.is_a?(Numeric)
        @value - other.to_f
      else
        raise TypeError, "#{other.class} cannot be subtracted from UnitScalar"
      end
    end

    def *(other)
      if other.is_a?(UnitValue)
        other.class.new(@value * other.value)
      elsif other.is_a?(Numeric)
        @value * other.to_f
      else
        raise TypeError, "#{other.class} cannot be multiplied with UnitScalar"
      end
    end

    def /(other)
      if other.is_a?(UnitValue)
        raise TypeError, "cannot divide UnitScalar by #{other.class}"
      elsif other.is_a?(Numeric)
        @value / other.to_f
      else
        raise TypeError, "#{other.class} cannot divide UnitScalar"
      end
    end
  end

  class UnitLength < UnitValue
    def mm
      self
    end

    alias millimeter mm
    alias millimeters mm

    def cm
      raise TypeError, "cannot convert length to centimeters once typed"
    end

    alias centimeter cm
    alias centimeters cm

    def m
      raise TypeError, "cannot convert length to metres once typed"
    end

    alias meter m
    alias meters m

    def inch
      raise TypeError, "cannot convert length to inches once typed"
    end

    alias inches inch

    def deg
      raise TypeError, "cannot convert length to angle"
    end

    alias degree deg
    alias degrees deg

    def rad
      raise TypeError, "cannot convert length to angle"
    end

    alias radian rad
    alias radians rad

    protected

    def suffix
      "mm"
    end
  end

  class UnitAngle < UnitValue
    def deg
      self
    end

    alias degree deg
    alias degrees deg

    def rad
      raise TypeError, "cannot convert angle to length"
    end

    alias radian rad
    alias radians rad

    def mm
      raise TypeError, "cannot convert angle to length"
    end

    alias millimeter mm
    alias millimeters mm
    alias cm mm
    alias centimeter mm
    alias centimeters mm
    alias m mm
    alias meter mm
    alias meters mm
    alias inch mm
    alias inches mm

    protected

    def suffix
      "deg"
    end
  end

  def self.length(value)
    UnitLength.new(value)
  end

  def self.angle(value)
    UnitAngle.new(value)
  end
end

class Numeric
  def mm
    RRCADUnits.length(self)
  end

  alias millimeter mm
  alias millimeters mm

  def cm
    RRCADUnits.length(self * 10.0)
  end

  alias centimeter cm
  alias centimeters cm

  def m
    RRCADUnits.length(self * 1000.0)
  end

  alias meter m
  alias meters m

  def inch
    RRCADUnits.length(self * 25.4)
  end

  alias inches inch

  def deg
    RRCADUnits.angle(self)
  end

  alias degree deg
  alias degrees deg

  def rad
    RRCADUnits.angle(self * 180.0 / Math::PI)
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
  attr_accessor :points, :lines, :constraints, :named, :profile

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

  def diagnostics
    analyze_diagnostics
  end

  def to_profile(diagnostics: false, strict: false)
    diagnostics_payload = (diagnostics || strict) ? analyze_diagnostics : nil
    if strict && diagnostics_payload && !diagnostics_payload[:redundant_constraints].empty?
      redundant = diagnostics_payload[:redundant_constraints]
      labels = redundant.map { |entry| entry[:summary] }.join(", ")
      raise RuntimeError, "sketch has redundant constraints: #{labels}"
    end

    if @profile
      solve_constraints
      shape = profile_shape
      shape.sketch_diagnostics = diagnostics_payload if diagnostics_payload
      return shape
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

    shape = polygon(pts)
    shape.sketch_diagnostics = diagnostics_payload if diagnostics_payload
    shape
  end

  def analyze_diagnostics
    baseline = clone_builder
    begin
      baseline.solve_constraints
    rescue RuntimeError => e
      components = component_report(baseline)
      return {
        status: :unsolved,
        error: e.message,
        component_count: components.length,
        components: components,
        estimated_dof: components.sum { |component| component[:estimated_dof] },
        redundant_constraint_count: 0,
        redundant_constraints: [],
      }
    end

    baseline_points = baseline.point_states

    redundant_constraints = []
    @constraints.each_with_index do |constraint, idx|
      trial = clone_builder
      trial.constraints.delete_at(idx)
      begin
        trial.solve_constraints
      rescue RuntimeError
        next
      end

      next unless same_point_states?(baseline_points, trial.point_states)

      redundant_constraints << {
        index: idx + 1,
        type: constraint[0],
        summary: constraint_summary(constraint),
      }
    end

    components = component_report(baseline)
    unsolved = components.any? { |component| !component[:unresolved_points].empty? }

    {
      status: unsolved ? :unsolved : (redundant_constraints.empty? ? :ok : :redundant_constraints),
      component_count: components.length,
      components: components,
      estimated_dof: components.sum { |component| component[:estimated_dof] },
      redundant_constraint_count: redundant_constraints.length,
      redundant_constraints: redundant_constraints,
    }
  end

  def component_report(builder)
    points = builder.points
    builder.connected_components.map.with_index do |indices, component_index|
      component_points = indices.map { |i| points[i] }
      unresolved_points = component_points.select { |point| !point.resolved? }
      {
        index: component_index + 1,
        point_count: component_points.length,
        points: component_points.map { |point| builder.point_label(point) },
        unresolved_points: unresolved_points.map { |point| builder.point_label(point) },
        estimated_dof: unresolved_points.sum { |point| builder.missing_coords(point).split("/").length },
      }
    end
  end

  def clone_builder
    clone = SketchBuilder.new
    point_map = {}

    @points.each do |point|
      copy = SketchPoint.new(point.x, point.y)
      clone.points << copy
      point_map[point] = copy
    end

    @named.each do |name, point|
      clone.named[name] = point_map.fetch(point)
    end

    @lines.each do |a, b|
      clone.lines << [point_map.fetch(a), point_map.fetch(b)]
    end

    @constraints.each do |constraint|
      clone.constraints << remap_constraint(constraint, point_map)
    end

    clone.profile = remap_profile(@profile, point_map) if @profile
    clone
  end

  def remap_constraint(constraint, point_map)
    type = constraint[0]
    case type
    when :fixed
      _type, p, x, y = constraint
      [:fixed, point_map.fetch(p), x, y]
    when :horizontal, :vertical
      _type, a, b = constraint
      [type, point_map.fetch(a), point_map.fetch(b)]
    when :coincident
      _type, a, b = constraint
      [:coincident, point_map.fetch(a), point_map.fetch(b)]
    when :dimension
      _type, a, b, length = constraint
      [:dimension, point_map.fetch(a), point_map.fetch(b), length]
    when :equal_length, :parallel, :perpendicular
      _type, a, b, c, d = constraint
      [type, point_map.fetch(a), point_map.fetch(b), point_map.fetch(c), point_map.fetch(d)]
    when :midpoint
      _type, p, a, b = constraint
      [:midpoint, point_map.fetch(p), point_map.fetch(a), point_map.fetch(b)]
    when :centered_dimension
      _type, center, a, b, attr, length = constraint
      [:centered_dimension, point_map.fetch(center), point_map.fetch(a), point_map.fetch(b), attr, length]
    when :symmetric
      _type, a, b, center = constraint
      [:symmetric, point_map.fetch(a), point_map.fetch(b), point_map.fetch(center)]
    when :mirror_x, :mirror_y
      _type, source, target, axis = constraint
      [type, point_map.fetch(source), point_map.fetch(target), axis]
    when :tangent
      _type, a, b, center, radius, side = constraint
      [:tangent, point_map.fetch(a), point_map.fetch(b), point_map.fetch(center), radius, side]
    when :polar_point
      _type, p, center, radius, angle_deg = constraint
      [:polar_point, point_map.fetch(p), point_map.fetch(center), radius, angle_deg]
    else
      constraint.dup
    end
  end

  def remap_profile(profile, point_map)
    return nil if profile.nil?

    type = profile[0]
    case type
    when :circle_at
      _type, center, radius = profile
      [:circle_at, point_map.fetch(center), radius]
    when :arc_at
      _type, center, radius, start_deg, end_deg = profile
      [:arc_at, point_map.fetch(center), radius, start_deg, end_deg]
    when :slot_between
      _type, a, b, radius = profile
      [:slot_between, point_map.fetch(a), point_map.fetch(b), radius]
    else
      profile.dup
    end
  end

  def point_states
    @points.map { |point| [point.x, point.y] }
  end

  def same_point_states?(a, b)
    return false unless a.length == b.length

    a.zip(b).all? do |lhs, rhs|
      same_value?(lhs[0], rhs[0]) && same_value?(lhs[1], rhs[1])
    end
  end

  def same_value?(lhs, rhs)
    return true if lhs.nil? && rhs.nil?
    return false if lhs.nil? || rhs.nil?

    (lhs - rhs).abs <= 1.0e-9
  end

  def connected_components
    parent = (0...@points.length).to_a

    find = lambda do |i|
      parent[i] = find.call(parent[i]) if parent[i] != i
      parent[i]
    end

    union = lambda do |a, b|
      ra = find.call(a)
      rb = find.call(b)
      parent[ra] = rb if ra != rb
    end

    index_for = {}
    @points.each_with_index { |point, idx| index_for[point] = idx }

    @lines.each do |a, b|
      union.call(index_for.fetch(a), index_for.fetch(b))
    end

    @constraints.each do |constraint|
      point_indices = constraint_points(constraint).map { |point| index_for[point] }.compact
      point_indices.each_cons(2) do |a, b|
        union.call(a, b)
      end
    end

    groups = Hash.new { |hash, key| hash[key] = [] }
    @points.each_with_index do |point, idx|
      groups[find.call(idx)] << idx
    end
    groups.values
  end

  def constraint_points(constraint)
    case constraint[0]
    when :fixed
      [constraint[1]]
    when :horizontal, :vertical, :coincident
      [constraint[1], constraint[2]]
    when :dimension
      [constraint[1], constraint[2]]
    when :equal_length, :parallel, :perpendicular
      constraint[1, 4]
    when :midpoint
      [constraint[1], constraint[2], constraint[3]]
    when :centered_dimension
      [constraint[1], constraint[2], constraint[3]]
    when :symmetric
      [constraint[1], constraint[2], constraint[3]]
    when :mirror_x, :mirror_y
      [constraint[1], constraint[2]]
    when :tangent
      [constraint[1], constraint[2], constraint[3]]
    when :polar_point
      [constraint[1], constraint[2]]
    else
      []
    end
  end

  def constraint_summary(constraint)
    type = constraint[0]
    case type
    when :fixed
      _type, point, x, y = constraint
      parts = ["fixed #{point_label(point)}"]
      parts << "x=#{format_num(x)}" unless x.nil?
      parts << "y=#{format_num(y)}" unless y.nil?
      parts.join(" ")
    when :horizontal, :vertical, :coincident
      _type, a, b = constraint
      "#{type} #{point_label(a)} #{point_label(b)}"
    when :dimension
      _type, a, b, length = constraint
      "dimension #{point_label(a)}→#{point_label(b)}=#{format_num(length)}"
    when :equal_length, :parallel, :perpendicular
      _type, a, b, c, d = constraint
      "#{type} #{point_label(a)}→#{point_label(b)} / #{point_label(c)}→#{point_label(d)}"
    when :midpoint
      _type, p, a, b = constraint
      "midpoint #{point_label(p)} = midpoint(#{point_label(a)}, #{point_label(b)})"
    when :centered_dimension
      _type, center, a, b, attr, length = constraint
      "centered_dimension #{point_label(center)} #{attr}=#{format_num(length)}"
    when :symmetric
      _type, a, b, center = constraint
      "symmetric #{point_label(a)} #{point_label(b)} around #{point_label(center)}"
    when :mirror_x, :mirror_y
      _type, source, target, axis = constraint
      "#{type} #{point_label(source)} #{point_label(target)} axis=#{format_num(axis)}"
    when :tangent
      _type, a, b, center, radius, side = constraint
      label = "tangent #{point_label(a)} #{point_label(b)} around #{point_label(center)} r=#{format_num(radius)}"
      side.nil? ? label : "#{label} side=#{side}"
    when :polar_point
      _type, p, center, radius, angle_deg = constraint
      "polar_point #{point_label(p)} around #{point_label(center)} r=#{format_num(radius)} angle=#{format_num(angle_deg)}"
    else
      type.to_s
    end
  end

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
    dx = RRCADUnits.scalar(b.x) - RRCADUnits.scalar(a.x)
    dy = RRCADUnits.scalar(b.y) - RRCADUnits.scalar(a.y)
    length = Math.sqrt(dx * dx + dy * dy)
    raise RuntimeError, "slot_between requires distinct points" if length <= 1.0e-9

    pts = arc_points(length, 0.0, radius, -90.0, 90.0, 12)
    pts += arc_points(0.0, 0.0, radius, 90.0, 270.0, 12)

    polygon(pts)
      .rotate(0, 0, 1, Math.atan2(dy, dx) * 180.0 / Math::PI)
      .translate(a.x, a.y, 0)
  end

  def arc_points(cx, cy, radius, start_deg, end_deg, segments)
    pts = []
    i = 0
    while i <= segments
      t = start_deg + (end_deg - start_deg) * i / segments
      rad = RRCADUnits.scalar(t) * Math::PI / 180.0
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

    rad = RRCADUnits.scalar(angle_deg) * Math::PI / 180.0
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
    abx = RRCADUnits.scalar(b.x) - RRCADUnits.scalar(a.x)
    aby = RRCADUnits.scalar(b.y) - RRCADUnits.scalar(a.y)
    apx = RRCADUnits.scalar(p.x) - RRCADUnits.scalar(a.x)
    apy = RRCADUnits.scalar(p.y) - RRCADUnits.scalar(a.y)
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
    dx = RRCADUnits.scalar(b.x) - RRCADUnits.scalar(a.x)
    dy = RRCADUnits.scalar(b.y) - RRCADUnits.scalar(a.y)
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
  attr_accessor :sketch_diagnostics

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
    mag = Math.sqrt(RRCADUnits.scalar(ax) * RRCADUnits.scalar(ax) +
                    RRCADUnits.scalar(ay) * RRCADUnits.scalar(ay) +
                    RRCADUnits.scalar(az) * RRCADUnits.scalar(az))
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

  # Return the feature graph as an array of hashes:
  #   [{ id: 1, parents: [], label: "box(...)", entry: "..." }, ...]
  def feature_graph
    raise NotImplementedError, "Shape#feature_graph is not yet implemented (Phase 10)"
  end

  # Rebuild this shape by replaying its stored feature graph.
  def rebuild
    raise NotImplementedError, "Shape#rebuild is not yet implemented (Phase 10)"
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

  # For a cylindrical face, return its axis information:
  #   { origin: [ox, oy, oz], axis: [ax, ay, az], radius: r }
  # The origin is a point on the axis; the axis vector is a unit direction.
  # Raises if the shape isn't a face or the underlying surface isn't a
  # cylinder.  Use #face_type or rescue the error to test for cylindricity.
  def cylinder_axis
    raise NotImplementedError, "Shape#cylinder_axis is not yet implemented (Phase 10)"
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

  def name_face(_name, _selector)
    raise NotImplementedError, "Shape#name_face is not yet implemented (Named faces, edges, and datums)"
  end

  def name_edge(_name, _selector)
    raise NotImplementedError, "Shape#name_edge is not yet implemented (Named faces, edges, and datums)"
  end

  def datum(_name, _shape)
    raise NotImplementedError, "Shape#datum is not yet implemented (Named faces, edges, and datums)"
  end

  def ref(_name)
    raise NotImplementedError, "Shape#ref is not yet implemented (Named faces, edges, and datums)"
  end

  def gdt_apply(_spec)
    raise NotImplementedError, "Shape#gdt_apply is not yet implemented (Standard GD&T implementation)"
  end
end

class GdtBuilder
  def initialize(shape, standard = :asme)
    unless shape.is_a?(Shape)
      raise ArgumentError, "gdt requires a Shape"
    end
    @shape = shape
    @standard = normalize_standard!(standard)
    @datum = nil
    @feature_control = nil
  end

  def datum(label = nil, face: nil, selector: nil, name: nil)
    label = name if label.nil?
    label = normalize_label!(label)
    anchor = resolve_anchor!(face, selector, "gdt datum")
    if anchor.nil? && selector.nil?
      raise ArgumentError, "gdt datum requires face: or selector:"
    end
    @datum = { label: label, selector: selector_name(selector), anchor: anchor }
    self
  end

  def feature_control(text: nil, frame: nil, value: nil, face: nil, selector: nil, datums: [],
                      modifiers: [])
    feature_text = text || frame || value
    if feature_text.nil?
      raise ArgumentError, "gdt feature_control requires text:, frame:, or value:"
    end
    feature_text = normalize_label!(feature_text)
    datums = normalize_name_list!(datums, "gdt feature_control datums")
    modifiers = normalize_name_list!(modifiers, "gdt feature_control modifiers")
    anchor = resolve_anchor!(face, selector, "gdt feature_control")
    if anchor.nil? && selector.nil?
      raise ArgumentError, "gdt feature_control requires face: or selector:"
    end
    @feature_control = {
      text: feature_text,
      selector: selector_name(selector),
      anchor: anchor,
      datums: datums,
      modifiers: modifiers,
    }
    self
  end

  def to_h
    spec = { standard: @standard }
    spec[:datum] = @datum if @datum
    spec[:feature_control] = @feature_control if @feature_control
    spec
  end

  def commit
    @shape.gdt_apply(to_h)
    @shape
  end

  private

  def normalize_standard!(standard)
    standard = standard.to_sym if standard.is_a?(String)
    unless [:asme, :iso].include?(standard)
      raise ArgumentError, "gdt standard must be :asme or :iso"
    end
    standard
  end

  def normalize_label!(value)
    unless value.is_a?(Symbol) || value.is_a?(String) || value.is_a?(Numeric)
      raise ArgumentError, "gdt label must be a Symbol, String, or Numeric"
    end
    value.to_s
  end

  def normalize_name_list!(value, name)
    values = value.is_a?(Array) ? value : [value]
    values.map do |item|
      unless item.is_a?(Symbol) || item.is_a?(String) || item.is_a?(Numeric)
        raise ArgumentError, "#{name} must be Symbols or Strings"
      end
      item.to_s
    end
  end

  def selector_name(selector)
    return nil if selector.nil?
    unless selector.is_a?(Symbol) || selector.is_a?(String)
      raise ArgumentError, "selector must be a Symbol or String"
    end
    selector.to_s
  end

  def resolve_anchor!(face, selector, name)
    if !face.nil?
      unless face.is_a?(Shape)
        raise ArgumentError, "#{name} face must be a Shape"
      end
      unless face.shape_type == :face
        raise ArgumentError, "#{name} face must be a Face"
      end
      face.centroid
    elsif !selector.nil?
      selector = selector.to_s
      selected = @shape.faces(selector)
      if selected.empty?
        raise ArgumentError, "#{name} selector matched no faces"
      end
      selected.first.centroid
    else
      nil
    end
  end
end

class Shape
  def gdt(standard: :asme, &block)
    builder = GdtBuilder.new(self, standard)
    if block_given?
      if block.arity == 1
        block.call(builder)
      else
        builder.instance_eval(&block)
      end
    end
    builder.commit
  end
end

# ---------------------------------------------------------------------------
# Assembly — groups named shapes; supports place; mate is Phase 5.
# ---------------------------------------------------------------------------
class Assembly
  SOLVER_TOLERANCE = 1.0e-6

  class FaceRef
    attr_reader :part_name, :selector

    def initialize(part_name, selector)
      @part_name = part_name
      @selector = selector
    end

    def inspect
      "#<Assembly::FaceRef #{part_name}:#{selector.inspect}>"
    end
  end

  class PartBuilder
    def initialize(assembly, part)
      @assembly = assembly
      @part = part
    end

    def face(part_name, selector)
      @assembly.face(part_name, selector)
    end

    def mate(from:, to:, offset: 0.0)
      from_sel = @assembly.__send__(:normalize_local_selector!, from, @part[:name], "mate from:")
      to_ref = @assembly.__send__(:normalize_face_ref!, to, "mate to:")
      @assembly.__send__(:validate_numeric!, offset, "mate offset")
      @part[:constraints] << [:mate, from_sel, to_ref, offset]
      @assembly.__send__(:mark_solver_dirty!)
      to_ref
    end

    def distance_mate(from:, to:, distance:)
      from_sel = @assembly.__send__(:normalize_local_selector!, from, @part[:name], "distance_mate from:")
      to_ref = @assembly.__send__(:normalize_face_ref!, to, "distance_mate to:")
      @assembly.__send__(:validate_positive_numeric!, distance, "distance_mate distance")
      @part[:constraints] << [:distance_mate, from_sel, to_ref, distance]
      @assembly.__send__(:mark_solver_dirty!)
      to_ref
    end

    def angle_mate(from:, to:, angle:, pivot:, axis_dir:, offset: 0.0)
      from_sel = @assembly.__send__(:normalize_local_selector!, from, @part[:name], "angle_mate from:")
      to_ref = @assembly.__send__(:normalize_face_ref!, to, "angle_mate to:")
      @assembly.__send__(:validate_numeric!, angle, "angle_mate angle")
      @assembly.__send__(:validate_point!, pivot, "angle_mate pivot")
      @assembly.__send__(:validate_point!, axis_dir, "angle_mate axis_dir")
      @assembly.__send__(:validate_numeric!, offset, "angle_mate offset")
      @part[:constraints] << [:angle_mate, from_sel, to_ref, angle, pivot, axis_dir, offset]
      @assembly.__send__(:mark_solver_dirty!)
      to_ref
    end
  end

  def initialize(name)
    @name = name
    @shapes = []
    @solver_parts = []
    @solver_parts_by_name = {}
    @solver_cache = nil
    @solver_dirty = true
  end

  def place(shape)
    @shapes << shape
    shape
  end

  # Declare a named rigid part to be solved lazily from constraints.
  # The first declared part is fixed by default unless fixed: false is given.
  def part(name, shape, fixed: nil, &block)
    unless shape.is_a?(Shape)
      raise ArgumentError, "part shape must be a Shape"
    end
    name = normalize_part_name(name)
    if @solver_parts_by_name.key?(name)
      raise ArgumentError, "duplicate assembly part #{name.inspect}"
    end
    fixed = @solver_parts.empty? if fixed.nil?
    part = { name: name, shape: shape, fixed: fixed, constraints: [] }
    @solver_parts << part
    @solver_parts_by_name[name] = part
    mark_solver_dirty!

    if block_given?
      builder = PartBuilder.new(self, part)
      if block.arity == 1
        block.call(builder)
      else
        builder.instance_eval(&block)
      end
    end
    shape
  end

  def ground(name, shape, &block)
    part(name, shape, fixed: true, &block)
  end

  def face(part_name, selector)
    FaceRef.new(normalize_part_name(part_name), selector)
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

  # Solve the declarative assembly graph and return a Hash of part name →
  # positioned Shape.  The result is cached until the assembly changes.
  def solve
    return @solver_cache unless @solver_dirty
    if @solver_parts.empty?
      @solver_cache = {}
      @solver_dirty = false
      return @solver_cache
    end

    validate_solver_refs!

    resolved = {}
    @solver_parts.each do |part|
      resolved[part[:name]] = part[:shape] if part[:fixed]
    end

    progress = true
    while progress
      progress = false
      @solver_parts.each do |part|
        next if resolved.key?(part[:name])
        candidate = solve_part_candidate(part, resolved)
        next unless candidate
        resolved[part[:name]] = candidate
        progress = true
      end
    end

    unresolved = @solver_parts.map { |part| part[:name] } - resolved.keys
    unless unresolved.empty?
      raise RuntimeError,
            "assembly '#{@name}' is under-constrained: unresolved parts #{unresolved.map { |name| ":#{name}" }.join(', ')}"
    end

    @solver_cache = resolved
    @solver_dirty = false
    resolved
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

  def vec_dot(a, b)
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
  end

  def vec_scale(v, s)
    [v[0] * s, v[1] * s, v[2] * s]
  end

  def vec_length(v)
    Math.sqrt(RRCADUnits.scalar(vec_dot(v, v)))
  end

  def vec_normalize(v, label = "axis")
    mag = Math.sqrt(RRCADUnits.scalar(v[0]) * RRCADUnits.scalar(v[0]) +
                    RRCADUnits.scalar(v[1]) * RRCADUnits.scalar(v[1]) +
                    RRCADUnits.scalar(v[2]) * RRCADUnits.scalar(v[2]))
    raise ArgumentError, "#{label} axis points must be distinct" if mag < 1.0e-12
    [v[0] / mag, v[1] / mag, v[2] / mag]
  end

  def format_num(v)
    return v.to_s unless v.is_a?(Numeric)
    rounded = (v * 1.0e6).round / 1.0e6
    rounded.to_s
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

  def normalize_part_name(name)
    case name
    when Symbol then name
    when String then name.to_sym
    else
      raise ArgumentError, "assembly part name must be a Symbol or String"
    end
  end

  def normalize_face_ref!(ref, label)
    case ref
    when FaceRef
      ref
    when Array
      if ref.length == 2
        FaceRef.new(normalize_part_name(ref[0]), ref[1])
      else
        raise ArgumentError, "#{label} must be an Assembly::FaceRef or [part_name, selector]"
      end
    else
      raise ArgumentError, "#{label} must be an Assembly::FaceRef or [part_name, selector]"
    end
  end

  def normalize_local_selector!(selector, part_name, label)
    case selector
    when Symbol, String
      selector
    when FaceRef
      if selector.part_name == part_name
        selector.selector
      else
        raise ArgumentError, "#{label} must refer to the current part"
      end
    else
      raise ArgumentError, "#{label} must be a Symbol, String, or Assembly::FaceRef"
    end
  end

  def validate_numeric!(value, label)
    unless value.is_a?(Numeric)
      raise ArgumentError, "#{label} must be a number"
    end
  end

  def validate_positive_numeric!(value, label)
    validate_numeric!(value, label)
    raise ArgumentError, "#{label} must be > 0" unless value > 0
  end

  def validate_point!(point, label)
    unless point.is_a?(Array) && point.length == 3 && point.all? { |v| v.is_a?(Numeric) }
      raise ArgumentError, "#{label} must be a 3-element numeric array"
    end
  end

  def mark_solver_dirty!
    @solver_dirty = true
    @solver_cache = nil
  end

  def resolve_face_ref(ref, resolved)
    part = @solver_parts_by_name[ref.part_name]
    raise RuntimeError, "assembly '#{@name}' references unknown part #{ref.part_name.inspect}" if part.nil?
    shape = resolved[ref.part_name]
    return nil unless shape
    faces = shape.faces(ref.selector)
    face = faces.first
    raise RuntimeError, "assembly '#{@name}' face #{ref.inspect} resolved to no face" if face.nil?
    face
  end

  def solve_part_candidate(part, resolved)
    return nil if part[:constraints].empty?
    return nil unless part[:constraints].all? { |constraint| constraint_target_resolved?(constraint, resolved) }

    candidate = part[:shape]
    part[:constraints].each do |constraint|
      kind, from_sel, to_ref, *rest = constraint
      target_face = resolve_face_ref(to_ref, resolved)
      case kind
      when :mate
        source_face = candidate.faces(from_sel).first
        raise RuntimeError, "assembly '#{@name}' face #{from_sel.inspect} resolved to no face" if source_face.nil?
        candidate = candidate.mate(source_face, target_face, rest[0])
      when :distance_mate
        source_face = candidate.faces(from_sel).first
        raise RuntimeError, "assembly '#{@name}' face #{from_sel.inspect} resolved to no face" if source_face.nil?
        candidate = candidate.mate(source_face, target_face, rest[0])
      when :angle_mate
        source_face = candidate.faces(from_sel).first
        raise RuntimeError, "assembly '#{@name}' face #{from_sel.inspect} resolved to no face" if source_face.nil?
        angle, pivot, axis_dir, offset = rest
        candidate = candidate.mate(source_face, target_face, offset)
        candidate = candidate.rotate_about(pivot, axis_dir, angle)
      else
        raise RuntimeError, "assembly '#{@name}' does not support constraint #{kind.inspect}"
      end
    end

    verify_part_constraints!(part, candidate, resolved)
    candidate
  end

  def constraint_target_resolved?(constraint, resolved)
    _kind, _from_sel, to_ref, *_rest = constraint
    resolved.key?(to_ref.part_name)
  end

  def verify_part_constraints!(part, candidate, resolved)
    part[:constraints].each do |constraint|
      kind, from_sel, to_ref, *rest = constraint
      next unless [:mate, :distance_mate, :angle_mate].include?(kind)
      target_face = resolve_face_ref(to_ref, resolved)
      source_face = candidate.faces(from_sel).first
      raise RuntimeError, "assembly '#{@name}' face #{from_sel.inspect} resolved to no face" if source_face.nil?
      expected_offset = kind == :angle_mate ? rest[3] : rest[0]
      verify_face_relation!(part[:name], kind, source_face, target_face, expected_offset)
    end
  end

  def verify_face_relation!(part_name, kind, source_face, target_face, expected_offset)
    source_centroid = source_face.centroid
    target_centroid = target_face.centroid
    target_normal = target_face.normal
    source_normal = source_face.normal

    dot = source_normal[0] * target_normal[0] + source_normal[1] * target_normal[1] +
          source_normal[2] * target_normal[2]
    unless (dot + 1.0).abs <= 1.0e-4
      raise RuntimeError,
            "assembly '#{@name}' conflicting #{kind} constraint on #{part_name}: face normals are not antiparallel"
    end

    delta = vec_sub(source_centroid, target_centroid)
    along = vec_dot(delta, target_normal)
    tangent = vec_sub(delta, vec_scale(target_normal, along))
    unless vec_length(tangent) <= SOLVER_TOLERANCE
      raise RuntimeError,
            "assembly '#{@name}' conflicting #{kind} constraint on #{part_name}: face centroids do not line up"
    end

    unless (along - expected_offset).abs <= 1.0e-4
      raise RuntimeError,
            "assembly '#{@name}' conflicting #{kind} constraint on #{part_name}: expected offset #{format_num(expected_offset)}, got #{format_num(along)}"
    end
  end

  def validate_solver_refs!
    @solver_parts.each do |part|
      part[:constraints].each do |constraint|
        _kind, _from_sel, to_ref, *_rest = constraint
        next if @solver_parts_by_name.key?(to_ref.part_name)
        raise RuntimeError,
              "assembly '#{@name}' references unknown part #{to_ref.part_name.inspect}"
      end
    end
  end

  private :validate_axis_pair!, :vec_sub, :vec_normalize, :vec_dot, :vec_scale, :vec_length,
          :apply_axis_rotation, :normalize_part_name, :normalize_face_ref!,
          :normalize_local_selector!, :validate_numeric!, :validate_positive_numeric!,
          :validate_point!, :mark_solver_dirty!, :resolve_face_ref,
          :solve_part_candidate, :constraint_target_resolved?, :verify_part_constraints!,
          :verify_face_relation!, :validate_solver_refs!

  def to_shape
    shapes = @shapes.dup
    shapes.concat(solve.values) unless @solver_parts.empty?
    raise RuntimeError, "Assembly '#{@name}' contains no shapes" if shapes.empty?
    shapes.inject { |acc, s| acc.fuse(s) }
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

  # clearance_hole(size, depth:) — standard close-clearance hole tool.
  # +size+ may be a Symbol/String naming a metric (`:m2`, `:m2_5`, `:m3`,
  # `:m4`, `:m5`) or imperial (`:"4-40"`, `:"6-32"`, `:"8-32"`, `:"10-32"`,
  # `:"10-24"`, `:"1/4-20"`, `:"5/16-18"`, `:"3/8-16"`) fastener, or a
  # numeric diameter in millimetres. Imperial values follow ASME B18.2.8
  # close-fit. Returns a cylindrical solid suitable for subtracting with
  # `.cut`.
  def clearance_hole(size, depth:)
    d = hardware_diameter(size, {
      # Metric (ISO close-fit)
      "m2" => 2.4,
      "m2_5" => 2.9,
      "m25" => 2.9,
      "m3" => 3.4,
      "m4" => 4.5,
      "m5" => 5.5,
      # Imperial (ASME B18.2.8 close-fit), values in mm
      "4_40" => 2.95,
      "6_32" => 3.66,
      "8_32" => 4.32,
      "10_32" => 4.98,
      "10_24" => 4.98,
      "1/4_20" => 6.53,
      "5/16_18" => 8.20,
      "3/8_16" => 9.80,
    }, "clearance_hole")
    validate_positive_dimension(depth, "clearance_hole depth")
    cylinder(d / 2.0, depth)
  end

  # tap_drill(size, depth:) — standard tap-drill hole tool (75% thread).
  # Metric coarse threads (`:m2`–`:m5`) and imperial UNC sizes (`:"4-40"`,
  # `:"6-32"`, `:"8-32"`, `:"10-32"`, `:"10-24"`, `:"1/4-20"`, `:"5/16-18"`,
  # `:"3/8-16"`) are supported, or pass a numeric drill diameter in
  # millimetres directly.
  def tap_drill(size, depth:)
    d = hardware_diameter(size, {
      # Metric coarse
      "m2" => 1.6,
      "m2_5" => 2.05,
      "m25" => 2.05,
      "m3" => 2.5,
      "m4" => 3.3,
      "m5" => 4.2,
      # Imperial UNC/UNF, 75% thread, values in mm
      "4_40" => 2.26,
      "6_32" => 2.71,
      "8_32" => 3.45,
      "10_32" => 4.04,
      "10_24" => 3.80,
      "1/4_20" => 5.11,
      "5/16_18" => 6.53,
      "3/8_16" => 7.94,
    }, "tap_drill")
    validate_positive_dimension(depth, "tap_drill depth")
    cylinder(d / 2.0, depth)
  end

  # heat_set_insert(size, depth:) — pilot-hole tool for common heat-set inserts.
  # Metric (`:m2`, `:m2_5`, `:m3`) and imperial (`:"4-40"`, `:"6-32"`,
  # `:"8-32"`, `:"10-32"`, `:"1/4-20"`) starter values are based on commonly
  # carried Tappex / E-Z LOK pilots; for unusual inserts pass a numeric
  # diameter in millimetres.
  def heat_set_insert(size, depth:)
    d = hardware_diameter(size, {
      "m2" => 3.2,
      "m2_5" => 3.8,
      "m25" => 3.8,
      "m3" => 4.6,
      "4_40" => 4.0,
      "6_32" => 4.5,
      "8_32" => 5.5,
      "10_32" => 6.0,
      "1/4_20" => 7.9,
    }, "heat_set_insert")
    validate_positive_dimension(depth, "heat_set_insert depth")
    cylinder(d / 2.0, depth)
  end

  # socket_head_cbore(size, depth:, head_depth:) — counterbore tool sized for
  # common socket-head cap screws.  Metric ISO 4762 (`:m2`–`:m5`) and imperial
  # ASME B18.3 inch sizes (`:"4-40"`, `:"6-32"`, `:"8-32"`, `:"10-32"`,
  # `:"10-24"`, `:"1/4-20"`, `:"5/16-18"`, `:"3/8-16"`) are supported.
  def socket_head_cbore(size, depth:, head_depth:)
    spec = hardware_spec(size, {
      "m2" => [2.4, 4.0],
      "m2_5" => [2.9, 5.0],
      "m25" => [2.9, 5.0],
      "m3" => [3.4, 6.0],
      "m4" => [4.5, 8.0],
      "m5" => [5.5, 10.0],
      # Imperial: [close-clearance, head OD], values in mm
      "4_40" => [2.95, 4.65],
      "6_32" => [3.66, 5.74],
      "8_32" => [4.32, 6.86],
      "10_32" => [4.98, 7.92],
      "10_24" => [4.98, 7.92],
      "1/4_20" => [6.53, 9.53],
      "5/16_18" => [8.20, 11.91],
      "3/8_16" => [9.80, 14.29],
    }, "socket_head_cbore")
    validate_positive_dimension(depth, "socket_head_cbore depth")
    validate_positive_dimension(head_depth, "socket_head_cbore head_depth")
    cbore(d: spec[0], cbore_d: spec[1], cbore_h: head_depth, depth: depth)
  end

  # flat_head_csink(size, depth:, angle: 45.0) — countersink tool sized for
  # common flat-head screws. +angle+ is the cone half-angle in degrees; the
  # default (45°) matches the 90° included angle of ISO 10642 metric flat
  # heads. Imperial flat heads (ANSI B18.3.5) use an 82° included angle —
  # pass `angle: 41` for those.  Metric (`:m2`–`:m5`) and imperial
  # (`:"4-40"`, `:"6-32"`, `:"8-32"`, `:"10-32"`, `:"1/4-20"`) sizes are
  # supported.
  def flat_head_csink(size, depth:, angle: 45.0)
    spec = hardware_spec(size, {
      "m2" => [2.4, 4.4],
      "m2_5" => [2.9, 5.5],
      "m25" => [2.9, 5.5],
      "m3" => [3.4, 6.3],
      "m4" => [4.5, 9.4],
      "m5" => [5.5, 10.4],
      # Imperial: [close-clearance, head OD], values in mm
      "4_40" => [2.95, 5.72],
      "6_32" => [3.66, 7.09],
      "8_32" => [4.32, 8.43],
      "10_32" => [4.98, 9.78],
      "1/4_20" => [6.53, 12.88],
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
  # +size+ may be metric (`:m2`, `:m2_5`, `:m3`, `:m4`, `:m5`) or imperial
  # UNC/UNF (`:"4-40"`, `:"6-32"`, `:"8-32"`, `:"10-32"`, `:"10-24"`,
  # `:"1/4-20"`, `:"5/16-18"`, `:"3/8-16"`). +length+ is the shank length
  # below the head, in millimetres. +style+ may be `:socket` (ISO 4762 /
  # ASME B18.3 cylindrical socket-head cap screw), `:button` (ISO 7380 /
  # ANSI B18.3.4 low dome head), or `:flat` (ISO 10642 90° / ANSI B18.3.5
  # 82° countersunk flat head — the body is approximated as a 90° cone for
  # both standards).
  #
  # Geometry: shank along +Z from z=0 to z=length; head sits above z=length.
  # For `:flat` the head is a conical frustum widening from shank_d at z=length
  # to head_d at z=length+head_h, suitable for sitting flush in a countersink.
  def screw(size, length:, style: :socket)
    spec = hardware_spec(size, {
      # [shaft_d, shcs_head_d, shcs_head_h, bhcs_head_d, bhcs_head_h, fhcs_head_d]
      "m2"   => [2.0, 3.8, 2.0, 3.5, 1.3, 3.8],
      "m2_5" => [2.5, 4.5, 2.5, 4.7, 1.5, 4.7],
      "m25"  => [2.5, 4.5, 2.5, 4.7, 1.5, 4.7],
      "m3"   => [3.0, 5.5, 3.0, 5.7, 1.65, 6.0],
      "m4"   => [4.0, 7.0, 4.0, 7.6, 2.2, 8.0],
      "m5"   => [5.0, 8.5, 5.0, 9.5, 2.75, 10.0],
      # Imperial UNC, values in mm (shaft = major dia)
      "4_40"    => [2.84, 4.65, 2.84, 4.80, 1.55, 5.72],
      "6_32"    => [3.51, 5.74, 3.51, 5.80, 1.85, 7.09],
      "8_32"    => [4.17, 6.86, 4.17, 6.90, 2.18, 8.43],
      "10_32"   => [4.83, 7.92, 4.83, 8.00, 2.50, 9.78],
      "10_24"   => [4.83, 7.92, 4.83, 8.00, 2.50, 9.78],
      "1/4_20"  => [6.35, 9.53, 6.35, 10.30, 3.30, 12.88],
      "5/16_18" => [7.94, 11.91, 7.94, 12.70, 4.10, 16.13],
      "3/8_16"  => [9.53, 14.29, 9.53, 15.20, 5.00, 19.35],
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
    sin_limit = Math.sin(RRCADUnits.scalar(max_angle_deg) * Math::PI / 180.0)
    part.faces("all").select do |face|
      nz = face.normal[2]
      nz < -sin_limit
    end
  end

  # hole_axes(part, orientation: nil, tolerance_deg: 5.0) — enumerate the
  # cylindrical-surface "holes" of +part+.  Returns an Array of Hashes:
  #   { origin: [ox, oy, oz], axis: [ax, ay, az], radius: r }
  # produced by `Shape#cylinder_axis` for every face whose underlying
  # surface is cylindrical (so external bosses and internal holes both
  # match — the radius alone doesn't distinguish them).
  #
  # When +orientation+ is given the result is filtered:
  #   :vertical   — axis is within +tolerance_deg+ of the world Z axis
  #   :horizontal — axis is within +tolerance_deg+ of the XY plane
  # The axis vector is always a unit direction; both +axis+ and −axis count
  # as the same orientation.
  def hole_axes(part, orientation: nil, tolerance_deg: 5.0)
    unless [nil, :vertical, :horizontal].include?(orientation)
      raise ArgumentError, "hole_axes orientation must be :vertical, :horizontal, or nil"
    end
    unless tolerance_deg.is_a?(Numeric) && tolerance_deg >= 0 && tolerance_deg <= 90
      raise ArgumentError, "hole_axes tolerance_deg must be in [0, 90]"
    end
    sin_tol = Math.sin(RRCADUnits.scalar(tolerance_deg) * Math::PI / 180.0)
    cos_tol = Math.cos(RRCADUnits.scalar(tolerance_deg) * Math::PI / 180.0)

    results = []
    part.faces("all").each do |face|
      begin
        info = face.cylinder_axis
      rescue RuntimeError
        next  # non-cylindrical face — skip
      end
      ax = info[:axis]
      az_abs = ax[2].abs

      keep = case orientation
             when :vertical   then az_abs >= cos_tol
             when :horizontal then az_abs <= sin_tol
             else true
             end
      results << info if keep
    end
    results
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
    mag = Math.sqrt(RRCADUnits.scalar(axis[0]) * RRCADUnits.scalar(axis[0]) +
                    RRCADUnits.scalar(axis[1]) * RRCADUnits.scalar(axis[1]) +
                    RRCADUnits.scalar(axis[2]) * RRCADUnits.scalar(axis[2]))
    raise ArgumentError, "draft_faces axis must be non-zero" if mag < 1.0e-12
    unless min_draft_deg.is_a?(Numeric) && min_draft_deg >= 0 && min_draft_deg <= 90
      raise ArgumentError, "draft_faces min_draft_deg must be in [0, 90]"
    end

    ux = axis[0] / mag
    uy = axis[1] / mag
    uz = axis[2] / mag
    sin_limit = Math.sin(RRCADUnits.scalar(min_draft_deg) * Math::PI / 180.0)

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

  def _cam_axis_spec(axis)
    case axis
    when :x
      [:yz, :x, 0, :x]
    when :y
      [:xz, :y, 1, :y]
    when :z
      [:xy, :z, 2, :z]
    when Array
      unless axis.length == 3
        raise ArgumentError,
              "unsupported_islands axis must be :x, :y, :z, or a 3-element numeric array"
      end
      i = 0
      while i < 3
        unless axis[i].is_a?(Numeric)
          raise ArgumentError,
                "unsupported_islands axis must be :x, :y, :z, or a 3-element numeric array"
        end
        i += 1
      end
      idx = 0
      max_abs = axis[0].abs
      i = 1
      while i < 3
        abs = axis[i].abs
        if abs > max_abs
          max_abs = abs
          idx = i
        end
        i += 1
      end
      raise ArgumentError, "unsupported_islands axis must be non-zero" if max_abs < 1.0e-12
      i = 0
      while i < 3
        if i != idx && axis[i].abs > 1.0e-12
          raise ArgumentError, "unsupported_islands axis must be aligned to X, Y, or Z"
        end
        i += 1
      end
      case idx
      when 0
        [:yz, :x, 0, :x]
      when 1
        [:xz, :y, 1, :y]
      when 2
        [:xy, :z, 2, :z]
      end
    else
      raise ArgumentError,
            "unsupported_islands axis must be :x, :y, :z, or a 3-element numeric array"
    end
  end

  def _cam_inflate_point(point2d, plane, offset)
    case plane
    when :xy
      [point2d[0], point2d[1], offset]
    when :xz
      [point2d[0], offset, point2d[1]]
    when :yz
      [offset, point2d[0], point2d[1]]
    else
      raise ArgumentError, "unsupported_islands plane must be :xy, :xz, or :yz"
    end
  end

  def _cam_edge_bbox_2d(edge, plane)
    bb = edge.bounding_box
    case plane
    when :xy
      [bb[:x], bb[:y], bb[:x] + bb[:dx], bb[:y] + bb[:dy]]
    when :xz
      [bb[:x], bb[:z], bb[:x] + bb[:dx], bb[:z] + bb[:dz]]
    when :yz
      [bb[:y], bb[:z], bb[:y] + bb[:dy], bb[:z] + bb[:dz]]
    else
      raise ArgumentError, "unsupported_islands plane must be :xy, :xz, or :yz"
    end
  end

  def _cam_bbox_union(a, b)
    [
      [a[0], b[0]].min,
      [a[1], b[1]].min,
      [a[2], b[2]].max,
      [a[3], b[3]].max,
    ]
  end

  def _cam_bbox_center(bbox)
    [
      (bbox[0] + bbox[2]) / 2.0,
      (bbox[1] + bbox[3]) / 2.0,
    ]
  end

  def _cam_bbox_area(bbox)
    width = [bbox[2] - bbox[0], 0.0].max
    height = [bbox[3] - bbox[1], 0.0].max
    width * height
  end

  def _cam_bbox_intersects?(a, b, tolerance)
    a[0] <= b[2] + tolerance &&
      a[2] + tolerance >= b[0] &&
      a[1] <= b[3] + tolerance &&
      a[3] + tolerance >= b[1]
  end

  def _cam_component_reports(section, plane, offset, tolerance, min_area)
    edges = section.edges("all")
    return [] if edges.empty?

    edge_boxes = edges.map { |edge| _cam_edge_bbox_2d(edge, plane) }
    adjacency = Hash.new { |h, k| h[k] = [] }

    edge_boxes.each_index do |i|
      ((i + 1)...edge_boxes.length).each do |j|
        next unless _cam_bbox_intersects?(edge_boxes[i], edge_boxes[j], tolerance)

        adjacency[i] << j
        adjacency[j] << i
      end
    end
    edge_boxes.each_index { |i| adjacency[i] ||= [] }

    visited = {}
    components = []
    adjacency.keys.sort.each do |start|
      next if visited[start]

      stack = [start]
      edge_ids = []
      until stack.empty?
        v = stack.pop
        next if visited[v]

        visited[v] = true
        edge_ids << v
        adjacency[v].each { |n| stack << n unless visited[n] }
      end

      bbox = edge_ids.map { |edge_id| edge_boxes[edge_id] }.reduce do |memo, edge_box|
        _cam_bbox_union(memo, edge_box)
      end
      area = _cam_bbox_area(bbox)
      next if area < min_area

      centroid_2d = _cam_bbox_center(bbox)
      components << {
        area: area,
        centroid: _cam_inflate_point(centroid_2d, plane, offset),
        bbox: bbox,
      }
    end

    components.sort_by { |c| [-c[:area], c[:centroid][0], c[:centroid][1], c[:centroid][2]] }
  end

  def _cam_unsupported_islands(part, layer_height: 0.2, axis: [0, 0, 1], min_area: 0.0, tolerance: 0.05)
    plane, offset_key, axis_index, axis_name = _cam_axis_spec(axis)

    unless layer_height.is_a?(Numeric) && layer_height > 0
      raise ArgumentError, "unsupported_islands layer_height must be > 0"
    end
    unless min_area.is_a?(Numeric) && min_area >= 0
      raise ArgumentError, "unsupported_islands min_area must be >= 0"
    end
    unless tolerance.is_a?(Numeric) && tolerance >= 0
      raise ArgumentError, "unsupported_islands tolerance must be >= 0"
    end

    bb = part.bounding_box
    min_coord = bb[axis_index == 0 ? :x : axis_index == 1 ? :y : :z]
    max_coord = min_coord + bb[axis_index == 0 ? :dx : axis_index == 1 ? :dy : :dz]

    report = []
    previous = []
    seen_any_layer = false
    offset = min_coord
    limit = max_coord + (layer_height * 1.0e-9)

    while offset <= limit
      section = part.slice(**{ plane: plane, offset_key => offset })
      current = _cam_component_reports(section, plane, offset, tolerance, min_area)

      if current.empty?
        previous = []
        offset += layer_height
        next
      end

      if seen_any_layer
        current.each do |component|
          component[:supported] = previous.any? do |prev|
            _cam_bbox_intersects?(component[:bbox], prev[:bbox], tolerance)
          end
        end
      else
        current.each { |component| component[:supported] = true }
      end

      report << {
        axis: axis_name,
        plane: plane,
        offset: offset,
        components: current,
        unsupported: current.select { |component| !component[:supported] },
      }

      previous = current
      seen_any_layer = true
      offset += layer_height
    end

    report
  end

  # unsupported_islands(part, layer_height: 0.2, axis: [0, 0, 1], min_area: 0.0,
  #                     tolerance: 0.05) — slice the part into layers and
  # report connected footprints that have no overlap with the previous layer.
  # Returns an Array of layer Hashes:
  #   { axis:, plane:, offset:, components:, unsupported: [...] }
  # where each unsupported component includes `:area`, `:centroid`, and `:bbox`.
  def unsupported_islands(part, layer_height: 0.2, axis: [0, 0, 1], min_area: 0.0, tolerance: 0.05)
    _cam_unsupported_islands(
      part,
      layer_height: layer_height,
      axis: axis,
      min_area: min_area,
      tolerance: tolerance,
    )
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

  def hardware_heuristic_dimension(size, label, scale)
    validate_positive_dimension(size, "#{label} size")
    size * scale
  end

  def nut_hex_profile(across_flats)
    validate_positive_dimension(across_flats, "nut across flats")
    radius = across_flats / Math.sqrt(3.0)
    points = 6.times.map do |i|
      angle = Math::PI / 6.0 + i * Math::PI / 3.0
      [radius * Math.cos(angle), radius * Math.sin(angle)]
    end
    polygon(points)
  end

  def nut_square_profile(across_flats)
    validate_positive_dimension(across_flats, "nut across flats")
    half = across_flats / 2.0
    polygon([
      [-half, -half],
      [half, -half],
      [half, half],
      [-half, half],
    ])
  end

  def nut_flange_diameter(across_flats)
    validate_positive_dimension(across_flats, "nut across flats")
    across_flats * 1.8
  end

  def nut_nyloc_collar_diameter(across_flats)
    validate_positive_dimension(across_flats, "nut across flats")
    across_flats * 1.15
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

  # washer(size, thickness:) — plain washer body for supported metric and
  # imperial fastener sizes.  Returns a centered ring solid with the washer
  # axis along +Z, sized for the same nominal screw families as the other
  # hardware helpers.
  def washer(size, thickness:)
    validate_positive_dimension(thickness, "washer thickness")

    outer_d = hardware_diameter(size, {
      "m2" => 4.0,
      "m2_5" => 5.0,
      "m25" => 5.0,
      "m3" => 7.0,
      "m4" => 9.0,
      "m5" => 10.0,
      "4_40" => 6.4,
      "6_32" => 9.5,
      "8_32" => 11.1,
      "10_32" => 14.3,
      "10_24" => 14.3,
      "1/4_20" => 18.6,
      "5/16_18" => 22.2,
      "3/8_16" => 25.4,
    }, "washer outer diameter")
    inner_d = hardware_diameter(size, {
      "m2" => 2.4,
      "m2_5" => 2.9,
      "m25" => 2.9,
      "m3" => 3.4,
      "m4" => 4.5,
      "m5" => 5.5,
      "4_40" => 2.95,
      "6_32" => 3.66,
      "8_32" => 4.32,
      "10_32" => 4.98,
      "10_24" => 4.98,
      "1/4_20" => 6.53,
      "5/16_18" => 8.20,
      "3/8_16" => 9.80,
    }, "washer inner diameter")

    if size.is_a?(Numeric)
      outer_d = hardware_heuristic_dimension(size, "washer", 2.5)
      inner_d = hardware_heuristic_dimension(size, "washer", 1.1)
    end

    ring = cylinder(outer_d / 2.0, thickness).cut(cylinder(inner_d / 2.0, thickness))
    ring.translate(0, 0, -thickness / 2.0)
  end

  # nut(size, thickness:, style: :hex) — nut body for supported metric and
  # imperial fastener families.  +style+ supports `:hex` and `:jam` (hex
  # profiles), `:square`, `:flange`, and `:nyloc`. Returns a centered solid
  # with a through clearance hole so it composes with the existing screw-body
  # helper in assemblies.
  def nut(size, thickness:, style: :hex)
    validate_positive_dimension(thickness, "nut thickness")

    across_flats = hardware_diameter(size, {
      "m2" => 4.0,
      "m2_5" => 5.0,
      "m25" => 5.0,
      "m3" => 5.5,
      "m4" => 7.0,
      "m5" => 8.0,
      "4_40" => 4.76,
      "6_32" => 7.94,
      "8_32" => 8.73,
      "10_32" => 9.53,
      "10_24" => 9.53,
      "1/4_20" => 11.11,
      "5/16_18" => 12.70,
      "3/8_16" => 14.29,
    }, "nut across flats")
    hole_d = hardware_diameter(size, {
      "m2" => 2.4,
      "m2_5" => 2.9,
      "m25" => 2.9,
      "m3" => 3.4,
      "m4" => 4.5,
      "m5" => 5.5,
      "4_40" => 2.95,
      "6_32" => 3.66,
      "8_32" => 4.32,
      "10_32" => 4.98,
      "10_24" => 4.98,
      "1/4_20" => 6.53,
      "5/16_18" => 8.20,
      "3/8_16" => 9.80,
    }, "nut hole diameter")

    if size.is_a?(Numeric)
      across_flats = hardware_heuristic_dimension(size, "nut", 1.7)
      hole_d = hardware_heuristic_dimension(size, "nut", 1.1)
    end

    body = case style
           when :hex, :jam
             nut_hex_profile(across_flats).extrude(thickness)
           when :square
             nut_square_profile(across_flats).extrude(thickness)
           when :flange
             flange_h = [thickness * 0.32, 1.0].max
             body_h = [thickness - flange_h, thickness * 0.4].max
             flange = cylinder(nut_flange_diameter(across_flats) / 2.0, flange_h)
             hex = nut_hex_profile(across_flats).extrude(body_h).translate(0, 0, flange_h)
             flange.fuse(hex)
           when :nyloc
             collar_h = [thickness * 0.35, 1.0].max
             body_h = [thickness - collar_h, thickness * 0.45].max
             hex = nut_hex_profile(across_flats).extrude(body_h)
             collar = cylinder(nut_nyloc_collar_diameter(across_flats) / 2.0, collar_h).translate(0, 0, body_h)
             hex.fuse(collar)
           else
             raise ArgumentError, "nut: unsupported style #{style.inspect}"
           end
    nut_body = body.cut(cylinder(hole_d / 2.0, thickness))
    nut_body.translate(0, 0, -thickness / 2.0)
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
    csink_h = (csink_d - d) / 2.0 / Math.tan(RRCADUnits.scalar(csink_angle) * Math::PI / 180.0)
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
  def sketch(diagnostics: false, strict: false, &block)
    builder = SketchBuilder.new
    result = if block.arity == 1
      block.call(builder)
    else
      builder.instance_eval(&block)
    end
    return result if result.is_a?(Shape)
    builder.to_profile(diagnostics: diagnostics, strict: strict)
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
      when Float then raw.to_f
      when RRCADUnits::UnitLength then RRCADUnits.length(raw.to_f)
      when RRCADUnits::UnitAngle then RRCADUnits.angle(raw.to_f)
      when TrueClass, FalseClass then raw == "true"
      else raw
      end
    elsif default.is_a?(RRCADUnits::UnitLength)
      RRCADUnits.length(raw)
    elsif default.is_a?(RRCADUnits::UnitAngle)
      RRCADUnits.angle(raw)
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
