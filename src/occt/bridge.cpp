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
#include <BRepCheck_ListOfStatus.hxx>
#include <BRepCheck_Result.hxx>
#include <BRepCheck_Status.hxx>

// --- OCCT: Phase 4 — query / introspection ---
#include <BRepBndLib.hxx>
#include <BRepGProp.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>

// --- OCCT: Phase 4 — 3-D operations ---
#include <BRepAlgoAPI_Defeaturing.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepLib.hxx>
#include <BRepOffsetAPI_MakeOffset.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepOffsetAPI_MakePipeShell.hxx>
#include <BRepOffsetAPI_MakeThickSolid.hxx>
#include <BRepOffsetAPI_ThruSections.hxx>
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <Standard_Failure.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS_Wire.hxx>
#include <cmath>
#include <limits>
#include <set>
#include <string>

// --- OCCT: Bézier surface patch ---
#include <Geom_BezierSurface.hxx>
#include <Precision.hxx>
#include <TColgp_Array2OfPnt.hxx>

// --- OCCT: Phase 8 Tier 1 — part design ---
#include <BRepFilletAPI_MakeFillet2d.hxx>
#include <ChFi2d_ConstructionError.hxx>
#include <gp_Ax3.hxx>

// --- OCCT: Phase 8 Tier 3 — inspection & clearance ---
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepGProp_Face.hxx>
#include <IntCurvesFace_ShapeIntersector.hxx>

// --- OCCT: Phase 8 Tier 2 — manufacturing features ---
#include <BRepOffsetAPI_DraftAngle.hxx>

// --- OCCT: Phase 8 Tier 4 — 2-D drawing output ---
#include <HLRAlgo_Projector.hxx>
#include <HLRBRep_PolyAlgo.hxx>
#include <HLRBRep_PolyHLRToShape.hxx>
#include <fstream>
#include <iomanip>

// --- OCCT: Phase 8 Tier 5 — advanced composition ---
#include <BRepAdaptor_CompCurve.hxx>
#include <BRepAlgoAPI_BuilderAlgo.hxx>
#include <BRepFill_TypeOfContact.hxx>
#include <GCPnts_UniformAbscissa.hxx>
#include <Poly_Triangulation.hxx>
#include <TopLoc_Location.hxx>

// --- OCCT: Phase 7 Tier 3 — surface modeling ---
#include <BRepAlgoAPI_Section.hxx>
#include <BRepFill.hxx>
#include <BRepFill_Filling.hxx>
#include <GeomAbs_Shape.hxx>

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

// Variable-radius fillet: each edge gets a radius that transitions linearly
// from r1 at one end-vertex to r2 at the other.
// BRepFilletAPI_MakeFillet::Add(r1, r2, edge) accepts two radii directly;
// OCCT interpolates smoothly along the edge.
std::unique_ptr<OcctShape> shape_fillet_var(const OcctShape& s, double r1, double r2) {
    BRepFilletAPI_MakeFillet builder(s.get());

    TopExp_Explorer exp(s.get(), TopAbs_EDGE);
    for (; exp.More(); exp.Next())
        builder.Add(r1, r2, TopoDS::Edge(exp.Current()));

    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepFilletAPI_MakeFillet (variable-radius) failed — "
                                 "check for degenerate edges or radii too large");
    return wrap(builder.Shape());
}

std::unique_ptr<OcctShape> shape_fillet_var_sel(const OcctShape& s, double r1, double r2,
                                                rust::Str selector) {
    std::string sel(selector.data(), selector.size());
    auto edges = collect_edges(s, sel);
    if (edges.empty())
        throw std::runtime_error("fillet: no edges match selector ':" + sel + "'");

    BRepFilletAPI_MakeFillet builder(s.get());
    for (const auto& edge : edges)
        builder.Add(r1, r2, edge);

    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepFilletAPI_MakeFillet (variable-radius, selective) failed — "
                                 "check for degenerate edges or radii too large");
    return wrap(builder.Shape());
}

// Asymmetric chamfer: d1 and d2 are the two bevel distances on each side of the edge.
// OCCT's BRepFilletAPI_MakeChamfer::Add(d1, d2, edge, face) requires a reference face
// to indicate on which side d1 applies.  We build an edge→adjacent-face map and use
// the first adjacent face for every edge.
std::unique_ptr<OcctShape> shape_chamfer_asym(const OcctShape& s, double d1, double d2) {
    TopTools_IndexedDataMapOfShapeListOfShape edge_face_map;
    TopExp::MapShapesAndAncestors(s.get(), TopAbs_EDGE, TopAbs_FACE, edge_face_map);

    BRepFilletAPI_MakeChamfer builder(s.get());

    TopExp_Explorer exp(s.get(), TopAbs_EDGE);
    for (; exp.More(); exp.Next()) {
        TopoDS_Edge edge = TopoDS::Edge(exp.Current());
        if (!edge_face_map.Contains(edge))
            continue;
        const TopTools_ListOfShape& faces = edge_face_map.FindFromKey(edge);
        if (faces.IsEmpty())
            continue;
        TopoDS_Face face = TopoDS::Face(faces.First());
        builder.Add(d1, d2, edge, face);
    }

    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepFilletAPI_MakeChamfer (asymmetric) failed");
    return wrap(builder.Shape());
}

// Selective asymmetric chamfer: only edges matching the selector are bevelled.
std::unique_ptr<OcctShape> shape_chamfer_asym_sel(const OcctShape& s, double d1, double d2,
                                                  rust::Str selector) {
    std::string sel(selector.data(), selector.size());
    auto edges = collect_edges(s, sel);
    if (edges.empty())
        throw std::runtime_error("chamfer: no edges match selector ':" + sel + "'");

    TopTools_IndexedDataMapOfShapeListOfShape edge_face_map;
    TopExp::MapShapesAndAncestors(s.get(), TopAbs_EDGE, TopAbs_FACE, edge_face_map);

    BRepFilletAPI_MakeChamfer builder(s.get());
    for (const auto& edge : edges) {
        if (!edge_face_map.Contains(edge))
            continue;
        const TopTools_ListOfShape& faces = edge_face_map.FindFromKey(edge);
        if (faces.IsEmpty())
            continue;
        TopoDS_Face face = TopoDS::Face(faces.First());
        builder.Add(d1, d2, edge, face);
    }

    builder.Build();
    if (!builder.IsDone())
        throw std::runtime_error("BRepFilletAPI_MakeChamfer (asymmetric, selective) failed");
    return wrap(builder.Shape());
}

// ---------------------------------------------------------------------------
// Bézier surface patch
// ---------------------------------------------------------------------------

// Build a single bicubic Bézier face from 16 control points (4×4 grid).
// `pts` is a flat array of 48 doubles: 16 points × (x, y, z) in row-major
// order (row 0 = first parameter direction, col 0 = second).
std::unique_ptr<OcctShape> make_bezier_patch(rust::Slice<const double> pts) {
    if (pts.size() != 48)
        throw std::runtime_error(
            "make_bezier_patch: expected 48 doubles (16 control points × 3 coords)");

    // OCCT array indices are 1-based.
    TColgp_Array2OfPnt poles(1, 4, 1, 4);
    for (int row = 0; row < 4; ++row) {
        for (int col = 0; col < 4; ++col) {
            int base = (row * 4 + col) * 3;
            poles.SetValue(row + 1, col + 1, gp_Pnt(pts[base], pts[base + 1], pts[base + 2]));
        }
    }

    Handle(Geom_BezierSurface) surf = new Geom_BezierSurface(poles);

    // BRepBuilderAPI_MakeFace with a parametric surface; Precision::Confusion() as tolerance.
    BRepBuilderAPI_MakeFace face_builder(surf, Precision::Confusion());
    if (!face_builder.IsDone())
        throw std::runtime_error("make_bezier_patch: BRepBuilderAPI_MakeFace failed");

    return wrap(face_builder.Face());
}

// ---------------------------------------------------------------------------
// Sewing builder
// ---------------------------------------------------------------------------

// Pimpl implementation: holds BRepBuilderAPI_Sewing inside bridge.cpp where
// the full OCCT header is available.
struct SewingBuilder::Impl {
    BRepBuilderAPI_Sewing sewing;
    explicit Impl(double tolerance) : sewing(tolerance) {}
};

SewingBuilder::SewingBuilder(double tolerance) : impl(std::make_unique<Impl>(tolerance)) {}

// Destructor must be out-of-line so the compiler sees the full Impl definition.
SewingBuilder::~SewingBuilder() = default;

std::unique_ptr<SewingBuilder> sewing_new(double tolerance) {
    return std::make_unique<SewingBuilder>(tolerance);
}

void sewing_add(SewingBuilder& builder, const OcctShape& shape) {
    builder.impl->sewing.Add(shape.get());
}

// Perform sewing, then attempt to close the resulting shell into a solid.
// Returns the solid on success; falls back to the open shell if MakeSolid fails.
std::unique_ptr<OcctShape> sewing_build(SewingBuilder& builder) {
    try {
        builder.impl->sewing.Perform();
        TopoDS_Shape sewn = builder.impl->sewing.SewedShape();
        if (sewn.IsNull())
            throw std::runtime_error("sewing_build: BRepBuilderAPI_Sewing produced a null shape");

        // Try to produce a closed solid from the sewn shell.
        if (sewn.ShapeType() == TopAbs_SHELL) {
            BRepBuilderAPI_MakeSolid solid_builder;
            solid_builder.Add(TopoDS::Shell(sewn));
            solid_builder.Build();
            if (solid_builder.IsDone()) {
                TopoDS_Solid solid = solid_builder.Solid();
                // Orient faces so all normals point outward.
                BRepLib::OrientClosedSolid(solid);
                BRepCheck_Analyzer check(solid);
                if (check.IsValid())
                    return wrap(solid);
            }
        }

        // Fall back: return the sewn shape as-is (open shell or compound).
        return wrap(sewn);
    } catch (Standard_Failure& e) {
        throw std::runtime_error(std::string("sewing_build failed: ") + e.GetMessageString());
    }
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
// Phase 3: PipeShellBuilder — variable-section sweep
// ---------------------------------------------------------------------------
//
// Strategy: translate each origin-centred section to the spine point at
// parameter t[i] = tFirst + i/(n-1)*(tLast-tFirst) before calling Add().
// MakePipeShell finds the placement position by projecting the section centroid
// onto the spine; pre-translating ensures each section has a unique projection
// point.  WithCorrection=true then rotates each profile perpendicular to the
// spine tangent, keeping circles truly circular.
//
// For highly-curved spines (e.g., the teapot handle C-arc), BRepOffsetAPI_
// MakePipeShell::MakeSolid() may fail.  In that case pipe_shell_build falls
// back to BRepOffsetAPI_ThruSections which is proven to produce valid solids
// for the same translated-circle sections.

// Internal state hidden from bridge.h so OCCT types don't leak into the header.
struct PipeShellBuilder::Impl {
    TopoDS_Wire spineWire;
    TopoDS_Edge spineEdge;              // single edge that forms the wire (from spline_3d)
    Handle(Geom_Curve) curve;           // the underlying BSpline curve
    Standard_Real tFirst = 0.0;         // curve parameter at spine start
    Standard_Real tLast = 1.0;          // curve parameter at spine end
    std::vector<TopoDS_Shape> sections; // collected cross-section shapes (wires/vertices)
};

PipeShellBuilder::PipeShellBuilder(const OcctShape& path) : impl(std::make_unique<Impl>()) {
    const TopoDS_Shape& s = path.get();
    if (s.ShapeType() != TopAbs_WIRE)
        throw std::runtime_error("sweep_sections: path must be a Wire (use spline_3d)");

    impl->spineWire = TopoDS::Wire(s);

    // Expect a single-edge wire (spline_3d always produces one).
    BRepTools_WireExplorer wExp(impl->spineWire);
    if (!wExp.More())
        throw std::runtime_error("sweep_sections: spine wire has no edges");
    impl->spineEdge = wExp.Current();

    impl->curve = BRep_Tool::Curve(impl->spineEdge, impl->tFirst, impl->tLast);
    if (impl->curve.IsNull())
        throw std::runtime_error("sweep_sections: could not extract curve from spine edge");
}

PipeShellBuilder::~PipeShellBuilder() = default;

std::unique_ptr<PipeShellBuilder> pipe_shell_new(const OcctShape& path) {
    return std::make_unique<PipeShellBuilder>(path);
}

void pipe_shell_add(PipeShellBuilder& b, const OcctShape& profile) {
    // Accumulate sections in order; they are placed in pipe_shell_build().
    const TopoDS_Shape& s = profile.get();
    if (s.ShapeType() == TopAbs_FACE) {
        // MakePipeShell works with wires; extract the outer boundary wire here.
        b.impl->sections.push_back(BRepTools::OuterWire(TopoDS::Face(s)));
    } else if (s.ShapeType() == TopAbs_WIRE) {
        b.impl->sections.push_back(s);
    } else if (s.ShapeType() == TopAbs_VERTEX) {
        b.impl->sections.push_back(s);
    } else {
        throw std::runtime_error("sweep_sections: each profile must be a Face, Wire, or Vertex");
    }
}

// Helper: translate section[i] to the spine point at evenly-distributed
// parameter t[i], return the moved shape.
static TopoDS_Shape moveToSpinePoint(const TopoDS_Shape& section, int i, int n,
                                     const Handle(Geom_Curve) & curve, Standard_Real tFirst,
                                     Standard_Real tLast) {
    const Standard_Real t = tFirst + static_cast<Standard_Real>(i) / (n - 1) * (tLast - tFirst);
    gp_Pnt spinePt;
    curve->D0(t, spinePt);

    gp_Trsf trsf;
    trsf.SetTranslation(gp_Vec(spinePt.X(), spinePt.Y(), spinePt.Z()));
    BRepBuilderAPI_Transform mover(section, trsf, /*Copy=*/Standard_True);
    return mover.Shape();
}

std::unique_ptr<OcctShape> pipe_shell_build(PipeShellBuilder& b) {
    const int n = static_cast<int>(b.impl->sections.size());
    if (n < 2)
        throw std::runtime_error("sweep_sections: at least 2 profiles required");

    try {
        // --- Primary path: BRepOffsetAPI_MakePipeShell ---
        //
        // All DSL profiles are origin-centred (circle(r), rect, etc.).  Translate
        // each section to its target spine point before Add() so that MakePipeShell's
        // centroid→spine projection maps to the correct unique parametric position.
        // WithCorrection=true asks OCCT to rotate each profile perpendicular to the
        // spine tangent, keeping circles truly circular in cross-section.
        BRepOffsetAPI_MakePipeShell mkPS(b.impl->spineWire);
        mkPS.SetMode(/*IsFrenet=*/Standard_True);

        for (int i = 0; i < n; i++) {
            TopoDS_Shape moved = moveToSpinePoint(b.impl->sections[i], i, n, b.impl->curve,
                                                  b.impl->tFirst, b.impl->tLast);
            if (moved.ShapeType() == TopAbs_VERTEX)
                mkPS.Add(TopoDS::Vertex(moved), Standard_False, Standard_False);
            else
                mkPS.Add(moved, Standard_False, /*WithCorrection=*/Standard_True);
        }

        if (mkPS.IsReady()) {
            mkPS.Build();
            if (mkPS.IsDone() && mkPS.MakeSolid())
                return wrap(mkPS.Shape());
        }

        // --- Fallback: BRepOffsetAPI_ThruSections (loft through spine-positioned sections) ---
        //
        // MakePipeShell can fail to close into a solid for highly-curved paths
        // (e.g., the teapot handle C-arc) where the built-in MakeSolid() is
        // too strict.  ThruSections is proven to produce valid closed solids for
        // the same translated-circle sections, and produces a geometrically
        // equivalent result for tubes that are substantially circular in cross-section.
        BRepOffsetAPI_ThruSections thru(/*isSolid=*/Standard_True, /*isRuled=*/Standard_False);
        thru.CheckCompatibility(Standard_False);

        for (int i = 0; i < n; i++) {
            TopoDS_Shape moved = moveToSpinePoint(b.impl->sections[i], i, n, b.impl->curve,
                                                  b.impl->tFirst, b.impl->tLast);
            if (moved.ShapeType() == TopAbs_VERTEX)
                thru.AddVertex(TopoDS::Vertex(moved));
            else if (moved.ShapeType() == TopAbs_WIRE)
                thru.AddWire(TopoDS::Wire(moved));
            // Faces already extracted to wires in pipe_shell_add; should not reach here.
        }

        thru.Build();
        if (thru.IsDone())
            return wrap(thru.Shape());

        throw std::runtime_error(
            "sweep_sections: both MakePipeShell and ThruSections fallback failed");

    } catch (const Standard_Failure& e) {
        // OCCT exceptions (Standard_Failure and subclasses) do not inherit from
        // std::exception, so cxx cannot catch them — they would terminate().
        // Re-throw as std::runtime_error so cxx can surface them as Rust errors.
        throw std::runtime_error(std::string("sweep_sections (OCCT): ") + e.GetMessageString());
    }
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
// Phase 7 Tier 1: .offset_2d(distance) — inward/outward offset of a Wire or Face
//
// BRepOffsetAPI_MakeOffset operates in the plane of the input shape.
// Positive distance expands the profile; negative shrinks it.
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_offset_2d(const OcctShape& shape, double distance) {
    TopAbs_ShapeEnum type = shape.get().ShapeType();
    if (type != TopAbs_FACE && type != TopAbs_WIRE)
        throw std::runtime_error("offset_2d: input must be a Face or Wire");

    // BRepOffsetAPI_MakeOffset has separate constructors for Face and Wire.
    if (type == TopAbs_FACE) {
        BRepOffsetAPI_MakeOffset offsetter(TopoDS::Face(shape.get()), GeomAbs_Arc);
        offsetter.Perform(distance);
        if (!offsetter.IsDone())
            throw std::runtime_error("BRepOffsetAPI_MakeOffset (offset_2d, face) failed");
        return wrap(offsetter.Shape());
    } else {
        BRepOffsetAPI_MakeOffset offsetter(TopoDS::Wire(shape.get()), GeomAbs_Arc);
        offsetter.Perform(distance);
        if (!offsetter.IsDone())
            throw std::runtime_error("BRepOffsetAPI_MakeOffset (offset_2d, wire) failed");
        return wrap(offsetter.Shape());
    }
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
// Phase 7 Tier 2: Validation & introspection
// ---------------------------------------------------------------------------

// Map TopAbs_ShapeEnum to a lowercase string name.
rust::String shape_type_str(const OcctShape& shape) {
    switch (shape.get().ShapeType()) {
    case TopAbs_COMPOUND:
        return rust::String("compound");
    case TopAbs_COMPSOLID:
        return rust::String("compsolid");
    case TopAbs_SOLID:
        return rust::String("solid");
    case TopAbs_SHELL:
        return rust::String("shell");
    case TopAbs_FACE:
        return rust::String("face");
    case TopAbs_WIRE:
        return rust::String("wire");
    case TopAbs_EDGE:
        return rust::String("edge");
    case TopAbs_VERTEX:
        return rust::String("vertex");
    default:
        return rust::String("other");
    }
}

// Centroid of the shape.  Uses VolumeProperties for solids and compounds;
// SurfaceProperties for shells/faces; LinearProperties for wires/edges.
void shape_centroid(const OcctShape& shape, rust::Slice<double> out) {
    if (out.size() < 3)
        throw std::runtime_error("centroid: output slice must have at least 3 elements");
    GProp_GProps props;
    switch (shape.get().ShapeType()) {
    case TopAbs_SOLID:
    case TopAbs_COMPOUND:
    case TopAbs_COMPSOLID:
        BRepGProp::VolumeProperties(shape.get(), props);
        break;
    case TopAbs_SHELL:
    case TopAbs_FACE:
        BRepGProp::SurfaceProperties(shape.get(), props);
        break;
    default: // wire, edge, vertex, compound of lower-dim shapes
        BRepGProp::LinearProperties(shape.get(), props);
        break;
    }
    gp_Pnt c = props.CentreOfMass();
    out[0] = c.X();
    out[1] = c.Y();
    out[2] = c.Z();
}

// A shape is "closed" if it has no free (boundary) edges — every edge is
// shared by at least two faces.  Empty shapes (no edges) return false.
bool shape_is_closed(const OcctShape& shape) {
    TopTools_IndexedDataMapOfShapeListOfShape edge_face_map;
    TopExp::MapShapesAndAncestors(shape.get(), TopAbs_EDGE, TopAbs_FACE, edge_face_map);
    if (edge_face_map.IsEmpty())
        return false;
    for (int i = 1; i <= edge_face_map.Extent(); ++i) {
        if (edge_face_map(i).Size() < 2)
            return false;
    }
    return true;
}

// A shape is "manifold" if every edge is shared by exactly two faces.
// This rules out both boundary edges (< 2 faces) and T-junction edges (> 2 faces).
bool shape_is_manifold(const OcctShape& shape) {
    TopTools_IndexedDataMapOfShapeListOfShape edge_face_map;
    TopExp::MapShapesAndAncestors(shape.get(), TopAbs_EDGE, TopAbs_FACE, edge_face_map);
    if (edge_face_map.IsEmpty())
        return false;
    for (int i = 1; i <= edge_face_map.Extent(); ++i) {
        if (edge_face_map(i).Size() != 2)
            return false;
    }
    return true;
}

// Convert a BRepCheck_Status value to a human-readable name string.
static const char* brep_check_status_name(BRepCheck_Status s) {
    switch (s) {
    case BRepCheck_NoError:
        return nullptr;
    case BRepCheck_InvalidPointOnCurve:
        return "InvalidPointOnCurve";
    case BRepCheck_InvalidPointOnCurveOnSurface:
        return "InvalidPointOnCurveOnSurface";
    case BRepCheck_InvalidPointOnSurface:
        return "InvalidPointOnSurface";
    case BRepCheck_No3DCurve:
        return "No3DCurve";
    case BRepCheck_Multiple3DCurve:
        return "Multiple3DCurve";
    case BRepCheck_Invalid3DCurve:
        return "Invalid3DCurve";
    case BRepCheck_NoCurveOnSurface:
        return "NoCurveOnSurface";
    case BRepCheck_InvalidCurveOnSurface:
        return "InvalidCurveOnSurface";
    case BRepCheck_InvalidCurveOnClosedSurface:
        return "InvalidCurveOnClosedSurface";
    case BRepCheck_InvalidSameRangeFlag:
        return "InvalidSameRangeFlag";
    case BRepCheck_InvalidSameParameterFlag:
        return "InvalidSameParameterFlag";
    case BRepCheck_InvalidDegeneratedFlag:
        return "InvalidDegeneratedFlag";
    case BRepCheck_FreeEdge:
        return "FreeEdge";
    case BRepCheck_InvalidMultiConnexity:
        return "InvalidMultiConnexity";
    case BRepCheck_InvalidRange:
        return "InvalidRange";
    case BRepCheck_EmptyWire:
        return "EmptyWire";
    case BRepCheck_RedundantEdge:
        return "RedundantEdge";
    case BRepCheck_SelfIntersectingWire:
        return "SelfIntersectingWire";
    case BRepCheck_NoSurface:
        return "NoSurface";
    case BRepCheck_InvalidWire:
        return "InvalidWire";
    case BRepCheck_RedundantWire:
        return "RedundantWire";
    case BRepCheck_IntersectingWires:
        return "IntersectingWires";
    case BRepCheck_InvalidImbricationOfWires:
        return "InvalidImbricationOfWires";
    case BRepCheck_EmptyShell:
        return "EmptyShell";
    case BRepCheck_RedundantFace:
        return "RedundantFace";
    case BRepCheck_UnorientableShape:
        return "UnorientableShape";
    case BRepCheck_NotClosed:
        return "NotClosed";
    case BRepCheck_NotConnected:
        return "NotConnected";
    case BRepCheck_SubshapeNotInShape:
        return "SubshapeNotInShape";
    case BRepCheck_BadOrientation:
        return "BadOrientation";
    case BRepCheck_BadOrientationOfSubshape:
        return "BadOrientationOfSubshape";
    case BRepCheck_InvalidToleranceValue:
        return "InvalidToleranceValue";
    case BRepCheck_CheckFail:
        return "CheckFail";
    default:
        return "UnknownError";
    }
}

// Collect BRepCheck errors for sub-shapes of a given type and add their
// names (deduplicated) into `errors`.
static void collect_check_errors(const BRepCheck_Analyzer& checker, const TopoDS_Shape& root,
                                 TopAbs_ShapeEnum sub_type, std::set<std::string>& errors) {
    for (TopExp_Explorer ex(root, sub_type); ex.More(); ex.Next()) {
        const Handle(BRepCheck_Result) & res = checker.Result(ex.Current());
        if (res.IsNull())
            continue;
        const BRepCheck_ListOfStatus& lst = res->StatusOnShape(ex.Current());
        for (BRepCheck_ListIteratorOfListOfStatus it(lst); it.More(); it.Next()) {
            const char* name = brep_check_status_name(it.Value());
            if (name)
                errors.insert(name);
        }
    }
}

// Run BRepCheck_Analyzer over the shape.  Returns "ok" if valid, or a
// newline-separated list of distinct error names if not.
rust::String shape_validate_str(const OcctShape& shape) {
    BRepCheck_Analyzer checker(shape.get());
    if (checker.IsValid())
        return rust::String("ok");

    std::set<std::string> errors;
    static const TopAbs_ShapeEnum sub_types[] = {TopAbs_SOLID, TopAbs_SHELL, TopAbs_FACE,
                                                 TopAbs_WIRE,  TopAbs_EDGE,  TopAbs_VERTEX};
    for (TopAbs_ShapeEnum t : sub_types)
        collect_check_errors(checker, shape.get(), t, errors);

    if (errors.empty()) {
        // Analyzer said invalid but returned no per-sub-shape errors:
        // the top-level shape itself may be flagged.
        const Handle(BRepCheck_Result) & res = checker.Result(shape.get());
        if (!res.IsNull()) {
            const BRepCheck_ListOfStatus& lst = res->StatusOnShape(shape.get());
            for (BRepCheck_ListIteratorOfListOfStatus it(lst); it.More(); it.Next()) {
                const char* name = brep_check_status_name(it.Value());
                if (name)
                    errors.insert(name);
            }
        }
    }

    if (errors.empty())
        return rust::String("invalid");

    std::string result;
    for (const auto& e : errors) {
        if (!result.empty())
            result += '\n';
        result += e;
    }
    return rust::String(result.c_str());
}

// ---------------------------------------------------------------------------
// Phase 7 Tier 3: Surface modeling
// ---------------------------------------------------------------------------

// Build a ruled surface (TopoDS_Shell) by connecting corresponding vertices
// of two wires with straight lines.  BRepFill::Shell handles wire orientation
// and produces a properly-connected shell suitable for further sewing.
std::unique_ptr<OcctShape> shape_ruled_surface(const OcctShape& wire_a, const OcctShape& wire_b) {
    if (wire_a.get().ShapeType() != TopAbs_WIRE)
        throw std::runtime_error("ruled_surface: first argument must be a Wire");
    if (wire_b.get().ShapeType() != TopAbs_WIRE)
        throw std::runtime_error("ruled_surface: second argument must be a Wire");

    TopoDS_Shell shell = BRepFill::Shell(TopoDS::Wire(wire_a.get()), TopoDS::Wire(wire_b.get()));
    if (shell.IsNull())
        throw std::runtime_error("ruled_surface: BRepFill::Shell failed");

    return wrap(shell);
}

// Build a smooth filling surface (TopoDS_Face) whose boundary follows the
// edges of a single closed wire.  Each edge is added as a C0 free boundary
// constraint; BRepFill_Filling solves a plate-energy minimisation problem to
// produce a fair surface inside.
std::unique_ptr<OcctShape> shape_fill_surface(const OcctShape& boundary_wire) {
    if (boundary_wire.get().ShapeType() != TopAbs_WIRE)
        throw std::runtime_error("fill_surface: argument must be a Wire");

    BRepFill_Filling filler;
    bool any = false;
    for (TopExp_Explorer ex(boundary_wire.get(), TopAbs_EDGE); ex.More(); ex.Next()) {
        filler.Add(TopoDS::Edge(ex.Current()), GeomAbs_C0, /* IsBound */ Standard_True);
        any = true;
    }
    if (!any)
        throw std::runtime_error("fill_surface: boundary wire contains no edges");

    filler.Build();
    if (!filler.IsDone())
        throw std::runtime_error("fill_surface: BRepFill_Filling failed");

    return wrap(filler.Face());
}

// Intersect a shape with an axis-aligned plane and return the cross-section
// as a compound of edges/wires.  The section curves can be extruded or used
// as profiles for further operations.
//
//   plane = "xy"  →  plane at z = offset  (normal +Z)
//   plane = "xz"  →  plane at y = offset  (normal +Y)
//   plane = "yz"  →  plane at x = offset  (normal +X)
std::unique_ptr<OcctShape> shape_slice(const OcctShape& shape, rust::Str plane, double offset) {
    std::string pname(plane.data(), plane.size());

    gp_Pnt origin(0, 0, 0);
    gp_Dir normal(0, 0, 1);

    if (pname == "xy") {
        origin = gp_Pnt(0, 0, offset);
        normal = gp_Dir(0, 0, 1);
    } else if (pname == "xz") {
        origin = gp_Pnt(0, offset, 0);
        normal = gp_Dir(0, 1, 0);
    } else if (pname == "yz") {
        origin = gp_Pnt(offset, 0, 0);
        normal = gp_Dir(1, 0, 0);
    } else {
        throw std::runtime_error("slice: plane must be \"xy\", \"xz\", or \"yz\"");
    }

    gp_Pln pln(origin, normal);
    BRepAlgoAPI_Section section(shape.get(), pln);
    section.Build();
    if (!section.IsDone())
        throw std::runtime_error("slice: BRepAlgoAPI_Section failed");

    return wrap(section.Shape());
}

// ---------------------------------------------------------------------------
// Phase 8 Tier 1: Core Part Design
// ---------------------------------------------------------------------------

// If shape is a TopoDS_Compound that wraps exactly one Solid (OCCT 7.6+
// boolean operations often do this), return that Solid.  Otherwise return
// the shape unchanged so callers always get the most specific type.
static TopoDS_Shape unwrap_single_solid(const TopoDS_Shape& shape) {
    if (shape.ShapeType() != TopAbs_COMPOUND)
        return shape;
    TopoDS_Iterator iter(shape);
    if (!iter.More())
        return shape; // empty compound
    TopoDS_Shape first = iter.Value();
    iter.Next();
    if (!iter.More() && first.ShapeType() == TopAbs_SOLID)
        return first; // single-solid compound → return the solid directly
    return shape;     // multiple sub-shapes or non-solid: keep as compound
}

// Build the coordinate system of a face: origin at face centroid, Z = outward
// normal, X = first in-plane direction (Gram-Schmidt from global X or Y).
static gp_Ax3 get_face_ax3(const TopoDS_Face& face) {
    // Centroid via surface properties.
    GProp_GProps props;
    BRepGProp::SurfaceProperties(face, props);
    gp_Pnt centroid = props.CentreOfMass();

    // Normal via parametric D1 at mid-parameter.
    BRepAdaptor_Surface surf(face);
    double u_mid = 0.5 * (surf.FirstUParameter() + surf.LastUParameter());
    double v_mid = 0.5 * (surf.FirstVParameter() + surf.LastVParameter());
    gp_Pnt pt;
    gp_Vec du, dv;
    surf.D1(u_mid, v_mid, pt, du, dv);
    gp_Dir normal(du.Crossed(dv));

    // Flip if the face orientation is reversed (OCCT convention).
    if (face.Orientation() == TopAbs_REVERSED)
        normal.Reverse();

    // Choose the X direction: project global X onto the face plane.
    // Fall back to global Y when the face normal is nearly parallel to global X.
    gp_Dir x_cand(1.0, 0.0, 0.0);
    if (std::abs(normal.Dot(x_cand)) > 0.9)
        x_cand = gp_Dir(0.0, 1.0, 0.0);

    // Gram-Schmidt: subtract the component along normal.
    gp_Vec x_proj = gp_Vec(x_cand) - gp_Vec(normal) * normal.Dot(x_cand);
    gp_Dir x_dir(x_proj);

    return gp_Ax3(centroid, normal, x_dir);
}

// Build the gp_Trsf that moves the standard XY plane (origin=(0,0,0),
// normal=(0,0,1)) onto the target face ax3.
//
// gp_Trsf::SetTransformation(ax3) creates the LOCAL-to-WORLD transform of ax3,
// i.e. it maps FROM ax3's local frame TO the standard world frame.  Inverting
// gives WORLD-to-ax3, which — when applied to a sketch already in world/standard
// coords — repositions it so that world (0,0,0) lands at ax3.Location() (the
// face centroid) and the sketch plane aligns with the face.
static gp_Trsf sketch_to_face_trsf(const gp_Ax3& face_ax3) {
    gp_Trsf trsf;
    trsf.SetTransformation(face_ax3); // ax3-local → world
    trsf.Invert();                    // world → ax3-local (places sketch on face)
    return trsf;
}

// Small overlap offset used in pad/pocket to avoid the "touching faces" problem.
// BRepAlgoAPI_Fuse/Cut may return a compound of disjoint shapes when inputs only
// touch at a common face boundary.  Offsetting the prism by this amount into the
// body creates proper topological overlap so the boolean always succeeds.
static constexpr double PAD_OVERLAP = 1e-3;

std::unique_ptr<OcctShape> shape_pad(const OcctShape& body, const OcctShape& face_ref,
                                     const OcctShape& sketch, double height) {
    if (face_ref.get().ShapeType() != TopAbs_FACE)
        throw std::runtime_error("pad: face_ref must be a Face");

    const TopoDS_Face& face = TopoDS::Face(face_ref.get());
    gp_Ax3 ax3 = get_face_ax3(face);

    // Transform the sketch from the XY plane onto the target face.
    gp_Trsf trsf = sketch_to_face_trsf(ax3);
    BRepBuilderAPI_Transform xform(sketch.get(), trsf);
    xform.Build();
    if (!xform.IsDone())
        throw std::runtime_error("pad: sketch transform failed");

    // Shift the sketch PAD_OVERLAP into the body so that BRepAlgoAPI_Fuse sees
    // proper topological overlap rather than a boundary-touching situation.
    gp_Dir n = ax3.Direction();
    gp_Trsf overlap_trsf;
    overlap_trsf.SetTranslation(
        gp_Vec(-n.X() * PAD_OVERLAP, -n.Y() * PAD_OVERLAP, -n.Z() * PAD_OVERLAP));
    BRepBuilderAPI_Transform shifted(xform.Shape(), overlap_trsf);
    shifted.Build();

    // Extrude by (height + PAD_OVERLAP) so the top of the pad is at the
    // desired height above the original face.
    gp_Vec extrude_dir(n.X() * (height + PAD_OVERLAP), n.Y() * (height + PAD_OVERLAP),
                       n.Z() * (height + PAD_OVERLAP));
    BRepPrimAPI_MakePrism extruder(shifted.Shape(), extrude_dir);
    extruder.Build();
    if (!extruder.IsDone())
        throw std::runtime_error("pad: BRepPrimAPI_MakePrism failed");

    // Fuse the extruded prism with the body.
    BRepAlgoAPI_Fuse fuser(body.get(), extruder.Shape());
    fuser.Build();
    if (!fuser.IsDone())
        throw std::runtime_error("pad: BRepAlgoAPI_Fuse failed");

    // OCCT 7.6+ may wrap the result in a single-solid Compound; unwrap it.
    return wrap(unwrap_single_solid(fuser.Shape()));
}

std::unique_ptr<OcctShape> shape_pocket(const OcctShape& body, const OcctShape& face_ref,
                                        const OcctShape& sketch, double depth) {
    if (face_ref.get().ShapeType() != TopAbs_FACE)
        throw std::runtime_error("pocket: face_ref must be a Face");

    const TopoDS_Face& face = TopoDS::Face(face_ref.get());
    gp_Ax3 ax3 = get_face_ax3(face);

    // Transform the sketch from the XY plane onto the target face.
    gp_Trsf trsf = sketch_to_face_trsf(ax3);
    BRepBuilderAPI_Transform xform(sketch.get(), trsf);
    xform.Build();
    if (!xform.IsDone())
        throw std::runtime_error("pocket: sketch transform failed");

    // Shift the sketch PAD_OVERLAP outside the body (in +normal direction) so
    // that the pocket tool (prism) extends fully through the face into the body.
    gp_Dir n = ax3.Direction();
    gp_Trsf overlap_trsf;
    overlap_trsf.SetTranslation(
        gp_Vec(n.X() * PAD_OVERLAP, n.Y() * PAD_OVERLAP, n.Z() * PAD_OVERLAP));
    BRepBuilderAPI_Transform shifted(xform.Shape(), overlap_trsf);
    shifted.Build();

    // Extrude along -normal by (depth + PAD_OVERLAP) to reach the desired depth.
    gp_Vec extrude_dir(-n.X() * (depth + PAD_OVERLAP), -n.Y() * (depth + PAD_OVERLAP),
                       -n.Z() * (depth + PAD_OVERLAP));
    BRepPrimAPI_MakePrism extruder(shifted.Shape(), extrude_dir);
    extruder.Build();
    if (!extruder.IsDone())
        throw std::runtime_error("pocket: BRepPrimAPI_MakePrism failed");

    // Cut the extruded tool from the body.
    BRepAlgoAPI_Cut cutter(body.get(), extruder.Shape());
    cutter.Build();
    if (!cutter.IsDone())
        throw std::runtime_error("pocket: BRepAlgoAPI_Cut failed");

    // OCCT 7.6+ may wrap the result in a single-solid Compound; unwrap it.
    return wrap(unwrap_single_solid(cutter.Shape()));
}

std::unique_ptr<OcctShape> shape_fillet_wire(const OcctShape& profile, double radius) {
    const TopoDS_Shape& s = profile.get();

    // Accept either a Face or a Wire; build a planar face from a Wire.
    TopoDS_Face face;
    if (s.ShapeType() == TopAbs_FACE) {
        face = TopoDS::Face(s);
    } else if (s.ShapeType() == TopAbs_WIRE) {
        BRepBuilderAPI_MakeFace mf(TopoDS::Wire(s));
        if (!mf.IsDone())
            throw std::runtime_error("fillet_wire: cannot build a planar face from wire");
        face = mf.Face();
    } else {
        throw std::runtime_error("fillet_wire: profile must be a Wire or Face");
    }

    BRepFilletAPI_MakeFillet2d filler(face);

    // Add a fillet at every vertex; non-corner vertices throw Standard_Failure
    // (e.g. a smooth tangent point) — skip those silently.
    TopTools_IndexedMapOfShape vmap;
    TopExp::MapShapes(face, TopAbs_VERTEX, vmap);
    for (int i = 1; i <= vmap.Extent(); ++i) {
        const TopoDS_Vertex& v = TopoDS::Vertex(vmap(i));
        try {
            filler.AddFillet(v, radius);
        } catch (const Standard_Failure&) {
            // Non-corner vertex — skip.
        }
    }

    filler.Build();
    if (filler.Status() != ChFi2d_IsDone)
        throw std::runtime_error("fillet_wire: BRepFilletAPI_MakeFillet2d failed");

    return wrap(filler.Shape());
}

std::unique_ptr<OcctShape> make_datum_plane(double ox, double oy, double oz, double nx, double ny,
                                            double nz, double xx, double xy, double xz) {
    gp_Ax3 ax3(gp_Pnt(ox, oy, oz), gp_Dir(nx, ny, nz), gp_Dir(xx, xy, xz));
    gp_Pln pln(ax3);
    // Create a finite face: ±50 units in each in-plane direction.
    BRepBuilderAPI_MakeFace mf(pln, -50.0, 50.0, -50.0, 50.0);
    if (!mf.IsDone())
        throw std::runtime_error("datum_plane: BRepBuilderAPI_MakeFace failed");
    return wrap(mf.Shape());
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

// ---------------------------------------------------------------------------
// Phase 8 Tier 3 — Inspection & clearance
// ---------------------------------------------------------------------------

// shape_distance_to — minimum distance between two shapes.
// BRepExtrema_DistShapeShape returns 0 when shapes intersect or touch.
double shape_distance_to(const OcctShape& a, const OcctShape& b) {
    BRepExtrema_DistShapeShape dist(a.get(), b.get());
    dist.Perform();
    if (!dist.IsDone())
        throw std::runtime_error("shape_distance_to: BRepExtrema_DistShapeShape failed");
    return dist.Value();
}

// shape_inertia — inertia tensor about the centre of mass.
// Fills out[] with [Ixx, Iyy, Izz, Ixy, Ixz, Iyz].
// The MatrixOfInertia diagonal is [Ixx, Iyy, Izz]; off-diagonal entries
// are the products of inertia (Ixy = MatrixOfInertia(1,2), etc.).
void shape_inertia(const OcctShape& shape, rust::Slice<double> out) {
    if (out.size() < 6)
        throw std::runtime_error("shape_inertia: output slice must have length >= 6");

    GProp_GProps props;
    // VolumeProperties computes mass (volume) and inertia for solid shapes.
    BRepGProp::VolumeProperties(shape.get(), props);
    gp_Mat m = props.MatrixOfInertia();
    // gp_Mat is 1-indexed; row/col order is [1..3][1..3].
    out[0] = m.Value(1, 1); // Ixx
    out[1] = m.Value(2, 2); // Iyy
    out[2] = m.Value(3, 3); // Izz
    out[3] = m.Value(1, 2); // Ixy
    out[4] = m.Value(1, 3); // Ixz
    out[5] = m.Value(2, 3); // Iyz
}

// shape_min_thickness — minimum wall thickness of a solid or shell.
//
// Strategy A — hollow solid (two or more shells, e.g. from .shell(t)):
//   BRepExtrema_DistShapeShape between the outer and inner shell directly
//   returns the nominal wall thickness t.
//
// Strategy B — single-boundary solid (e.g. a plain box):
//   Binary-search for the largest inward offset δ that
//   BRepOffsetAPI_MakeOffsetShape accepts.  The largest successful δ
//   approximates the inscribed sphere radius (≈ min_thickness for a sphere,
//   ≈ half the shortest dimension for a box).
double shape_min_thickness(const OcctShape& shape) {
    const TopoDS_Shape& s = shape.get();

    if (s.ShapeType() != TopAbs_SOLID && s.ShapeType() != TopAbs_SHELL)
        throw std::runtime_error("min_thickness: shape must be a Solid or Shell");

    // Ray-casting approach: for each face, shoot a ray from its UV-centre along
    // the inward surface normal, intersect with the whole shape, and record the
    // shortest non-trivial intersection distance.  The minimum over all faces is
    // the minimum wall thickness.
    //
    // This works for both hollow solids (.shell(t) gives one connected shell
    // with inner + outer surfaces joined at the rim) and simple solid boxes
    // (ray from each face hits the opposite face at a distance = wall thickness).
    IntCurvesFace_ShapeIntersector inter;
    inter.Load(s, 1e-6);

    double min_t = std::numeric_limits<double>::max();

    TopExp_Explorer fexp(s, TopAbs_FACE);
    for (; fexp.More(); fexp.Next()) {
        const TopoDS_Face& face = TopoDS::Face(fexp.Current());

        // UV mid-point of the face.
        double u1, u2, v1, v2;
        BRepTools::UVBounds(face, u1, u2, v1, v2);
        double um = (u1 + u2) * 0.5, vm = (v1 + v2) * 0.5;

        // Surface normal at (um, vm).
        BRepGProp_Face gpf(face);
        gp_Pnt p;
        gp_Vec n;
        gpf.Normal(um, vm, p, n);
        if (n.Magnitude() < 1e-10)
            continue;
        n.Normalize();

        // The OCCT surface normal points outward for FORWARD-oriented faces.
        // Reverse it to obtain the inward direction (into the material).
        if (face.Orientation() != TopAbs_REVERSED)
            n.Reverse();

        gp_Dir indir(n);
        // Offset the ray origin slightly inward to avoid self-intersection.
        gp_Pnt origin = p.Translated(gp_Vec(indir) * 1e-4);
        gp_Lin ray(origin, indir);

        inter.Perform(ray, 0.0, 1e6);
        for (int i = 1; i <= inter.NbPnt(); ++i) {
            double t = inter.WParameter(i);
            // Ignore hits within 1e-3 (numerical noise / same face self-hit).
            if (t > 1e-3 && t < min_t)
                min_t = t;
        }
    }

    if (min_t == std::numeric_limits<double>::max())
        throw std::runtime_error(
            "min_thickness: could not compute — shape may be open or degenerate");

    return min_t;
}

// ---------------------------------------------------------------------------
// Phase 8 Tier 2 — Manufacturing features
// ---------------------------------------------------------------------------

// shape_extrude_draft — extrude a 2D profile then apply a draft angle to all
// lateral (non-Z-normal) faces so the solid tapers from base to top.
//
// Strategy:
//   1. Straight prism via BRepPrimAPI_MakePrism (same as shape_extrude).
//   2. Walk faces; skip the top/bottom (|normal · Z| > 0.5); apply
//      BRepOffsetAPI_DraftAngle to each lateral planar face.
//   3. Build — returns the tapered solid.
//
// Neutral plane = XY at Z=0 (the base of the extrusion) so base edges
// stay fixed and the top edges move inward.
// draft_deg > 0 → standard mould taper (narrower at top).
std::unique_ptr<OcctShape> shape_extrude_draft(const OcctShape& profile, double height,
                                               double draft_deg) {
    if (draft_deg == 0.0)
        return shape_extrude(profile, height);

    // Step 1: straight prism.
    BRepPrimAPI_MakePrism prism_builder(profile.get(), gp_Vec(0, 0, height));
    prism_builder.Build();
    if (!prism_builder.IsDone())
        throw std::runtime_error("shape_extrude_draft: BRepPrimAPI_MakePrism failed");
    TopoDS_Shape solid = prism_builder.Shape();

    // Step 2: add draft to each lateral planar face.
    BRepOffsetAPI_DraftAngle drafter(solid);
    gp_Dir pull_dir(0, 0, 1);
    double angle_rad = draft_deg * M_PI / 180.0;
    // Neutral plane: the XY plane at Z=0 anchors the base edges.
    gp_Pln neutral_plane(gp_Ax3(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));

    bool any_added = false;
    TopExp_Explorer exp(solid, TopAbs_FACE);
    for (; exp.More(); exp.Next()) {
        TopoDS_Face face = TopoDS::Face(exp.Current());
        BRepAdaptor_Surface surf(face);
        if (surf.GetType() != GeomAbs_Plane)
            continue; // skip non-planar faces (e.g. fillet arcs)
        gp_Dir face_normal = surf.Plane().Axis().Direction();
        // Skip top/bottom: their normal is nearly parallel to the pull direction.
        if (std::abs(face_normal.Dot(pull_dir)) > 0.5)
            continue;
        drafter.Add(face, pull_dir, angle_rad, neutral_plane);
        any_added = true;
    }

    if (!any_added)
        throw std::runtime_error("shape_extrude_draft: no lateral planar faces found — "
                                 "profile may already be 3-D or have no straight edges");

    drafter.Build();
    if (!drafter.IsDone())
        throw std::runtime_error("shape_extrude_draft: BRepOffsetAPI_DraftAngle failed");

    return wrap(drafter.Shape());
}

// make_helix — helical Wire path built from a dense BSpline interpolation.
//
// Parametric form: x(t) = radius*cos(t), y(t) = radius*sin(t),
//                  z(t) = pitch * t / (2π)
// for t in [0, 2π * (height/pitch)].
//
// 32 sample points per full turn give a smooth enough curve for thread
// profiles at any practical pitch/radius combination.
std::unique_ptr<OcctShape> make_helix(double radius, double pitch, double height) {
    if (radius <= 0.0)
        throw std::runtime_error("helix: radius must be > 0");
    if (pitch <= 0.0)
        throw std::runtime_error("helix: pitch must be > 0");
    if (height <= 0.0)
        throw std::runtime_error("helix: height must be > 0");

    double n_turns = height / pitch;
    // 16 samples per turn is sufficient for thread-profile sweeps and keeps
    // the BSpline degree low enough that BRepOffsetAPI_MakePipe stays stable.
    // Cap at 512 total points (≥32 turns at 16/turn) to avoid OCCT internal
    // limits on very long helices.
    int n_pts = std::max(3, static_cast<int>(n_turns * 16.0) + 2);
    if (n_pts > 512)
        n_pts = 512;

    Handle(TColgp_HArray1OfPnt) hPts = new TColgp_HArray1OfPnt(1, n_pts);
    for (int i = 0; i < n_pts; i++) {
        double t = (2.0 * M_PI * n_turns * i) / (n_pts - 1);
        double z = height * i / (n_pts - 1);
        hPts->SetValue(i + 1, gp_Pnt(radius * std::cos(t), radius * std::sin(t), z));
    }

    GeomAPI_Interpolate interp(hPts, /*isPeriodic=*/Standard_False, /*Tolerance=*/1e-6);
    interp.Perform();
    if (!interp.IsDone())
        throw std::runtime_error("make_helix: GeomAPI_Interpolate failed");

    Handle(Geom_BSplineCurve) curve = interp.Curve();
    TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(curve).Edge();
    TopoDS_Wire wire = BRepBuilderAPI_MakeWire(edge).Wire();
    return wrap(wire);
}

// ---------------------------------------------------------------------------
// Phase 8 Tier 4 — 2-D drawing output (SVG + DXF)
//
// Both functions share the same HLR pipeline:
//   1. Tessellate the shape (required by HLRBRep_PolyAlgo).
//   2. Set an orthographic projector for the requested view direction.
//   3. Run HLRBRep_PolyAlgo::Update() to compute visible/silhouette edges.
//   4. Discretise each projected edge into a polyline.
//
// The output edges from HLRBRep_PolyHLRToShape are in the HLR view plane
// (Z = 0); their X and Y values are the 2-D drawing coordinates.
//
// View conventions (matching standard engineering drawing orientation):
//   "top"   — gp_Ax2 with Z-dir = (0,0,1), X-dir = (1,0,0)
//             → 2-D coords are (world.X, world.Y)
//   "front" — gp_Ax2 with Z-dir = (0,-1,0), X-dir = (1,0,0)
//             → 2-D coords are (world.X, world.Z)
//   "side"  — gp_Ax2 with Z-dir = (1,0,0), X-dir = (0,1,0)
//             → 2-D coords are (world.Y, world.Z)
// ---------------------------------------------------------------------------

// Number of parametric samples per projected edge when building polylines.
// 32 gives smooth curves while keeping SVG/DXF files compact.
static const int HLR_SAMPLES_PER_EDGE = 32;

// Project the shape onto the chosen view plane, return all visible polylines
// as a vector of (x, y) point lists.
static std::vector<std::vector<std::pair<double, double>>> hlr_project(const OcctShape& shape,
                                                                       const std::string& view) {
    // Tessellate (required before loading into PolyAlgo).
    BRepMesh_IncrementalMesh mesher(shape.get(), 0.05, false, 0.5, true);
    mesher.Perform();

    // Build orthographic projector for the requested view.
    gp_Ax2 cs;
    if (view == "front") {
        // Looking along −Y; X→right in drawing, Z→up in drawing.
        cs = gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, -1, 0), gp_Dir(1, 0, 0));
    } else if (view == "side") {
        // Looking along +X; Y→right in drawing, Z→up in drawing.
        cs = gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0), gp_Dir(0, 1, 0));
    } else {
        // "top" (default): looking along −Z; X→right, Y→up in drawing.
        cs = gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1), gp_Dir(1, 0, 0));
    }

    Handle(HLRBRep_PolyAlgo) algo = new HLRBRep_PolyAlgo();
    algo->Projector(HLRAlgo_Projector(cs));
    algo->Load(shape.get());
    algo->Update();

    HLRBRep_PolyHLRToShape gen;
    gen.Update(algo);

    // Collect projected edges: sharp visible + silhouette outline.
    std::vector<std::vector<std::pair<double, double>>> polylines;

    auto process_compound = [&](const TopoDS_Shape& compound) {
        if (compound.IsNull())
            return;
        TopExp_Explorer eexp(compound, TopAbs_EDGE);
        for (; eexp.More(); eexp.Next()) {
            BRepAdaptor_Curve curve(TopoDS::Edge(eexp.Current()));
            double t0 = curve.FirstParameter();
            double t1 = curve.LastParameter();
            if (t1 <= t0)
                continue;

            std::vector<std::pair<double, double>> pts;
            pts.reserve(HLR_SAMPLES_PER_EDGE + 1);
            for (int i = 0; i <= HLR_SAMPLES_PER_EDGE; ++i) {
                double t = t0 + (t1 - t0) * i / HLR_SAMPLES_PER_EDGE;
                gp_Pnt p = curve.Value(t);
                pts.emplace_back(p.X(), p.Y());
            }
            polylines.push_back(std::move(pts));
        }
    };

    process_compound(gen.VCompound());
    process_compound(gen.OutLineVCompound());

    if (polylines.empty())
        throw std::runtime_error("export_svg/dxf: no visible edges found — "
                                 "shape may be degenerate or face the wrong direction");
    return polylines;
}

// ---------------------------------------------------------------------------
// SVG export
// ---------------------------------------------------------------------------
void export_svg(const OcctShape& shape, rust::Str path, rust::Str view) {
    std::string path_str(path.data(), path.size());
    std::string view_str(view.data(), view.size());

    auto polylines = hlr_project(shape, view_str);

    // Compute axis-aligned bounding box in drawing coordinates.
    double xmin = 1e30, xmax = -1e30, ymin = 1e30, ymax = -1e30;
    for (auto& pl : polylines) {
        for (auto& [x, y] : pl) {
            xmin = std::min(xmin, x);
            xmax = std::max(xmax, x);
            ymin = std::min(ymin, y);
            ymax = std::max(ymax, y);
        }
    }

    const double margin = 5.0;
    double w = (xmax - xmin) + 2.0 * margin;
    double h = (ymax - ymin) + 2.0 * margin;
    // SVG viewBox origin: left edge = xmin−margin, top edge = −(ymax+margin)
    // (SVG Y increases downward; drawing Y increases upward — hence the negation).
    double vb_x = xmin - margin;
    double vb_y = -(ymax + margin);

    std::ofstream f(path_str);
    if (!f.is_open())
        throw std::runtime_error("export_svg: cannot open file: " + path_str);

    f << std::fixed << std::setprecision(4);
    f << "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
    f << "<svg xmlns=\"http://www.w3.org/2000/svg\"";
    f << " width=\"" << w << "\" height=\"" << h << "\"";
    f << " viewBox=\"" << vb_x << " " << vb_y << " " << w << " " << h << "\">\n";
    f << "  <!-- Generated by rrcad — view: " << view_str << " -->\n";
    f << "  <g stroke=\"black\" stroke-width=\"0.3\" fill=\"none\"";
    f << " stroke-linecap=\"round\" stroke-linejoin=\"round\">\n";

    for (auto& pts : polylines) {
        if (pts.size() < 2)
            continue;
        f << "    <polyline points=\"";
        for (auto& [x, y] : pts) {
            // Negate Y: drawing Y-up → SVG Y-down.
            f << x << "," << -y << " ";
        }
        f << "\"/>\n";
    }

    f << "  </g>\n</svg>\n";
    if (!f.good())
        throw std::runtime_error("export_svg: write error on file: " + path_str);
}

// ---------------------------------------------------------------------------
// DXF export (ASCII DXF R12 — universally supported)
//
// Each polyline segment is written as a LINE entity.  DXF uses Y-up
// coordinates (standard math / CAD convention) so no Y-flip is applied.
// ---------------------------------------------------------------------------
void export_dxf(const OcctShape& shape, rust::Str path, rust::Str view) {
    std::string path_str(path.data(), path.size());
    std::string view_str(view.data(), view.size());

    auto polylines = hlr_project(shape, view_str);

    std::ofstream f(path_str);
    if (!f.is_open())
        throw std::runtime_error("export_dxf: cannot open file: " + path_str);

    f << std::fixed << std::setprecision(6);

    // Minimal DXF R12 header.
    f << "  0\nSECTION\n  2\nHEADER\n";
    f << "  9\n$ACADVER\n  1\nAC1009\n"; // AutoCAD R12
    f << "  0\nENDSEC\n";

    // ENTITIES section: one LINE entity per polyline segment.
    f << "  0\nSECTION\n  2\nENTITIES\n";

    for (auto& pts : polylines) {
        for (std::size_t i = 0; i + 1 < pts.size(); ++i) {
            auto [x1, y1] = pts[i];
            auto [x2, y2] = pts[i + 1];
            // Skip zero-length degenerate segments.
            if (std::abs(x2 - x1) < 1e-9 && std::abs(y2 - y1) < 1e-9)
                continue;
            f << "  0\nLINE\n";
            f << "  8\n0\n"; // layer "0"
            f << " 10\n" << x1 << "\n";
            f << " 20\n" << y1 << "\n";
            f << " 30\n0.0\n"; // Z1 = 0
            f << " 11\n" << x2 << "\n";
            f << " 21\n" << y2 << "\n";
            f << " 31\n0.0\n"; // Z2 = 0
        }
    }

    f << "  0\nENDSEC\n  0\nEOF\n";
    if (!f.good())
        throw std::runtime_error("export_dxf: write error on file: " + path_str);
}

// ---------------------------------------------------------------------------
// Phase 8 Tier 5: Advanced composition
// ---------------------------------------------------------------------------

// --- fragment builder -------------------------------------------------------

struct FragmentBuilder::Impl {
    TopTools_ListOfShape shapes;
};

FragmentBuilder::FragmentBuilder() : impl(std::make_unique<Impl>()) {}
FragmentBuilder::~FragmentBuilder() = default;

std::unique_ptr<FragmentBuilder> fragment_new() { return std::make_unique<FragmentBuilder>(); }

void fragment_add(FragmentBuilder& builder, const OcctShape& shape) {
    builder.impl->shapes.Append(shape.get());
}

std::unique_ptr<OcctShape> fragment_build(FragmentBuilder& builder) {
    if (builder.impl->shapes.IsEmpty())
        throw std::runtime_error("fragment: no shapes added");

    // BRepAlgoAPI_BuilderAlgo requires at least 2 shapes.  For a single shape,
    // just wrap it in a compound and return immediately.
    if (builder.impl->shapes.Size() == 1) {
        TopoDS_Compound compound;
        BRep_Builder bb;
        bb.MakeCompound(compound);
        bb.Add(compound, builder.impl->shapes.First());
        return wrap(compound);
    }

    BRepAlgoAPI_BuilderAlgo algo;
    algo.SetArguments(builder.impl->shapes);
    algo.Build();
    if (!algo.IsDone())
        throw std::runtime_error("BRepAlgoAPI_BuilderAlgo (fragment) failed");

    return wrap(algo.Shape());
}

// --- convex_hull ------------------------------------------------------------
//
// 3-D incremental convex hull (QuickHull variant).
// All internal helpers live in an anonymous namespace so they do not pollute
// the rrcad namespace.

namespace {

struct CHPt {
    double x, y, z;
};

struct CHFace {
    int a, b, c;
    bool dead; // true when the face has been removed during expansion
};

// Signed volume (×6) of the tetrahedron (pa, pb, pc, pp).
// Positive means pp is on the outward side of the face (cross(pb−pa, pc−pa)).
static double ch_signed_vol(const CHPt& pa, const CHPt& pb, const CHPt& pc,
                            const CHPt& pp) noexcept {
    double ax = pb.x - pa.x, ay = pb.y - pa.y, az = pb.z - pa.z;
    double bx = pc.x - pa.x, by = pc.y - pa.y, bz = pc.z - pa.z;
    double nx = ay * bz - az * by;
    double ny = az * bx - ax * bz;
    double nz = ax * by - ay * bx;
    return nx * (pp.x - pa.x) + ny * (pp.y - pa.y) + nz * (pp.z - pa.z);
}

// Build the convex hull from a set of points and return a vector of oriented
// outward-facing triangles (by index into `pts`).
static std::vector<CHFace> build_convex_hull(const std::vector<CHPt>& pts) {
    int n = static_cast<int>(pts.size());
    if (n < 4)
        throw std::runtime_error("convex_hull: need at least 4 non-coplanar points");

    // 1. Find extreme points: min-X and max-X.
    int i0 = 0, i1 = 0;
    for (int i = 1; i < n; i++) {
        if (pts[i].x < pts[i0].x)
            i0 = i;
        if (pts[i].x > pts[i1].x)
            i1 = i;
    }
    if (i0 == i1) {
        // Fallback: try Y axis
        i0 = 0;
        i1 = 0;
        for (int i = 1; i < n; i++) {
            if (pts[i].y < pts[i0].y)
                i0 = i;
            if (pts[i].y > pts[i1].y)
                i1 = i;
        }
    }

    // 2. Find point i2 farthest from the line(i0, i1).
    double dx = pts[i1].x - pts[i0].x;
    double dy = pts[i1].y - pts[i0].y;
    double dz = pts[i1].z - pts[i0].z;
    double best2 = -1.0;
    int i2 = -1;
    for (int i = 0; i < n; i++) {
        if (i == i0 || i == i1)
            continue;
        double ex = pts[i].x - pts[i0].x;
        double ey = pts[i].y - pts[i0].y;
        double ez = pts[i].z - pts[i0].z;
        // |d × e|² = distance² from line × |d|²
        double cx = dy * ez - dz * ey;
        double cy = dz * ex - dx * ez;
        double cz = dx * ey - dy * ex;
        double dist2 = cx * cx + cy * cy + cz * cz;
        if (dist2 > best2) {
            best2 = dist2;
            i2 = i;
        }
    }
    if (i2 < 0 || best2 < 1e-20)
        throw std::runtime_error("convex_hull: all points are collinear");

    // 3. Find point i3 farthest from the plane(i0, i1, i2).
    double best3 = -1.0;
    int i3 = -1;
    for (int i = 0; i < n; i++) {
        if (i == i0 || i == i1 || i == i2)
            continue;
        double sv = std::fabs(ch_signed_vol(pts[i0], pts[i1], pts[i2], pts[i]));
        if (sv > best3) {
            best3 = sv;
            i3 = i;
        }
    }
    if (i3 < 0 || best3 < 1e-20)
        throw std::runtime_error("convex_hull: all points are coplanar");

    // 4. Centroid of the initial tetrahedron (always interior to the hull).
    CHPt interior{(pts[i0].x + pts[i1].x + pts[i2].x + pts[i3].x) / 4.0,
                  (pts[i0].y + pts[i1].y + pts[i2].y + pts[i3].y) / 4.0,
                  (pts[i0].z + pts[i1].z + pts[i2].z + pts[i3].z) / 4.0};

    // 5. Build the initial 4 faces; orient each so that `interior` (and the
    //    4th vertex) is on the negative side.
    auto make_face = [&](int a, int b, int c) -> CHFace {
        if (ch_signed_vol(pts[a], pts[b], pts[c], interior) > 0)
            std::swap(b, c);
        return {a, b, c, false};
    };

    std::vector<CHFace> faces = {
        make_face(i0, i1, i2),
        make_face(i0, i1, i3),
        make_face(i0, i2, i3),
        make_face(i1, i2, i3),
    };

    // 6. Incrementally expand the hull.
    for (int i = 0; i < n; i++) {
        if (i == i0 || i == i1 || i == i2 || i == i3)
            continue;
        const CHPt& p = pts[i];

        // Collect indices of faces visible from p (p is on their outward side).
        std::vector<int> visible;
        for (int fi = 0; fi < static_cast<int>(faces.size()); fi++) {
            if (faces[fi].dead)
                continue;
            if (ch_signed_vol(pts[faces[fi].a], pts[faces[fi].b], pts[faces[fi].c], p) > 1e-12)
                visible.push_back(fi);
        }
        if (visible.empty())
            continue; // p is inside the current hull

        // Build a set of all directed edges from visible faces.
        std::set<std::pair<int, int>> vis_edges;
        for (int fi : visible) {
            auto& f = faces[fi];
            vis_edges.insert({f.a, f.b});
            vis_edges.insert({f.b, f.c});
            vis_edges.insert({f.c, f.a});
        }

        // Horizon: directed edge (a,b) whose reverse (b,a) is NOT in a visible face.
        std::vector<std::pair<int, int>> horizon;
        for (auto& e : vis_edges) {
            if (vis_edges.find({e.second, e.first}) == vis_edges.end())
                horizon.push_back(e);
        }

        // Mark visible faces as dead.
        for (int fi : visible)
            faces[fi].dead = true;

        // Add new faces connecting horizon edges to p.
        // Orient each new face so that interior is on the negative side.
        for (auto& e : horizon)
            faces.push_back(make_face(e.first, e.second, i));
    }

    return faces;
}

} // anonymous namespace

std::unique_ptr<OcctShape> shape_convex_hull(const OcctShape& shape) {
    // Tessellate the shape.
    BRepMesh_IncrementalMesh mesh(shape.get(), 0.5);
    (void)mesh;

    // Collect all mesh vertices (with location transforms applied).
    std::vector<CHPt> pts;
    for (TopExp_Explorer ex(shape.get(), TopAbs_FACE); ex.More(); ex.Next()) {
        const TopoDS_Face& face = TopoDS::Face(ex.Current());
        TopLoc_Location loc;
        Handle(Poly_Triangulation) tri = BRep_Tool::Triangulation(face, loc);
        if (tri.IsNull())
            continue;
        gp_Trsf trsf;
        if (!loc.IsIdentity())
            trsf = loc.Transformation();
        for (int i = 1; i <= tri->NbNodes(); i++) {
            gp_Pnt pnt = tri->Node(i).Transformed(trsf);
            pts.push_back({pnt.X(), pnt.Y(), pnt.Z()});
        }
    }

    if (pts.size() < 4)
        throw std::runtime_error("convex_hull: shape has fewer than 4 mesh vertices");

    auto hull_faces = build_convex_hull(pts);

    // Build BRep solid from hull triangles using sewing.
    BRepBuilderAPI_Sewing sewing(1e-6);
    for (auto& f : hull_faces) {
        if (f.dead)
            continue;
        gp_Pnt P0(pts[f.a].x, pts[f.a].y, pts[f.a].z);
        gp_Pnt P1(pts[f.b].x, pts[f.b].y, pts[f.b].z);
        gp_Pnt P2(pts[f.c].x, pts[f.c].y, pts[f.c].z);

        BRepBuilderAPI_MakePolygon poly;
        poly.Add(P0);
        poly.Add(P1);
        poly.Add(P2);
        poly.Close();
        if (!poly.IsDone())
            continue;

        BRepBuilderAPI_MakeFace face_maker(poly.Wire(), true);
        if (!face_maker.IsDone())
            continue;
        sewing.Add(face_maker.Face());
    }

    sewing.Perform();
    TopoDS_Shape sewn = sewing.SewedShape();

    // Attempt to close into a solid.
    BRepBuilderAPI_MakeSolid solid_maker;
    for (TopExp_Explorer ex(sewn, TopAbs_SHELL); ex.More(); ex.Next())
        solid_maker.Add(TopoDS::Shell(ex.Current()));
    if (solid_maker.IsDone())
        return wrap(solid_maker.Solid());
    return wrap(sewn);
}

// --- path_pattern -----------------------------------------------------------

std::unique_ptr<OcctShape> shape_path_pattern(const OcctShape& shape, const OcctShape& path,
                                              int32_t n) {
    if (n < 1)
        throw std::runtime_error("path_pattern: n must be >= 1");

    // Convert path to a Wire (accept Wire or bare Edge).
    const TopoDS_Shape& psh = path.get();
    TopoDS_Wire path_wire;
    if (psh.ShapeType() == TopAbs_WIRE) {
        path_wire = TopoDS::Wire(psh);
    } else if (psh.ShapeType() == TopAbs_EDGE) {
        BRepBuilderAPI_MakeWire mw(TopoDS::Edge(psh));
        if (!mw.IsDone())
            throw std::runtime_error("path_pattern: failed to convert Edge to Wire");
        path_wire = mw.Wire();
    } else {
        throw std::runtime_error("path_pattern: path must be a Wire or Edge");
    }

    BRepAdaptor_CompCurve adaptor(path_wire, /*KnotByCurvilinearAbcissa=*/false);
    double t0 = adaptor.FirstParameter();
    double t1 = adaptor.LastParameter();

    // Collect n arc-length-evenly-spaced parameter values.
    std::vector<double> params(n);
    if (n == 1) {
        params[0] = t0;
    } else {
        GCPnts_UniformAbscissa splitter;
        splitter.Initialize(adaptor, n, t0, t1);
        if (!splitter.IsDone())
            throw std::runtime_error("path_pattern: GCPnts_UniformAbscissa failed");
        for (int i = 0; i < n; i++)
            params[i] = splitter.Parameter(i + 1); // 1-indexed
    }

    // Build compound of n oriented copies.
    TopoDS_Compound compound;
    BRep_Builder bb;
    bb.MakeCompound(compound);

    // The shape's canonical "up" direction (local Z-axis to align with the tangent).
    gp_Dir z_axis(0.0, 0.0, 1.0);

    for (int i = 0; i < n; i++) {
        gp_Pnt pnt;
        gp_Vec tangent;
        adaptor.D1(params[i], pnt, tangent);
        if (tangent.Magnitude() < 1e-10)
            tangent = gp_Vec(0.0, 0.0, 1.0);
        tangent.Normalize();

        gp_Dir tang_dir(tangent);
        double cos_angle = tang_dir.Dot(z_axis);

        // Rotation that maps Z → tangent.
        gp_Trsf rot_trsf; // identity by default
        if (std::fabs(cos_angle + 1.0) < 1e-9) {
            // Antiparallel (tangent ≈ −Z): rotate 180° around X.
            rot_trsf.SetRotation(gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0)), M_PI);
        } else if (std::fabs(cos_angle - 1.0) > 1e-9) {
            // General case: rotate around cross(Z, tangent).
            gp_Vec cross = gp_Vec(z_axis).Crossed(tangent);
            if (cross.Magnitude() > 1e-10) {
                double angle = std::acos(std::max(-1.0, std::min(1.0, cos_angle)));
                rot_trsf.SetRotation(gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(cross)), angle);
            }
        }

        // Translation to the sample point on the path.
        gp_Trsf trans_trsf;
        trans_trsf.SetTranslation(gp_Vec(pnt.X(), pnt.Y(), pnt.Z()));

        // Compose: first rotate (align Z→tangent), then translate.
        gp_Trsf combined = trans_trsf;
        combined.Multiply(rot_trsf);

        TopoDS_Shape copy = BRepBuilderAPI_Transform(shape.get(), combined, /*copy=*/true).Shape();
        bb.Add(compound, copy);
    }

    return wrap(compound);
}

// --- sweep_guide ------------------------------------------------------------

std::unique_ptr<OcctShape> shape_sweep_guide(const OcctShape& profile, const OcctShape& path,
                                             const OcctShape& guide) {
    if (path.get().ShapeType() != TopAbs_WIRE)
        throw std::runtime_error("sweep_guide: path must be a Wire (created with spline_3d)");
    if (guide.get().ShapeType() != TopAbs_WIRE)
        throw std::runtime_error("sweep_guide: guide must be a Wire (created with spline_3d)");

    TopoDS_Wire spine = TopoDS::Wire(path.get());
    TopoDS_Wire aux = TopoDS::Wire(guide.get());

    BRepOffsetAPI_MakePipeShell pipe(spine);
    // Auxiliary spine mode: the profile's local X-axis tracks the guide wire.
    // CurvilinearEquivalence=true, contact type=BRepFill_ContactAtX.
    pipe.SetMode(aux, Standard_True, BRepFill_Contact);

    // Extract a Wire from the profile (accept Face or Wire).
    TopoDS_Wire profile_wire;
    if (profile.get().ShapeType() == TopAbs_FACE) {
        profile_wire = BRepTools::OuterWire(TopoDS::Face(profile.get()));
    } else if (profile.get().ShapeType() == TopAbs_WIRE) {
        profile_wire = TopoDS::Wire(profile.get());
    } else {
        throw std::runtime_error("sweep_guide: profile must be a Wire or Face");
    }

    pipe.Add(profile_wire);
    pipe.Build();
    if (!pipe.IsDone())
        throw std::runtime_error(
            "BRepOffsetAPI_MakePipeShell (sweep_guide) failed — check that path and guide are "
            "compatible Wires");

    pipe.MakeSolid();
    return wrap(pipe.Shape());
}

} // namespace rrcad
