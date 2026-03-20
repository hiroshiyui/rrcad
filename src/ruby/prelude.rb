# rrcad DSL prelude — loaded automatically into every interpreter session.
#
# This file is embedded in the binary (include_str!) and evaluated during
# MrubyVm::new().  Users never need to write `require` or `require_relative`.
#
# After the prelude runs, MrubyVm::new() calls rrcad_register_shape_class()
# which registers native implementations for Shape, box(), cylinder(), and
# sphere().  Those native methods shadow the Ruby stubs below.

# ---------------------------------------------------------------------------
# Shape — backing class for all solid geometry objects.
#
# Native instances (created via box/cylinder/sphere) are mRuby RData objects
# holding a raw pointer to a heap-allocated Rust Shape.  The native `inspect`
# returns "#<Shape>".
#
# Stub instances (created via Shape.new for testing Phase 2/3 stub methods)
# carry @kind / @args as instance variables; their inspect uses the prelude
# definition below until the native override runs.
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

  # --- Methods not yet implemented natively (Phase 2) ----------------------

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

  # --- Stubs overridden by native after prelude runs -----------------------
  # (kept here only so that Shape.new(:box).export / .fuse etc. work from
  # tests that intentionally exercise the stub path before native registration)

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
end

# ---------------------------------------------------------------------------
# Top-level primitive constructors — stubs overridden by native after prelude.
# ---------------------------------------------------------------------------
module Kernel
  def box(_x, _y, _z)
    raise NotImplementedError, "box() is not yet implemented (Phase 1)"
  end

  def cylinder(_r, _h)
    raise NotImplementedError, "cylinder() is not yet implemented (Phase 1)"
  end

  def sphere(_r)
    raise NotImplementedError, "sphere() is not yet implemented (Phase 1)"
  end

  # `solid do ... end` builder — Phase 2.
  def solid
    raise NotImplementedError, "solid {} is not yet implemented (Phase 2)"
  end

  # Open the live browser preview for a shape — Phase 3.
  def preview(_shape)
    raise NotImplementedError, "preview() is not yet implemented (Phase 3)"
  end
end
