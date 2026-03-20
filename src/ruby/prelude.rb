# rrcad DSL prelude — loaded automatically into every interpreter session.
#
# This file is embedded in the binary (include_str!) and evaluated during
# MrubyVm::new().  Users never need to write `require` or `require_relative`.
#
# Phase 1 note: everything here is a pure-Ruby stub that raises
# NotImplementedError.  Once the native Rust/OCCT bindings are registered via
# mrb_define_class / mrb_define_method, those registrations happen *before*
# this prelude runs, so any method redefined here will take precedence only if
# no native version was registered.  Native methods should be registered for
# the same names and will shadow these stubs automatically.

# ---------------------------------------------------------------------------
# Shape — backing class for all solid geometry objects.
#
# In Phase 1 this will be a native class whose instances hold a u64 SlotMap
# key into the Rust-side OcctShape store.  For now it is a pure Ruby stub.
# ---------------------------------------------------------------------------
class Shape
  def initialize(kind, *args)
    @kind = kind
    @args = args
  end

  def to_s
    "#<Shape:#{@kind}(#{@args.map(&:inspect).join(', ')})>"
  end

  alias inspect to_s

  # --- export ---------------------------------------------------------------

  def export(_path)
    raise NotImplementedError, "Shape#export is not yet implemented (Phase 1)"
  end

  # --- boolean operations ---------------------------------------------------

  def fuse(_other)
    raise NotImplementedError, "Shape#fuse is not yet implemented (Phase 1)"
  end

  def cut(_other)
    raise NotImplementedError, "Shape#cut is not yet implemented (Phase 1)"
  end

  def common(_other)
    raise NotImplementedError, "Shape#common is not yet implemented (Phase 1)"
  end

  # --- transforms -----------------------------------------------------------

  def translate(_x, _y, _z)
    raise NotImplementedError, "Shape#translate is not yet implemented (Phase 1)"
  end

  def rotate(_ax, _ay, _az, _angle)
    raise NotImplementedError, "Shape#rotate is not yet implemented (Phase 1)"
  end

  def scale(_factor)
    raise NotImplementedError, "Shape#scale is not yet implemented (Phase 1)"
  end

  # --- modifiers ------------------------------------------------------------

  def fillet(_radius)
    raise NotImplementedError, "Shape#fillet is not yet implemented (Phase 2)"
  end

  def chamfer(_dist)
    raise NotImplementedError, "Shape#chamfer is not yet implemented (Phase 2)"
  end
end

# ---------------------------------------------------------------------------
# Top-level primitive constructors.
#
# These are defined as Kernel methods so they are available everywhere without
# a receiver — `box(10, 20, 30)` just works from any context.
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
