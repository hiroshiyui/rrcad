#pragma once

#include <TopoDS_Shape.hxx>
#include <memory>
#include <rust/cxx.h>

// Forward-declare OCCT builder types (global namespace — OCCT uses no namespace)
// so that builder classes can hold unique_ptr<T> without pulling in full headers.
class BRepOffsetAPI_ThruSections;
class BRepOffsetAPI_MakePipeShell;
class BRepBuilderAPI_Sewing;

namespace rrcad {

// OcctShape — opaque wrapper around a TopoDS_Shape, safe to cross the cxx bridge.
//
// Rules for cxx opaque types:
//   - Non-copyable and non-movable (cxx manages lifetime via unique_ptr).
//   - Heap-allocated only; always transferred across the bridge as unique_ptr<OcctShape>.
//
// TopoDS_Shape itself uses BRep handle-based reference counting, so constructing
// a new OcctShape from a TopoDS_Shape copy is cheap.
class OcctShape {
public:
    explicit OcctShape(TopoDS_Shape s) noexcept : shape_(std::move(s)) {}

    // Constructor with an sRGB color attached (r/g/b in [0, 1]).
    OcctShape(TopoDS_Shape s, float r, float g, float b) noexcept
        : shape_(std::move(s)), color_r_(r), color_g_(g), color_b_(b) {}

    ~OcctShape() = default;

    OcctShape(const OcctShape&) = delete;
    OcctShape& operator=(const OcctShape&) = delete;
    OcctShape(OcctShape&&) = delete;
    OcctShape& operator=(OcctShape&&) = delete;

    const TopoDS_Shape& get() const noexcept { return shape_; }
    TopoDS_Shape& get() noexcept { return shape_; }

    // Color accessors.  has_color() returns false when no color has been set.
    bool has_color() const noexcept { return color_r_ >= 0.0f; }
    float color_r() const noexcept { return color_r_; }
    float color_g() const noexcept { return color_g_; }
    float color_b() const noexcept { return color_b_; }

private:
    TopoDS_Shape shape_;
    // Negative sentinel means "no color set".
    float color_r_ = -1.0f;
    float color_g_ = -1.0f;
    float color_b_ = -1.0f;
};

// Convenience factories used throughout bridge.cpp.
inline std::unique_ptr<OcctShape> wrap(TopoDS_Shape s) {
    return std::make_unique<OcctShape>(std::move(s));
}
inline std::unique_ptr<OcctShape> wrap_colored(TopoDS_Shape s, float r, float g, float b) {
    return std::make_unique<OcctShape>(std::move(s), r, g, b);
}

// --- Color ---
// Return a copy of shape with an sRGB color tag (r/g/b each in [0, 1]).
// The color is written into the XDE document during GLB/glTF/OBJ export.
std::unique_ptr<OcctShape> shape_set_color(const OcctShape& shape, double r, double g, double b);

// --- Assembly mating ---
// Return a copy of `shape` rigidly transformed so that `from_face` (a planar
// face belonging to `shape`) lies flush against `to_face` (a planar face on
// the fixed reference geometry).
//
// The transform is computed as:
//   1. Rotation that maps from_face's outward normal → −(to_face's outward normal)
//      (antiparallel normals = contact, not overlap), pivoting around from_face's centroid.
//   2. Translation that moves from_face's centroid onto to_face's centroid.
//
// `offset` shifts the mated shape along to_face's outward normal:
//   offset > 0 → gap between the faces.
//   offset < 0 → intentional interference / overlap.
//
// Throws std::runtime_error if either face is non-planar.
std::unique_ptr<OcctShape> shape_mate(const OcctShape& shape, const OcctShape& from_face,
                                      const OcctShape& to_face, double offset);

// --- Primitives ---
std::unique_ptr<OcctShape> make_box(double dx, double dy, double dz);
std::unique_ptr<OcctShape> make_cylinder(double radius, double height);
std::unique_ptr<OcctShape> make_sphere(double radius);
std::unique_ptr<OcctShape> make_cone(double r1, double r2, double height);
std::unique_ptr<OcctShape> make_torus(double r1, double r2);
std::unique_ptr<OcctShape> make_wedge(double dx, double dy, double dz, double ltx);

// --- Boolean operations ---
std::unique_ptr<OcctShape> shape_fuse(const OcctShape& a, const OcctShape& b);
std::unique_ptr<OcctShape> shape_cut(const OcctShape& a, const OcctShape& b);
std::unique_ptr<OcctShape> shape_common(const OcctShape& a, const OcctShape& b);

// --- Fillets and chamfers ---
// All-edge forms (no selector).
std::unique_ptr<OcctShape> shape_fillet(const OcctShape& shape, double radius);
std::unique_ptr<OcctShape> shape_chamfer(const OcctShape& shape, double dist);
// Selective forms: only edges matching the edge selector (:all / :vertical / :horizontal).
std::unique_ptr<OcctShape> shape_fillet_sel(const OcctShape& shape, double radius,
                                            rust::Str selector);
std::unique_ptr<OcctShape> shape_chamfer_sel(const OcctShape& shape, double dist,
                                             rust::Str selector);
// Variable-radius fillet: radius transitions smoothly from r1 at one vertex
// of each edge to r2 at the other.  Uses BRepFilletAPI_MakeFillet::Add(r1,r2,edge).
// DSL spelling: .fillet(r1..r2) or .fillet(r1..r2, :selector)
std::unique_ptr<OcctShape> shape_fillet_var(const OcctShape& shape, double r1, double r2);
std::unique_ptr<OcctShape> shape_fillet_var_sel(const OcctShape& shape, double r1, double r2,
                                                rust::Str selector);
// Asymmetric chamfer: d1 and d2 are the two bevel distances on each side of the edge.
// DSL spelling: .chamfer(d1, d2) or .chamfer(d1, d2, :selector)
std::unique_ptr<OcctShape> shape_chamfer_asym(const OcctShape& shape, double d1, double d2);
std::unique_ptr<OcctShape> shape_chamfer_asym_sel(const OcctShape& shape, double d1, double d2,
                                                  rust::Str selector);

// --- Transforms (return new shapes; inputs are unchanged) ---
std::unique_ptr<OcctShape> shape_translate(const OcctShape& shape, double dx, double dy, double dz);
std::unique_ptr<OcctShape> shape_rotate(const OcctShape& shape, double axis_x, double axis_y,
                                        double axis_z, double angle_deg);
std::unique_ptr<OcctShape> shape_scale(const OcctShape& shape, double factor);
// Non-uniform scale: independent factors along each axis.
// Uses BRepBuilderAPI_GTransform + gp_GTrsf (general affine).
std::unique_ptr<OcctShape> shape_scale_xyz(const OcctShape& shape, double sx, double sy, double sz);

// --- Phase 2: mirror ---
std::unique_ptr<OcctShape> shape_mirror(const OcctShape& shape, rust::Str plane);

// --- Phase 2: 2D sketch faces ---
std::unique_ptr<OcctShape> make_rect(double w, double h);
std::unique_ptr<OcctShape> make_circle_face(double r);

// --- Phase 4: sketch profiles ---
std::unique_ptr<OcctShape> make_polygon(rust::Slice<const double> pts);
std::unique_ptr<OcctShape> make_ellipse_face(double rx, double ry);
std::unique_ptr<OcctShape> make_arc(double r, double start_deg, double end_deg);

// --- Phase 2: extrude / revolve ---
std::unique_ptr<OcctShape> shape_extrude(const OcctShape& shape, double height);
std::unique_ptr<OcctShape> shape_revolve(const OcctShape& shape, double angle_deg);

// --- Phase 4: ThruSections (loft) builder ---
//
// Opaque builder wrapping BRepOffsetAPI_ThruSections.  Create via
// thru_sections_new(), add profiles via thru_sections_add(), then call
// thru_sections_build() to get the finished shape.
//
// Rules identical to OcctShape: non-copyable, non-movable, heap-allocated,
// always transferred across the cxx bridge as unique_ptr<ThruSectionsBuilder>.
class ThruSectionsBuilder {
public:
    // solid=true → closed solid; ruled=true → ruled surface between sections.
    ThruSectionsBuilder(bool solid, bool ruled);
    // Destructor must be defined in bridge.cpp (where the full OCCT type is visible).
    ~ThruSectionsBuilder();

    ThruSectionsBuilder(const ThruSectionsBuilder&) = delete;
    ThruSectionsBuilder& operator=(const ThruSectionsBuilder&) = delete;
    ThruSectionsBuilder(ThruSectionsBuilder&&) = delete;
    ThruSectionsBuilder& operator=(ThruSectionsBuilder&&) = delete;

    // Use the global-namespace OCCT type (forward-declared above the rrcad namespace).
    std::unique_ptr<::BRepOffsetAPI_ThruSections> impl;
};

std::unique_ptr<ThruSectionsBuilder> thru_sections_new(bool solid, bool ruled);
// profile must be a Face, Wire, or Vertex (for a pointed cap/base)
void thru_sections_add(ThruSectionsBuilder& builder, const OcctShape& profile);
std::unique_ptr<OcctShape> thru_sections_build(ThruSectionsBuilder& builder);

// --- Phase 3: PipeShellBuilder (variable-section sweep) ---
//
// Builder for BRepOffsetAPI_MakePipeShell.  Create via pipe_shell_new(path),
// add section profiles via pipe_shell_add(), then call pipe_shell_build() to
// get the finished solid.
//
// The path must be a Wire (from spline_3d).  Each section must be a Face,
// Wire, or Vertex.  Sections are distributed along the spine at evenly-spaced
// parametric positions: first profile at t=0, last at t=1, others between.
//
// Rules: non-copyable, non-movable, heap-allocated, always transferred as
// unique_ptr<PipeShellBuilder>.
class PipeShellBuilder {
public:
    explicit PipeShellBuilder(const OcctShape& path);
    ~PipeShellBuilder(); // defined in bridge.cpp (Impl is an incomplete type here)

    PipeShellBuilder(const PipeShellBuilder&) = delete;
    PipeShellBuilder& operator=(const PipeShellBuilder&) = delete;
    PipeShellBuilder(PipeShellBuilder&&) = delete;
    PipeShellBuilder& operator=(PipeShellBuilder&&) = delete;

    struct Impl; // fully defined in bridge.cpp; holds OCCT types
    std::unique_ptr<Impl> impl;
};

std::unique_ptr<PipeShellBuilder> pipe_shell_new(const OcctShape& path);
// profile must be a Face (outer wire extracted), Wire, or Vertex.
// WithCorrection=true rotates each profile to be orthogonal to the spine tangent.
void pipe_shell_add(PipeShellBuilder& builder, const OcctShape& profile);
std::unique_ptr<OcctShape> pipe_shell_build(PipeShellBuilder& builder);

// --- Phase 4: 3-D operations ---
// .shell(thickness) — hollow out a solid by removing the topmost face
//   (highest Z centroid) and offsetting all other faces inward by `thickness`.
//   Uses BRepOffsetAPI_MakeThickSolid::MakeThickSolidByJoin.
std::unique_ptr<OcctShape> shape_shell(const OcctShape& shape, double thickness);

// .offset(distance) — inflate (distance>0) or deflate (distance<0) a solid.
//   Uses BRepOffsetAPI_MakeOffsetShape::PerformByJoin.
std::unique_ptr<OcctShape> shape_offset(const OcctShape& shape, double distance);

// .offset_2d(distance) — offset a 2D Face or Wire inward (negative) or outward (positive).
//   Uses BRepOffsetAPI_MakeOffset, which operates on Wire/Face shapes in their own plane.
std::unique_ptr<OcctShape> shape_offset_2d(const OcctShape& shape, double distance);

// .simplify(min_feature_size) — remove small holes and fillets.
//   Faces with surface area < min_feature_size² are passed to
//   BRepAlgoAPI_Defeaturing.  Returns the original shape if no faces qualify.
std::unique_ptr<OcctShape> shape_simplify(const OcctShape& shape, double min_feature_size);

// .extrude(h, twist_deg, scale) — extended extrude with end twist and scale.
//   Falls back to BRepPrimAPI_MakePrism when twist_deg≈0 and scale≈1.
//   Otherwise discretises the extrusion into sections via ThruSections.
std::unique_ptr<OcctShape> shape_extrude_ex(const OcctShape& shape, double height, double twist_deg,
                                            double scale);

// --- Phase 3: spline profiles and pipe sweep ---
std::unique_ptr<OcctShape> make_spline_2d(rust::Slice<const double> pts);
std::unique_ptr<OcctShape> make_spline_3d(rust::Slice<const double> pts);

// Tangent-constrained variants: explicit start/end tangent vectors suppress
// natural-boundary oscillation on short splines.
// Tangents for 2D live in the XZ plane: (t0x, t0z) and (t1x, t1z).
// Tangents for 3D are full 3-D vectors: (t0x,t0y,t0z) and (t1x,t1y,t1z).
// The vectors do not need to be unit-length; they are normalised internally.
std::unique_ptr<OcctShape> make_spline_2d_tan(rust::Slice<const double> pts, double t0x, double t0z,
                                              double t1x, double t1z);
std::unique_ptr<OcctShape> make_spline_3d_tan(rust::Slice<const double> pts, double t0x, double t0y,
                                              double t0z, double t1x, double t1y, double t1z);

std::unique_ptr<OcctShape> shape_sweep(const OcctShape& profile, const OcctShape& path);

// --- Phase 3: sub-shape selectors ---
// Face selectors: "all", "top", "bottom", "side"
//   Direction-based (Phase 4): ">X", "<X", ">Y", "<Y", ">Z", "<Z"
//   (">Z" = faces whose outward normal has a positive Z component > 0.5,
//    "<Z" = negative Z component < -0.5, and so on for X and Y axes.)
// Edge selectors: "all", "vertical", "horizontal"
// Vertex selector: "all"
// Throws std::runtime_error for unknown selector strings.
int32_t shape_faces_count(const OcctShape& shape, rust::Str selector);
std::unique_ptr<OcctShape> shape_faces_get(const OcctShape& shape, rust::Str selector, int32_t idx);
int32_t shape_edges_count(const OcctShape& shape, rust::Str selector);
std::unique_ptr<OcctShape> shape_edges_get(const OcctShape& shape, rust::Str selector, int32_t idx);
int32_t shape_vertices_count(const OcctShape& shape, rust::Str selector);
std::unique_ptr<OcctShape> shape_vertices_get(const OcctShape& shape, rust::Str selector,
                                              int32_t idx);

// --- Patterns ---
// Both functions return a TopoDS_Compound containing n translated/rotated copies
// of the input shape. Copies include i=0 (original position).
// linear_pattern: copy i is translated by i*[dx,dy,dz].
// polar_pattern:  copy i is rotated by i*(angle_deg/n) degrees around the Z axis.
std::unique_ptr<OcctShape> shape_linear_pattern(const OcctShape& shape, int32_t n, double dx,
                                                double dy, double dz);
std::unique_ptr<OcctShape> shape_polar_pattern(const OcctShape& shape, int32_t n, double angle_deg);

// --- Import ---
std::unique_ptr<OcctShape> import_step(rust::Str path);
std::unique_ptr<OcctShape> import_stl(rust::Str path);

// --- Bézier surface patch ---
//
// pts: flat array of 48 doubles — 16 control points (4×4 row-major grid) each
// given as (x, y, z).  Row 0 = first parameter direction; column 0 = second.
// Returns a Face from Geom_BezierSurface via BRepBuilderAPI_MakeFace.
// Throws std::runtime_error if pts.size() != 48 or MakeFace fails.
std::unique_ptr<OcctShape> make_bezier_patch(rust::Slice<const double> pts);

// --- Sewing builder ---
//
// Sews a collection of Faces/Shells into a closed Shell, then attempts to
// close it into a Solid.  If MakeSolid fails the Shell is returned as-is.
//
// Usage:
//   auto b = sewing_new(tolerance);
//   sewing_add(*b, face1);
//   sewing_add(*b, face2);
//   ...
//   auto solid = sewing_build(*b);
//
// Rules: non-copyable, non-movable, heap-allocated, always transferred as
// unique_ptr<SewingBuilder>.
class SewingBuilder {
public:
    explicit SewingBuilder(double tolerance);
    ~SewingBuilder(); // defined in bridge.cpp (Impl is incomplete here)

    SewingBuilder(const SewingBuilder&) = delete;
    SewingBuilder& operator=(const SewingBuilder&) = delete;
    SewingBuilder(SewingBuilder&&) = delete;
    SewingBuilder& operator=(SewingBuilder&&) = delete;

    struct Impl; // fully defined in bridge.cpp; holds BRepBuilderAPI_Sewing
    std::unique_ptr<Impl> impl;
};

std::unique_ptr<SewingBuilder> sewing_new(double tolerance);
void sewing_add(SewingBuilder& builder, const OcctShape& shape);
std::unique_ptr<OcctShape> sewing_build(SewingBuilder& builder);

// --- Query / introspection ---
// Fills out[0..6] with [xmin, ymin, zmin, xmax, ymax, zmax].
void shape_bounding_box(const OcctShape& shape, rust::Slice<double> out);
double shape_volume(const OcctShape& shape);
double shape_surface_area(const OcctShape& shape);

// --- Phase 7 Tier 2: Validation & introspection ---
// Returns the shape type as a lowercase string: "solid", "shell", "face",
// "wire", "edge", "vertex", "compound", or "other".
rust::String shape_type_str(const OcctShape& shape);
// Fills out[0..3] with [x, y, z] centroid of the shape.
// Uses VolumeProperties for solids, SurfaceProperties for shells/faces.
void shape_centroid(const OcctShape& shape, rust::Slice<double> out);
// True if the shape has no free (boundary) edges — i.e., every edge is
// shared by at least two faces.  Implies the shape is a closed manifold
// when combined with is_manifold.
bool shape_is_closed(const OcctShape& shape);
// True if every edge is shared by exactly two faces (no T-junctions or
// non-manifold edges).  A valid solid satisfies both is_closed and is_manifold.
bool shape_is_manifold(const OcctShape& shape);
// Returns "ok" if BRepCheck_Analyzer reports no errors, or a
// newline-separated list of BRepCheck_Status names otherwise.
rust::String shape_validate_str(const OcctShape& shape);

// --- Phase 8 Tier 1: Core Part Design ---

// Pad: extrude a sketch Face/Wire along face_ref's outward normal by height,
// then fuse the prism with body.
// face_ref must be a Face (typically obtained via body.faces(:top).first).
// sketch must be a Face or Wire in the XY plane at Z=0.
// The sketch is transformed onto face_ref before extrusion.
std::unique_ptr<OcctShape> shape_pad(const OcctShape& body, const OcctShape& face_ref,
                                     const OcctShape& sketch, double height);

// Pocket: extrude a sketch along -normal by depth and cut from body.
// Same constraints as shape_pad, but cuts instead of fuses.
std::unique_ptr<OcctShape> shape_pocket(const OcctShape& body, const OcctShape& face_ref,
                                        const OcctShape& sketch, double depth);

// Fillet Wire: round all corners of a 2D Wire or Face profile with radius.
// Uses BRepFilletAPI_MakeFillet2d.  Non-corner vertices are silently skipped.
std::unique_ptr<OcctShape> shape_fillet_wire(const OcctShape& profile, double radius);

// Datum Plane: construct a finite reference plane from origin, normal, and X direction.
// Returns a Face from a gp_Pln (±50 units in each direction) for use as a reference.
std::unique_ptr<OcctShape> make_datum_plane(double ox, double oy, double oz, double nx, double ny,
                                            double nz, double xx, double xy, double xz);

// --- Phase 8 Tier 3: Inspection & clearance ---

// Minimum distance between two shapes.  Returns 0 if the shapes overlap.
// Uses BRepExtrema_DistShapeShape with default deflection.
double shape_distance_to(const OcctShape& a, const OcctShape& b);

// Inertia tensor of a solid about its centre of mass.
// Fills out[0..6] with [Ixx, Iyy, Izz, Ixy, Ixz, Iyz] (in the world frame).
// Uses BRepGProp::VolumeProperties → GProp_GProps::MatrixOfInertia.
// Throws std::runtime_error for non-solid shapes.
void shape_inertia(const OcctShape& shape, rust::Slice<double> out);

// Minimum wall thickness of a solid: smallest distance between any two
// opposed faces.  Strategy: extract the outer shell, build an inward-offset
// shell, then use BRepExtrema_DistShapeShape between the two shells.
// Throws std::runtime_error if the shape is not a Solid or Shell, or if
// the offset fails.
double shape_min_thickness(const OcctShape& shape);

// --- Phase 8 Tier 2: Manufacturing features ---

// Draft angle extrude: straight prism then taper all lateral faces via
// BRepOffsetAPI_DraftAngle.  draft_deg > 0 tapers inward (standard mould
// taper — wider at base, narrower at top).  Neutral plane is Z=0.
std::unique_ptr<OcctShape> shape_extrude_draft(const OcctShape& profile, double height,
                                               double draft_deg);

// Helix path: Wire approximated by GeomAPI_Interpolate (32 samples/turn).
// radius: distance from Z axis.
// pitch:  axial rise per full revolution.
// height: total Z extent of the helix (= pitch × number of turns).
// Returns a Wire suitable for use as a sweep path.
std::unique_ptr<OcctShape> make_helix(double radius, double pitch, double height);

// --- Phase 7 Tier 3: Surface modeling ---

// Create a ruled surface (shell) between two wires using BRepFill::Shell.
// Both arguments must be Wires.
std::unique_ptr<OcctShape> shape_ruled_surface(const OcctShape& wire_a, const OcctShape& wire_b);
// Fill the interior of a closed boundary wire with a smooth NURBS surface.
// The wire's edges are added as C0 boundary constraints to BRepFill_Filling.
std::unique_ptr<OcctShape> shape_fill_surface(const OcctShape& boundary_wire);
// Cross-section of a shape by an axis-aligned plane.
// plane: "xy" (offset along Z), "xz" (offset along Y), "yz" (offset along X).
// Returns the section as a compound of edges / wires.
std::unique_ptr<OcctShape> shape_slice(const OcctShape& shape, rust::Str plane, double offset);

// --- Export ---
void export_step(const OcctShape& shape, rust::Str path);
void export_stl(const OcctShape& shape, rust::Str path);
void export_gltf(const OcctShape& shape, rust::Str path, double linear_deflection);
// Binary glTF (GLB) — single-file format, suitable for HTTP serving.
void export_glb(const OcctShape& shape, rust::Str path, double linear_deflection);
// OBJ — Wavefront OBJ text format via RWObj_CafWriter (OCCT 7.6+).
void export_obj(const OcctShape& shape, rust::Str path, double linear_deflection);

// --- Phase 8 Tier 4: 2-D drawing output ---
//
// export_svg / export_dxf project the shape's visible edges onto a flat drawing
// plane using HLRBRep_PolyAlgo (polygon-based hidden-line removal).
//
// `view` selects the projection direction:
//   "top"   — looking down the −Z axis; drawing plane is XY.
//   "front" — looking along the −Y axis; drawing plane is XZ.
//   "side"  — looking along the +X axis; drawing plane is YZ.
//
// SVG uses Y-down coordinates (standard for SVG/HTML).
// DXF uses Y-up coordinates (standard CAD convention).
void export_svg(const OcctShape& shape, rust::Str path, rust::Str view);
void export_dxf(const OcctShape& shape, rust::Str path, rust::Str view);

// --- Phase 8 Tier 5: Advanced composition ---

// fragment — General Boolean fragment: split all shapes at their mutual
// intersection boundaries and return a Compound of all non-overlapping pieces.
// Uses BRepAlgoAPI_BuilderAlgo (the "splitter" algorithm).
// Builder pattern: create → add shapes → build → get result.
//
// Rules: non-copyable, non-movable, heap-allocated, always transferred as
// unique_ptr<FragmentBuilder>.
class FragmentBuilder {
public:
    FragmentBuilder();
    ~FragmentBuilder(); // defined in bridge.cpp (Impl is incomplete here)

    FragmentBuilder(const FragmentBuilder&) = delete;
    FragmentBuilder& operator=(const FragmentBuilder&) = delete;
    FragmentBuilder(FragmentBuilder&&) = delete;
    FragmentBuilder& operator=(FragmentBuilder&&) = delete;

    struct Impl; // fully defined in bridge.cpp; holds TopTools_ListOfShape
    std::unique_ptr<Impl> impl;
};

std::unique_ptr<FragmentBuilder> fragment_new();
void fragment_add(FragmentBuilder& builder, const OcctShape& shape);
std::unique_ptr<OcctShape> fragment_build(FragmentBuilder& builder);

// convex_hull — 3-D convex hull of the shape's tessellated mesh vertices.
// Tessellates the shape with BRepMesh_IncrementalMesh, collects all triangle
// mesh vertices (applying TopLoc_Location transforms), runs an incremental
// QuickHull algorithm, then sews the hull triangles into a closed solid via
// BRepBuilderAPI_Sewing + BRepBuilderAPI_MakeSolid.
// Throws std::runtime_error if fewer than 4 non-coplanar points are found.
std::unique_ptr<OcctShape> shape_convex_hull(const OcctShape& shape);

// path_pattern — Distribute n evenly-spaced (arc-length) copies of `shape`
// along `path` (a Wire or Edge), orienting each copy so its local Z-axis
// aligns with the path tangent at that point.
// Uses GCPnts_UniformAbscissa on BRepAdaptor_CompCurve for arc-length spacing.
// Returns a TopoDS_Compound of n copies.
std::unique_ptr<OcctShape> shape_path_pattern(const OcctShape& shape, const OcctShape& path,
                                              int32_t n);

// sweep_guide — Guided sweep: sweep `profile` along `path` (the main spine Wire)
// while keeping the profile's orientation controlled by `guide` (an auxiliary
// spine Wire, i.e. a guide rail).
// Uses BRepOffsetAPI_MakePipeShell::SetMode(auxiliary_spine, …) with
// BRepFill_Contact so the profile tracks the guide rail at all points.
// Both `path` and `guide` must be Wires.  `profile` may be a Wire or Face.
std::unique_ptr<OcctShape> shape_sweep_guide(const OcctShape& profile, const OcctShape& path,
                                             const OcctShape& guide);

} // namespace rrcad
