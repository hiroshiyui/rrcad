#pragma once

#include <TopoDS_Shape.hxx>
#include <memory>
#include <rust/cxx.h>

// Forward-declare the OCCT loft builder in the global namespace (OCCT itself
// uses no namespace) so that ThruSectionsBuilder can store a unique_ptr to it
// without pulling in the full OCCT header here.
class BRepOffsetAPI_ThruSections;

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
    ~OcctShape() = default;

    OcctShape(const OcctShape&) = delete;
    OcctShape& operator=(const OcctShape&) = delete;
    OcctShape(OcctShape&&) = delete;
    OcctShape& operator=(OcctShape&&) = delete;

    const TopoDS_Shape& get() const noexcept { return shape_; }
    TopoDS_Shape& get() noexcept { return shape_; }

private:
    TopoDS_Shape shape_;
};

// Convenience factory used throughout bridge.cpp.
inline std::unique_ptr<OcctShape> wrap(TopoDS_Shape s) {
    return std::make_unique<OcctShape>(std::move(s));
}

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

// --- Fillets and chamfers (applied to all edges) ---
std::unique_ptr<OcctShape> shape_fillet(const OcctShape& shape, double radius);
std::unique_ptr<OcctShape> shape_chamfer(const OcctShape& shape, double dist);

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

// --- Phase 4: 3-D operations ---
// .shell(thickness) — hollow out a solid by removing the topmost face
//   (highest Z centroid) and offsetting all other faces inward by `thickness`.
//   Uses BRepOffsetAPI_MakeThickSolid::MakeThickSolidByJoin.
std::unique_ptr<OcctShape> shape_shell(const OcctShape& shape, double thickness);

// .offset(distance) — inflate (distance>0) or deflate (distance<0) a solid.
//   Uses BRepOffsetAPI_MakeOffsetShape::PerformByJoin.
std::unique_ptr<OcctShape> shape_offset(const OcctShape& shape, double distance);

// .extrude(h, twist_deg, scale) — extended extrude with end twist and scale.
//   Falls back to BRepPrimAPI_MakePrism when twist_deg≈0 and scale≈1.
//   Otherwise discretises the extrusion into sections via ThruSections.
std::unique_ptr<OcctShape> shape_extrude_ex(const OcctShape& shape, double height, double twist_deg,
                                            double scale);

// --- Phase 3: spline profiles and pipe sweep ---
std::unique_ptr<OcctShape> make_spline_2d(rust::Slice<const double> pts);
std::unique_ptr<OcctShape> make_spline_3d(rust::Slice<const double> pts);
std::unique_ptr<OcctShape> shape_sweep(const OcctShape& profile, const OcctShape& path);

// --- Phase 3: sub-shape selectors ---
// Face selectors: "all", "top", "bottom", "side"
// Edge selectors: "all", "vertical", "horizontal"
// Throws std::runtime_error for unknown selector strings.
int32_t shape_faces_count(const OcctShape& shape, rust::Str selector);
std::unique_ptr<OcctShape> shape_faces_get(const OcctShape& shape, rust::Str selector, int32_t idx);
int32_t shape_edges_count(const OcctShape& shape, rust::Str selector);
std::unique_ptr<OcctShape> shape_edges_get(const OcctShape& shape, rust::Str selector, int32_t idx);

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

} // namespace rrcad
