#include "bridge.h"

// --- OCCT: geometry ---
#include <gp_Ax1.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>

// --- OCCT: topology ---
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>

// --- OCCT: primitives ---
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRepPrimAPI_MakeWedge.hxx>

// --- OCCT: boolean ops ---
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>

// --- OCCT: fillets and chamfers ---
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>

// --- OCCT: transforms ---
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <gp_GTrsf.hxx>
#include <gp_Mat.hxx>

// --- OCCT: Phase 2 ---
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Pln.hxx>

// --- OCCT: Phase 5 — assembly mating ---
#include <Geom_Plane.hxx>

// --- OCCT: Phase 4 sketch profiles ---
#include <GC_MakeArcOfCircle.hxx>
#include <GC_MakeEllipse.hxx>

// --- OCCT: Phase 3 — splines and pipe sweep ---
#include <BRepOffsetAPI_MakePipe.hxx>
#include <GeomAPI_Interpolate.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TColgp_HArray1OfPnt.hxx>

// --- OCCT: Phase 3 — sub-shape selectors ---
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepLProp_SLProps.hxx>
#include <BRep_Tool.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>

// --- OCCT: tessellation (required before glTF export) ---
#include <BRepMesh_IncrementalMesh.hxx>

// --- OCCT: shape validity check ---
#include <BRepCheck_Analyzer.hxx>

// --- OCCT: Phase 4 — query / introspection ---
#include <BRepBndLib.hxx>
#include <BRepGProp.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>

// --- OCCT: Phase 4 — 3-D operations ---
#include <BRepAlgoAPI_Defeaturing.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepOffsetAPI_MakeThickSolid.hxx>
#include <BRepOffsetAPI_ThruSections.hxx>
#include <BRepTools.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS_Wire.hxx>
#include <cmath>
#include <limits>

// --- OCCT: STEP import / export ---
#include <IFSelect_ReturnStatus.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_StepModelType.hxx>
#include <STEPControl_Writer.hxx>

// --- OCCT: STL import / export ---
#include <BRep_Builder.hxx>
#include <RWStl.hxx>
#include <StlAPI_Writer.hxx>

// --- OCCT: glTF / OBJ export (XDE pipeline) ---
#include <Message_ProgressRange.hxx>
#include <Quantity_Color.hxx>
#include <RWGltf_CafWriter.hxx>
#include <RWObj_CafWriter.hxx>
#include <TColStd_IndexedDataMapOfStringString.hxx>
#include <TCollection_AsciiString.hxx>
#include <TCollection_ExtendedString.hxx>
#include <TDF_Label.hxx>
#include <TDocStd_Document.hxx>
#include <XCAFApp_Application.hxx>
#include <XCAFDoc_ColorTool.hxx>
#include <XCAFDoc_ColorType.hxx>
#include <XCAFDoc_DocumentTool.hxx>
#include <XCAFDoc_ShapeTool.hxx>

#include <cmath>
#include <stdexcept>
#include <string>

namespace rrcad {

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> make_box(double dx, double dy, double dz) {
    BRepPrimAPI_MakeBox builder(dx, dy, dz);
    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepPrimAPI_MakeBox failed");
    return wrap(builder.Shape());
}

std::unique_ptr<OcctShape> make_cylinder(double radius, double height) {
    BRepPrimAPI_MakeCylinder builder(radius, height);
    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepPrimAPI_MakeCylinder failed");
    return wrap(builder.Shape());
}

std::unique_ptr<OcctShape> make_sphere(double radius) {
    BRepPrimAPI_MakeSphere builder(radius);
    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepPrimAPI_MakeSphere failed");
    return wrap(builder.Shape());
}

std::unique_ptr<OcctShape> make_cone(double r1, double r2, double height) {
    BRepPrimAPI_MakeCone builder(r1, r2, height);
    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepPrimAPI_MakeCone failed");
    return wrap(builder.Shape());
}

std::unique_ptr<OcctShape> make_torus(double r1, double r2) {
    BRepPrimAPI_MakeTorus builder(r1, r2);
    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepPrimAPI_MakeTorus failed");
    return wrap(builder.Shape());
}

std::unique_ptr<OcctShape> make_wedge(double dx, double dy, double dz, double ltx) {
    BRepPrimAPI_MakeWedge builder(dx, dy, dz, ltx);
    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepPrimAPI_MakeWedge failed");
    return wrap(builder.Shape());
}

// ---------------------------------------------------------------------------
// Color

std::unique_ptr<OcctShape> shape_set_color(const OcctShape& shape, double r, double g, double b) {
    // Return a new OcctShape wrapping the same BRep topology (cheap — TopoDS
    // uses handle-based reference counting) with the sRGB color tag attached.
    return wrap_colored(shape.get(), static_cast<float>(r), static_cast<float>(g),
                        static_cast<float>(b));
}

// ---------------------------------------------------------------------------
// Assembly mating
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_mate(const OcctShape& shape, const OcctShape& from_face_shape,
                                      const OcctShape& to_face_shape, double offset) {
    TopoDS_Face from_face = TopoDS::Face(from_face_shape.get());
    TopoDS_Face to_face = TopoDS::Face(to_face_shape.get());

    Handle(Geom_Surface) from_surf = BRep_Tool::Surface(from_face);
    Handle(Geom_Surface) to_surf = BRep_Tool::Surface(to_face);

    Handle(Geom_Plane) from_plane = Handle(Geom_Plane)::DownCast(from_surf);
    Handle(Geom_Plane) to_plane = Handle(Geom_Plane)::DownCast(to_surf);

    if (from_plane.IsNull())
        throw std::runtime_error("mate: 'from' face must be planar (non-planar surface given)");
    if (to_plane.IsNull())
        throw std::runtime_error("mate: 'to' face must be planar (non-planar surface given)");

    // Outward normals — honour TopoDS_Face orientation (Forward vs Reversed).
    gp_Dir n_from = from_plane->Axis().Direction();
    if (from_face.Orientation() == TopAbs_REVERSED)
        n_from.Reverse();

    gp_Dir n_to = to_plane->Axis().Direction();
    if (to_face.Orientation() == TopAbs_REVERSED)
        n_to.Reverse();

    // Use face centroids as reference points so the mate aligns face centres.
    GProp_GProps from_props, to_props;
    BRepGProp::SurfaceProperties(from_face, from_props);
    BRepGProp::SurfaceProperties(to_face, to_props);
    gp_Pnt p_from = from_props.CentreOfMass();
    gp_Pnt p_to = to_props.CentreOfMass();

    // Positive offset shifts the target point along n_to (away from the surface).
    if (offset != 0.0)
        p_to.Translate(gp_Vec(n_to).Multiplied(offset));

    // For contact, from-face normal must be antiparallel to to-face normal.
    gp_Dir target_normal = n_to.Reversed();

    // -----------------------------------------------------------------------
    // Step 1: rotation that maps n_from → target_normal, pivoting at p_from
    // so that p_from is stationary after the rotation.
    // -----------------------------------------------------------------------
    gp_Trsf rot;
    gp_Vec v_from(n_from), v_target(target_normal);
    double dot = v_from.Dot(v_target);

    if (std::abs(dot - 1.0) < 1e-7) {
        // n_from already equals target_normal — identity rotation (gp_Trsf default).
    } else if (std::abs(dot + 1.0) < 1e-7) {
        // n_from and target_normal are antiparallel: 180° around any perpendicular axis.
        gp_Vec perp = v_from.Crossed(gp_Vec(1.0, 0.0, 0.0));
        if (perp.Magnitude() < 1e-7)
            perp = v_from.Crossed(gp_Vec(0.0, 1.0, 0.0));
        rot.SetRotation(gp_Ax1(p_from, gp_Dir(perp)), M_PI);
    } else {
        gp_Vec axis = v_from.Crossed(v_target);
        double angle = v_from.Angle(v_target);
        rot.SetRotation(gp_Ax1(p_from, gp_Dir(axis)), angle);
    }

    // -----------------------------------------------------------------------
    // Step 2: translation that moves p_from (which is on the rotation axis,
    // so it didn't move in step 1) to p_to.
    // -----------------------------------------------------------------------
    gp_Trsf trans;
    trans.SetTranslation(gp_Vec(p_from, p_to));

    // Combined: rotate first (around p_from), then translate.
    // Multiply semantics: combined = trans * rot  →  rot applied first.
    gp_Trsf combined = trans;
    combined.Multiply(rot);

    BRepBuilderAPI_Transform transformer(shape.get(), combined, /*copy=*/Standard_True);
    if (!transformer.IsDone())
        throw std::runtime_error("mate: BRepBuilderAPI_Transform failed");
    return wrap(transformer.Shape());
}

// ---------------------------------------------------------------------------
// Boolean operations
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_fuse(const OcctShape& a, const OcctShape& b) {
    // Builder-style API: explicit args/tools, fuzzy tolerance for near-coincident
    // faces, and TBB parallel evaluation (OCCT 7.4+).
    TopTools_ListOfShape args, tools;
    args.Append(a.get());
    tools.Append(b.get());
    BRepAlgoAPI_Fuse op;
    op.SetArguments(args);
    op.SetTools(tools);
    op.SetRunParallel(Standard_True);
    op.SetFuzzyValue(1e-6);
    op.Build();
    if (!op.IsDone())
        throw std::runtime_error("BRepAlgoAPI_Fuse failed");
    return wrap(op.Shape());
}

std::unique_ptr<OcctShape> shape_cut(const OcctShape& a, const OcctShape& b) {
    TopTools_ListOfShape args, tools;
    args.Append(a.get());
    tools.Append(b.get());
    BRepAlgoAPI_Cut op;
    op.SetArguments(args);
    op.SetTools(tools);
    op.SetRunParallel(Standard_True);
    op.SetFuzzyValue(1e-6);
    op.Build();
    if (!op.IsDone())
        throw std::runtime_error("BRepAlgoAPI_Cut failed");
    return wrap(op.Shape());
}

std::unique_ptr<OcctShape> shape_common(const OcctShape& a, const OcctShape& b) {
    TopTools_ListOfShape args, tools;
    args.Append(a.get());
    tools.Append(b.get());
    BRepAlgoAPI_Common op;
    op.SetArguments(args);
    op.SetTools(tools);
    op.SetRunParallel(Standard_True);
    op.SetFuzzyValue(1e-6);
    op.Build();
    if (!op.IsDone())
        throw std::runtime_error("BRepAlgoAPI_Common failed");
    return wrap(op.Shape());
}

// ---------------------------------------------------------------------------
// Fillets and chamfers
// ---------------------------------------------------------------------------

// Forward declaration — collect_edges is defined in the selectors section below.
static std::vector<TopoDS_Edge> collect_edges(const OcctShape& shape, const std::string& sel);

std::unique_ptr<OcctShape> shape_fillet(const OcctShape& s, double radius) {
    BRepFilletAPI_MakeFillet builder(s.get());

    // Add every edge to the fillet operation.
    TopExp_Explorer exp(s.get(), TopAbs_EDGE);
    for (; exp.More(); exp.Next()) {
        builder.Add(radius, TopoDS::Edge(exp.Current()));
    }

    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepFilletAPI_MakeFillet failed — "
                                 "check for degenerate edges or zero-length edges");
    return wrap(builder.Shape());
}

std::unique_ptr<OcctShape> shape_chamfer(const OcctShape& s, double dist) {
    BRepFilletAPI_MakeChamfer builder(s.get());

    TopExp_Explorer exp(s.get(), TopAbs_EDGE);
    for (; exp.More(); exp.Next()) {
        builder.Add(dist, TopoDS::Edge(exp.Current()));
    }

    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepFilletAPI_MakeChamfer failed");
    return wrap(builder.Shape());
}

// Selective fillet: only edges matching the edge selector are rounded.
// Reuses collect_edges for deduplication and validation.
std::unique_ptr<OcctShape> shape_fillet_sel(const OcctShape& s, double radius, rust::Str selector) {
    std::string sel(selector.data(), selector.size());
    auto edges = collect_edges(s, sel);
    if (edges.empty())
        throw std::runtime_error("fillet: no edges match selector ':" + sel + "'");

    BRepFilletAPI_MakeFillet builder(s.get());
    for (const auto& edge : edges)
        builder.Add(radius, edge);

    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepFilletAPI_MakeFillet (selective) failed — "
                                 "check for degenerate edges or too-large radius");
    return wrap(builder.Shape());
}

// Selective chamfer: only edges matching the edge selector are bevelled.
std::unique_ptr<OcctShape> shape_chamfer_sel(const OcctShape& s, double dist, rust::Str selector) {
    std::string sel(selector.data(), selector.size());
    auto edges = collect_edges(s, sel);
    if (edges.empty())
        throw std::runtime_error("chamfer: no edges match selector ':" + sel + "'");

    BRepFilletAPI_MakeChamfer builder(s.get());
    for (const auto& edge : edges)
        builder.Add(dist, edge);

    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepFilletAPI_MakeChamfer (selective) failed");
    return wrap(builder.Shape());
}

// ---------------------------------------------------------------------------
// Transforms
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_translate(const OcctShape& s, double dx, double dy, double dz) {
    gp_Trsf trsf;
    trsf.SetTranslation(gp_Vec(dx, dy, dz));
    BRepBuilderAPI_Transform xform(s.get(), trsf, /*copy=*/Standard_True);
    xform.Build();
    if (!xform.IsDone())
        throw std::runtime_error("BRepBuilderAPI_Transform (translate) failed");
    return wrap(xform.Shape());
}

std::unique_ptr<OcctShape> shape_rotate(const OcctShape& s, double axis_x, double axis_y,
                                        double axis_z, double angle_deg) {
    // gp_Dir normalizes automatically; throws Standard_ConstructionError on zero vector.
    gp_Dir dir(axis_x, axis_y, axis_z);
    gp_Ax1 axis(gp_Pnt(0.0, 0.0, 0.0), dir);
    const double angle_rad = angle_deg * (M_PI / 180.0);

    gp_Trsf trsf;
    trsf.SetRotation(axis, angle_rad);
    BRepBuilderAPI_Transform xform(s.get(), trsf, /*copy=*/Standard_True);
    xform.Build();
    if (!xform.IsDone())
        throw std::runtime_error("BRepBuilderAPI_Transform (rotate) failed");
    return wrap(xform.Shape());
}

std::unique_ptr<OcctShape> shape_scale(const OcctShape& s, double factor) {
    gp_Trsf trsf;
    trsf.SetScaleFactor(factor);
    BRepBuilderAPI_Transform xform(s.get(), trsf, /*copy=*/Standard_True);
    xform.Build();
    if (!xform.IsDone())
        throw std::runtime_error("BRepBuilderAPI_Transform (scale) failed");
    return wrap(xform.Shape());
}

// Non-uniform scale — independent factors for X, Y, Z.
//
// gp_Trsf only supports uniform scale (single scalar), so we use gp_GTrsf
// (general affine transform) with a diagonal 3x3 matrix.
// BRepBuilderAPI_GTransform may approximate curved edges; the result is still
// topologically valid and suitable for all downstream operations.
std::unique_ptr<OcctShape> shape_scale_xyz(const OcctShape& s, double sx, double sy, double sz) {
    // Build a diagonal 3×3 matrix: diag(sx, sy, sz).
    // gp_Mat(row0col0, row0col1, row0col2, row1col0, ...)
    gp_GTrsf gtrsf;
    gtrsf.SetVectorialPart(gp_Mat(sx, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, sz));
    // Translation part stays zero (SetTranslationPart not called).
    BRepBuilderAPI_GTransform xform(s.get(), gtrsf, /*copy=*/Standard_True);
    if (!xform.IsDone())
        throw std::runtime_error("BRepBuilderAPI_GTransform (scale_xyz) failed");
    return wrap(xform.Shape());
}

// ---------------------------------------------------------------------------
// Phase 2: Mirror
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_mirror(const OcctShape& s, rust::Str plane) {
    gp_Trsf trsf;
    if (plane == "xy") {
        trsf.SetMirror(gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)));
    } else if (plane == "xz") {
        trsf.SetMirror(gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 1.0, 0.0)));
    } else if (plane == "yz") {
        trsf.SetMirror(gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(1.0, 0.0, 0.0)));
    } else {
        std::string msg = "mirror: unknown plane '";
        msg += std::string(plane.data(), plane.size());
        msg += "' — expected :xy, :xz, or :yz";
        throw std::runtime_error(msg);
    }
    BRepBuilderAPI_Transform xform(s.get(), trsf, /*copy=*/Standard_True);
    xform.Build();
    if (!xform.IsDone())
        throw std::runtime_error("BRepBuilderAPI_Transform (mirror) failed");
    return wrap(xform.Shape());
}

// ---------------------------------------------------------------------------
// Phase 2: 2D sketch faces
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> make_rect(double w, double h) {
    BRepBuilderAPI_MakePolygon poly;
    poly.Add(gp_Pnt(0.0, 0.0, 0.0));
    poly.Add(gp_Pnt(w, 0.0, 0.0));
    poly.Add(gp_Pnt(w, h, 0.0));
    poly.Add(gp_Pnt(0.0, h, 0.0));
    poly.Close();
    if (!poly.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakePolygon (rect) failed");
    BRepBuilderAPI_MakeFace face(poly.Wire());
    if (!face.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakeFace (rect) failed");
    return wrap(face.Face());
}

std::unique_ptr<OcctShape> make_circle_face(double r) {
    gp_Circ circ(gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), r);
    TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(circ).Edge();
    TopoDS_Wire wire = BRepBuilderAPI_MakeWire(edge).Wire();
    BRepBuilderAPI_MakeFace face(wire);
    if (!face.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakeFace (circle) failed");
    return wrap(face.Face());
}

// ---------------------------------------------------------------------------
// Phase 4: Sketch profiles — polygon, ellipse, arc
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> make_polygon(rust::Slice<const double> pts) {
    int n = (int)(pts.size() / 2);
    if (n < 3)
        throw std::runtime_error("polygon requires at least 3 points");
    BRepBuilderAPI_MakePolygon poly;
    for (int i = 0; i < n; i++)
        poly.Add(gp_Pnt(pts[(size_t)(i * 2)], pts[(size_t)(i * 2 + 1)], 0.0));
    poly.Close();
    if (!poly.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakePolygon failed");
    BRepBuilderAPI_MakeFace face(poly.Wire());
    if (!face.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakeFace (polygon) failed");
    return wrap(face.Face());
}

std::unique_ptr<OcctShape> make_ellipse_face(double rx, double ry) {
    // OCCT requires major radius >= minor radius for GC_MakeEllipse.
    if (rx < ry)
        std::swap(rx, ry);
    gp_Ax2 axes(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0), gp_Dir(1.0, 0.0, 0.0));
    GC_MakeEllipse ellipse_maker(axes, rx, ry);
    if (!ellipse_maker.IsDone())
        throw std::runtime_error("GC_MakeEllipse failed");
    TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(ellipse_maker.Value()).Edge();
    TopoDS_Wire wire = BRepBuilderAPI_MakeWire(edge).Wire();
    BRepBuilderAPI_MakeFace face(wire);
    if (!face.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakeFace (ellipse) failed");
    return wrap(face.Face());
}

std::unique_ptr<OcctShape> make_arc(double r, double start_deg, double end_deg) {
    gp_Ax2 axes(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0), gp_Dir(1.0, 0.0, 0.0));
    gp_Circ circ(axes, r);
    double start_rad = start_deg * M_PI / 180.0;
    double end_rad = end_deg * M_PI / 180.0;
    GC_MakeArcOfCircle arc_maker(circ, start_rad, end_rad, Standard_True);
    if (!arc_maker.IsDone())
        throw std::runtime_error("GC_MakeArcOfCircle failed");
    TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(arc_maker.Value()).Edge();
    BRepBuilderAPI_MakeWire wire_maker(edge);
    if (!wire_maker.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakeWire (arc) failed");
    return wrap(wire_maker.Wire());
}

// ---------------------------------------------------------------------------
// Phase 2: Extrude / Revolve
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_extrude(const OcctShape& s, double height) {
    BRepPrimAPI_MakePrism prism(s.get(), gp_Vec(0.0, 0.0, height));
    prism.Build();
    if (!prism.IsDone())
        throw std::runtime_error("BRepPrimAPI_MakePrism (extrude) failed");
    return wrap(prism.Shape());
}

std::unique_ptr<OcctShape> shape_revolve(const OcctShape& s, double angle_deg) {
    gp_Ax1 axis(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0));
    const double angle_rad = angle_deg * (M_PI / 180.0);
    BRepPrimAPI_MakeRevol revol(s.get(), axis, angle_rad);
    revol.Build();
    if (!revol.IsDone())
        throw std::runtime_error("BRepPrimAPI_MakeRevol (revolve) failed");
    return wrap(revol.Shape());
}

// ---------------------------------------------------------------------------
// Phase 4: ThruSections (loft) builder
// ---------------------------------------------------------------------------

ThruSectionsBuilder::ThruSectionsBuilder(bool solid, bool ruled)
    : impl(std::make_unique<BRepOffsetAPI_ThruSections>(solid, ruled)) {}

ThruSectionsBuilder::~ThruSectionsBuilder() = default;

std::unique_ptr<ThruSectionsBuilder> thru_sections_new(bool solid, bool ruled) {
    return std::make_unique<ThruSectionsBuilder>(solid, ruled);
}

void thru_sections_add(ThruSectionsBuilder& b, const OcctShape& profile) {
    const TopoDS_Shape& s = profile.get();
    if (s.ShapeType() == TopAbs_FACE) {
        // Extract the outer wire from the face so ThruSections can work with it.
        TopoDS_Wire wire = BRepTools::OuterWire(TopoDS::Face(s));
        b.impl->AddWire(wire);
    } else if (s.ShapeType() == TopAbs_WIRE) {
        b.impl->AddWire(TopoDS::Wire(s));
    } else if (s.ShapeType() == TopAbs_VERTEX) {
        b.impl->AddVertex(TopoDS::Vertex(s));
    } else {
        throw std::runtime_error("loft: each profile must be a Face, Wire, or Vertex");
    }
}

std::unique_ptr<OcctShape> thru_sections_build(ThruSectionsBuilder& b) {
    b.impl->Build();
    if (!b.impl->IsDone())
        throw std::runtime_error("BRepOffsetAPI_ThruSections (loft) failed");
    return wrap(b.impl->Shape());
}

// ---------------------------------------------------------------------------
// Phase 4: .shell(thickness) — hollow out a solid
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_shell(const OcctShape& shape, double thickness) {
    // Select the face with the highest Z centroid as the opening (top face).
    TopoDS_Face top_face;
    double max_z = -std::numeric_limits<double>::max();
    bool found = false;

    for (TopExp_Explorer exp(shape.get(), TopAbs_FACE); exp.More(); exp.Next()) {
        const TopoDS_Face& face = TopoDS::Face(exp.Current());
        GProp_GProps props;
        BRepGProp::SurfaceProperties(face, props);
        double z = props.CentreOfMass().Z();
        if (z > max_z) {
            max_z = z;
            top_face = face;
            found = true;
        }
    }

    if (!found)
        throw std::runtime_error("shell: shape has no faces");

    TopTools_ListOfShape faces_to_remove;
    faces_to_remove.Append(top_face);

    // Negative offset moves surfaces inward, creating a wall of `thickness`.
    BRepOffsetAPI_MakeThickSolid thick;
    thick.MakeThickSolidByJoin(shape.get(), faces_to_remove, -thickness, 1e-3);
    if (!thick.IsDone())
        throw std::runtime_error("BRepOffsetAPI_MakeThickSolid (shell) failed");
    return wrap(thick.Shape());
}

// ---------------------------------------------------------------------------
// Phase 4: .offset(distance) — inflate / deflate a solid
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_offset(const OcctShape& shape, double distance) {
    BRepOffsetAPI_MakeOffsetShape offsetter;
    offsetter.PerformByJoin(shape.get(), distance, 1e-3);
    if (!offsetter.IsDone())
        throw std::runtime_error("BRepOffsetAPI_MakeOffsetShape (offset) failed");
    return wrap(offsetter.Shape());
}

// ---------------------------------------------------------------------------
// Phase 4 / Tier 4: .simplify(min_feature_size)
//
// Removes small holes and fillets from a solid using BRepAlgoAPI_Defeaturing.
// Faces with surface area smaller than min_feature_size² are treated as
// belonging to small features and are passed to AddFaceToRemove.
//
// If no faces are below the threshold the original shape is returned unchanged.
// If the algorithm fails after selecting faces a std::runtime_error is thrown.
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_simplify(const OcctShape& shape, double min_feature_size) {
    double area_threshold = min_feature_size * min_feature_size;

    // Collect faces smaller than the area threshold.
    TopTools_ListOfShape small_faces;
    for (TopExp_Explorer ex(shape.get(), TopAbs_FACE); ex.More(); ex.Next()) {
        GProp_GProps face_props;
        BRepGProp::SurfaceProperties(ex.Current(), face_props);
        if (face_props.Mass() < area_threshold)
            small_faces.Append(ex.Current());
    }

    // If nothing qualifies, return a copy of the original unchanged.
    if (small_faces.IsEmpty())
        return wrap(shape.get());

    BRepAlgoAPI_Defeaturing df;
    df.SetShape(shape.get());
    df.AddFacesToRemove(small_faces);
    df.SetRunParallel(Standard_True);
    df.SetToFillHistory(Standard_False);
    df.Build();
    if (!df.IsDone())
        throw std::runtime_error("BRepAlgoAPI_Defeaturing (simplify) failed");
    return wrap(df.Shape());
}

// ---------------------------------------------------------------------------
// Phase 4: .extrude(h, twist_deg, scale) — extended extrusion
//
// When twist_deg≈0 and scale≈1 the fast path (MakePrism) is used.  Otherwise
// the extrusion is discretised into N cross-sections (proportional to the
// twist angle) and lofted through them via BRepOffsetAPI_ThruSections.
// Each section is the original profile scaled by lerp(1,scale,t) and rotated
// by lerp(0,twist_deg,t) around Z, then translated to z = t*height.
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_extrude_ex(const OcctShape& profile, double height,
                                            double twist_deg, double scale) {
    // Fast path: delegate to existing MakePrism implementation.
    if (std::abs(twist_deg) < 1e-10 && std::abs(scale - 1.0) < 1e-10)
        return shape_extrude(profile, height);

    // Number of sections: more sections for larger twist angles, minimum 4.
    const int N = std::max(4, static_cast<int>(std::abs(twist_deg) / 5.0) + 2);

    // isSolid=true produces a closed solid; isRuled=false gives smooth loft.
    BRepOffsetAPI_ThruSections loft(/*isSolid=*/Standard_True, /*isRuled=*/Standard_False);

    for (int i = 0; i < N; i++) {
        double t = static_cast<double>(i) / static_cast<double>(N - 1);
        double z = t * height;
        double rot_rad = t * twist_deg * (M_PI / 180.0);
        double s = 1.0 + t * (scale - 1.0); // linear scale interpolation

        // Build combined transform: scale → rotate around Z → translate to z.
        // In OCCT, T1.Multiply(T2) means T1 = T1 * T2 (T2 applied first).
        gp_Trsf trsf_translate;
        trsf_translate.SetTranslation(gp_Vec(0.0, 0.0, z));
        gp_Trsf trsf_rotate;
        trsf_rotate.SetRotation(gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), rot_rad);
        gp_Trsf trsf_scale;
        trsf_scale.SetScaleFactor(s);

        // combined = translate * rotate * scale → P' = translate(rotate(scale(P)))
        gp_Trsf combined = trsf_translate;
        combined.Multiply(trsf_rotate);
        combined.Multiply(trsf_scale);

        // Apply transform to a copy of the profile (Standard_True = make copy).
        BRepBuilderAPI_Transform transformer(profile.get(), combined, Standard_True);
        const TopoDS_Shape& transformed = transformer.Shape();

        // Add to loft: extract outer wire from faces.
        if (transformed.ShapeType() == TopAbs_FACE) {
            loft.AddWire(BRepTools::OuterWire(TopoDS::Face(transformed)));
        } else if (transformed.ShapeType() == TopAbs_WIRE) {
            loft.AddWire(TopoDS::Wire(transformed));
        } else {
            throw std::runtime_error("extrude_ex: profile must be a Face or Wire");
        }
    }

    loft.Build();
    if (!loft.IsDone())
        throw std::runtime_error("BRepOffsetAPI_ThruSections (extrude_ex) failed");
    return wrap(loft.Shape());
}

// ---------------------------------------------------------------------------
// Phase 3: Spline profiles and pipe sweep
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> make_spline_2d(rust::Slice<const double> pts) {
    int n = static_cast<int>(pts.size()) / 2;
    if (n < 2)
        throw std::runtime_error("spline_2d: need at least 2 points");

    // Build 3D point array in XZ plane: (r, z) → gp_Pnt(r, 0, z)
    Handle(TColgp_HArray1OfPnt) hPts = new TColgp_HArray1OfPnt(1, n);
    for (int i = 0; i < n; i++) {
        hPts->SetValue(i + 1, gp_Pnt(pts[2 * i], 0.0, pts[2 * i + 1]));
    }

    // Interpolate BSpline through the points
    GeomAPI_Interpolate interp(hPts, /*isPeriodic=*/Standard_False, /*Tolerance=*/1e-6);
    interp.Perform();
    if (!interp.IsDone())
        throw std::runtime_error("GeomAPI_Interpolate (spline_2d) failed");

    Handle(Geom_BSplineCurve) curve = interp.Curve();
    TopoDS_Edge spline_edge = BRepBuilderAPI_MakeEdge(curve).Edge();

    // Close the profile: if first and last points differ, add a straight line back
    gp_Pnt p_first(pts[0], 0.0, pts[1]);
    gp_Pnt p_last(pts[2 * (n - 1)], 0.0, pts[2 * (n - 1) + 1]);

    BRepBuilderAPI_MakeWire wire_builder;
    wire_builder.Add(spline_edge);
    if (p_first.Distance(p_last) > 1e-7) {
        TopoDS_Edge close_edge = BRepBuilderAPI_MakeEdge(p_last, p_first).Edge();
        wire_builder.Add(close_edge);
    }
    if (!wire_builder.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakeWire (spline_2d) failed");

    TopoDS_Wire wire = wire_builder.Wire();

    // The profile is planar (Y=0); specify the XZ plane explicitly for robustness
    gp_Pln xz_plane(gp_Pnt(0, 0, 0), gp_Dir(0, 1, 0));
    BRepBuilderAPI_MakeFace face(xz_plane, wire);
    if (!face.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakeFace (spline_2d) failed");
    return wrap(face.Face());
}

std::unique_ptr<OcctShape> make_spline_3d(rust::Slice<const double> pts) {
    int n = static_cast<int>(pts.size()) / 3;
    if (n < 2)
        throw std::runtime_error("spline_3d: need at least 2 points");

    Handle(TColgp_HArray1OfPnt) hPts = new TColgp_HArray1OfPnt(1, n);
    for (int i = 0; i < n; i++) {
        hPts->SetValue(i + 1, gp_Pnt(pts[3 * i], pts[3 * i + 1], pts[3 * i + 2]));
    }

    GeomAPI_Interpolate interp(hPts, /*isPeriodic=*/Standard_False, /*Tolerance=*/1e-6);
    interp.Perform();
    if (!interp.IsDone())
        throw std::runtime_error("GeomAPI_Interpolate (spline_3d) failed");

    Handle(Geom_BSplineCurve) curve = interp.Curve();
    TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(curve).Edge();
    TopoDS_Wire wire = BRepBuilderAPI_MakeWire(edge).Wire();
    return wrap(wire);
}

std::unique_ptr<OcctShape> make_spline_2d_tan(rust::Slice<const double> pts, double t0x, double t0z,
                                              double t1x, double t1z) {
    int n = static_cast<int>(pts.size()) / 2;
    if (n < 2)
        throw std::runtime_error("spline_2d: need at least 2 points");

    Handle(TColgp_HArray1OfPnt) hPts = new TColgp_HArray1OfPnt(1, n);
    for (int i = 0; i < n; i++) {
        hPts->SetValue(i + 1, gp_Pnt(pts[2 * i], 0.0, pts[2 * i + 1]));
    }

    GeomAPI_Interpolate interp(hPts, /*isPeriodic=*/Standard_False, /*Tolerance=*/1e-6);
    // Apply explicit end tangents — suppresses natural-boundary oscillation.
    // Tangents are in the XZ plane (Y=0); Load() normalises them internally.
    gp_Vec start_tan(t0x, 0.0, t0z);
    gp_Vec end_tan(t1x, 0.0, t1z);
    interp.Load(start_tan, end_tan);
    interp.Perform();
    if (!interp.IsDone())
        throw std::runtime_error("GeomAPI_Interpolate (spline_2d) failed");

    Handle(Geom_BSplineCurve) curve = interp.Curve();
    TopoDS_Edge spline_edge = BRepBuilderAPI_MakeEdge(curve).Edge();

    gp_Pnt p_first(pts[0], 0.0, pts[1]);
    gp_Pnt p_last(pts[2 * (n - 1)], 0.0, pts[2 * (n - 1) + 1]);

    BRepBuilderAPI_MakeWire wire_builder;
    wire_builder.Add(spline_edge);
    if (p_first.Distance(p_last) > 1e-7) {
        TopoDS_Edge close_edge = BRepBuilderAPI_MakeEdge(p_last, p_first).Edge();
        wire_builder.Add(close_edge);
    }
    if (!wire_builder.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakeWire (spline_2d) failed");

    gp_Pln xz_plane(gp_Pnt(0, 0, 0), gp_Dir(0, 1, 0));
    BRepBuilderAPI_MakeFace face(xz_plane, wire_builder.Wire());
    if (!face.IsDone())
        throw std::runtime_error("BRepBuilderAPI_MakeFace (spline_2d) failed");
    return wrap(face.Face());
}

std::unique_ptr<OcctShape> make_spline_3d_tan(rust::Slice<const double> pts, double t0x, double t0y,
                                              double t0z, double t1x, double t1y, double t1z) {
    int n = static_cast<int>(pts.size()) / 3;
    if (n < 2)
        throw std::runtime_error("spline_3d: need at least 2 points");

    Handle(TColgp_HArray1OfPnt) hPts = new TColgp_HArray1OfPnt(1, n);
    for (int i = 0; i < n; i++) {
        hPts->SetValue(i + 1, gp_Pnt(pts[3 * i], pts[3 * i + 1], pts[3 * i + 2]));
    }

    GeomAPI_Interpolate interp(hPts, /*isPeriodic=*/Standard_False, /*Tolerance=*/1e-6);
    // Apply explicit end tangents — suppresses natural-boundary oscillation.
    gp_Vec start_tan(t0x, t0y, t0z);
    gp_Vec end_tan(t1x, t1y, t1z);
    interp.Load(start_tan, end_tan);
    interp.Perform();
    if (!interp.IsDone())
        throw std::runtime_error("GeomAPI_Interpolate (spline_3d) failed");

    Handle(Geom_BSplineCurve) curve = interp.Curve();
    TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(curve).Edge();
    TopoDS_Wire wire = BRepBuilderAPI_MakeWire(edge).Wire();
    return wrap(wire);
}

std::unique_ptr<OcctShape> shape_sweep(const OcctShape& profile, const OcctShape& path) {
    const TopoDS_Shape& path_shape = path.get();
    if (path_shape.ShapeType() != TopAbs_WIRE)
        throw std::runtime_error("sweep: path must be a Wire (create with spline_3d)");

    TopoDS_Wire path_wire = TopoDS::Wire(path_shape);
    BRepOffsetAPI_MakePipe pipe(path_wire, profile.get());
    pipe.Build();
    if (!pipe.IsDone())
        throw std::runtime_error("BRepOffsetAPI_MakePipe (sweep) failed");
    return wrap(pipe.Shape());
}

// ---------------------------------------------------------------------------
// Phase 3: Sub-shape selectors
// ---------------------------------------------------------------------------

// Parse a direction-based face selector like ">Z", "<X", ">Y".
// Returns true on success and fills *axis (0=X,1=Y,2=Z) and *positive.
static bool parse_dir_selector(const std::string& sel, int* axis, bool* positive) {
    if (sel.size() != 2)
        return false;
    char sign = sel[0];
    char ax = sel[1];
    if (sign != '>' && sign != '<')
        return false;
    if (ax != 'X' && ax != 'Y' && ax != 'Z')
        return false;
    *positive = (sign == '>');
    *axis = (ax == 'X') ? 0 : (ax == 'Y') ? 1 : 2;
    return true;
}

// Returns true if face matches the given selector.
// BRepLProp_SLProps returns the geometric surface normal; we must flip it
// when the face orientation is TopAbs_REVERSED so the normal reflects the
// outward (shell-facing) direction rather than the underlying surface direction.
// Supports both named selectors (all/top/bottom/side) and direction-based
// selectors (>X, <X, >Y, <Y, >Z, <Z).
static bool face_matches(const TopoDS_Face& face, const std::string& sel) {
    // Named shorthand selectors that don't need a normal computation.
    if (sel == "all")
        return true;

    BRepAdaptor_Surface adaptor(face);
    double umid = 0.5 * (adaptor.FirstUParameter() + adaptor.LastUParameter());
    double vmid = 0.5 * (adaptor.FirstVParameter() + adaptor.LastVParameter());
    BRepLProp_SLProps props(adaptor, umid, vmid, 1, 1e-6);
    if (!props.IsNormalDefined())
        return false; // degenerate face — skip

    gp_Dir normal = props.Normal();
    if (face.Orientation() == TopAbs_REVERSED)
        normal.Reverse();

    const double threshold = 0.5;
    const double dz = normal.Z();

    if (sel == "top")
        return dz > threshold;
    if (sel == "bottom")
        return dz < -threshold;
    if (sel == "side")
        return std::fabs(dz) <= threshold;

    // Direction-based selectors: ">Z", "<X", etc.
    int axis;
    bool positive;
    if (parse_dir_selector(sel, &axis, &positive)) {
        double component = (axis == 0) ? normal.X() : (axis == 1) ? normal.Y() : normal.Z();
        return positive ? component > threshold : component < -threshold;
    }

    throw std::runtime_error(std::string("faces: unknown selector ':") + sel +
                             "' — use :all, :top, :bottom, :side, or a direction like :>Z or :<X");
}

// Returns true if edge matches the given selector.
// Degenerate edges are always excluded.
static bool edge_matches(const TopoDS_Edge& edge, const std::string& sel) {
    if (BRep_Tool::Degenerated(edge))
        return false;

    BRepAdaptor_Curve adaptor(edge);
    double tmid = 0.5 * (adaptor.FirstParameter() + adaptor.LastParameter());
    gp_Pnt pnt;
    gp_Vec tangent;
    adaptor.D1(tmid, pnt, tangent);
    if (tangent.Magnitude() < 1e-10)
        return false; // zero-length edge

    const double tz = std::fabs(tangent.Z()) / tangent.Magnitude();
    const double threshold = 0.5;

    if (sel == "all")
        return true;
    if (sel == "vertical")
        return tz > threshold;
    if (sel == "horizontal")
        return tz <= threshold;
    throw std::runtime_error(std::string("edges: unknown selector ':") + sel +
                             "' — use :all, :vertical, or :horizontal");
}

int32_t shape_faces_count(const OcctShape& shape, rust::Str selector) {
    std::string sel(selector.data(), selector.size());
    // Validate selector upfront before iterating.
    int dir_axis;
    bool dir_positive;
    if (sel != "all" && sel != "top" && sel != "bottom" && sel != "side" &&
        !parse_dir_selector(sel, &dir_axis, &dir_positive))
        throw std::runtime_error(
            std::string("faces: unknown selector ':") + sel +
            "' — use :all, :top, :bottom, :side, or a direction like :>Z or :<X");
    int32_t count = 0;
    TopExp_Explorer exp(shape.get(), TopAbs_FACE);
    for (; exp.More(); exp.Next()) {
        TopoDS_Face face = TopoDS::Face(exp.Current());
        if (face_matches(face, sel))
            ++count;
    }
    return count;
}

std::unique_ptr<OcctShape> shape_faces_get(const OcctShape& shape, rust::Str selector,
                                           int32_t idx) {
    std::string sel(selector.data(), selector.size());
    int32_t cur = 0;
    TopExp_Explorer exp(shape.get(), TopAbs_FACE);
    for (; exp.More(); exp.Next()) {
        TopoDS_Face face = TopoDS::Face(exp.Current());
        if (face_matches(face, sel)) {
            if (cur == idx)
                return wrap(face);
            ++cur;
        }
    }
    throw std::runtime_error("shape_faces_get: index out of range");
}

// Build a deduplicated list of matching edges.
// TopExp_Explorer may visit shared edges multiple times; TopTools_IndexedMapOfShape
// guarantees each unique TShape appears exactly once.
static std::vector<TopoDS_Edge> collect_edges(const OcctShape& shape, const std::string& sel) {
    // Validate selector upfront before iterating.
    if (sel != "all" && sel != "vertical" && sel != "horizontal")
        throw std::runtime_error(std::string("edges: unknown selector ':") + sel +
                                 "' — use :all, :vertical, or :horizontal");

    TopTools_IndexedMapOfShape edge_map;
    TopExp::MapShapes(shape.get(), TopAbs_EDGE, edge_map);

    std::vector<TopoDS_Edge> result;
    for (int i = 1; i <= edge_map.Extent(); i++) {
        TopoDS_Edge edge = TopoDS::Edge(edge_map(i));
        if (edge_matches(edge, sel))
            result.push_back(edge);
    }
    return result;
}

int32_t shape_edges_count(const OcctShape& shape, rust::Str selector) {
    std::string sel(selector.data(), selector.size());
    return static_cast<int32_t>(collect_edges(shape, sel).size());
}

std::unique_ptr<OcctShape> shape_edges_get(const OcctShape& shape, rust::Str selector,
                                           int32_t idx) {
    std::string sel(selector.data(), selector.size());
    auto edges = collect_edges(shape, sel);
    auto i = static_cast<size_t>(idx);
    if (i >= edges.size())
        throw std::runtime_error("shape_edges_get: index out of range");
    return wrap(edges[i]);
}

// ---------------------------------------------------------------------------
// Phase 4: Vertices selector
// ---------------------------------------------------------------------------

// Only selector currently supported is "all" — a positional / direction
// filter on vertices is not meaningful in the same way as faces/edges.
// The API is symmetric with faces/edges so callers can iterate all vertices
// without special-casing.
int32_t shape_vertices_count(const OcctShape& shape, rust::Str selector) {
    std::string sel(selector.data(), selector.size());
    if (sel != "all")
        throw std::runtime_error(std::string("vertices: unknown selector ':") + sel +
                                 "' — only :all is supported");

    // Use IndexedMapOfShape for deduplication (shared vertices appear once).
    TopTools_IndexedMapOfShape vertex_map;
    TopExp::MapShapes(shape.get(), TopAbs_VERTEX, vertex_map);
    return static_cast<int32_t>(vertex_map.Extent());
}

std::unique_ptr<OcctShape> shape_vertices_get(const OcctShape& shape, rust::Str selector,
                                              int32_t idx) {
    std::string sel(selector.data(), selector.size());
    if (sel != "all")
        throw std::runtime_error(std::string("vertices: unknown selector ':") + sel +
                                 "' — only :all is supported");

    TopTools_IndexedMapOfShape vertex_map;
    TopExp::MapShapes(shape.get(), TopAbs_VERTEX, vertex_map);

    // IndexedMapOfShape is 1-based; idx is 0-based from the caller.
    int one_based = idx + 1;
    if (one_based < 1 || one_based > vertex_map.Extent())
        throw std::runtime_error("shape_vertices_get: index out of range");
    return wrap(vertex_map(one_based));
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Patterns
// ---------------------------------------------------------------------------

// Returns a compound of n translated copies.
// Copy i sits at i*[dx, dy, dz] relative to the input shape's position.
std::unique_ptr<OcctShape> shape_linear_pattern(const OcctShape& s, int32_t n, double dx, double dy,
                                                double dz) {
    if (n < 1)
        throw std::runtime_error("linear_pattern: n must be >= 1");

    TopoDS_Compound compound;
    BRep_Builder builder;
    builder.MakeCompound(compound);

    for (int32_t i = 0; i < n; i++) {
        gp_Trsf trsf;
        trsf.SetTranslation(gp_Vec(i * dx, i * dy, i * dz));
        BRepBuilderAPI_Transform xform(s.get(), trsf, /*copy=*/Standard_True);
        builder.Add(compound, xform.Shape());
    }
    return wrap(compound);
}

// Returns a compound of n copies rotated around the Z axis.
// Copy i is rotated by i * (angle_deg / n) degrees.
// e.g. polar_pattern(shape, 6, 360) places 6 copies every 60° around a full circle.
std::unique_ptr<OcctShape> shape_polar_pattern(const OcctShape& s, int32_t n, double angle_deg) {
    if (n < 1)
        throw std::runtime_error("polar_pattern: n must be >= 1");

    const double step_rad = (angle_deg / n) * (M_PI / 180.0);

    TopoDS_Compound compound;
    BRep_Builder builder;
    builder.MakeCompound(compound);

    const gp_Ax1 z_axis(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1));
    for (int32_t i = 0; i < n; i++) {
        gp_Trsf trsf;
        trsf.SetRotation(z_axis, i * step_rad);
        BRepBuilderAPI_Transform xform(s.get(), trsf, /*copy=*/Standard_True);
        builder.Add(compound, xform.Shape());
    }
    return wrap(compound);
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> import_step(rust::Str path) {
    std::string path_str(path.data(), path.size());
    STEPControl_Reader reader;
    IFSelect_ReturnStatus status = reader.ReadFile(path_str.c_str());
    if (status != IFSelect_RetDone)
        throw std::runtime_error("STEPControl_Reader::ReadFile failed for: " + path_str);
    reader.TransferRoots();
    TopoDS_Shape shape = reader.OneShape();
    if (shape.IsNull())
        throw std::runtime_error("import_step: no shapes found in: " + path_str);
    return wrap(shape);
}

std::unique_ptr<OcctShape> import_stl(rust::Str path) {
    std::string path_str(path.data(), path.size());
    Handle(Poly_Triangulation) tri = RWStl::ReadFile(path_str.c_str(), Message_ProgressRange());
    if (tri.IsNull())
        throw std::runtime_error("RWStl::ReadFile failed or file is empty: " + path_str);
    // Attach the triangulation to a face, then wrap that face in a compound.
    // Returning a compound (same shape type as import_step) keeps callers
    // consistent: a bare TopoDS_Face cannot be passed to boolean ops and its
    // bounding-box / volume results differ from solid behaviour.
    TopoDS_Face face;
    BRep_Builder builder;
    builder.MakeFace(face);
    builder.UpdateFace(face, tri);
    TopoDS_Compound compound;
    builder.MakeCompound(compound);
    builder.Add(compound, face);
    return wrap(compound);
}

// ---------------------------------------------------------------------------
// Phase 4: Query / introspection
// ---------------------------------------------------------------------------

void shape_bounding_box(const OcctShape& shape, rust::Slice<double> out) {
    if (out.size() < 6)
        throw std::runtime_error("bounding_box: output slice must have at least 6 elements");
    Bnd_Box bndBox;
    // AddOptimal gives tighter bounds than Add (avoids inflated gap/tolerance).
    BRepBndLib::AddOptimal(shape.get(), bndBox, /*useTriangulation=*/Standard_False,
                           /*useShapeTolerance=*/Standard_False);
    if (bndBox.IsVoid())
        throw std::runtime_error("bounding_box: shape has no geometry (void bounding box)");
    double xmin, ymin, zmin, xmax, ymax, zmax;
    bndBox.Get(xmin, ymin, zmin, xmax, ymax, zmax);
    out[0] = xmin;
    out[1] = ymin;
    out[2] = zmin;
    out[3] = xmax;
    out[4] = ymax;
    out[5] = zmax;
}

double shape_volume(const OcctShape& shape) {
    GProp_GProps props;
    BRepGProp::VolumeProperties(shape.get(), props);
    return props.Mass();
}

double shape_surface_area(const OcctShape& shape) {
    GProp_GProps props;
    BRepGProp::SurfaceProperties(shape.get(), props);
    return props.Mass();
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

void export_step(const OcctShape& shape, rust::Str path) {
    // Guard against degenerate geometry that would produce a corrupt STEP file.
    BRepCheck_Analyzer checker(shape.get());
    if (!checker.IsValid())
        throw std::runtime_error("shape is topologically invalid (degenerate faces or open shells) "
                                 "— check upstream boolean operations or fillet radii");

    STEPControl_Writer writer;
    IFSelect_ReturnStatus status = writer.Transfer(shape.get(), STEPControl_AsIs);
    if (status != IFSelect_RetDone)
        throw std::runtime_error("STEPControl_Writer::Transfer failed");

    std::string path_str(path.data(), path.size());
    status = writer.Write(path_str.c_str());
    if (status != IFSelect_RetDone)
        throw std::runtime_error("STEPControl_Writer::Write failed for: " + path_str);
}

void export_stl(const OcctShape& shape, rust::Str path) {
    // Tessellate before writing — StlAPI_Writer requires a pre-meshed shape
    // in OCCT 7.7+.  isParallel=true uses the TBB thread pool (OCCT 7.4+).
    BRepMesh_IncrementalMesh mesher(shape.get(), 0.1, /*isRelative=*/Standard_False,
                                    /*angularDeflection=*/0.5, /*isParallel=*/Standard_True);
    mesher.Perform();

    std::string path_str(path.data(), path.size());
    StlAPI_Writer writer;
    Standard_Boolean ok = writer.Write(shape.get(), path_str.c_str());
    if (!ok)
        throw std::runtime_error("StlAPI_Writer::Write failed for: " + path_str);
}

// Shared setup for glTF / GLB export: tessellate, create XDE document, add shape.
static Handle(TDocStd_Document)
    make_xde_doc(const OcctShape& shape, double linear_deflection, const char* label) {
    // isParallel=true uses the TBB thread pool — dominant cost on complex shapes.
    BRepMesh_IncrementalMesh mesher(shape.get(), linear_deflection,
                                    /*isRelative=*/Standard_False,
                                    /*angularDeflection=*/0.5, /*isParallel=*/Standard_True);
    mesher.Perform();

    Handle(XCAFApp_Application) app = XCAFApp_Application::GetApplication();
    Handle(TDocStd_Document) doc;
    app->NewDocument(TCollection_ExtendedString("BinXCAF"), doc);
    if (doc.IsNull())
        throw std::runtime_error(std::string("Failed to create XDE document for ") + label);

    Handle(XCAFDoc_ShapeTool) shape_tool = XCAFDoc_DocumentTool::ShapeTool(doc->Main());
    TDF_Label shape_label = shape_tool->AddShape(shape.get());

    // Attach sRGB surface color when the shape carries one (set via .color(r,g,b)).
    if (shape.has_color()) {
        Handle(XCAFDoc_ColorTool) color_tool = XCAFDoc_DocumentTool::ColorTool(doc->Main());
        Quantity_Color color(shape.color_r(), shape.color_g(), shape.color_b(), Quantity_TOC_sRGB);
        color_tool->SetColor(shape_label, color, XCAFDoc_ColorSurf);
    }

    return doc;
}

void export_gltf(const OcctShape& shape, rust::Str path, double linear_deflection) {
    Handle(TDocStd_Document) doc = make_xde_doc(shape, linear_deflection, "glTF export");
    std::string path_str(path.data(), path.size());
    TCollection_AsciiString gltf_path(path_str.c_str());
    RWGltf_CafWriter writer(gltf_path, /*isBinary=*/Standard_False);
    TColStd_IndexedDataMapOfStringString metadata;
    Message_ProgressRange progress;
    if (!writer.Perform(doc, metadata, progress))
        throw std::runtime_error("RWGltf_CafWriter::Perform failed for: " + path_str);
}

void export_glb(const OcctShape& shape, rust::Str path, double linear_deflection) {
    // Guard against degenerate geometry that would produce a corrupt GLB file.
    BRepCheck_Analyzer checker(shape.get());
    if (!checker.IsValid())
        throw std::runtime_error("shape is topologically invalid (degenerate faces or open shells) "
                                 "— check upstream boolean operations or fillet radii");

    Handle(TDocStd_Document) doc = make_xde_doc(shape, linear_deflection, "GLB export");
    std::string path_str(path.data(), path.size());
    TCollection_AsciiString glb_path(path_str.c_str());
    RWGltf_CafWriter writer(glb_path, /*isBinary=*/Standard_True);
    // TRS decomposition (translation/rotation/scale) is lighter and more
    // interoperable with animation tools than the default 4×4 matrix.
    writer.SetTransformationFormat(RWGltf_WriterTrsfFormat_TRS);
    TColStd_IndexedDataMapOfStringString metadata;
    Message_ProgressRange progress;
    if (!writer.Perform(doc, metadata, progress))
        throw std::runtime_error("RWGltf_CafWriter::Perform (GLB) failed for: " + path_str);
}

// OBJ export via RWObj_CafWriter (OCCT 7.6+).
// Uses the same XDE pipeline as glTF/GLB so material handling is consistent.
void export_obj(const OcctShape& shape, rust::Str path, double linear_deflection) {
    Handle(TDocStd_Document) doc = make_xde_doc(shape, linear_deflection, "OBJ export");
    std::string path_str(path.data(), path.size());
    TCollection_AsciiString obj_path(path_str.c_str());
    RWObj_CafWriter writer(obj_path);
    TColStd_IndexedDataMapOfStringString metadata;
    Message_ProgressRange progress;
    if (!writer.Perform(doc, metadata, progress))
        throw std::runtime_error("RWObj_CafWriter::Perform failed for: " + path_str);
}

} // namespace rrcad
