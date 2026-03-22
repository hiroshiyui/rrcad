#pragma once

#include <TopoDS_Shape.hxx>
#include <memory>
#include <rust/cxx.h>

// Forward-declare the OCCT loft builder in the global namespace (OCCT itself
// uses no namespace) so that ThruSectionsBuilder can store a unique_ptr to it
// without pulling in the full OCCT header here.
class BRepOffsetAPI_ThruSections;
class BRepOffsetAPI_MakePipeShell;

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

// --- Query / introspection ---
// Fills out[0..6] with [xmin, ymin, zmin, xmax, ymax, zmax].
void shape_bounding_box(const OcctShape& shape, rust::Slice<double> out);
double shape_volume(const OcctShape& shape);
double shape_surface_area(const OcctShape& shape);

// --- Export ---
void export_step(const OcctShape& shape, rust::Str path);
void export_stl(const OcctShape& shape, rust::Str path);
void export_gltf(const OcctShape& shape, rust::Str path, double linear_deflection);
// Binary glTF (GLB) — single-file format, suitable for HTTP serving.
void export_glb(const OcctShape& shape, rust::Str path, double linear_deflection);
// OBJ — Wavefront OBJ text format via RWObj_CafWriter (OCCT 7.6+).
void export_obj(const OcctShape& shape, rust::Str path, double linear_deflection);

} // namespace rrcad
