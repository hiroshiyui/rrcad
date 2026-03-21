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
#include <BRepBuilderAPI_Transform.hxx>

// --- OCCT: Phase 2 ---
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Pln.hxx>

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

// --- OCCT: STEP import / export ---
#include <IFSelect_ReturnStatus.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_StepModelType.hxx>
#include <STEPControl_Writer.hxx>

// --- OCCT: STL import / export ---
#include <BRep_Builder.hxx>
#include <RWStl.hxx>
#include <StlAPI_Writer.hxx>

// --- OCCT: glTF export (XDE pipeline) ---
#include <Message_ProgressRange.hxx>
#include <RWGltf_CafWriter.hxx>
#include <TColStd_IndexedDataMapOfStringString.hxx>
#include <TCollection_AsciiString.hxx>
#include <TCollection_ExtendedString.hxx>
#include <TDocStd_Document.hxx>
#include <XCAFApp_Application.hxx>
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
// Boolean operations
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_fuse(const OcctShape& a, const OcctShape& b) {
    BRepAlgoAPI_Fuse op(a.get(), b.get());
    op.Build();
    if (!op.IsDone())
        throw std::runtime_error("BRepAlgoAPI_Fuse failed");
    return wrap(op.Shape());
}

std::unique_ptr<OcctShape> shape_cut(const OcctShape& a, const OcctShape& b) {
    BRepAlgoAPI_Cut op(a.get(), b.get());
    op.Build();
    if (!op.IsDone())
        throw std::runtime_error("BRepAlgoAPI_Cut failed");
    return wrap(op.Shape());
}

std::unique_ptr<OcctShape> shape_common(const OcctShape& a, const OcctShape& b) {
    BRepAlgoAPI_Common op(a.get(), b.get());
    op.Build();
    if (!op.IsDone())
        throw std::runtime_error("BRepAlgoAPI_Common failed");
    return wrap(op.Shape());
}

// ---------------------------------------------------------------------------
// Fillets and chamfers
// ---------------------------------------------------------------------------

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

// Returns true if face matches the given selector.
// BRepLProp_SLProps returns the geometric surface normal; we must flip it
// when the face orientation is TopAbs_REVERSED so dz reflects the outward
// (shell-facing) direction rather than the underlying surface direction.
static bool face_matches(const TopoDS_Face& face, const std::string& sel) {
    BRepAdaptor_Surface adaptor(face);
    double umid = 0.5 * (adaptor.FirstUParameter() + adaptor.LastUParameter());
    double vmid = 0.5 * (adaptor.FirstVParameter() + adaptor.LastVParameter());
    BRepLProp_SLProps props(adaptor, umid, vmid, 1, 1e-6);
    if (!props.IsNormalDefined())
        return false; // degenerate face — skip

    gp_Dir normal = props.Normal();
    if (face.Orientation() == TopAbs_REVERSED)
        normal.Reverse();

    const double dz = normal.Z();
    const double threshold = 0.5;

    if (sel == "all")
        return true;
    if (sel == "top")
        return dz > threshold;
    if (sel == "bottom")
        return dz < -threshold;
    if (sel == "side")
        return std::fabs(dz) <= threshold;
    throw std::runtime_error(std::string("faces: unknown selector ':") + sel +
                             "' — use :all, :top, :bottom, or :side");
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
    // Validate selector by calling face_matches on a dummy check; easier to
    // iterate and count (also validates selector via exception on first call).
    int32_t count = 0;
    bool validated = false;
    TopExp_Explorer exp(shape.get(), TopAbs_FACE);
    for (; exp.More(); exp.Next()) {
        TopoDS_Face face = TopoDS::Face(exp.Current());
        if (!validated) {
            // This call throws on unknown selector — propagates to Rust as Err
            face_matches(face, sel);
            validated = true;
        }
        if (face_matches(face, sel))
            ++count;
    }
    if (!validated && sel != "all" && sel != "top" && sel != "bottom" && sel != "side") {
        // Shape has no faces — still validate selector
        throw std::runtime_error(std::string("faces: unknown selector ':") + sel +
                                 "' — use :all, :top, :bottom, or :side");
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
    TopTools_IndexedMapOfShape edge_map;
    TopExp::MapShapes(shape.get(), TopAbs_EDGE, edge_map);

    std::vector<TopoDS_Edge> result;
    bool validated = false;
    for (int i = 1; i <= edge_map.Extent(); i++) {
        TopoDS_Edge edge = TopoDS::Edge(edge_map(i));
        if (!validated) {
            edge_matches(edge, sel); // throws on unknown selector
            validated = true;
        }
        if (edge_matches(edge, sel))
            result.push_back(edge);
    }
    if (!validated && sel != "all" && sel != "vertical" && sel != "horizontal") {
        throw std::runtime_error(std::string("edges: unknown selector ':") + sel +
                                 "' — use :all, :vertical, or :horizontal");
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
    TopoDS_Face face;
    BRep_Builder builder;
    builder.MakeFace(face);
    builder.UpdateFace(face, tri);
    return wrap(face);
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

void export_step(const OcctShape& shape, rust::Str path) {
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
    // in OCCT 7.7+.
    BRepMesh_IncrementalMesh mesher(shape.get(), 0.1, /*isRelative=*/Standard_False,
                                    /*angularDeflection=*/0.5);
    mesher.Perform();

    std::string path_str(path.data(), path.size());
    StlAPI_Writer writer;
    Standard_Boolean ok = writer.Write(shape.get(), path_str.c_str());
    if (!ok)
        throw std::runtime_error("StlAPI_Writer::Write failed for: " + path_str);
}

void export_gltf(const OcctShape& shape, rust::Str path, double linear_deflection) {
    // 1. Tessellate the shape (required before glTF export).
    BRepMesh_IncrementalMesh mesher(shape.get(), linear_deflection,
                                    /*isRelative=*/Standard_False,
                                    /*angularDeflection=*/0.5);
    mesher.Perform();

    // 2. Set up an XDE document and add the shape.
    Handle(XCAFApp_Application) app = XCAFApp_Application::GetApplication();
    Handle(TDocStd_Document) doc;
    app->NewDocument(TCollection_ExtendedString("BinXCAF"), doc);
    if (doc.IsNull())
        throw std::runtime_error("Failed to create XDE document for glTF export");

    Handle(XCAFDoc_ShapeTool) shape_tool = XCAFDoc_DocumentTool::ShapeTool(doc->Main());
    shape_tool->AddShape(shape.get());

    // 3. Write glTF.
    std::string path_str(path.data(), path.size());
    TCollection_AsciiString gltf_path(path_str.c_str());
    RWGltf_CafWriter writer(gltf_path, /*isBinary=*/Standard_False);
    TColStd_IndexedDataMapOfStringString metadata;
    Message_ProgressRange progress;

    if (!writer.Perform(doc, metadata, progress))
        throw std::runtime_error("RWGltf_CafWriter::Perform failed for: " + path_str);
}

void export_glb(const OcctShape& shape, rust::Str path, double linear_deflection) {
    BRepMesh_IncrementalMesh mesher(shape.get(), linear_deflection,
                                    /*isRelative=*/Standard_False,
                                    /*angularDeflection=*/0.5);
    mesher.Perform();

    Handle(XCAFApp_Application) app = XCAFApp_Application::GetApplication();
    Handle(TDocStd_Document) doc;
    app->NewDocument(TCollection_ExtendedString("BinXCAF"), doc);
    if (doc.IsNull())
        throw std::runtime_error("Failed to create XDE document for GLB export");

    Handle(XCAFDoc_ShapeTool) shape_tool = XCAFDoc_DocumentTool::ShapeTool(doc->Main());
    shape_tool->AddShape(shape.get());

    std::string path_str(path.data(), path.size());
    TCollection_AsciiString glb_path(path_str.c_str());
    RWGltf_CafWriter writer(glb_path, /*isBinary=*/Standard_True);
    TColStd_IndexedDataMapOfStringString metadata;
    Message_ProgressRange progress;

    if (!writer.Perform(doc, metadata, progress))
        throw std::runtime_error("RWGltf_CafWriter::Perform (GLB) failed for: " + path_str);
}

} // namespace rrcad
