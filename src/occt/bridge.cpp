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
#include <sstream>
#include <string>
#include <utility>

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
#include <GCPnts_QuasiUniformDeflection.hxx>
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
#include <ShapeFix_Face.hxx>
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

#include <algorithm>
#include <atomic>
#include <cmath>
#include <filesystem>
#include <stdexcept>
#include <string>
#include <unistd.h>
#include <vector>

namespace rrcad {

namespace {

std::filesystem::path atomic_export_temp_path(const std::string& final_path) {
    static std::atomic<unsigned long long> seq{1};
    std::filesystem::path final(final_path);
    auto id = seq.fetch_add(1, std::memory_order_relaxed);
    auto pid = static_cast<unsigned long long>(::getpid());
    auto name = final.stem().string() + ".rrcad-tmp." + std::to_string(pid) + "." +
                std::to_string(id) + final.extension().string();
    return final.parent_path() / name;
}

void rename_export_artifact(const std::filesystem::path& from, const std::filesystem::path& to) {
    if (std::filesystem::exists(from)) {
        std::filesystem::rename(from, to);
    }
}

} // namespace

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> make_box(double dx, double dy, double dz) {
    try {
        BRepPrimAPI_MakeBox builder(dx, dy, dz);
        builder.Build();
        if (!builder.IsDone())
            throw std::runtime_error("BRepPrimAPI_MakeBox failed");
        return wrap(builder.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_cylinder(double radius, double height) {
    try {
        BRepPrimAPI_MakeCylinder builder(radius, height);
        builder.Build();
        if (!builder.IsDone())
            throw std::runtime_error("BRepPrimAPI_MakeCylinder failed");
        return wrap(builder.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_sphere(double radius) {
    try {
        BRepPrimAPI_MakeSphere builder(radius);
        builder.Build();
        if (!builder.IsDone())
            throw std::runtime_error("BRepPrimAPI_MakeSphere failed");
        return wrap(builder.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_cone(double r1, double r2, double height) {
    try {
        BRepPrimAPI_MakeCone builder(r1, r2, height);
        builder.Build();
        if (!builder.IsDone())
            throw std::runtime_error("BRepPrimAPI_MakeCone failed");
        return wrap(builder.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_torus(double r1, double r2) {
    try {
        BRepPrimAPI_MakeTorus builder(r1, r2);
        builder.Build();
        if (!builder.IsDone())
            throw std::runtime_error("BRepPrimAPI_MakeTorus failed");
        return wrap(builder.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_wedge(double dx, double dy, double dz, double ltx) {
    try {
        BRepPrimAPI_MakeWedge builder(dx, dy, dz, ltx);
        builder.Build();
        if (!builder.IsDone())
            throw std::runtime_error("BRepPrimAPI_MakeWedge failed");
        return wrap(builder.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Color

std::unique_ptr<OcctShape> shape_set_color(const OcctShape& shape, double r, double g, double b) {
    try {
        // Return a new OcctShape wrapping the same BRep topology (cheap — TopoDS
        // uses handle-based reference counting) with the sRGB color tag attached.
        return wrap_colored(shape.get(), static_cast<float>(r), static_cast<float>(g),
                            static_cast<float>(b));
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_copy(const OcctShape& shape) {
    try {
        if (shape.has_color())
            return wrap_colored(shape.get(), shape.color_r(), shape.color_g(), shape.color_b());
        return wrap(shape.get());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Assembly mating
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_mate(const OcctShape& shape, const OcctShape& from_face_shape,
                                      const OcctShape& to_face_shape, double offset) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Boolean operations
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_fuse(const OcctShape& a, const OcctShape& b) {
    try {
        // Builder-style API: explicit args/tools, fuzzy tolerance for near-coincident
        // faces, and TBB parallel evaluation (OCCT 7.4+).
        TopTools_ListOfShape args, tools;
        args.Append(a.get());
        tools.Append(b.get());
        BRepAlgoAPI_Fuse op;
        op.SetArguments(args);
        op.SetTools(tools);
        // SetRunParallel: use TBB thread pool for sub-operations (OCCT 7.4+).
        // SetFuzzyValue: merge vertices/edges within 1e-6 mm to tolerate near-coincident
        // geometry that would otherwise leave gap artefacts.
        op.SetRunParallel(Standard_True);
        op.SetFuzzyValue(1e-6);
        op.Build();
        if (!op.IsDone())
            throw std::runtime_error("BRepAlgoAPI_Fuse failed");
        return wrap(op.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_cut(const OcctShape& a, const OcctShape& b) {
    try {
        TopTools_ListOfShape args, tools;
        args.Append(a.get());
        tools.Append(b.get());
        BRepAlgoAPI_Cut op;
        op.SetArguments(args);
        op.SetTools(tools);
        // Same parallel + fuzzy settings as shape_fuse — see comments there.
        op.SetRunParallel(Standard_True);
        op.SetFuzzyValue(1e-6);
        op.Build();
        if (!op.IsDone())
            throw std::runtime_error("BRepAlgoAPI_Cut failed");
        return wrap(op.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_common(const OcctShape& a, const OcctShape& b) {
    try {
        TopTools_ListOfShape args, tools;
        args.Append(a.get());
        tools.Append(b.get());
        BRepAlgoAPI_Common op;
        op.SetArguments(args);
        op.SetTools(tools);
        // Same parallel + fuzzy settings as shape_fuse — see comments there.
        op.SetRunParallel(Standard_True);
        op.SetFuzzyValue(1e-6);
        op.Build();
        if (!op.IsDone())
            throw std::runtime_error("BRepAlgoAPI_Common failed");
        return wrap(op.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Fillets and chamfers
// ---------------------------------------------------------------------------

// Forward declaration — collect_edges is defined in the selectors section below.
static std::vector<TopoDS_Edge> collect_edges(const OcctShape& shape, const std::string& sel);

std::unique_ptr<OcctShape> shape_fillet(const OcctShape& s, double radius) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_chamfer(const OcctShape& s, double dist) {
    try {
        BRepFilletAPI_MakeChamfer builder(s.get());

        TopExp_Explorer exp(s.get(), TopAbs_EDGE);
        for (; exp.More(); exp.Next()) {
            builder.Add(dist, TopoDS::Edge(exp.Current()));
        }

        builder.Build();
        if (!builder.IsDone())
            throw std::runtime_error("BRepFilletAPI_MakeChamfer failed");
        return wrap(builder.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Selective fillet: only edges matching the edge selector are rounded.
// Reuses collect_edges for deduplication and validation.
std::unique_ptr<OcctShape> shape_fillet_sel(const OcctShape& s, double radius, rust::Str selector) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Selective chamfer: only edges matching the edge selector are bevelled.
std::unique_ptr<OcctShape> shape_chamfer_sel(const OcctShape& s, double dist, rust::Str selector) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Variable-radius fillet: each edge gets a radius that transitions linearly
// from r1 at one end-vertex to r2 at the other.
// BRepFilletAPI_MakeFillet::Add(r1, r2, edge) accepts two radii directly;
// OCCT interpolates smoothly along the edge.
std::unique_ptr<OcctShape> shape_fillet_var(const OcctShape& s, double r1, double r2) {
    try {
        BRepFilletAPI_MakeFillet builder(s.get());

        TopExp_Explorer exp(s.get(), TopAbs_EDGE);
        for (; exp.More(); exp.Next())
            builder.Add(r1, r2, TopoDS::Edge(exp.Current()));

        builder.Build();
        if (!builder.IsDone())
            throw std::runtime_error("BRepFilletAPI_MakeFillet (variable-radius) failed — "
                                     "check for degenerate edges or radii too large");
        return wrap(builder.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_fillet_var_sel(const OcctShape& s, double r1, double r2,
                                                rust::Str selector) {
    try {
        std::string sel(selector.data(), selector.size());
        auto edges = collect_edges(s, sel);
        if (edges.empty())
            throw std::runtime_error("fillet: no edges match selector ':" + sel + "'");

        BRepFilletAPI_MakeFillet builder(s.get());
        for (const auto& edge : edges)
            builder.Add(r1, r2, edge);

        builder.Build();
        if (!builder.IsDone())
            throw std::runtime_error(
                "BRepFilletAPI_MakeFillet (variable-radius, selective) failed — "
                "check for degenerate edges or radii too large");
        return wrap(builder.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Asymmetric chamfer: d1 and d2 are the two bevel distances on each side of the edge.
// OCCT's BRepFilletAPI_MakeChamfer::Add(d1, d2, edge, face) requires a reference face
// to indicate on which side d1 applies.  We build an edge→adjacent-face map and use
// the first adjacent face for every edge.
std::unique_ptr<OcctShape> shape_chamfer_asym(const OcctShape& s, double d1, double d2) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Selective asymmetric chamfer: only edges matching the selector are bevelled.
std::unique_ptr<OcctShape> shape_chamfer_asym_sel(const OcctShape& s, double d1, double d2,
                                                  rust::Str selector) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Bézier surface patch
// ---------------------------------------------------------------------------

// Build a single bicubic Bézier face from 16 control points (4×4 grid).
// `pts` is a flat array of 48 doubles: 16 points × (x, y, z) in row-major
// order (row 0 = first parameter direction, col 0 = second).
std::unique_ptr<OcctShape> make_bezier_patch(rust::Slice<const double> pts) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    // OCCT Standard_Failure does not derive std::exception; without this guard an
    // escaping OCCT exception would abort the process at the cxx bridge boundary.
    try {
        return std::make_unique<SewingBuilder>(tolerance);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("sewing_new failed: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    } catch (...) {
        throw std::runtime_error("unknown C++ exception in sewing_new");
    }
}

void sewing_add(SewingBuilder& builder, const OcctShape& shape) {
    try {
        builder.impl->sewing.Add(shape.get());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("sewing_add failed: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    } catch (...) {
        throw std::runtime_error("unknown C++ exception in sewing_add");
    }
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("sewing_build failed: ") + e.GetMessageString());
    } catch (const std::exception&) {
        // Re-throw std::exception subclasses (e.g. from BRepBuilderAPI_MakeSolid or
        // BRepLib::OrientClosedSolid) so they cross the cxx bridge as Rust errors
        // rather than terminating the process as unhandled exceptions.
        throw;
    }
}

// ---------------------------------------------------------------------------
// Transforms
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_translate(const OcctShape& s, double dx, double dy, double dz) {
    try {
        gp_Trsf trsf;
        trsf.SetTranslation(gp_Vec(dx, dy, dz));
        BRepBuilderAPI_Transform xform(s.get(), trsf, /*copy=*/Standard_True);
        xform.Build();
        if (!xform.IsDone())
            throw std::runtime_error("BRepBuilderAPI_Transform (translate) failed");
        return wrap(xform.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_rotate(const OcctShape& s, double axis_x, double axis_y,
                                        double axis_z, double angle_deg) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_scale(const OcctShape& s, double factor) {
    try {
        gp_Trsf trsf;
        trsf.SetScaleFactor(factor);
        BRepBuilderAPI_Transform xform(s.get(), trsf, /*copy=*/Standard_True);
        xform.Build();
        if (!xform.IsDone())
            throw std::runtime_error("BRepBuilderAPI_Transform (scale) failed");
        return wrap(xform.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Non-uniform scale — independent factors for X, Y, Z.
//
// gp_Trsf only supports uniform scale (single scalar), so we use gp_GTrsf
// (general affine transform) with a diagonal 3x3 matrix.
// BRepBuilderAPI_GTransform may approximate curved edges; the result is still
// topologically valid and suitable for all downstream operations.
std::unique_ptr<OcctShape> shape_scale_xyz(const OcctShape& s, double sx, double sy, double sz) {
    try {
        // Build a diagonal 3×3 matrix: diag(sx, sy, sz).
        // gp_Mat(row0col0, row0col1, row0col2, row1col0, ...)
        gp_GTrsf gtrsf;
        gtrsf.SetVectorialPart(gp_Mat(sx, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, sz));
        // Translation part stays zero (SetTranslationPart not called).
        BRepBuilderAPI_GTransform xform(s.get(), gtrsf, /*copy=*/Standard_True);
        if (!xform.IsDone())
            throw std::runtime_error("BRepBuilderAPI_GTransform (scale_xyz) failed");
        return wrap(xform.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Mirror
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_mirror(const OcctShape& s, rust::Str plane) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 2: 2D sketch faces
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> make_rect(double w, double h) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_circle_face(double r) {
    try {
        gp_Circ circ(gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), r);
        TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(circ).Edge();
        TopoDS_Wire wire = BRepBuilderAPI_MakeWire(edge).Wire();
        BRepBuilderAPI_MakeFace face(wire);
        if (!face.IsDone())
            throw std::runtime_error("BRepBuilderAPI_MakeFace (circle) failed");
        return wrap(face.Face());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 4: Sketch profiles — polygon, ellipse, arc
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Phase 11: make_profile_2d — a closed XY profile mixing straight and curved
// segments, so a constraint sketch can carry spline edges without falling back
// to a polyline approximation of them.
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> make_profile_2d(rust::Slice<const double> pts,
                                           rust::Slice<const int32_t> counts,
                                           rust::Slice<const int32_t> kinds) {
    try {
        if (counts.size() != kinds.size())
            throw std::runtime_error("make_profile_2d: counts and kinds must have equal length");
        if (counts.size() < 2)
            throw std::runtime_error("make_profile_2d: need at least 2 segments");

        std::size_t total = 0;
        for (std::size_t i = 0; i < counts.size(); ++i) {
            if (counts[i] < 2)
                throw std::runtime_error("make_profile_2d: every segment needs at least 2 points");
            total += static_cast<std::size_t>(counts[i]);
        }
        if (total * 2 != pts.size())
            throw std::runtime_error("make_profile_2d: point count does not match the segments");

        BRepBuilderAPI_MakeWire wire_builder;
        std::size_t offset = 0;

        for (std::size_t seg = 0; seg < counts.size(); ++seg) {
            int n = counts[seg];

            if (kinds[seg] == 0) {
                // Straight run: one edge per pair, skipping repeated points.
                for (int i = 0; i + 1 < n; ++i) {
                    gp_Pnt a(pts[(offset + i) * 2], pts[(offset + i) * 2 + 1], 0.0);
                    gp_Pnt b(pts[(offset + i + 1) * 2], pts[(offset + i + 1) * 2 + 1], 0.0);
                    if (a.Distance(b) <= Precision::Confusion())
                        continue;
                    wire_builder.Add(BRepBuilderAPI_MakeEdge(a, b).Edge());
                }
            } else {
                // Curved run: interpolate a BSpline through every point.
                Handle(TColgp_HArray1OfPnt) curve_pts = new TColgp_HArray1OfPnt(1, n);
                for (int i = 0; i < n; ++i) {
                    curve_pts->SetValue(
                        i + 1, gp_Pnt(pts[(offset + i) * 2], pts[(offset + i) * 2 + 1], 0.0));
                }

                GeomAPI_Interpolate interp(curve_pts, /*isPeriodic=*/Standard_False,
                                           /*Tolerance=*/1e-6);
                interp.Perform();
                if (!interp.IsDone())
                    throw std::runtime_error(
                        "GeomAPI_Interpolate (make_profile_2d) failed: a spline segment's points "
                        "may be duplicated or collinear to within tolerance");

                wire_builder.Add(BRepBuilderAPI_MakeEdge(interp.Curve()).Edge());
            }

            offset += static_cast<std::size_t>(n);
        }

        // Close the loop if the caller left a gap between the last and first
        // point, mirroring what make_spline_2d does.
        gp_Pnt first(pts[0], pts[1], 0.0);
        gp_Pnt last(pts[(total - 1) * 2], pts[(total - 1) * 2 + 1], 0.0);
        if (first.Distance(last) > Precision::Confusion())
            wire_builder.Add(BRepBuilderAPI_MakeEdge(last, first).Edge());

        if (!wire_builder.IsDone())
            throw std::runtime_error("BRepBuilderAPI_MakeWire (make_profile_2d) failed: the "
                                     "segments do not form one connected loop");

        BRepBuilderAPI_MakeFace face(wire_builder.Wire());
        if (!face.IsDone())
            throw std::runtime_error("BRepBuilderAPI_MakeFace (make_profile_2d) failed");

        return wrap(face.Face());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_polygon(rust::Slice<const double> pts) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_ellipse_face(double rx, double ry) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_arc(double r, double start_deg, double end_deg) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Extrude / Revolve
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_extrude(const OcctShape& s, double height) {
    try {
        BRepPrimAPI_MakePrism prism(s.get(), gp_Vec(0.0, 0.0, height));
        prism.Build();
        if (!prism.IsDone())
            throw std::runtime_error("BRepPrimAPI_MakePrism (extrude) failed");
        return wrap(prism.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_revolve(const OcctShape& s, double angle_deg) {
    try {
        gp_Ax1 axis(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0));
        const double angle_rad = angle_deg * (M_PI / 180.0);
        BRepPrimAPI_MakeRevol revol(s.get(), axis, angle_rad);
        revol.Build();
        if (!revol.IsDone())
            throw std::runtime_error("BRepPrimAPI_MakeRevol (revolve) failed");
        return wrap(revol.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 4: ThruSections (loft) builder
// ---------------------------------------------------------------------------

ThruSectionsBuilder::ThruSectionsBuilder(bool solid, bool ruled)
    : impl(std::make_unique<BRepOffsetAPI_ThruSections>(solid, ruled)) {}

ThruSectionsBuilder::~ThruSectionsBuilder() = default;

std::unique_ptr<ThruSectionsBuilder> thru_sections_new(bool solid, bool ruled) {
    // OCCT Standard_Failure does not derive std::exception; without this guard an
    // escaping OCCT exception would abort the process at the cxx bridge boundary.
    try {
        return std::make_unique<ThruSectionsBuilder>(solid, ruled);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("thru_sections_new failed: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    } catch (...) {
        throw std::runtime_error("unknown C++ exception in thru_sections_new");
    }
}

void thru_sections_add(ThruSectionsBuilder& b, const OcctShape& profile) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> thru_sections_build(ThruSectionsBuilder& b) {
    try {
        b.impl->Build();
        if (!b.impl->IsDone())
            throw std::runtime_error("BRepOffsetAPI_ThruSections (loft) failed");
        return wrap(b.impl->Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    // OCCT Standard_Failure does not derive std::exception; without this guard an
    // escaping OCCT exception would abort the process at the cxx bridge boundary.
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("sweep_sections path setup failed: ") +
                                 e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    } catch (...) {
        throw std::runtime_error("unknown C++ exception in PipeShellBuilder constructor");
    }
}

PipeShellBuilder::~PipeShellBuilder() = default;

std::unique_ptr<PipeShellBuilder> pipe_shell_new(const OcctShape& path) {
    return std::make_unique<PipeShellBuilder>(path);
}

void pipe_shell_add(PipeShellBuilder& b, const OcctShape& profile) {
    // Accumulate sections in order; they are placed in pipe_shell_build().
    // Guarded: BRepTools::OuterWire may raise Standard_Failure, which does not
    // derive std::exception and would otherwise abort at the cxx bridge boundary.
    try {
        const TopoDS_Shape& s = profile.get();
        if (s.ShapeType() == TopAbs_FACE) {
            // MakePipeShell works with wires; extract the outer boundary wire here.
            b.impl->sections.push_back(BRepTools::OuterWire(TopoDS::Face(s)));
        } else if (s.ShapeType() == TopAbs_WIRE) {
            b.impl->sections.push_back(s);
        } else if (s.ShapeType() == TopAbs_VERTEX) {
            b.impl->sections.push_back(s);
        } else {
            throw std::runtime_error(
                "sweep_sections: each profile must be a Face, Wire, or Vertex");
        }
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("pipe_shell_add failed: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    } catch (...) {
        throw std::runtime_error("unknown C++ exception in pipe_shell_add");
    }
}

// Helper: translate section[i] to the spine point at evenly-distributed
// parameter t[i], return the moved shape.
//
// Note: positions are evenly spaced in *parametric* space, not arc-length.
// This is intentional — for the common case of circular profiles on smooth
// curves the difference is negligible, and parametric spacing avoids an extra
// GCPnts_UniformAbscissa pass at a cost of slight non-uniformity on highly
// curved paths.
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
        // Frenet mode: at each spine point, OCCT computes the Frenet frame
        // (tangent / normal / binormal) and aligns the profile to it.  This keeps
        // circular cross-sections truly circular even on curved paths.
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 4: .offset(distance) — inflate / deflate a solid
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> shape_offset(const OcctShape& shape, double distance) {
    try {
        BRepOffsetAPI_MakeOffsetShape offsetter;
        offsetter.PerformByJoin(shape.get(), distance, 1e-3);
        if (!offsetter.IsDone())
            throw std::runtime_error("BRepOffsetAPI_MakeOffsetShape (offset) failed");
        return wrap(offsetter.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 7 Tier 1: .offset_2d(distance) — inward/outward offset of a Wire or Face
//
// BRepOffsetAPI_MakeOffset operates in the plane of the input shape.
// Positive distance expands the profile; negative shrinks it.
// ---------------------------------------------------------------------------

// Area of the region a planar wire encloses, used to tell the offset result's
// outer boundary from the holes inside it.
static double planar_wire_area(const gp_Pln& plane, const TopoDS_Wire& wire) {
    BRepBuilderAPI_MakeFace mf(plane, wire);
    if (!mf.IsDone())
        return 0.0;

    GProp_GProps props;
    BRepGProp::SurfaceProperties(mf.Face(), props);
    return std::abs(props.Mass());
}

// The plane a 2-D profile lives in.  Offsetting only makes sense for a planar
// face, and the plane is needed to rebuild the result.
static gp_Pln planar_face_plane(const TopoDS_Face& face) {
    Handle(Geom_Surface) surf = BRep_Tool::Surface(face);
    Handle(Geom_Plane) plane = Handle(Geom_Plane)::DownCast(surf);
    if (plane.IsNull())
        throw std::runtime_error("offset_2d: the face must be planar");
    return plane->Pln();
}

// Fallback for faces BRepOffsetAPI_MakeOffset cannot offset in one go — an
// all-circular annulus, for one.  Each boundary wire is offset on its own and
// the results handed back for reassembly.
//
// A standalone wire offsets by its own geometry, losing the sense it had as a
// hole in the face, so hole wires take the opposite sign: growing the material
// (distance > 0) has to shrink the holes.
static TopoDS_Shape offset_face_wires_individually(const TopoDS_Face& face, const gp_Pln& plane,
                                                   double distance) {
    std::vector<TopoDS_Wire> wires;
    for (TopExp_Explorer ex(face, TopAbs_WIRE); ex.More(); ex.Next())
        wires.push_back(TopoDS::Wire(ex.Current()));

    // The wire enclosing the most area is the outer boundary; the rest are
    // holes.
    std::size_t outer = 0;
    for (std::size_t i = 1; i < wires.size(); ++i) {
        if (planar_wire_area(plane, wires[i]) > planar_wire_area(plane, wires[outer]))
            outer = i;
    }

    BRep_Builder builder;
    TopoDS_Compound compound;
    builder.MakeCompound(compound);

    for (std::size_t i = 0; i < wires.size(); ++i) {
        BRepOffsetAPI_MakeOffset wire_offsetter(wires[i], GeomAbs_Arc);
        wire_offsetter.Perform(i == outer ? distance : -distance);
        if (!wire_offsetter.IsDone())
            throw std::runtime_error("BRepOffsetAPI_MakeOffset (offset_2d, face) failed");
        builder.Add(compound, wire_offsetter.Shape());
    }

    return compound;
}

// Rebuild a planar Face from the wires BRepOffsetAPI_MakeOffset produced.
//
// The offsetter returns a Wire (or a Compound of them) rather than a Face, so
// the raw result cannot be extruded into a solid — the largest wire becomes
// the new outer boundary and every smaller one is added back as a hole.
static TopoDS_Shape face_from_offset_wires(const gp_Pln& plane, const TopoDS_Shape& offset_result,
                                           double distance) {
    std::vector<TopoDS_Wire> wires;
    if (offset_result.ShapeType() == TopAbs_WIRE) {
        wires.push_back(TopoDS::Wire(offset_result));
    } else {
        for (TopExp_Explorer ex(offset_result, TopAbs_WIRE); ex.More(); ex.Next())
            wires.push_back(TopoDS::Wire(ex.Current()));
    }

    // An inward offset wider than the profile collapses it entirely; OCCT
    // signals that with an empty result rather than an error.
    if (wires.empty())
        throw std::runtime_error("offset_2d(" + std::to_string(distance) +
                                 ") leaves no profile: the inward offset consumed the whole face");

    // Largest enclosed area first: that wire is the outer boundary.
    std::stable_sort(wires.begin(), wires.end(), [&](const TopoDS_Wire& a, const TopoDS_Wire& b) {
        return planar_wire_area(plane, a) > planar_wire_area(plane, b);
    });

    BRepBuilderAPI_MakeFace outer(plane, wires[0]);
    if (!outer.IsDone())
        throw std::runtime_error("offset_2d: could not rebuild the offset profile as a face");

    TopoDS_Face result = outer.Face();
    for (std::size_t i = 1; i < wires.size(); ++i) {
        // A hole wire must run opposite to the outer boundary.
        BRepBuilderAPI_MakeFace with_hole(result, TopoDS::Wire(wires[i].Reversed()));
        if (!with_hole.IsDone())
            throw std::runtime_error("offset_2d: could not add an offset hole to the profile");
        result = with_hole.Face();
    }

    // Let OCCT settle the wire orientations so the holes really read as holes.
    ShapeFix_Face fixer(result);
    fixer.Perform();
    fixer.FixOrientation();
    return fixer.Face();
}

std::unique_ptr<OcctShape> shape_offset_2d(const OcctShape& shape, double distance) {
    try {
        // A profile built by a boolean (e.g. rect(40, 20).cut(circle(5)))
        // arrives as a Compound wrapping a single Face; unwrap it so profiles
        // with holes offset like any other.
        TopoDS_Shape input = shape.get();
        if (input.ShapeType() == TopAbs_COMPOUND) {
            TopExp_Explorer ex(input, TopAbs_FACE);
            if (ex.More()) {
                TopoDS_Shape only_face = ex.Current();
                ex.Next();
                if (!ex.More())
                    input = only_face;
            }
        }

        TopAbs_ShapeEnum type = input.ShapeType();
        if (type != TopAbs_FACE && type != TopAbs_WIRE)
            throw std::runtime_error("offset_2d: input must be a Face or Wire");

        // BRepOffsetAPI_MakeOffset has separate constructors for Face and Wire.
        if (type == TopAbs_FACE) {
            TopoDS_Face face = TopoDS::Face(input);
            gp_Pln plane = planar_face_plane(face);

            BRepOffsetAPI_MakeOffset offsetter(face, GeomAbs_Arc);
            offsetter.Perform(distance);

            // Offsetting the face as a whole fails on some profiles with holes;
            // offsetting each boundary wire on its own still succeeds.
            TopoDS_Shape offset_wires = offsetter.IsDone()
                                            ? offsetter.Shape()
                                            : offset_face_wires_individually(face, plane, distance);

            // A Face in stays a Face out, so the offset profile can be padded,
            // pocketed, or extruded like any other profile.
            return wrap(face_from_offset_wires(plane, offset_wires, distance));
        } else {
            BRepOffsetAPI_MakeOffset offsetter(TopoDS::Wire(input), GeomAbs_Arc);
            offsetter.Perform(distance);
            if (!offsetter.IsDone())
                throw std::runtime_error("BRepOffsetAPI_MakeOffset (offset_2d, wire) failed");
            return wrap(offsetter.Shape());
        }
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 3: Spline profiles and pipe sweep
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> make_spline_2d(rust::Slice<const double> pts) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_spline_3d(rust::Slice<const double> pts) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_spline_2d_tan(rust::Slice<const double> pts, double t0x, double t0z,
                                              double t1x, double t1z) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_spline_3d_tan(rust::Slice<const double> pts, double t0x, double t0y,
                                              double t0z, double t1x, double t1y, double t1z) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_sweep(const OcctShape& profile, const OcctShape& path) {
    try {
        const TopoDS_Shape& path_shape = path.get();
        if (path_shape.ShapeType() != TopAbs_WIRE)
            throw std::runtime_error("sweep: path must be a Wire (create with spline_3d)");

        TopoDS_Wire path_wire = TopoDS::Wire(path_shape);
        BRepOffsetAPI_MakePipe pipe(path_wire, profile.get());
        pipe.Build();
        if (!pipe.IsDone())
            throw std::runtime_error("BRepOffsetAPI_MakePipe (sweep) failed");
        return wrap(pipe.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_faces_get(const OcctShape& shape, rust::Str selector,
                                           int32_t idx) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
        std::string sel(selector.data(), selector.size());
        return static_cast<int32_t>(collect_edges(shape, sel).size());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_edges_get(const OcctShape& shape, rust::Str selector,
                                           int32_t idx) {
    try {
        std::string sel(selector.data(), selector.size());
        auto edges = collect_edges(shape, sel);
        auto i = static_cast<size_t>(idx);
        if (i >= edges.size())
            throw std::runtime_error("shape_edges_get: index out of range");
        return wrap(edges[i]);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 4: Vertices selector
// ---------------------------------------------------------------------------

// Only selector currently supported is "all" — a positional / direction
// filter on vertices is not meaningful in the same way as faces/edges.
// The API is symmetric with faces/edges so callers can iterate all vertices
// without special-casing.
int32_t shape_vertices_count(const OcctShape& shape, rust::Str selector) {
    try {
        std::string sel(selector.data(), selector.size());
        if (sel != "all")
            throw std::runtime_error(std::string("vertices: unknown selector ':") + sel +
                                     "' — only :all is supported");

        // Use IndexedMapOfShape for deduplication (shared vertices appear once).
        TopTools_IndexedMapOfShape vertex_map;
        TopExp::MapShapes(shape.get(), TopAbs_VERTEX, vertex_map);
        return static_cast<int32_t>(vertex_map.Extent());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_vertices_get(const OcctShape& shape, rust::Str selector,
                                              int32_t idx) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Returns a compound of n copies rotated around the Z axis.
// Copy i is rotated by i * (angle_deg / n) degrees.
// e.g. polar_pattern(shape, 6, 360) places 6 copies every 60° around a full circle.
std::unique_ptr<OcctShape> shape_polar_pattern(const OcctShape& s, int32_t n, double angle_deg) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

std::unique_ptr<OcctShape> import_step(rust::Str path) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> import_stl(rust::Str path) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 4: Query / introspection
// ---------------------------------------------------------------------------

void shape_bounding_box(const OcctShape& shape, rust::Slice<double> out) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

double shape_volume(const OcctShape& shape) {
    try {
        GProp_GProps props;
        BRepGProp::VolumeProperties(shape.get(), props);
        return props.Mass();
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

double shape_surface_area(const OcctShape& shape) {
    try {
        GProp_GProps props;
        BRepGProp::SurfaceProperties(shape.get(), props);
        return props.Mass();
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 7 Tier 2: Validation & introspection
// ---------------------------------------------------------------------------

// Map TopAbs_ShapeEnum to a lowercase string name.
rust::String shape_type_str(const OcctShape& shape) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Centroid of the shape.  Uses VolumeProperties for solids and compounds;
// SurfaceProperties for shells/faces; LinearProperties for wires/edges.
void shape_cylinder_axis(const OcctShape& face_shape, rust::Slice<double> out) {
    try {
        if (out.size() < 7)
            throw std::runtime_error("cylinder_axis: output slice must have at least 7 elements");
        const TopoDS_Shape& s = face_shape.get();
        if (s.ShapeType() != TopAbs_FACE)
            throw std::runtime_error("cylinder_axis: shape is not a face");

        const TopoDS_Face& face = TopoDS::Face(s);
        BRepAdaptor_Surface surf(face);
        if (surf.GetType() != GeomAbs_Cylinder)
            throw std::runtime_error("cylinder_axis: face is not a cylindrical surface");

        gp_Cylinder cyl = surf.Cylinder();
        const gp_Ax1& axis = cyl.Axis();
        gp_Pnt loc = axis.Location();
        gp_Dir dir = axis.Direction();

        out[0] = loc.X();
        out[1] = loc.Y();
        out[2] = loc.Z();
        out[3] = dir.X();
        out[4] = dir.Y();
        out[5] = dir.Z();
        out[6] = cyl.Radius();
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

void shape_face_normal(const OcctShape& face_shape, rust::Slice<double> out) {
    try {
        if (out.size() < 3)
            throw std::runtime_error("face_normal: output slice must have at least 3 elements");
        const TopoDS_Shape& s = face_shape.get();
        if (s.ShapeType() != TopAbs_FACE)
            throw std::runtime_error("face_normal: shape is not a face");

        const TopoDS_Face& face = TopoDS::Face(s);
        BRepAdaptor_Surface adaptor(face);
        double umid = 0.5 * (adaptor.FirstUParameter() + adaptor.LastUParameter());
        double vmid = 0.5 * (adaptor.FirstVParameter() + adaptor.LastVParameter());
        BRepLProp_SLProps props(adaptor, umid, vmid, 1, 1e-6);
        if (!props.IsNormalDefined())
            throw std::runtime_error("face_normal: normal is not defined at the sample point");

        gp_Dir normal = props.Normal();
        if (face.Orientation() == TopAbs_REVERSED)
            normal.Reverse();

        out[0] = normal.X();
        out[1] = normal.Y();
        out[2] = normal.Z();
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

void shape_centroid(const OcctShape& shape, rust::Slice<double> out) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// A shape is "closed" if it has no free (boundary) edges — every edge is
// shared by at least two faces.  Empty shapes (no edges) return false.
bool shape_is_closed(const OcctShape& shape) {
    try {
        TopTools_IndexedDataMapOfShapeListOfShape edge_face_map;
        TopExp::MapShapesAndAncestors(shape.get(), TopAbs_EDGE, TopAbs_FACE, edge_face_map);
        if (edge_face_map.IsEmpty())
            return false;
        for (int i = 1; i <= edge_face_map.Extent(); ++i) {
            if (edge_face_map(i).Size() < 2)
                return false;
        }
        return true;
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// A shape is "manifold" if every edge is shared by exactly two faces.
// This rules out both boundary edges (< 2 faces) and T-junction edges (> 2 faces).
bool shape_is_manifold(const OcctShape& shape) {
    try {
        TopTools_IndexedDataMapOfShapeListOfShape edge_face_map;
        TopExp::MapShapesAndAncestors(shape.get(), TopAbs_EDGE, TopAbs_FACE, edge_face_map);
        if (edge_face_map.IsEmpty())
            return false;
        for (int i = 1; i <= edge_face_map.Extent(); ++i) {
            if (edge_face_map(i).Size() != 2)
                return false;
        }
        return true;
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 7 Tier 3: Surface modeling
// ---------------------------------------------------------------------------

// Build a ruled surface (TopoDS_Shell) by connecting corresponding vertices
// of two wires with straight lines.  BRepFill::Shell handles wire orientation
// and produces a properly-connected shell suitable for further sewing.
std::unique_ptr<OcctShape> shape_ruled_surface(const OcctShape& wire_a, const OcctShape& wire_b) {
    try {
        if (wire_a.get().ShapeType() != TopAbs_WIRE)
            throw std::runtime_error("ruled_surface: first argument must be a Wire");
        if (wire_b.get().ShapeType() != TopAbs_WIRE)
            throw std::runtime_error("ruled_surface: second argument must be a Wire");

        TopoDS_Shell shell =
            BRepFill::Shell(TopoDS::Wire(wire_a.get()), TopoDS::Wire(wire_b.get()));
        if (shell.IsNull())
            throw std::runtime_error("ruled_surface: BRepFill::Shell failed");

        return wrap(shell);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Build a smooth filling surface (TopoDS_Face) whose boundary follows the
// edges of a single closed wire.  Each edge is added as a C0 free boundary
// constraint; BRepFill_Filling solves a plate-energy minimisation problem to
// produce a fair surface inside.
std::unique_ptr<OcctShape> shape_fill_surface(const OcctShape& boundary_wire) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// Intersect a shape with an axis-aligned plane and return the cross-section
// as a compound of edges/wires.  The section curves can be extruded or used
// as profiles for further operations.
//
//   plane = "xy"  →  plane at z = offset  (normal +Z)
//   plane = "xz"  →  plane at y = offset  (normal +Y)
//   plane = "yz"  →  plane at x = offset  (normal +X)
std::unique_ptr<OcctShape> shape_slice(const OcctShape& shape, rust::Str plane, double offset) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_pocket(const OcctShape& body, const OcctShape& face_ref,
                                        const OcctShape& sketch, double depth) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> shape_fillet_wire(const OcctShape& profile, double radius) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> make_datum_plane(double ox, double oy, double oz, double nx, double ny,
                                            double nz, double xx, double xy, double xz) {
    try {
        gp_Ax3 ax3(gp_Pnt(ox, oy, oz), gp_Dir(nx, ny, nz), gp_Dir(xx, xy, xz));
        gp_Pln pln(ax3);
        // Create a finite face: ±50 units in each in-plane direction.
        BRepBuilderAPI_MakeFace mf(pln, -50.0, 50.0, -50.0, 50.0);
        if (!mf.IsDone())
            throw std::runtime_error("datum_plane: BRepBuilderAPI_MakeFace failed");
        return wrap(mf.Shape());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

void export_step(const OcctShape& shape, rust::Str path) {
    try {
        // Guard against degenerate geometry that would produce a corrupt STEP file.
        BRepCheck_Analyzer checker(shape.get());
        if (!checker.IsValid())
            throw std::runtime_error(
                "shape is topologically invalid (degenerate faces or open shells) "
                "— check upstream boolean operations or fillet radii");

        STEPControl_Writer writer;
        IFSelect_ReturnStatus status = writer.Transfer(shape.get(), STEPControl_AsIs);
        if (status != IFSelect_RetDone)
            throw std::runtime_error("STEPControl_Writer::Transfer failed");

        std::string path_str(path.data(), path.size());
        auto temp_path = atomic_export_temp_path(path_str);
        status = writer.Write(temp_path.string().c_str());
        if (status != IFSelect_RetDone)
            throw std::runtime_error("STEPControl_Writer::Write failed for: " + path_str);
        rename_export_artifact(temp_path, path_str);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

void export_stl(const OcctShape& shape, rust::Str path) {
    try {
        // Tessellate before writing — StlAPI_Writer requires a pre-meshed shape
        // in OCCT 7.7+.  isParallel=true uses the TBB thread pool (OCCT 7.4+).
        BRepMesh_IncrementalMesh mesher(shape.get(), 0.1, /*isRelative=*/Standard_False,
                                        /*angularDeflection=*/0.5, /*isParallel=*/Standard_True);
        mesher.Perform();

        std::string path_str(path.data(), path.size());
        StlAPI_Writer writer;
        auto temp_path = atomic_export_temp_path(path_str);
        Standard_Boolean ok = writer.Write(shape.get(), temp_path.string().c_str());
        if (!ok)
            throw std::runtime_error("StlAPI_Writer::Write failed for: " + path_str);
        rename_export_artifact(temp_path, path_str);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
        Handle(TDocStd_Document) doc = make_xde_doc(shape, linear_deflection, "glTF export");
        std::string path_str(path.data(), path.size());
        auto temp_path = atomic_export_temp_path(path_str);
        TCollection_AsciiString gltf_path(temp_path.string().c_str());
        RWGltf_CafWriter writer(gltf_path, /*isBinary=*/Standard_False);
        TColStd_IndexedDataMapOfStringString metadata;
        Message_ProgressRange progress;
        if (!writer.Perform(doc, metadata, progress))
            throw std::runtime_error("RWGltf_CafWriter::Perform failed for: " + path_str);
        rename_export_artifact(temp_path, path_str);
        auto temp_bin = temp_path;
        temp_bin.replace_extension(".bin");
        auto final_bin = std::filesystem::path(path_str);
        final_bin.replace_extension(".bin");
        rename_export_artifact(temp_bin, final_bin);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

void export_glb(const OcctShape& shape, rust::Str path, double linear_deflection) {
    try {
        // Guard against degenerate geometry that would produce a corrupt GLB file.
        BRepCheck_Analyzer checker(shape.get());
        if (!checker.IsValid())
            throw std::runtime_error(
                "shape is topologically invalid (degenerate faces or open shells) "
                "— check upstream boolean operations or fillet radii");

        Handle(TDocStd_Document) doc = make_xde_doc(shape, linear_deflection, "GLB export");
        std::string path_str(path.data(), path.size());
        auto temp_path = atomic_export_temp_path(path_str);
        TCollection_AsciiString glb_path(temp_path.string().c_str());
        RWGltf_CafWriter writer(glb_path, /*isBinary=*/Standard_True);
        // TRS decomposition (translation/rotation/scale) is lighter and more
        // interoperable with animation tools than the default 4×4 matrix.
        writer.SetTransformationFormat(RWGltf_WriterTrsfFormat_TRS);
        TColStd_IndexedDataMapOfStringString metadata;
        Message_ProgressRange progress;
        if (!writer.Perform(doc, metadata, progress))
            throw std::runtime_error("RWGltf_CafWriter::Perform (GLB) failed for: " + path_str);
        rename_export_artifact(temp_path, path_str);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// OBJ export via RWObj_CafWriter (OCCT 7.6+).
// Uses the same XDE pipeline as glTF/GLB so material handling is consistent.
void export_obj(const OcctShape& shape, rust::Str path, double linear_deflection) {
    try {
        Handle(TDocStd_Document) doc = make_xde_doc(shape, linear_deflection, "OBJ export");
        std::string path_str(path.data(), path.size());
        auto temp_path = atomic_export_temp_path(path_str);
        TCollection_AsciiString obj_path(temp_path.string().c_str());
        RWObj_CafWriter writer(obj_path);
        TColStd_IndexedDataMapOfStringString metadata;
        Message_ProgressRange progress;
        if (!writer.Perform(doc, metadata, progress))
            throw std::runtime_error("RWObj_CafWriter::Perform failed for: " + path_str);
        rename_export_artifact(temp_path, path_str);
        auto temp_mtl = temp_path;
        temp_mtl.replace_extension(".mtl");
        auto final_mtl = std::filesystem::path(path_str);
        final_mtl.replace_extension(".mtl");
        rename_export_artifact(temp_mtl, final_mtl);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Phase 8 Tier 3 — Inspection & clearance
// ---------------------------------------------------------------------------

// shape_distance_to — minimum distance between two shapes.
// BRepExtrema_DistShapeShape returns 0 when shapes intersect or touch.
double shape_distance_to(const OcctShape& a, const OcctShape& b) {
    try {
        BRepExtrema_DistShapeShape dist(a.get(), b.get());
        dist.Perform();
        if (!dist.IsDone())
            throw std::runtime_error("shape_distance_to: BRepExtrema_DistShapeShape failed");
        return dist.Value();
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// shape_inertia — inertia tensor about the shape's own centre of mass.
// Fills out[] with [Ixx, Iyy, Izz, Ixy, Ixz, Iyz].
//
// The MatrixOfInertia diagonal is [Ixx, Iyy, Izz]. The off-diagonal entries
// are true inertia-tensor entries (-∫xy dV), NOT products of inertia
// (+∫xy dV) — verified against two unit-density cubes on a diagonal, which
// report Ixy = -50000 where ∫xy dV = +50000. Any axis transfer built on this
// must subtract the m·dx·dy term to match.
//
// VolumeProperties integrates with density 1, so the result is in volume
// units (mm^5), not mass units; callers scale by mass / volume.
void shape_inertia(const OcctShape& shape, rust::Slice<double> out) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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

using DrawingPolyline = std::vector<std::pair<double, double>>;
using DrawingPolylines = std::vector<DrawingPolyline>;

struct DrawingMark {
    double x;
    double y;
    double size;
};

struct DrawingCallout {
    double x;
    double y;
    double leader_x;
    double leader_y;
    std::string text;
};

// One ordinate dimension: a witness line running from a feature's centre out to
// a common baseline, labelled with that feature's distance from the view's
// datum corner.  Ordinate dimensioning is what a plate full of holes actually
// gets on a drawing — a chain of individual dimensions between every pair would
// be unreadable and would accumulate tolerance.
struct DrawingOrdinate {
    // true: measured along the drawing's X, witness drawn below the view.
    // false: measured along Y, witness drawn to the left.
    bool horizontal;
    // The feature's position on the measured axis, in drawing coordinates.
    double at;
    // The measured distance from the datum corner, in model units.
    std::string label;
};

// Ordinate baselines sit outside the overall dimension lines so the two never
// overlap, with room beyond for the rotated labels.
static const double ORDINATE_BASELINE = 20.0;
static const double ORDINATE_LABEL_ROOM = 12.0;

// One balloon callout: a numbered circle keyed to a parts-list row, with a
// leader pointing at the component it identifies.
struct DrawingBalloon {
    std::string label; // the item number, matching the table's first column
    double x, y;       // leader anchor: the component's centroid, in drawing coords
    double bx, by;     // balloon centre, placed clear of the geometry
};

// Balloon geometry, in final drawing units.  The ring keeps the circles clear
// of the part; the leaders fan out from it to each component.
static const double BALLOON_RADIUS = 4.0;
static const double BALLOON_RING_GAP = 14.0;

// Parts-list table geometry.  A monospace glyph at font size 3 is a little
// under 1.8 units wide, which is what sizes the columns.
static const double BOM_ROW_HEIGHT = 5.0;
static const double BOM_FONT_SIZE = 3.0;
static const double BOM_CHAR_WIDTH = 1.8;
static const double BOM_CELL_PAD = 3.0;
static const double BOM_TABLE_GAP = 10.0;

// Split on a single delimiter, keeping empty fields — the table's cells are
// positional, so an empty material must not shift the columns after it.
static std::vector<std::string> split_on(const std::string& text, char delim) {
    std::vector<std::string> out;
    std::string current;
    for (char c : text) {
        if (c == delim) {
            out.push_back(current);
            current.clear();
        } else {
            current.push_back(c);
        }
    }
    out.push_back(current);
    return out;
}

// Parse the tab/newline-delimited parts list.  Per-component data cannot travel
// as scalars — the row count is not known until the assembly is walked — so it
// arrives as one delimited string, with the first record as the header.
static std::vector<std::vector<std::string>> parse_bom_rows(const std::string& text) {
    std::vector<std::vector<std::string>> rows;
    if (text.empty())
        return rows;
    for (const auto& line : split_on(text, '\n')) {
        if (line.empty())
            continue;
        rows.push_back(split_on(line, '\t'));
    }
    return rows;
}

// Parse balloon records ("label\tx\ty"), with x/y in model units on the view's
// drawing plane; `scale` lifts them into drawing coordinates.
static std::vector<DrawingBalloon> parse_balloons(const std::string& text, double scale) {
    std::vector<DrawingBalloon> balloons;
    if (text.empty())
        return balloons;
    for (const auto& line : split_on(text, '\n')) {
        if (line.empty())
            continue;
        auto fields = split_on(line, '\t');
        if (fields.size() < 3)
            throw std::runtime_error("export_svg/dxf: malformed balloon record \"" + line + "\"");
        DrawingBalloon b;
        b.label = fields[0];
        try {
            b.x = std::stod(fields[1]) * scale;
            b.y = std::stod(fields[2]) * scale;
        } catch (const std::exception&) {
            throw std::runtime_error("export_svg/dxf: balloon \"" + fields[0] +
                                     "\" has a non-numeric anchor");
        }
        b.bx = b.x;
        b.by = b.y;
        balloons.push_back(b);
    }
    return balloons;
}

// A requested section (cutting) plane for a drawing view.
// `active` is false when the caller passed no `section:` option, in which case
// the drawing is a plain projection and no cutting is performed at all.
struct SectionSpec {
    bool active = false;
    std::string plane; // "xy", "xz", or "yz"
    double offset = 0.0;
};

// Hatch line spacing, in final (scaled) drawing units.  Standard mechanical
// drawings use an evenly-spaced 45° pattern; 2.5 mm reads well at 1:1.
static const double SECTION_HATCH_SPACING = 2.5;

// A requested detail view: a magnified close-up of one circular region of the
// parent projection.  `x` / `y` / `radius` are in *model* units on the view's
// own drawing plane (top → X/Y, front → X/Z, side → Y/Z), so the caller states
// the region using the same numbers they modelled with, independent of the
// drawing `scale`.  `scale` is the magnification relative to the parent view,
// i.e. the conventional "4:1" of a detail bubble.  `active` is false when the
// caller passed no `detail:` option at all.
struct DetailSpec {
    bool active = false;
    double x = 0.0;
    double y = 0.0;
    double radius = 0.0;
    double scale = 2.0;
    std::string label = "A";
};

// Gap between the parent view and the detail view placed beside it, and the
// clearance left under a view for its caption, both in drawing units.
static const double DETAIL_VIEW_GAP = 12.0;
static const double DETAIL_CAPTION_GAP = 7.0;

struct DrawingViewData {
    std::string name;
    DrawingPolylines visible;
    DrawingPolylines hidden;
    // Closed 2-D outlines of the cut faces (drawn at visible-edge weight).
    DrawingPolylines section_outline;
    // Individual 45° hatch segments clipped to the cut faces.
    DrawingPolylines hatch;
    std::vector<DrawingMark> marks;
    std::vector<DrawingCallout> callouts;
    std::vector<DrawingOrdinate> ordinates;
    double geom_xmin = 0.0;
    double geom_xmax = 0.0;
    double geom_ymin = 0.0;
    double geom_ymax = 0.0;
    double xmin = 0.0;
    double xmax = 0.0;
    double ymin = 0.0;
    double ymax = 0.0;
    double width = 0.0;
    double height = 0.0;
    // Detail-view annotation.  On a parent view this is the thin circle marking
    // the region that was magnified; on the detail view itself it is the border
    // circle enclosing the magnified geometry.  Coordinates are in this view's
    // own (already scaled) drawing space, like every other member here.
    bool detail_marker = false;
    double detail_x = 0.0;
    double detail_y = 0.0;
    double detail_r = 0.0;
    // Short label drawn beside the marker circle ("A"); empty on the detail
    // view, which carries the full text in `caption` instead.
    std::string detail_label;
    // Caption centred under the view ("DETAIL A (4:1)"); empty when unused.
    std::string caption;
};

struct DrawingCanvasBounds {
    double xmin = 0.0;
    double xmax = 0.0;
    double ymin = 0.0;
    double ymax = 0.0;
};

struct HlrProjection {
    DrawingPolylines visible;
    DrawingPolylines hidden;
};

static std::pair<double, double> project_point(const std::string& view, const gp_Pnt& p) {
    if (view == "front")
        return {p.X(), p.Z()};
    if (view == "side")
        return {p.Y(), p.Z()};
    return {p.X(), p.Y()};
}

static gp_Dir view_direction(const std::string& view) {
    if (view == "front")
        return gp_Dir(0, -1, 0);
    if (view == "side")
        return gp_Dir(1, 0, 0);
    return gp_Dir(0, 0, 1);
}

static void collect_hlr_compound(const TopoDS_Shape& compound, DrawingPolylines& polylines) {
    if (compound.IsNull())
        return;
    TopExp_Explorer eexp(compound, TopAbs_EDGE);
    for (; eexp.More(); eexp.Next()) {
        BRepAdaptor_Curve curve(TopoDS::Edge(eexp.Current()));
        double t0 = curve.FirstParameter();
        double t1 = curve.LastParameter();
        if (t1 <= t0)
            continue;

        DrawingPolyline pts;
        pts.reserve(HLR_SAMPLES_PER_EDGE + 1);
        for (int i = 0; i <= HLR_SAMPLES_PER_EDGE; ++i) {
            double t = t0 + (t1 - t0) * i / HLR_SAMPLES_PER_EDGE;
            gp_Pnt p = curve.Value(t);
            pts.emplace_back(p.X(), p.Y());
        }
        polylines.push_back(std::move(pts));
    }
}

static void scale_polylines(DrawingPolylines& polylines, double scale) {
    for (auto& pl : polylines) {
        for (auto& [x, y] : pl) {
            x *= scale;
            y *= scale;
        }
    }
}

static std::string format_measurement(double value) {
    std::ostringstream oss;
    oss << std::fixed << std::setprecision(3) << value;
    std::string s = oss.str();
    while (s.size() > 1 && s.find('.') != std::string::npos && s.back() == '0')
        s.pop_back();
    if (!s.empty() && s.back() == '.')
        s.pop_back();
    return s;
}

static std::string format_tolerance(double plus, double minus) {
    if (plus == minus) {
        if (plus <= 0.0)
            return std::string();
        return std::string("\xC2\xB1") + format_measurement(plus);
    }

    std::string text =
        std::string("+") + format_measurement(plus) + "/-" + format_measurement(minus);
    return text;
}

static std::vector<DrawingMark> collect_center_marks(const OcctShape& shape,
                                                     const std::string& view) {
    std::vector<DrawingMark> marks;
    const gp_Dir draw_dir = view_direction(view);

    for (TopExp_Explorer exp(shape.get(), TopAbs_FACE); exp.More(); exp.Next()) {
        const TopoDS_Face& face = TopoDS::Face(exp.Current());
        BRepAdaptor_Surface surf(face);
        if (surf.GetType() != GeomAbs_Cylinder)
            continue;

        gp_Cylinder cyl = surf.Cylinder();
        const gp_Dir& axis = cyl.Axis().Direction();
        if (std::abs(axis.Dot(draw_dir)) < 0.98)
            continue;

        gp_Pnt loc = cyl.Axis().Location();
        auto [x, y] = project_point(view, loc);
        double size = std::max(cyl.Radius() * 0.35, 1.5);
        bool duplicate = false;
        for (const auto& mark : marks) {
            if (std::abs(mark.x - x) < 1e-6 && std::abs(mark.y - y) < 1e-6 &&
                std::abs(mark.size - size) < 1e-6) {
                duplicate = true;
                break;
            }
        }
        if (!duplicate)
            marks.push_back({x, y, size});
    }

    return marks;
}

// Turn the located feature centres into ordinate dimensions measured from the
// view's datum corner — the lower-left of the projected geometry, which is what
// a shop sets its zero to.
//
// Coordinates arrive already multiplied by the drawing `scale`, so the label is
// divided back down: an ordinate states where the feature is on the *part*, not
// where it landed on the page.
//
// Features sharing a coordinate collapse to one ordinate. A row of holes at the
// same Y needs one Y dimension, not four stacked on top of each other.
static std::vector<DrawingOrdinate> collect_ordinates(const std::vector<DrawingMark>& marks,
                                                      double origin_x, double origin_y,
                                                      double scale) {
    std::vector<DrawingOrdinate> ordinates;
    auto add = [&](bool horizontal, double at, double origin) {
        for (const auto& o : ordinates)
            if (o.horizontal == horizontal && std::abs(o.at - at) < 1e-6)
                return;
        // A zero-length ordinate is the datum itself and carries no information.
        const double measured = (at - origin) / scale;
        if (std::abs(measured) < 1e-9)
            return;
        ordinates.push_back({horizontal, at, format_measurement(measured)});
    };
    for (const auto& mark : marks) {
        add(true, mark.x, origin_x);
        add(false, mark.y, origin_y);
    }
    return ordinates;
}

static std::vector<DrawingCallout> collect_callouts(const OcctShape& shape,
                                                    const std::string& view) {
    std::vector<DrawingCallout> callouts;
    const gp_Dir draw_dir = view_direction(view);

    for (TopExp_Explorer exp(shape.get(), TopAbs_FACE); exp.More(); exp.Next()) {
        const TopoDS_Face& face = TopoDS::Face(exp.Current());
        BRepAdaptor_Surface surf(face);
        if (surf.GetType() != GeomAbs_Cylinder)
            continue;

        gp_Cylinder cyl = surf.Cylinder();
        const gp_Dir& axis = cyl.Axis().Direction();
        if (std::abs(axis.Dot(draw_dir)) < 0.98)
            continue;

        gp_Pnt loc = cyl.Axis().Location();
        auto [x, y] = project_point(view, loc);
        double radius = cyl.Radius();
        double size = std::max(radius * 0.35, 1.5);
        std::string text = std::string("\xE2\x8C\x80") + format_measurement(2.0 * radius);
        double leader_x = x + size * 2.8;
        double leader_y = y + size * 1.6;
        bool duplicate = false;
        for (const auto& callout : callouts) {
            if (std::abs(callout.x - x) < 1e-6 && std::abs(callout.y - y) < 1e-6 &&
                callout.text == text) {
                duplicate = true;
                break;
            }
        }
        if (!duplicate)
            callouts.push_back({x, y, leader_x, leader_y, text});
    }

    return callouts;
}

// ---------------------------------------------------------------------------
// Section views
//
// A section view cuts the solid with an axis-aligned plane, throws away the
// material in front of the plane (on the +normal side), projects what is left
// exactly like a normal view, and additionally draws the exposed cut faces with
// their outline plus standard 45° hatching.
// ---------------------------------------------------------------------------

// Result of cutting the shape with the section plane.
struct SectionGeometry {
    // Material behind the cutting plane; this is what gets HLR-projected.
    std::unique_ptr<OcctShape> retained;
    // One entry per exposed cut face, each holding that face's boundary
    // polylines already projected into 2-D view space (unscaled).
    std::vector<DrawingPolylines> face_loops;
};

// Map a section plane name to its normal and the index (0=X, 1=Y, 2=Z) of the
// coordinate the offset is measured along.  Mirrors `shape_slice`.
static gp_Dir section_plane_normal(const std::string& plane, int& axis) {
    if (plane == "xy") {
        axis = 2;
        return gp_Dir(0, 0, 1);
    }
    if (plane == "xz") {
        axis = 1;
        return gp_Dir(0, 1, 0);
    }
    if (plane == "yz") {
        axis = 0;
        return gp_Dir(1, 0, 0);
    }
    throw std::runtime_error("export_svg/dxf: section plane must be \"xy\", \"xz\", or \"yz\"");
}

// Clip evenly-spaced 45° hatch lines to the interior of one cut face.
//
// `loops` are that face's boundary polylines in final (scaled) drawing
// coordinates — outer wire plus any inner wires.  A hatch line is the set of
// points satisfying x - y == c; for each such line every crossing with the
// boundary is collected, sorted along the line, and alternate spans are kept
// (even-odd fill rule), which drops the parts that fall inside holes.
static DrawingPolylines build_section_hatch(const DrawingPolylines& loops, double spacing) {
    DrawingPolylines out;
    if (loops.empty() || !(spacing > 0.0) || !std::isfinite(spacing))
        return out;

    double xmin = 1e30, xmax = -1e30, ymin = 1e30, ymax = -1e30;
    for (const auto& loop : loops) {
        for (const auto& [x, y] : loop) {
            xmin = std::min(xmin, x);
            xmax = std::max(xmax, x);
            ymin = std::min(ymin, y);
            ymax = std::max(ymax, y);
        }
    }
    if (xmin > xmax || ymin > ymax)
        return out;

    // Perpendicular spacing `spacing` between 45° lines corresponds to a step
    // of spacing * sqrt(2) in the c = x - y family.
    const double step = spacing * std::sqrt(2.0);
    const double c_lo = xmin - ymax;
    const double c_hi = xmax - ymin;
    // Hard cap so a huge face (or a tiny spacing) can never emit unboundedly.
    const std::size_t max_segments = 20000;

    for (double c = std::ceil(c_lo / step) * step; c <= c_hi; c += step) {
        std::vector<double> crossings;
        for (const auto& loop : loops) {
            for (std::size_t i = 0; i + 1 < loop.size(); ++i) {
                const double x1 = loop[i].first, y1 = loop[i].second;
                const double x2 = loop[i + 1].first, y2 = loop[i + 1].second;
                const double f1 = x1 - y1 - c;
                const double f2 = x2 - y2 - c;
                // Half-open crossing test: a vertex sitting exactly on the
                // hatch line is counted by one of its two segments only.
                const bool crosses = (f1 <= 0.0 && f2 > 0.0) || (f2 <= 0.0 && f1 > 0.0);
                if (!crosses)
                    continue;
                const double s = f1 / (f1 - f2);
                // Points on the line are (t, t - c), so its x coordinate is a
                // valid monotonic parameter along the line.
                crossings.push_back(x1 + s * (x2 - x1));
            }
        }
        if (crossings.size() < 2)
            continue;
        std::sort(crossings.begin(), crossings.end());
        for (std::size_t i = 0; i + 1 < crossings.size(); i += 2) {
            const double a = crossings[i];
            const double b = crossings[i + 1];
            if (b - a < 1e-9)
                continue;
            out.push_back(DrawingPolyline{{a, a - c}, {b, b - c}});
            if (out.size() >= max_segments)
                return out;
        }
    }
    return out;
}

// Cut `shape` with the requested plane and collect the exposed cut faces.
// Throws (with a descriptive message) for every degenerate case: non-solid
// input, a plane that misses the solid, or a zero-area cross-section.
static SectionGeometry compute_section_geometry(const OcctShape& shape, const std::string& view,
                                                const SectionSpec& spec) {
    int axis = 2;
    const gp_Dir normal = section_plane_normal(spec.plane, axis);
    // NaN is the "no offset given" sentinel: the caller wants the shape's
    // mid-plane, resolved below once the bounding box is known.  Infinities
    // are still a hard error.
    if (!std::isfinite(spec.offset) && !std::isnan(spec.offset))
        throw std::runtime_error("export_svg/dxf: section offset must be finite");

    // Sectioning only makes sense for solids — a wire, face, or shell has no
    // material to cut away and would yield no cross-section at all.
    if (!TopExp_Explorer(shape.get(), TopAbs_SOLID).More())
        throw std::runtime_error("export_svg/dxf: section requires a solid shape");

    Bnd_Box bbox;
    BRepBndLib::Add(shape.get(), bbox);
    if (bbox.IsVoid())
        throw std::runtime_error("export_svg/dxf: section input has an empty bounding box");
    double lo[3] = {0, 0, 0};
    double hi[3] = {0, 0, 0};
    bbox.Get(lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);

    const double span = std::max({hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2], 1.0});
    const double tol = 1e-7 * span;

    // Default to the shape's mid-plane along the cut axis.  Defaulting to 0
    // instead would put the plane on (or outside) the boundary of any part
    // that does not straddle the origin — which is the common case, since
    // `box()` and friends start at the origin.
    const double offset = std::isnan(spec.offset) ? 0.5 * (lo[axis] + hi[axis]) : spec.offset;

    if (offset <= lo[axis] + tol || offset >= hi[axis] - tol) {
        std::ostringstream oss;
        oss << "export_svg/dxf: section plane \"" << spec.plane << "\" at offset " << offset
            << " does not intersect the shape (extent along that axis is " << lo[axis] << " … "
            << hi[axis] << ")";
        throw std::runtime_error(oss.str());
    }

    // Cutting tool: an axis-aligned box that covers the whole half-space in
    // front of the plane (and comfortably overshoots the shape elsewhere).
    double box_lo[3] = {lo[0] - span, lo[1] - span, lo[2] - span};
    double box_hi[3] = {hi[0] + span, hi[1] + span, hi[2] + span};
    box_lo[axis] = offset;
    BRepPrimAPI_MakeBox tool(gp_Pnt(box_lo[0], box_lo[1], box_lo[2]),
                             gp_Pnt(box_hi[0], box_hi[1], box_hi[2]));
    tool.Build();
    if (!tool.IsDone())
        throw std::runtime_error("export_svg/dxf: section could not build the cutting half-space");

    BRepAlgoAPI_Cut cut(shape.get(), tool.Shape());
    cut.Build();
    if (!cut.IsDone())
        throw std::runtime_error("export_svg/dxf: section boolean cut failed");
    const TopoDS_Shape retained = cut.Shape();
    if (retained.IsNull() || !TopExp_Explorer(retained, TopAbs_SOLID).More())
        throw std::runtime_error("export_svg/dxf: section removed the entire shape — "
                                 "check the plane offset");

    SectionGeometry out;
    double total_area = 0.0;
    for (TopExp_Explorer fexp(retained, TopAbs_FACE); fexp.More(); fexp.Next()) {
        const TopoDS_Face& face = TopoDS::Face(fexp.Current());
        BRepAdaptor_Surface surf(face);
        if (surf.GetType() != GeomAbs_Plane)
            continue;
        const gp_Pln pln = surf.Plane();
        // Keep only the planar faces that lie exactly on the cutting plane:
        // parallel normal, and a reference point at the requested offset.
        if (std::abs(pln.Axis().Direction().Dot(normal)) < 0.999)
            continue;
        if (std::abs(pln.Location().Coord(axis + 1) - offset) > 1e-6 * span)
            continue;

        GProp_GProps props;
        BRepGProp::SurfaceProperties(face, props);
        total_area += props.Mass();

        DrawingPolylines loops;
        for (TopExp_Explorer eexp(face, TopAbs_EDGE); eexp.More(); eexp.Next()) {
            BRepAdaptor_Curve curve(TopoDS::Edge(eexp.Current()));
            const double t0 = curve.FirstParameter();
            const double t1 = curve.LastParameter();
            if (!(t1 > t0))
                continue;
            DrawingPolyline pts;
            pts.reserve(HLR_SAMPLES_PER_EDGE + 1);
            for (int i = 0; i <= HLR_SAMPLES_PER_EDGE; ++i) {
                const double t = t0 + (t1 - t0) * i / HLR_SAMPLES_PER_EDGE;
                pts.push_back(project_point(view, curve.Value(t)));
            }
            loops.push_back(std::move(pts));
        }
        if (!loops.empty())
            out.face_loops.push_back(std::move(loops));
    }

    if (out.face_loops.empty() || total_area <= 1e-9)
        throw std::runtime_error("export_svg/dxf: section produced a zero-area cross-section");

    out.retained = wrap(retained);
    return out;
}

static HlrProjection hlr_project(const OcctShape& shape, const std::string& view);

static DrawingViewData build_drawing_view(const OcctShape& shape, const std::string& view,
                                          double scale, bool hidden, bool center_marks,
                                          bool dimensions, bool callouts, bool ordinate,
                                          const SectionSpec& section) {
    // When a section is requested, everything downstream (projection, centre
    // marks, callouts) works on the material left behind the cutting plane.
    std::unique_ptr<OcctShape> cut_shape;
    std::vector<DrawingPolylines> section_faces;
    if (section.active) {
        SectionGeometry geom = compute_section_geometry(shape, view, section);
        cut_shape = std::move(geom.retained);
        section_faces = std::move(geom.face_loops);
    }
    const OcctShape& source = cut_shape ? *cut_shape : shape;

    // Cut-face outlines and hatching, both in final drawing coordinates.
    DrawingPolylines section_outline;
    DrawingPolylines hatch;
    for (auto& loops : section_faces) {
        scale_polylines(loops, scale);
        DrawingPolylines face_hatch = build_section_hatch(loops, SECTION_HATCH_SPACING);
        hatch.insert(hatch.end(), std::make_move_iterator(face_hatch.begin()),
                     std::make_move_iterator(face_hatch.end()));
        section_outline.insert(section_outline.end(), std::make_move_iterator(loops.begin()),
                               std::make_move_iterator(loops.end()));
    }

    auto projection = hlr_project(source, view);
    scale_polylines(projection.visible, scale);
    scale_polylines(projection.hidden, scale);

    // Feature centres serve two options: the crosshair marks and the ordinate
    // dimensions, so they are located whenever either is asked for.
    auto marks = (center_marks || ordinate) ? collect_center_marks(source, view)
                                            : std::vector<DrawingMark>{};
    for (auto& mark : marks) {
        mark.x *= scale;
        mark.y *= scale;
        mark.size *= scale;
    }

    auto callout_list = callouts ? collect_callouts(source, view) : std::vector<DrawingCallout>{};
    for (auto& callout : callout_list) {
        callout.x *= scale;
        callout.y *= scale;
        callout.leader_x *= scale;
        callout.leader_y *= scale;
    }

    double geom_xmin = 1e30, geom_xmax = -1e30, geom_ymin = 1e30, geom_ymax = -1e30;
    auto include_bounds = [&](const DrawingPolylines& polylines) {
        for (auto& pl : polylines) {
            for (auto& [x, y] : pl) {
                geom_xmin = std::min(geom_xmin, x);
                geom_xmax = std::max(geom_xmax, x);
                geom_ymin = std::min(geom_ymin, y);
                geom_ymax = std::max(geom_ymax, y);
            }
        }
    };
    include_bounds(projection.visible);
    if (hidden)
        include_bounds(projection.hidden);
    include_bounds(section_outline);
    include_bounds(hatch);
    // Only the drawn crosshairs affect the extents; centres located purely to
    // place ordinates lie inside the geometry anyway.
    if (center_marks) {
        for (const auto& mark : marks) {
            geom_xmin = std::min(geom_xmin, mark.x - mark.size);
            geom_xmax = std::max(geom_xmax, mark.x + mark.size);
            geom_ymin = std::min(geom_ymin, mark.y - mark.size);
            geom_ymax = std::max(geom_ymax, mark.y + mark.size);
        }
    }
    for (const auto& callout : callout_list) {
        geom_xmin = std::min(geom_xmin, std::min(callout.x, callout.leader_x));
        geom_xmax = std::max(geom_xmax, std::max(callout.x, callout.leader_x));
        geom_ymin = std::min(geom_ymin, std::min(callout.y, callout.leader_y));
        geom_ymax = std::max(geom_ymax, std::max(callout.y, callout.leader_y));
        geom_xmax = std::max(geom_xmax, callout.leader_x + callout.text.size() * 1.8);
        geom_ymax = std::max(geom_ymax, callout.leader_y + 2.0);
    }
    if (geom_xmin == 1e30)
        throw std::runtime_error("export_svg/dxf: no drawing edges found after projection");

    double xmin = geom_xmin;
    double xmax = geom_xmax;
    double ymin = geom_ymin;
    double ymax = geom_ymax;
    if (dimensions) {
        xmin -= 14.0;
        xmax += 14.0;
        ymin -= 14.0;
        ymax += 7.0;
    }

    // The datum corner is the lower-left of the projected geometry, so the
    // ordinates read as distances across the part itself.
    auto ordinate_list = ordinate ? collect_ordinates(marks, geom_xmin, geom_ymin, scale)
                                  : std::vector<DrawingOrdinate>{};
    if (ordinate) {
        xmin = std::min(xmin, geom_xmin - ORDINATE_BASELINE - ORDINATE_LABEL_ROOM);
        ymin = std::min(ymin, geom_ymin - ORDINATE_BASELINE - ORDINATE_LABEL_ROOM);
    }

    DrawingViewData view_data;
    view_data.name = view;
    view_data.visible = std::move(projection.visible);
    view_data.hidden = std::move(projection.hidden);
    view_data.section_outline = std::move(section_outline);
    view_data.hatch = std::move(hatch);
    view_data.marks = std::move(marks);
    view_data.callouts = std::move(callout_list);
    view_data.ordinates = std::move(ordinate_list);
    view_data.geom_xmin = geom_xmin;
    view_data.geom_xmax = geom_xmax;
    view_data.geom_ymin = geom_ymin;
    view_data.geom_ymax = geom_ymax;
    view_data.xmin = xmin;
    view_data.xmax = xmax;
    view_data.ymin = ymin;
    view_data.ymax = ymax;
    view_data.width = geom_xmax - geom_xmin;
    view_data.height = geom_ymax - geom_ymin;
    return view_data;
}

// Parametric interval of the segment P0→P1 that lies inside the circle.
//
// Solving |P0 + t·d − C|² = r² for t gives the two boundary crossings; clamping
// the root interval to [0, 1] yields the part of *this* segment that is inside.
// Returns false when the segment misses the circle entirely.  Working
// analytically rather than by testing endpoints means an edge that crosses the
// boundary is cut exactly at the boundary, so a detail view's geometry stops on
// its border circle instead of overshooting to the next vertex.
static bool circle_clip_interval(double x0, double y0, double x1, double y1, double cx, double cy,
                                 double r, double& t0, double& t1) {
    const double dx = x1 - x0, dy = y1 - y0;
    const double fx = x0 - cx, fy = y0 - cy;
    const double a = dx * dx + dy * dy;
    const double c = fx * fx + fy * fy - r * r;
    if (a <= 1e-18) {
        // Degenerate (zero-length) segment: it is either in or out as a point.
        if (c > 0.0)
            return false;
        t0 = 0.0;
        t1 = 1.0;
        return true;
    }
    const double b = 2.0 * (fx * dx + fy * dy);
    const double disc = b * b - 4.0 * a * c;
    if (disc < 0.0)
        return false; // the infinite line misses the circle
    const double sq = std::sqrt(disc);
    t0 = std::max(0.0, (-b - sq) / (2.0 * a));
    t1 = std::min(1.0, (-b + sq) / (2.0 * a));
    return t1 > t0;
}

// Clip every polyline to the interior of a circle, splitting one that leaves
// and re-enters into separate pieces so the gap is not bridged by a false edge.
static DrawingPolylines clip_polylines_to_circle(const DrawingPolylines& src, double cx, double cy,
                                                 double r) {
    DrawingPolylines out;
    for (const auto& pl : src) {
        if (pl.size() < 2)
            continue;
        DrawingPolyline run;
        auto flush = [&]() {
            if (run.size() >= 2)
                out.push_back(run);
            run.clear();
        };
        for (std::size_t i = 0; i + 1 < pl.size(); ++i) {
            const auto [x0, y0] = pl[i];
            const auto [x1, y1] = pl[i + 1];
            double t0 = 0.0, t1 = 0.0;
            if (!circle_clip_interval(x0, y0, x1, y1, cx, cy, r, t0, t1)) {
                flush();
                continue;
            }
            const double ax = x0 + (x1 - x0) * t0, ay = y0 + (y1 - y0) * t0;
            const double bx = x0 + (x1 - x0) * t1, by = y0 + (y1 - y0) * t1;
            // Continue the current run only when this piece starts where the
            // last one ended; otherwise the polyline left the circle in between.
            const bool joins = !run.empty() && std::abs(run.back().first - ax) < 1e-9 &&
                               std::abs(run.back().second - ay) < 1e-9;
            if (!joins) {
                flush();
                run.emplace_back(ax, ay);
            }
            run.emplace_back(bx, by);
        }
        flush();
    }
    return out;
}

// Render a magnification ratio the way a drawing states it: "4:1" when the
// factor is a whole number, "2.5:1" otherwise.
static std::string format_detail_ratio(double scale) {
    std::ostringstream oss;
    if (std::abs(scale - std::round(scale)) < 1e-9)
        oss << static_cast<long long>(std::llround(scale));
    else
        oss << format_measurement(scale);
    oss << ":1";
    return oss.str();
}

// Mark on the parent view the region a detail view magnifies, growing the
// view's bounds so the circle and its label are not clipped by the canvas.
static void attach_detail_marker(DrawingViewData& view, const DetailSpec& spec, double scale) {
    view.detail_marker = true;
    view.detail_x = spec.x * scale;
    view.detail_y = spec.y * scale;
    view.detail_r = spec.radius * scale;
    view.detail_label = spec.label;
    // The label sits up and to the right of the circle; allow for both.
    view.xmin = std::min(view.xmin, view.detail_x - view.detail_r);
    view.xmax = std::max(view.xmax, view.detail_x + view.detail_r + 4.0);
    view.ymin = std::min(view.ymin, view.detail_y - view.detail_r);
    view.ymax = std::max(view.ymax, view.detail_y + view.detail_r + 4.0);
}

// Build the magnified close-up itself.
//
// A detail view is just another DrawingViewData: the parent's already-projected
// polylines clipped to the region, recentred on the origin and multiplied by the
// magnification.  Reusing the type means the existing SVG and DXF writers render
// it with no special cases, and hidden lines, section outlines and hatching all
// come along for free.
//
// Centre marks and diameter callouts are deliberately *not* carried over: they
// are placed relative to the parent's geometry and would need re-deriving from
// the clipped region to land correctly, which is not worth doing until someone
// asks for it.
static DrawingViewData build_detail_view(const DrawingViewData& parent, const DetailSpec& spec,
                                         double scale) {
    const double cx = spec.x * scale;
    const double cy = spec.y * scale;
    const double r = spec.radius * scale;
    const double m = spec.scale;

    // Clip in the parent's coordinates, then recentre and magnify.
    auto transform = [&](DrawingPolylines polylines) {
        for (auto& pl : polylines)
            for (auto& [x, y] : pl) {
                x = (x - cx) * m;
                y = (y - cy) * m;
            }
        return polylines;
    };

    DrawingViewData detail;
    detail.name = parent.name + "-detail";
    detail.visible = transform(clip_polylines_to_circle(parent.visible, cx, cy, r));
    detail.hidden = transform(clip_polylines_to_circle(parent.hidden, cx, cy, r));
    detail.section_outline = transform(clip_polylines_to_circle(parent.section_outline, cx, cy, r));
    detail.hatch = transform(clip_polylines_to_circle(parent.hatch, cx, cy, r));

    if (detail.visible.empty() && detail.section_outline.empty()) {
        std::ostringstream oss;
        oss << "export_svg/dxf: detail region at (" << format_measurement(spec.x) << ", "
            << format_measurement(spec.y) << ") radius " << format_measurement(spec.radius)
            << " contains no drawing geometry — check the centre is stated in the " << parent.name
            << " view's own axes";
        throw std::runtime_error(oss.str());
    }

    // The border circle bounds the view exactly: everything inside was clipped
    // to it, so there is no need to walk the points to find the extents.
    const double br = r * m;
    detail.detail_marker = true;
    detail.detail_x = 0.0;
    detail.detail_y = 0.0;
    detail.detail_r = br;
    detail.caption = "DETAIL " + spec.label + " (" + format_detail_ratio(m) + ")";
    detail.geom_xmin = -br;
    detail.geom_xmax = br;
    detail.geom_ymin = -br;
    detail.geom_ymax = br;
    detail.xmin = -br;
    detail.xmax = br;
    detail.ymin = -br - DETAIL_CAPTION_GAP;
    detail.ymax = br;
    detail.width = 2.0 * br;
    detail.height = 2.0 * br;
    return detail;
}

// Radius of the ring the balloons sit on, measured from the view's geometry
// centre.  Shared by placement and by the canvas-bounds calculation so the two
// cannot disagree and clip a balloon off the page.
static double balloon_ring_radius(const DrawingViewData& view) {
    const double half_w = (view.geom_xmax - view.geom_xmin) * 0.5;
    const double half_h = (view.geom_ymax - view.geom_ymin) * 0.5;
    return std::sqrt(half_w * half_w + half_h * half_h) + BALLOON_RING_GAP + BALLOON_RADIUS;
}

// Arrange balloons on a ring around the view's geometry.
//
// Slots are assigned in order of each anchor's bearing from the centre, so the
// leaders fan out in the same rotational order as the parts they point at and
// therefore do not cross.  Spacing the slots evenly rather than leaving each
// balloon on its own bearing is what stops two components in the same corner
// from landing on top of each other.
static void place_balloons(std::vector<DrawingBalloon>& balloons, const DrawingViewData& view) {
    if (balloons.empty())
        return;
    const double cx = (view.geom_xmin + view.geom_xmax) * 0.5;
    const double cy = (view.geom_ymin + view.geom_ymax) * 0.5;
    const double ring = balloon_ring_radius(view);

    auto angle_of = [&](std::size_t i) {
        return std::atan2(balloons[i].y - cy, balloons[i].x - cx);
    };
    std::vector<std::size_t> order(balloons.size());
    for (std::size_t i = 0; i < order.size(); ++i)
        order[i] = i;
    std::stable_sort(order.begin(), order.end(),
                     [&](std::size_t a, std::size_t b) { return angle_of(a) < angle_of(b); });

    const double start = angle_of(order[0]);
    const double step = 2.0 * M_PI / static_cast<double>(balloons.size());
    for (std::size_t slot = 0; slot < order.size(); ++slot) {
        DrawingBalloon& b = balloons[order[slot]];
        const double theta = start + step * static_cast<double>(slot);
        b.bx = cx + ring * std::cos(theta);
        b.by = cy + ring * std::sin(theta);
    }
}

// Where the leader meets the balloon: on the circle's edge, facing the anchor.
// Starting at the centre would draw a line through the number.
static std::pair<double, double> balloon_leader_start(const DrawingBalloon& b) {
    const double dx = b.x - b.bx;
    const double dy = b.y - b.by;
    const double len = std::sqrt(dx * dx + dy * dy);
    if (len < 1e-9)
        return {b.bx, b.by};
    return {b.bx + BALLOON_RADIUS * dx / len, b.by + BALLOON_RADIUS * dy / len};
}

// Column widths for the parts list, from the longest cell in each column.
static std::vector<double> bom_column_widths(const std::vector<std::vector<std::string>>& rows) {
    std::size_t columns = 0;
    for (const auto& row : rows)
        columns = std::max(columns, row.size());
    std::vector<double> widths(columns, 0.0);
    for (const auto& row : rows)
        for (std::size_t c = 0; c < row.size(); ++c)
            widths[c] = std::max(widths[c], static_cast<double>(row[c].size()) * BOM_CHAR_WIDTH +
                                                BOM_CELL_PAD);
    return widths;
}

static double bom_table_width(const std::vector<std::vector<std::string>>& rows) {
    double total = 0.0;
    for (double w : bom_column_widths(rows))
        total += w;
    return total;
}

static double bom_table_height(const std::vector<std::vector<std::string>>& rows) {
    return static_cast<double>(rows.size()) * BOM_ROW_HEIGHT;
}

static void include_placed_bounds(double& xmin, double& xmax, double& ymin, double& ymax,
                                  const DrawingViewData& view, double offset_x, double offset_y) {
    xmin = std::min(xmin, view.xmin + offset_x);
    xmax = std::max(xmax, view.xmax + offset_x);
    ymin = std::min(ymin, view.ymin + offset_y);
    ymax = std::max(ymax, view.ymax + offset_y);
}

static std::pair<double, double> project_point_2d(const std::string& view, double x, double y,
                                                  double z) {
    if (view == "front")
        return {x, z};
    if (view == "side")
        return {y, z};
    return {x, y};
}

// Placement geometry shared by the SVG and DXF dimension annotations:
// extension-line extents, label midpoints, and the offset dimension lines.
struct DimensionLayout {
    static constexpr double gap = 8.0;          // gap between geometry and dimension line
    static constexpr double tick = 1.5;         // slash tick half-length at line ends
    static constexpr double label_offset = 3.5; // distance from dimension line to label
    static constexpr double font_size = 3.0;    // annotation text height
    double xmin, xmax, ymin, ymax;              // placed geometry bounds
    double width, height;                       // measured extents (unplaced)
    double hx, hy;                              // label midpoints
    double dim_x, dim_y;                        // vertical / horizontal dimension line positions
};

static DimensionLayout compute_dimension_layout(const DrawingViewData& view, double offset_x,
                                                double offset_y) {
    DimensionLayout d{};
    d.xmin = view.geom_xmin + offset_x;
    d.xmax = view.geom_xmax + offset_x;
    d.ymin = view.geom_ymin + offset_y;
    d.ymax = view.geom_ymax + offset_y;
    d.width = view.width;
    d.height = view.height;
    d.hx = (d.xmin + d.xmax) * 0.5;
    d.hy = (d.ymin + d.ymax) * 0.5;
    d.dim_y = d.ymin - DimensionLayout::gap;
    d.dim_x = d.xmin - DimensionLayout::gap;
    return d;
}

// Compose a dimension label ("[axis ]value[ tol]") shared by SVG and DXF output.
static std::string compose_dim_label(const char* axis, double value, const std::string& tol) {
    std::string label = format_measurement(value);
    if (!tol.empty())
        label += " " + tol;
    if (axis)
        label = std::string(axis) + " " + label;
    return label;
}

// Escape text destined for an SVG text node.
//
// Parts-list cells carry user-chosen component names and materials, so an
// ampersand or angle bracket would otherwise produce a document no parser will
// open.  (The older annotation paths — datum labels, feature-control frames,
// diameter callouts — still emit raw text; see Project Improvements.)
static std::string svg_text(const std::string& raw) {
    std::string out;
    out.reserve(raw.size());
    for (char c : raw) {
        switch (c) {
        case '&':
            out += "&amp;";
            break;
        case '<':
            out += "&lt;";
            break;
        case '>':
            out += "&gt;";
            break;
        default:
            out.push_back(c);
        }
    }
    return out;
}

static void write_svg_view(std::ofstream& f, const DrawingViewData& view, double offset_x,
                           double offset_y, bool hidden, bool center_marks, bool dimensions,
                           bool callouts, bool sheet_mode, const char* width_axis = nullptr,
                           const char* height_axis = nullptr, double tolerance_plus = 0.0,
                           double tolerance_minus = 0.0) {
    auto write_polyline_points = [&](const DrawingPolyline& polyline) {
        for (auto& [x, y] : polyline) {
            f << (x + offset_x) << "," << (-(y + offset_y)) << " ";
        }
    };

    if (sheet_mode) {
        f << "  <g class=\"view view-" << view.name << "\"";
        f << " stroke=\"black\" stroke-width=\"0.3\" fill=\"none\"";
        f << " stroke-linecap=\"round\" stroke-linejoin=\"round\">\n";
    } else {
        f << "  <g class=\"visible\" stroke=\"black\" stroke-width=\"0.3\" fill=\"none\"";
        f << " stroke-linecap=\"round\" stroke-linejoin=\"round\">\n";
    }
    for (auto& pts : view.visible) {
        if (pts.size() < 2)
            continue;
        f << "    <polyline points=\"";
        write_polyline_points(pts);
        f << "\"/>\n";
    }
    f << "  </g>\n";
    // Section hatching first, so the cut outline is drawn on top of it.
    if (!view.hatch.empty()) {
        f << "  <g class=\"";
        if (sheet_mode)
            f << "view view-" << view.name << " ";
        f << "hatch\" stroke=\"#111827\" stroke-width=\"0.15\" fill=\"none\"";
        f << " stroke-linecap=\"round\">\n";
        for (auto& pts : view.hatch) {
            if (pts.size() < 2)
                continue;
            f << "    <line x1=\"" << (pts.front().first + offset_x) << "\" y1=\""
              << (-(pts.front().second + offset_y)) << "\" x2=\"" << (pts.back().first + offset_x)
              << "\" y2=\"" << (-(pts.back().second + offset_y)) << "\"/>\n";
        }
        f << "  </g>\n";
    }
    // Cut boundary: same weight as the visible outline.
    if (!view.section_outline.empty()) {
        f << "  <g class=\"";
        if (sheet_mode)
            f << "view view-" << view.name << " ";
        f << "section\" stroke=\"black\" stroke-width=\"0.3\" fill=\"none\"";
        f << " stroke-linecap=\"round\" stroke-linejoin=\"round\">\n";
        for (auto& pts : view.section_outline) {
            if (pts.size() < 2)
                continue;
            f << "    <polyline points=\"";
            write_polyline_points(pts);
            f << "\"/>\n";
        }
        f << "  </g>\n";
    }
    if (hidden) {
        if (sheet_mode) {
            f << "  <g class=\"view view-" << view.name
              << " hidden\" stroke=\"#888\" stroke-width=\"0.25\" fill=\"none\"";
            f << " stroke-dasharray=\"2 1.5\" stroke-linecap=\"round\" "
                 "stroke-linejoin=\"round\">\n";
        } else {
            f << "  <g class=\"hidden\" stroke=\"#888\" stroke-width=\"0.25\" fill=\"none\"";
            f << " stroke-dasharray=\"2 1.5\" stroke-linecap=\"round\" "
                 "stroke-linejoin=\"round\">\n";
        }
        for (auto& pts : view.hidden) {
            if (pts.size() < 2)
                continue;
            f << "    <polyline points=\"";
            write_polyline_points(pts);
            f << "\"/>\n";
        }
        f << "  </g>\n";
    }
    if (center_marks && !view.marks.empty()) {
        if (sheet_mode) {
            f << "  <g class=\"view view-" << view.name
              << " center-marks\" stroke=\"#6b7280\" stroke-width=\"0.25\" fill=\"none\"";
            f << " stroke-linecap=\"round\">\n";
        } else {
            f << "  <g class=\"center-marks\" stroke=\"#6b7280\" stroke-width=\"0.25\" "
                 "fill=\"none\"";
            f << " stroke-linecap=\"round\">\n";
        }
        for (const auto& mark : view.marks) {
            const double x = mark.x + offset_x;
            const double y = mark.y + offset_y;
            f << "    <line x1=\"" << (x - mark.size) << "\" y1=\"" << (-y) << "\" x2=\""
              << (x + mark.size) << "\" y2=\"" << (-y) << "\"/>\n";
            f << "    <line x1=\"" << x << "\" y1=\"" << (-(y - mark.size)) << "\" x2=\"" << x
              << "\" y2=\"" << (-(y + mark.size)) << "\"/>\n";
        }
        f << "  </g>\n";
    }
    if (dimensions) {
        const DimensionLayout d = compute_dimension_layout(view, offset_x, offset_y);
        const double tick = DimensionLayout::tick;
        if (sheet_mode) {
            f << "  <g class=\"view view-" << view.name
              << " dimensions\" stroke=\"#9ca3af\" stroke-width=\"0.25\" fill=\"none\"";
        } else {
            f << "  <g class=\"dimensions\" stroke=\"#9ca3af\" stroke-width=\"0.25\" fill=\"none\"";
        }
        f << " stroke-linecap=\"round\" stroke-linejoin=\"round\" font-family=\"monospace\"";
        f << " font-size=\"" << DimensionLayout::font_size << "\">\n";
        f << "    <line x1=\"" << d.xmin << "\" y1=\"" << d.dim_y << "\" x2=\"" << d.xmax
          << "\" y2=\"" << d.dim_y << "\"/>\n";
        f << "    <line x1=\"" << d.xmin << "\" y1=\"" << d.dim_y << "\" x2=\"" << d.xmin
          << "\" y2=\"" << d.ymin << "\"/>\n";
        f << "    <line x1=\"" << d.xmax << "\" y1=\"" << d.dim_y << "\" x2=\"" << d.xmax
          << "\" y2=\"" << d.ymin << "\"/>\n";
        f << "    <line x1=\"" << (d.xmin + tick) << "\" y1=\"" << (d.dim_y - tick) << "\" x2=\""
          << (d.xmin - tick) << "\" y2=\"" << (d.dim_y + tick) << "\"/>\n";
        f << "    <line x1=\"" << (d.xmax + tick) << "\" y1=\"" << (d.dim_y - tick) << "\" x2=\""
          << (d.xmax - tick) << "\" y2=\"" << (d.dim_y + tick) << "\"/>\n";
        const bool axis_labels = sheet_mode && width_axis && height_axis;
        const std::string tol = format_tolerance(tolerance_plus, tolerance_minus);
        f << "    <text x=\"" << d.hx << "\" y=\"" << (d.dim_y - DimensionLayout::label_offset)
          << "\" text-anchor=\"middle\" fill=\"#6b7280\">"
          << compose_dim_label(axis_labels ? width_axis : nullptr, d.width, tol) << "</text>\n";
        f << "    <line x1=\"" << d.dim_x << "\" y1=\"" << d.ymin << "\" x2=\"" << d.dim_x
          << "\" y2=\"" << d.ymax << "\"/>\n";
        f << "    <line x1=\"" << (d.dim_x - tick) << "\" y1=\"" << (d.ymin + tick) << "\" x2=\""
          << (d.dim_x + tick) << "\" y2=\"" << (d.ymin - tick) << "\"/>\n";
        f << "    <line x1=\"" << (d.dim_x - tick) << "\" y1=\"" << (d.ymax + tick) << "\" x2=\""
          << (d.dim_x + tick) << "\" y2=\"" << (d.ymax - tick) << "\"/>\n";
        f << "    <text x=\"" << (d.dim_x - DimensionLayout::label_offset) << "\" y=\"" << d.hy
          << "\" text-anchor=\"middle\" fill=\"#6b7280\" transform=\"rotate(-90 "
          << (d.dim_x - DimensionLayout::label_offset) << " " << d.hy << ")\">"
          << compose_dim_label(axis_labels ? height_axis : nullptr, d.height, tol) << "</text>\n";
        f << "  </g>\n";
    }
    if (callouts && !view.callouts.empty()) {
        if (sheet_mode) {
            f << "  <g class=\"view view-" << view.name
              << " callouts\" stroke=\"#b45309\" stroke-width=\"0.25\" fill=\"none\"";
        } else {
            f << "  <g class=\"callouts\" stroke=\"#b45309\" stroke-width=\"0.25\" fill=\"none\"";
        }
        f << " stroke-linecap=\"round\" stroke-linejoin=\"round\" font-family=\"monospace\"";
        f << " font-size=\"3.0\">\n";
        for (const auto& callout : view.callouts) {
            const double x = callout.x + offset_x;
            const double y = callout.y + offset_y;
            const double lx = callout.leader_x + offset_x;
            const double ly = callout.leader_y + offset_y;
            f << "    <line x1=\"" << x << "\" y1=\"" << (-y) << "\" x2=\"" << lx << "\" y2=\""
              << (-ly) << "\"/>\n";
            f << "    <text x=\"" << (lx + 2.0) << "\" y=\"" << (-ly) << "\" fill=\"#b45309\">";
            f << callout.text << "</text>\n";
        }
        f << "  </g>\n";
    }
    // Ordinate dimensions: witness lines from each feature centre out to a
    // baseline along the bottom and left, labelled with the distance from the
    // datum corner.  The datum itself is drawn as a short cross at the corner.
    if (!view.ordinates.empty()) {
        const double x0 = view.geom_xmin + offset_x;
        const double y0 = view.geom_ymin + offset_y;
        const double base_y = y0 - ORDINATE_BASELINE;
        const double base_x = x0 - ORDINATE_BASELINE;
        f << "  <g class=\"";
        if (sheet_mode)
            f << "view view-" << view.name << " ";
        f << "ordinates\" stroke=\"#0f766e\" stroke-width=\"0.2\" fill=\"none\"";
        f << " stroke-linecap=\"round\" font-family=\"monospace\" font-size=\"2.6\">\n";
        // Datum corner marker.
        f << "    <line x1=\"" << (x0 - 2.0) << "\" y1=\"" << (-y0) << "\" x2=\"" << (x0 + 2.0)
          << "\" y2=\"" << (-y0) << "\"/>\n";
        f << "    <line x1=\"" << x0 << "\" y1=\"" << (-(y0 - 2.0)) << "\" x2=\"" << x0
          << "\" y2=\"" << (-(y0 + 2.0)) << "\"/>\n";
        for (const auto& ord : view.ordinates) {
            if (ord.horizontal) {
                const double x = ord.at + offset_x;
                f << "    <line x1=\"" << x << "\" y1=\"" << (-y0) << "\" x2=\"" << x << "\" y2=\""
                  << (-base_y) << "\"/>\n";
                // Rotated so neighbouring labels cannot collide when two
                // features sit a few millimetres apart.
                f << "    <text x=\"" << x << "\" y=\"" << (-(base_y - 1.5))
                  << "\" text-anchor=\"end\" fill=\"#0f766e\" transform=\"rotate(-90 " << x << " "
                  << (-(base_y - 1.5)) << ")\">" << ord.label << "</text>\n";
            } else {
                const double y = ord.at + offset_y;
                f << "    <line x1=\"" << x0 << "\" y1=\"" << (-y) << "\" x2=\"" << base_x
                  << "\" y2=\"" << (-y) << "\"/>\n";
                f << "    <text x=\"" << (base_x - 1.5) << "\" y=\"" << (-y)
                  << "\" text-anchor=\"end\" fill=\"#0f766e\">" << ord.label << "</text>\n";
            }
        }
        f << "  </g>\n";
    }
    // Detail bubble: the region marker on a parent view, or the border circle
    // and caption on the magnified view itself.
    if (view.detail_marker) {
        const double x = view.detail_x + offset_x;
        const double y = view.detail_y + offset_y;
        f << "  <g class=\"";
        if (sheet_mode)
            f << "view view-" << view.name << " ";
        f << "detail\" stroke=\"#1d4ed8\" stroke-width=\"0.25\" fill=\"none\"";
        f << " font-family=\"monospace\" font-size=\"3.0\">\n";
        f << "    <circle cx=\"" << x << "\" cy=\"" << (-y) << "\" r=\"" << view.detail_r
          << "\"/>\n";
        if (!view.detail_label.empty()) {
            // Up and to the right of the circle, clear of the geometry inside.
            const double lx = x + view.detail_r * 0.71 + 1.5;
            const double ly = y + view.detail_r * 0.71 + 1.5;
            f << "    <text x=\"" << lx << "\" y=\"" << (-ly) << "\" fill=\"#1d4ed8\">"
              << svg_text(view.detail_label) << "</text>\n";
        }
        if (!view.caption.empty()) {
            f << "    <text x=\"" << x << "\" y=\""
              << (-(y - view.detail_r - DETAIL_CAPTION_GAP + 2.0))
              << "\" text-anchor=\"middle\" fill=\"#1d4ed8\">" << svg_text(view.caption)
              << "</text>\n";
        }
        f << "  </g>\n";
    }
}

static void write_dxf_view(std::ofstream& f, const DrawingViewData& view, double offset_x,
                           double offset_y, bool hidden, bool center_marks, bool dimensions,
                           bool callouts, bool sheet_mode, const char* width_axis = nullptr,
                           const char* height_axis = nullptr, double tolerance_plus = 0.0,
                           double tolerance_minus = 0.0) {
    auto write_lines = [&](const DrawingPolylines& polylines, const char* layer) {
        for (auto& pts : polylines) {
            for (std::size_t i = 0; i + 1 < pts.size(); ++i) {
                auto [x1, y1] = pts[i];
                auto [x2, y2] = pts[i + 1];
                x1 += offset_x;
                y1 += offset_y;
                x2 += offset_x;
                y2 += offset_y;
                if (std::abs(x2 - x1) < 1e-9 && std::abs(y2 - y1) < 1e-9)
                    continue;
                f << "  0\nLINE\n";
                f << "  8\n" << layer << "\n";
                f << " 10\n" << x1 << "\n";
                f << " 20\n" << y1 << "\n";
                f << " 30\n0.0\n";
                f << " 11\n" << x2 << "\n";
                f << " 21\n" << y2 << "\n";
                f << " 31\n0.0\n";
            }
        }
    };

    write_lines(view.visible, "0");
    // Section hatching gets its own layer, mirroring how HIDDEN is handled;
    // the cut boundary shares layer 0 so it keeps the visible-edge weight.
    write_lines(view.hatch, "HATCH");
    write_lines(view.section_outline, "0");
    if (hidden)
        write_lines(view.hidden, "HIDDEN");
    if (center_marks && !view.marks.empty()) {
        for (const auto& mark : view.marks) {
            const double x = mark.x + offset_x;
            const double y = mark.y + offset_y;
            f << "  0\nLINE\n";
            f << "  8\nCENTER\n";
            f << " 10\n" << (x - mark.size) << "\n";
            f << " 20\n" << y << "\n";
            f << " 30\n0.0\n";
            f << " 11\n" << (x + mark.size) << "\n";
            f << " 21\n" << y << "\n";
            f << " 31\n0.0\n";
            f << "  0\nLINE\n";
            f << "  8\nCENTER\n";
            f << " 10\n" << x << "\n";
            f << " 20\n" << (y - mark.size) << "\n";
            f << " 30\n0.0\n";
            f << " 11\n" << x << "\n";
            f << " 21\n" << (y + mark.size) << "\n";
            f << " 31\n0.0\n";
        }
    }
    if (dimensions) {
        const DimensionLayout d = compute_dimension_layout(view, offset_x, offset_y);
        const double tick = DimensionLayout::tick;

        auto write_line = [&](double x1, double y1, double x2, double y2) {
            f << "  0\nLINE\n";
            f << "  8\nDIMENSION\n";
            f << " 10\n" << x1 << "\n";
            f << " 20\n" << y1 << "\n";
            f << " 30\n0.0\n";
            f << " 11\n" << x2 << "\n";
            f << " 21\n" << y2 << "\n";
            f << " 31\n0.0\n";
        };
        auto write_text = [&](double x, double y, const std::string& text, double rotation) {
            f << "  0\nTEXT\n";
            f << "  8\nDIMENSION\n";
            f << " 10\n" << x << "\n";
            f << " 20\n" << y << "\n";
            f << " 30\n0.0\n";
            f << " 40\n" << DimensionLayout::font_size << "\n";
            f << "  1\n" << text << "\n";
            if (rotation != 0.0)
                f << " 50\n" << rotation << "\n";
        };

        // Dimension lines: horizontal below the geometry, vertical to its left.
        write_line(d.xmin, d.dim_y, d.xmax, d.dim_y);
        write_line(d.dim_x, d.ymin, d.dim_x, d.ymax);
        // Slash ticks at the horizontal dimension line ends.
        write_line(d.xmin + tick, d.dim_y - tick, d.xmin - tick, d.dim_y + tick);
        write_line(d.xmax + tick, d.dim_y - tick, d.xmax - tick, d.dim_y + tick);

        const bool axis_labels = sheet_mode && width_axis && height_axis;
        const std::string tol = format_tolerance(tolerance_plus, tolerance_minus);
        write_text(d.hx, d.dim_y - DimensionLayout::label_offset,
                   compose_dim_label(axis_labels ? width_axis : nullptr, d.width, tol), 0.0);

        // Slash ticks at the vertical dimension line ends.
        write_line(d.dim_x - tick, d.ymin + tick, d.dim_x + tick, d.ymin - tick);
        write_line(d.dim_x - tick, d.ymax + tick, d.dim_x + tick, d.ymax - tick);

        write_text(d.dim_x - DimensionLayout::label_offset, d.hy,
                   compose_dim_label(axis_labels ? height_axis : nullptr, d.height, tol), -90.0);
    }
    if (callouts && !view.callouts.empty()) {
        auto write_text = [&](double x, double y, const std::string& text) {
            f << "  0\nTEXT\n";
            f << "  8\nCALLOUT\n";
            f << " 10\n" << x << "\n";
            f << " 20\n" << y << "\n";
            f << " 30\n0.0\n";
            f << " 40\n3.0\n";
            f << "  1\n" << text << "\n";
        };
        for (const auto& callout : view.callouts) {
            const double x = callout.x + offset_x;
            const double y = callout.y + offset_y;
            const double lx = callout.leader_x + offset_x;
            const double ly = callout.leader_y + offset_y;
            f << "  0\nLINE\n";
            f << "  8\nCALLOUT\n";
            f << " 10\n" << x << "\n";
            f << " 20\n" << y << "\n";
            f << " 30\n0.0\n";
            f << " 11\n" << lx << "\n";
            f << " 21\n" << ly << "\n";
            f << " 31\n0.0\n";
            write_text(lx + 2.0, ly, callout.text);
        }
    }
    // Ordinate dimensions on their own layer, mirroring the SVG group.
    if (!view.ordinates.empty()) {
        const double x0 = view.geom_xmin + offset_x;
        const double y0 = view.geom_ymin + offset_y;
        const double base_y = y0 - ORDINATE_BASELINE;
        const double base_x = x0 - ORDINATE_BASELINE;
        auto write_line = [&](double x1, double y1, double x2, double y2) {
            f << "  0\nLINE\n";
            f << "  8\nORDINATE\n";
            f << " 10\n" << x1 << "\n";
            f << " 20\n" << y1 << "\n";
            f << " 30\n0.0\n";
            f << " 11\n" << x2 << "\n";
            f << " 21\n" << y2 << "\n";
            f << " 31\n0.0\n";
        };
        // Right-aligned (group code 72 = 2), which needs the second alignment
        // point 11/21 as well.  Alignment matters here: a rotated left-aligned
        // label would grow back over the drawing instead of away from it, which
        // is also why the SVG uses text-anchor="end".
        auto write_text = [&](double x, double y, const std::string& text, double rotation) {
            f << "  0\nTEXT\n";
            f << "  8\nORDINATE\n";
            f << " 10\n" << x << "\n";
            f << " 20\n" << y << "\n";
            f << " 30\n0.0\n";
            f << " 40\n2.6\n";
            f << " 50\n" << rotation << "\n";
            f << " 72\n2\n";
            f << " 11\n" << x << "\n";
            f << " 21\n" << y << "\n";
            f << " 31\n0.0\n";
            f << "  1\n" << text << "\n";
        };
        // Datum corner marker.
        write_line(x0 - 2.0, y0, x0 + 2.0, y0);
        write_line(x0, y0 - 2.0, x0, y0 + 2.0);
        for (const auto& ord : view.ordinates) {
            if (ord.horizontal) {
                const double x = ord.at + offset_x;
                write_line(x, y0, x, base_y);
                write_text(x, base_y - 1.5, ord.label, 90.0);
            } else {
                const double y = ord.at + offset_y;
                write_line(x0, y, base_x, y);
                write_text(base_x - 1.5, y, ord.label, 0.0);
            }
        }
    }
    // Detail bubble on its own layer, so a shop can turn the annotation off
    // without losing geometry — the same treatment HIDDEN and HATCH get.
    if (view.detail_marker) {
        const double x = view.detail_x + offset_x;
        const double y = view.detail_y + offset_y;
        f << "  0\nCIRCLE\n";
        f << "  8\nDETAIL\n";
        f << " 10\n" << x << "\n";
        f << " 20\n" << y << "\n";
        f << " 30\n0.0\n";
        f << " 40\n" << view.detail_r << "\n";
        auto write_detail_text = [&](double tx, double ty, const std::string& text) {
            f << "  0\nTEXT\n";
            f << "  8\nDETAIL\n";
            f << " 10\n" << tx << "\n";
            f << " 20\n" << ty << "\n";
            f << " 30\n0.0\n";
            f << " 40\n3.0\n";
            f << "  1\n" << text << "\n";
        };
        if (!view.detail_label.empty())
            write_detail_text(x + view.detail_r * 0.71 + 1.5, y + view.detail_r * 0.71 + 1.5,
                              view.detail_label);
        if (!view.caption.empty())
            write_detail_text(x - view.detail_r, y - view.detail_r - DETAIL_CAPTION_GAP,
                              view.caption);
    }
}

// Numbered balloons keyed to the parts list, each with a leader ending in a dot
// on the component it identifies.
static void write_svg_balloons(std::ofstream& f, const std::vector<DrawingBalloon>& balloons,
                               double offset_x, double offset_y) {
    if (balloons.empty())
        return;
    f << "  <g class=\"balloons\" stroke=\"#7c2d12\" stroke-width=\"0.25\" fill=\"none\"";
    f << " font-family=\"monospace\" font-size=\"3.0\">\n";
    for (const auto& b : balloons) {
        const double bx = b.bx + offset_x;
        const double by = b.by + offset_y;
        const double ax = b.x + offset_x;
        const double ay = b.y + offset_y;
        auto [lx, ly] = balloon_leader_start(b);
        lx += offset_x;
        ly += offset_y;
        f << "    <line x1=\"" << lx << "\" y1=\"" << (-ly) << "\" x2=\"" << ax << "\" y2=\""
          << (-ay) << "\"/>\n";
        f << "    <circle cx=\"" << bx << "\" cy=\"" << (-by) << "\" r=\"" << BALLOON_RADIUS
          << "\"/>\n";
        f << "    <circle cx=\"" << ax << "\" cy=\"" << (-ay)
          << "\" r=\"0.6\" fill=\"#7c2d12\" stroke=\"none\"/>\n";
        f << "    <text x=\"" << bx << "\" y=\"" << (-(by - 1.1))
          << "\" text-anchor=\"middle\" fill=\"#7c2d12\" stroke=\"none\">" << svg_text(b.label)
          << "</text>\n";
    }
    f << "  </g>\n";
}

// The parts list itself, ruled under the header and closed at the bottom, sat
// below the drawing and left-aligned with it.
static void write_svg_bom_table(std::ofstream& f, const DrawingCanvasBounds& canvas,
                                const std::vector<std::vector<std::string>>& rows) {
    if (rows.empty())
        return;
    const auto widths = bom_column_widths(rows);
    const double total = bom_table_width(rows);
    const double top = canvas.ymin - BOM_TABLE_GAP;
    const double x0 = canvas.xmin;

    f << "  <g class=\"bom\" stroke=\"#334155\" stroke-width=\"0.2\" fill=\"none\"";
    f << " font-family=\"monospace\" font-size=\"" << BOM_FONT_SIZE << "\">\n";
    auto rule = [&](double y) {
        f << "    <line x1=\"" << x0 << "\" y1=\"" << (-y) << "\" x2=\"" << (x0 + total)
          << "\" y2=\"" << (-y) << "\"/>\n";
    };
    rule(top);
    rule(top - BOM_ROW_HEIGHT); // under the header
    rule(top - bom_table_height(rows));
    for (std::size_t r = 0; r < rows.size(); ++r) {
        double x = x0;
        const double text_y = top - static_cast<double>(r) * BOM_ROW_HEIGHT - BOM_ROW_HEIGHT * 0.68;
        for (std::size_t c = 0; c < rows[r].size(); ++c) {
            f << "    <text x=\"" << (x + BOM_CELL_PAD * 0.5) << "\" y=\"" << (-text_y)
              << "\" fill=\"#334155\" stroke=\"none\">" << svg_text(rows[r][c]) << "</text>\n";
            x += widths[c];
        }
    }
    f << "  </g>\n";
}

// DXF equivalents: balloons on a BALLOON layer, the parts list on a BOM layer,
// so a shop can turn either off without touching geometry.
static void write_dxf_balloons(std::ofstream& f, const std::vector<DrawingBalloon>& balloons,
                               double offset_x, double offset_y) {
    for (const auto& b : balloons) {
        const double bx = b.bx + offset_x;
        const double by = b.by + offset_y;
        const double ax = b.x + offset_x;
        const double ay = b.y + offset_y;
        auto [lx, ly] = balloon_leader_start(b);
        lx += offset_x;
        ly += offset_y;
        f << "  0\nLINE\n  8\nBALLOON\n";
        f << " 10\n" << lx << "\n 20\n" << ly << "\n 30\n0.0\n";
        f << " 11\n" << ax << "\n 21\n" << ay << "\n 31\n0.0\n";
        f << "  0\nCIRCLE\n  8\nBALLOON\n";
        f << " 10\n" << bx << "\n 20\n" << by << "\n 30\n0.0\n";
        f << " 40\n" << BALLOON_RADIUS << "\n";
        // Centred text (group code 72 = 1) needs the 11/21 alignment point.
        f << "  0\nTEXT\n  8\nBALLOON\n";
        f << " 10\n" << bx << "\n 20\n" << (by - 1.1) << "\n 30\n0.0\n";
        f << " 40\n3.0\n 72\n1\n";
        f << " 11\n" << bx << "\n 21\n" << (by - 1.1) << "\n 31\n0.0\n";
        f << "  1\n" << b.label << "\n";
    }
}

static void write_dxf_bom_table(std::ofstream& f, const DrawingCanvasBounds& canvas,
                                const std::vector<std::vector<std::string>>& rows) {
    if (rows.empty())
        return;
    const auto widths = bom_column_widths(rows);
    const double total = bom_table_width(rows);
    const double top = canvas.ymin - BOM_TABLE_GAP;
    const double x0 = canvas.xmin;

    auto rule = [&](double y) {
        f << "  0\nLINE\n  8\nBOM\n";
        f << " 10\n" << x0 << "\n 20\n" << y << "\n 30\n0.0\n";
        f << " 11\n" << (x0 + total) << "\n 21\n" << y << "\n 31\n0.0\n";
    };
    rule(top);
    rule(top - BOM_ROW_HEIGHT);
    rule(top - bom_table_height(rows));
    for (std::size_t r = 0; r < rows.size(); ++r) {
        double x = x0;
        const double text_y = top - static_cast<double>(r) * BOM_ROW_HEIGHT - BOM_ROW_HEIGHT * 0.68;
        for (std::size_t c = 0; c < rows[r].size(); ++c) {
            f << "  0\nTEXT\n  8\nBOM\n";
            f << " 10\n" << (x + BOM_CELL_PAD * 0.5) << "\n 20\n" << text_y << "\n 30\n0.0\n";
            f << " 40\n" << BOM_FONT_SIZE << "\n";
            f << "  1\n" << rows[r][c] << "\n";
            x += widths[c];
        }
    }
}

static void write_svg_title_block(std::ofstream& f, const DrawingCanvasBounds& canvas,
                                  const std::string& sheet_name, const std::string& view_name,
                                  bool sheet_mode, double scale, double tolerance_plus,
                                  double tolerance_minus) {
    const double block_w = 42.0;
    const double block_h = 18.0;
    const double x0 = canvas.xmax - block_w - 4.0;
    const double y0 = canvas.ymin + 4.0;
    const double mid_y = y0 + 6.0;
    const double top_y = y0 + 12.0;

    f << "  <g class=\"title-block\" stroke=\"#111827\" stroke-width=\"0.25\" fill=\"none\"";
    f << " font-family=\"monospace\" font-size=\"3.0\">\n";
    f << "    <rect x=\"" << x0 << "\" y=\"" << (-y0 - block_h) << "\" width=\"" << block_w
      << "\" height=\"" << block_h << "\"/>\n";
    f << "    <line x1=\"" << x0 << "\" y1=\"" << (-(y0 + 6.0)) << "\" x2=\"" << (x0 + block_w)
      << "\" y2=\"" << (-(y0 + 6.0)) << "\"/>\n";
    f << "    <line x1=\"" << x0 << "\" y1=\"" << (-(y0 + 12.0)) << "\" x2=\"" << (x0 + block_w)
      << "\" y2=\"" << (-(y0 + 12.0)) << "\"/>\n";
    f << "    <text x=\"" << (x0 + 2.0) << "\" y=\"" << (-(top_y - 1.0))
      << "\" fill=\"#111827\">rrcad</text>\n";
    f << "    <text x=\"" << (x0 + 2.0) << "\" y=\"" << (-(mid_y - 1.0)) << "\" fill=\"#111827\">"
      << (sheet_mode ? sheet_name : view_name) << "</text>\n";
    f << "    <text x=\"" << (x0 + 2.0) << "\" y=\"" << (-(y0 + 1.0))
      << "\" fill=\"#111827\">scale 1:" << (1.0 / scale) << "</text>\n";
    std::string tol = format_tolerance(tolerance_plus, tolerance_minus);
    if (!tol.empty()) {
        f << "    <text x=\"" << (x0 + 22.0) << "\" y=\"" << (-(y0 + 1.0))
          << "\" fill=\"#111827\">tol " << tol << "</text>\n";
    }
    f << "  </g>\n";
}

static void write_svg_gdt_frame(std::ofstream& f, const DrawingCanvasBounds& canvas,
                                const std::string& datum, const std::string& feature_control,
                                bool datum_anchor_valid, double datum_anchor_x,
                                double datum_anchor_y, bool feature_control_anchor_valid,
                                double feature_control_anchor_x, double feature_control_anchor_y) {
    if (datum.empty() && feature_control.empty())
        return;

    const bool has_both = !datum.empty() && !feature_control.empty();
    const double block_w = std::max(42.0, std::max(datum.size(), feature_control.size()) * 1.8);
    const double block_h = has_both ? 12.0 : 6.0;
    const double x0 = canvas.xmin + 4.0;
    const double y0 = canvas.ymin + 4.0;
    const double line_y = y0 + 6.0;

    f << "  <g class=\"gdt-frame\" stroke=\"#111827\" stroke-width=\"0.25\" fill=\"none\"";
    f << " font-family=\"monospace\" font-size=\"3.0\">\n";
    f << "    <rect x=\"" << x0 << "\" y=\"" << (-y0 - block_h) << "\" width=\"" << block_w
      << "\" height=\"" << block_h << "\"/>\n";
    if (has_both) {
        f << "    <line x1=\"" << x0 << "\" y1=\"" << (-line_y) << "\" x2=\"" << (x0 + block_w)
          << "\" y2=\"" << (-line_y) << "\"/>\n";
    }
    const std::string datum_text = datum.empty() ? feature_control : std::string("DATUM ") + datum;
    const std::string fc_text = feature_control;
    f << "    <text x=\"" << (x0 + 2.0) << "\" y=\"" << (-(y0 + (has_both ? 1.0 : 2.0)))
      << "\" fill=\"#111827\">" << datum_text << "</text>\n";
    if (has_both) {
        f << "    <text x=\"" << (x0 + 2.0) << "\" y=\"" << (-(y0 + 7.0)) << "\" fill=\"#111827\">"
          << fc_text << "</text>\n";
    }
    if (datum_anchor_valid) {
        const double leader_x = x0 + block_w;
        const double leader_y = y0 + (has_both ? 3.0 : 2.0);
        f << "    <line x1=\"" << leader_x << "\" y1=\"" << (-leader_y) << "\" x2=\""
          << datum_anchor_x << "\" y2=\"" << (-datum_anchor_y) << "\"/>\n";
        f << "    <circle class=\"datum-anchor\" cx=\"" << datum_anchor_x << "\" cy=\""
          << (-datum_anchor_y) << "\" r=\"0.9\" fill=\"#111827\" stroke=\"none\"/>\n";
    }
    if (feature_control_anchor_valid) {
        const double leader_x = x0;
        const double leader_y = y0 + (has_both ? 9.0 : 3.0);
        f << "    <line x1=\"" << leader_x << "\" y1=\"" << (-leader_y) << "\" x2=\""
          << feature_control_anchor_x << "\" y2=\"" << (-feature_control_anchor_y) << "\"/>\n";
        f << "    <circle class=\"feature-control-anchor\" cx=\"" << feature_control_anchor_x
          << "\" cy=\"" << (-feature_control_anchor_y)
          << "\" r=\"0.9\" fill=\"#111827\" stroke=\"none\"/>\n";
    }
    f << "  </g>\n";
}

static void write_dxf_title_block(std::ofstream& f, const DrawingCanvasBounds& canvas,
                                  const std::string& sheet_name, const std::string& view_name,
                                  bool sheet_mode, double scale, double tolerance_plus,
                                  double tolerance_minus) {
    const double block_w = 42.0;
    const double block_h = 18.0;
    const double x0 = canvas.xmax - block_w - 4.0;
    const double y0 = canvas.ymin + 4.0;
    const double row1 = y0 + 6.0;
    const double row2 = y0 + 12.0;

    auto line = [&](double x1, double y1, double x2, double y2) {
        f << "  0\nLINE\n";
        f << "  8\nTITLEBLOCK\n";
        f << " 10\n" << x1 << "\n";
        f << " 20\n" << y1 << "\n";
        f << " 30\n0.0\n";
        f << " 11\n" << x2 << "\n";
        f << " 21\n" << y2 << "\n";
        f << " 31\n0.0\n";
    };
    auto text = [&](double x, double y, const std::string& s) {
        f << "  0\nTEXT\n";
        f << "  8\nTITLEBLOCK\n";
        f << " 10\n" << x << "\n";
        f << " 20\n" << y << "\n";
        f << " 30\n0.0\n";
        f << " 40\n3.0\n";
        f << "  1\n" << s << "\n";
    };

    line(x0, y0, x0 + block_w, y0);
    line(x0, y0 + block_h, x0 + block_w, y0 + block_h);
    line(x0, y0, x0, y0 + block_h);
    line(x0 + block_w, y0, x0 + block_w, y0 + block_h);
    line(x0, y0 + 6.0, x0 + block_w, y0 + 6.0);
    line(x0, y0 + 12.0, x0 + block_w, y0 + 12.0);
    text(x0 + 2.0, row2 - 1.0, "rrcad");
    text(x0 + 2.0, row1 - 1.0, sheet_mode ? sheet_name : view_name);
    text(x0 + 2.0, y0 + 1.0, std::string("scale 1:") + std::to_string(1.0 / scale));
    std::string tol = format_tolerance(tolerance_plus, tolerance_minus);
    if (!tol.empty())
        text(x0 + 22.0, y0 + 1.0, std::string("tol ") + tol);
}

static void write_dxf_gdt_frame(std::ofstream& f, const DrawingCanvasBounds& canvas,
                                const std::string& datum, const std::string& feature_control,
                                bool datum_anchor_valid, double datum_anchor_x,
                                double datum_anchor_y, bool feature_control_anchor_valid,
                                double feature_control_anchor_x, double feature_control_anchor_y) {
    if (datum.empty() && feature_control.empty())
        return;

    const bool has_both = !datum.empty() && !feature_control.empty();
    const double block_w = std::max(42.0, std::max(datum.size(), feature_control.size()) * 1.8);
    const double block_h = has_both ? 12.0 : 6.0;
    const double x0 = canvas.xmin + 4.0;
    const double y0 = canvas.ymin + 4.0;
    const double line_y = y0 + 6.0;

    auto line = [&](double x1, double y1, double x2, double y2) {
        f << "  0\nLINE\n";
        f << "  8\nGDT\n";
        f << " 10\n" << x1 << "\n";
        f << " 20\n" << y1 << "\n";
        f << " 30\n0.0\n";
        f << " 11\n" << x2 << "\n";
        f << " 21\n" << y2 << "\n";
        f << " 31\n0.0\n";
    };
    auto text = [&](double x, double y, const std::string& s) {
        f << "  0\nTEXT\n";
        f << "  8\nGDT\n";
        f << " 10\n" << x << "\n";
        f << " 20\n" << y << "\n";
        f << " 30\n0.0\n";
        f << " 40\n3.0\n";
        f << "  1\n" << s << "\n";
    };

    line(x0, y0, x0 + block_w, y0);
    line(x0, y0 + block_h, x0 + block_w, y0 + block_h);
    line(x0, y0, x0, y0 + block_h);
    line(x0 + block_w, y0, x0 + block_w, y0 + block_h);
    if (has_both)
        line(x0, line_y, x0 + block_w, line_y);
    text(x0 + 2.0, y0 + (has_both ? 1.0 : 2.0),
         datum.empty() ? feature_control : std::string("DATUM ") + datum);
    if (has_both)
        text(x0 + 2.0, y0 + 7.0, feature_control);
    if (datum_anchor_valid) {
        const double leader_x = x0 + block_w;
        const double leader_y = y0 + (has_both ? 3.0 : 2.0);
        line(leader_x, leader_y, datum_anchor_x, datum_anchor_y);
        f << "  0\nCIRCLE\n";
        f << "  8\nGDT\n";
        f << " 10\n" << datum_anchor_x << "\n";
        f << " 20\n" << datum_anchor_y << "\n";
        f << " 30\n0.0\n";
        f << " 40\n0.9\n";
    }
    if (feature_control_anchor_valid) {
        const double leader_x = x0;
        const double leader_y = y0 + (has_both ? 9.0 : 3.0);
        line(leader_x, leader_y, feature_control_anchor_x, feature_control_anchor_y);
        f << "  0\nCIRCLE\n";
        f << "  8\nGDT\n";
        f << " 10\n" << feature_control_anchor_x << "\n";
        f << " 20\n" << feature_control_anchor_y << "\n";
        f << " 30\n0.0\n";
        f << " 40\n0.9\n";
    }
}

// Project the shape onto the chosen view plane, return visible and hidden
// polylines as (x, y) point lists.
static HlrProjection hlr_project(const OcctShape& shape, const std::string& view) {
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

    HlrProjection projection;
    collect_hlr_compound(gen.VCompound(), projection.visible);
    collect_hlr_compound(gen.OutLineVCompound(), projection.visible);
    collect_hlr_compound(gen.HCompound(), projection.hidden);
    collect_hlr_compound(gen.OutLineHCompound(), projection.hidden);

    if (projection.visible.empty() && projection.hidden.empty())
        throw std::runtime_error("export_svg/dxf: no visible edges found — "
                                 "shape may be degenerate or face the wrong direction");
    return projection;
}

// Parsed and validated arguments shared by export_svg and export_dxf.
struct DrawingExportSetup {
    std::string path;
    std::string view;
    std::string datum;
    std::string feature_control;
    bool sheet_mode = false;
    // Requested section plane; inactive when the caller passed no section.
    SectionSpec section;
    // Requested detail view; inactive when the caller passed no detail.
    DetailSpec detail;
    // Parts list and balloon callouts, supplied by the assembly layer as
    // delimited records because their row count is not known up front.
    std::vector<std::vector<std::string>> bom_rows;
    std::vector<DrawingBalloon> balloons;
    // Datum anchor projected into 2-D view space (unscaled).
    std::pair<double, double> anchor_2d{0.0, 0.0};
    // Feature-control anchor already scaled into canvas coordinates.
    double feature_anchor_canvas_x = 0.0;
    double feature_anchor_canvas_y = 0.0;
};

// Common front matter for export_svg / export_dxf: copy the rust::Str inputs
// into owned strings, validate the scale, and project both GD&T anchors into
// the drawing plane of the requested view.
static DrawingExportSetup prepare_drawing_export(
    const char* fn_name, rust::Str path, rust::Str view, double scale, rust::Str datum,
    rust::Str feature_control, double datum_anchor_x, double datum_anchor_y, double datum_anchor_z,
    double feature_control_anchor_x, double feature_control_anchor_y,
    double feature_control_anchor_z, rust::Str section_plane, double section_offset,
    bool detail_active, double detail_x, double detail_y, double detail_radius, double detail_scale,
    rust::Str detail_label, rust::Str bom_rows, rust::Str balloons) {
    DrawingExportSetup s;
    s.path = std::string(path.data(), path.size());
    s.view = std::string(view.data(), view.size());
    s.datum = std::string(datum.data(), datum.size());
    s.feature_control = std::string(feature_control.data(), feature_control.size());
    if (!(scale > 0.0) || !std::isfinite(scale))
        throw std::runtime_error(std::string(fn_name) + ": scale must be positive and finite");
    s.sheet_mode = (s.view == "sheet");
    // In sheet mode the anchors are drawn relative to the top view.
    const std::string anchor_view = s.sheet_mode ? "top" : s.view;
    s.anchor_2d = project_point_2d(anchor_view, datum_anchor_x, datum_anchor_y, datum_anchor_z);
    const auto feature_anchor_2d = project_point_2d(
        anchor_view, feature_control_anchor_x, feature_control_anchor_y, feature_control_anchor_z);
    s.feature_anchor_canvas_x = feature_anchor_2d.first * scale;
    s.feature_anchor_canvas_y = feature_anchor_2d.second * scale;
    // An empty section plane name means "no section": draw a plain projection.
    const std::string section_plane_name(section_plane.data(), section_plane.size());
    if (!section_plane_name.empty()) {
        int axis = 0;
        // Validate the plane name up front so a typo fails before any geometry
        // work; the returned normal is recomputed where it is actually used.
        (void)section_plane_normal(section_plane_name, axis);
        // NaN means "no offset given" — the cut defaults to the shape's
        // mid-plane, resolved in compute_section_geometry.
        if (!std::isfinite(section_offset) && !std::isnan(section_offset))
            throw std::runtime_error(std::string(fn_name) + ": section offset must be finite");
        s.section.active = true;
        s.section.plane = section_plane_name;
        s.section.offset = section_offset;
    }
    if (detail_active) {
        // A detail view magnifies one region of one projection.  On a three-view
        // sheet there is no single parent to magnify, and silently picking one
        // would put the bubble on a view the caller did not name — so refuse.
        if (s.sheet_mode)
            throw std::runtime_error(std::string(fn_name) +
                                     ": detail views need a single view (top/front/side), not the "
                                     "three-view sheet — export the detail separately");
        if (!std::isfinite(detail_x) || !std::isfinite(detail_y))
            throw std::runtime_error(std::string(fn_name) + ": detail centre must be finite");
        if (!(detail_radius > 0.0) || !std::isfinite(detail_radius))
            throw std::runtime_error(std::string(fn_name) +
                                     ": detail radius must be positive and finite");
        if (!(detail_scale > 0.0) || !std::isfinite(detail_scale))
            throw std::runtime_error(std::string(fn_name) +
                                     ": detail scale must be positive and finite");
        s.detail.active = true;
        s.detail.x = detail_x;
        s.detail.y = detail_y;
        s.detail.radius = detail_radius;
        s.detail.scale = detail_scale;
        s.detail.label = std::string(detail_label.data(), detail_label.size());
        if (s.detail.label.empty())
            s.detail.label = "A";
    }
    s.bom_rows = parse_bom_rows(std::string(bom_rows.data(), bom_rows.size()));
    s.balloons = parse_balloons(std::string(balloons.data(), balloons.size()), scale);
    return s;
}

// Placement offsets and overall bounds for the three-view "sheet" layout.
struct SheetLayout {
    double top_dx = 0.0;
    double top_dy = 0.0;
    double front_dx = 0.0;
    double front_dy = 0.0;
    double side_dx = 0.0;
    double side_dy = 0.0;
    DrawingCanvasBounds canvas;
};

// `top_dy` is supplied by the caller: the SVG and DXF exporters historically
// compute the top-view vertical offset differently, so it is not unified here.
static SheetLayout compute_sheet_layout(const DrawingViewData& top_view,
                                        const DrawingViewData& front_view,
                                        const DrawingViewData& side_view, double sheet_gap,
                                        double top_dy) {
    SheetLayout l;
    // Centre the top view horizontally over the front view.
    l.top_dx = (front_view.geom_xmin + front_view.geom_xmax) * 0.5 -
               (top_view.geom_xmin + top_view.geom_xmax) * 0.5;
    l.top_dy = top_dy;
    // Side view sits to the right of the front view, vertically centred on it.
    l.side_dx = front_view.geom_xmax - side_view.geom_xmin + sheet_gap;
    l.side_dy = (front_view.geom_ymin + front_view.geom_ymax) * 0.5 -
                (side_view.geom_ymin + side_view.geom_ymax) * 0.5;
    double xmin = 1e30, xmax = -1e30, ymin = 1e30, ymax = -1e30;
    include_placed_bounds(xmin, xmax, ymin, ymax, top_view, l.top_dx, l.top_dy);
    include_placed_bounds(xmin, xmax, ymin, ymax, front_view, l.front_dx, l.front_dy);
    include_placed_bounds(xmin, xmax, ymin, ymax, side_view, l.side_dx, l.side_dy);
    l.canvas = DrawingCanvasBounds{xmin, xmax, ymin, ymax};
    return l;
}

// Offset that places a detail view beside its parent: to the right, with the
// two vertical centres aligned, which is how a drawing normally reads.
struct DetailPlacement {
    double dx = 0.0;
    double dy = 0.0;
};

static DetailPlacement place_detail_view(const DrawingViewData& parent,
                                         const DrawingViewData& detail) {
    DetailPlacement p;
    p.dx = parent.xmax + DETAIL_VIEW_GAP - detail.xmin;
    p.dy = (parent.ymin + parent.ymax) * 0.5 - (detail.ymin + detail.ymax) * 0.5;
    return p;
}

// Grow the page so the balloons and the parts list are not clipped off it.
//
// Only the viewBox uses the result: the title block, the GD&T frame, and the
// table itself stay anchored to the *drawing's* bounds, so adding a parts list
// does not push the title block down with it.
static DrawingCanvasBounds
extend_page_for_annotations(DrawingCanvasBounds page, const std::vector<DrawingBalloon>& balloons,
                            double offset_x, double offset_y,
                            const std::vector<std::vector<std::string>>& bom_rows) {
    // The table hangs off the *drawing's* bounds, so measure its extent before
    // the balloons move them — otherwise a balloon below the part would push
    // the page down by the table's height a second time.
    const double table_bottom =
        bom_rows.empty() ? page.ymin : page.ymin - BOM_TABLE_GAP - bom_table_height(bom_rows);
    const double table_right =
        bom_rows.empty() ? page.xmax : std::max(page.xmax, page.xmin + bom_table_width(bom_rows));

    for (const auto& b : balloons) {
        page.xmin = std::min(page.xmin, b.bx + offset_x - BALLOON_RADIUS);
        page.xmax = std::max(page.xmax, b.bx + offset_x + BALLOON_RADIUS);
        page.ymin = std::min(page.ymin, b.by + offset_y - BALLOON_RADIUS);
        page.ymax = std::max(page.ymax, b.by + offset_y + BALLOON_RADIUS);
    }
    page.ymin = std::min(page.ymin, table_bottom);
    page.xmax = std::max(page.xmax, table_right);
    return page;
}

// ---------------------------------------------------------------------------
// SVG export
// ---------------------------------------------------------------------------
void export_svg(const OcctShape& shape, rust::Str path, rust::Str view, double scale, bool hidden,
                bool center_marks, bool dimensions, bool title_block, bool callouts,
                rust::Str datum, bool datum_anchor_valid, double datum_anchor_x,
                double datum_anchor_y, double datum_anchor_z, rust::Str feature_control,
                bool feature_control_anchor_valid, double feature_control_anchor_x,
                double feature_control_anchor_y, double feature_control_anchor_z,
                double tolerance_plus, double tolerance_minus, rust::Str section_plane,
                double section_offset, bool detail_active, double detail_x, double detail_y,
                double detail_radius, double detail_scale, rust::Str detail_label, bool ordinate,
                rust::Str bom_rows, rust::Str balloons) {
    try {
        const DrawingExportSetup setup = prepare_drawing_export(
            "export_svg", path, view, scale, datum, feature_control, datum_anchor_x, datum_anchor_y,
            datum_anchor_z, feature_control_anchor_x, feature_control_anchor_y,
            feature_control_anchor_z, section_plane, section_offset, detail_active, detail_x,
            detail_y, detail_radius, detail_scale, detail_label, bom_rows, balloons);
        bool has_anchor = datum_anchor_valid;
        double anchor_canvas_x = 0.0;
        double anchor_canvas_y = 0.0;

        const double margin = 5.0;
        const double sheet_gap = 16.0;

        if (setup.sheet_mode) {
            auto top_view = build_drawing_view(shape, "top", scale, hidden, center_marks,
                                               dimensions, callouts, ordinate, setup.section);
            auto front_view = build_drawing_view(shape, "front", scale, hidden, center_marks,
                                                 dimensions, callouts, ordinate, setup.section);
            auto side_view = build_drawing_view(shape, "side", scale, hidden, center_marks,
                                                dimensions, callouts, ordinate, setup.section);

            const SheetLayout layout =
                compute_sheet_layout(top_view, front_view, side_view, sheet_gap,
                                     front_view.geom_ymax - top_view.geom_ymin + sheet_gap);
            const DrawingCanvasBounds& canvas = layout.canvas;

            // Balloons ring the top view: on a three-view sheet that is the one
            // plan every component appears somewhere on.
            auto balloon_list = setup.balloons;
            place_balloons(balloon_list, top_view);
            const DrawingCanvasBounds page = extend_page_for_annotations(
                canvas, balloon_list, layout.top_dx, layout.top_dy, setup.bom_rows);

            const double w = (page.xmax - page.xmin) + 2.0 * margin;
            const double h = (page.ymax - page.ymin) + 2.0 * margin;
            const double vb_x = page.xmin - margin;
            const double vb_y = -(page.ymax + margin);

            std::ofstream f(setup.path);
            if (!f.is_open())
                throw std::runtime_error("export_svg: cannot open file: " + setup.path);

            f << std::fixed << std::setprecision(4);
            f << "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
            f << "<svg xmlns=\"http://www.w3.org/2000/svg\"";
            f << " width=\"" << w << "\" height=\"" << h << "\"";
            f << " viewBox=\"" << vb_x << " " << vb_y << " " << w << " " << h << "\">\n";
            f << "  <!-- Generated by rrcad — sheet view, scale: " << scale << " -->\n";
            write_svg_view(f, top_view, layout.top_dx, layout.top_dy, hidden, center_marks,
                           dimensions, callouts, true, "X", "Y", tolerance_plus, tolerance_minus);
            write_svg_view(f, front_view, layout.front_dx, layout.front_dy, hidden, center_marks,
                           dimensions, callouts, true, "X", "Z", tolerance_plus, tolerance_minus);
            write_svg_view(f, side_view, layout.side_dx, layout.side_dy, hidden, center_marks,
                           dimensions, callouts, true, "Y", "Z", tolerance_plus, tolerance_minus);
            if (has_anchor) {
                anchor_canvas_x = setup.anchor_2d.first * scale + layout.top_dx;
                anchor_canvas_y = setup.anchor_2d.second * scale + layout.top_dy;
            }
            write_svg_gdt_frame(f, canvas, setup.datum, setup.feature_control, has_anchor,
                                anchor_canvas_x, anchor_canvas_y, feature_control_anchor_valid,
                                setup.feature_anchor_canvas_x + layout.top_dx,
                                setup.feature_anchor_canvas_y + layout.top_dy);
            write_svg_balloons(f, balloon_list, layout.top_dx, layout.top_dy);
            write_svg_bom_table(f, canvas, setup.bom_rows);
            if (title_block)
                write_svg_title_block(f, canvas, "sheet", setup.view, true, scale, tolerance_plus,
                                      tolerance_minus);
            f << "</svg>\n";
            if (!f.good())
                throw std::runtime_error("export_svg: write error on file: " + setup.path);
            return;
        }

        auto single_view = build_drawing_view(shape, setup.view, scale, hidden, center_marks,
                                              dimensions, callouts, ordinate, setup.section);
        // Build the close-up from the parent's projected geometry, then mark the
        // region on the parent — in that order, since marking grows its bounds.
        DrawingViewData detail_view;
        DetailPlacement detail_at;
        if (setup.detail.active) {
            detail_view = build_detail_view(single_view, setup.detail, scale);
            attach_detail_marker(single_view, setup.detail, scale);
            detail_at = place_detail_view(single_view, detail_view);
        }

        double cx_min = single_view.xmin, cx_max = single_view.xmax;
        double cy_min = single_view.ymin, cy_max = single_view.ymax;
        if (setup.detail.active)
            include_placed_bounds(cx_min, cx_max, cy_min, cy_max, detail_view, detail_at.dx,
                                  detail_at.dy);

        const DrawingCanvasBounds canvas{cx_min, cx_max, cy_min, cy_max};
        auto balloon_list = setup.balloons;
        place_balloons(balloon_list, single_view);
        const DrawingCanvasBounds page =
            extend_page_for_annotations(canvas, balloon_list, 0.0, 0.0, setup.bom_rows);

        const double w = (page.xmax - page.xmin) + 2.0 * margin;
        const double h = (page.ymax - page.ymin) + 2.0 * margin;
        const double vb_x = page.xmin - margin;
        const double vb_y = -(page.ymax + margin);

        std::ofstream f(setup.path);
        if (!f.is_open())
            throw std::runtime_error("export_svg: cannot open file: " + setup.path);

        f << std::fixed << std::setprecision(4);
        f << "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
        f << "<svg xmlns=\"http://www.w3.org/2000/svg\"";
        f << " width=\"" << w << "\" height=\"" << h << "\"";
        f << " viewBox=\"" << vb_x << " " << vb_y << " " << w << " " << h << "\">\n";
        f << "  <!-- Generated by rrcad — view: " << setup.view << ", scale: " << scale << " -->\n";
        if (has_anchor) {
            anchor_canvas_x = setup.anchor_2d.first * scale;
            anchor_canvas_y = setup.anchor_2d.second * scale;
        }
        write_svg_view(f, single_view, 0.0, 0.0, hidden, center_marks, dimensions, callouts, false,
                       nullptr, nullptr, tolerance_plus, tolerance_minus);
        if (setup.detail.active) {
            // The close-up carries no dimensions of its own: they would repeat
            // the parent's overall extents at the wrong scale.
            write_svg_view(f, detail_view, detail_at.dx, detail_at.dy, hidden, false, false, false,
                           false);
        }
        write_svg_gdt_frame(f, canvas, setup.datum, setup.feature_control, has_anchor,
                            anchor_canvas_x, anchor_canvas_y, feature_control_anchor_valid,
                            setup.feature_anchor_canvas_x, setup.feature_anchor_canvas_y);
        write_svg_balloons(f, balloon_list, 0.0, 0.0);
        write_svg_bom_table(f, canvas, setup.bom_rows);
        if (title_block)
            write_svg_title_block(f, canvas, "sheet", setup.view, false, scale, tolerance_plus,
                                  tolerance_minus);
        f << "</svg>\n";
        if (!f.good())
            throw std::runtime_error("export_svg: write error on file: " + setup.path);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    } catch (...) {
        throw std::runtime_error("unknown C++ exception in export_svg");
    }
}

// ---------------------------------------------------------------------------
// DXF export (ASCII DXF R12 — universally supported)
//
// Each polyline segment is written as a LINE entity.  DXF uses Y-up
// coordinates (standard math / CAD convention) so no Y-flip is applied.
// ---------------------------------------------------------------------------
void export_dxf(const OcctShape& shape, rust::Str path, rust::Str view, double scale, bool hidden,
                bool center_marks, bool dimensions, bool title_block, bool callouts,
                rust::Str datum, bool datum_anchor_valid, double datum_anchor_x,
                double datum_anchor_y, double datum_anchor_z, rust::Str feature_control,
                bool feature_control_anchor_valid, double feature_control_anchor_x,
                double feature_control_anchor_y, double feature_control_anchor_z,
                double tolerance_plus, double tolerance_minus, rust::Str section_plane,
                double section_offset, bool detail_active, double detail_x, double detail_y,
                double detail_radius, double detail_scale, rust::Str detail_label, bool ordinate,
                rust::Str bom_rows, rust::Str balloons) {
    try {
        const DrawingExportSetup setup = prepare_drawing_export(
            "export_dxf", path, view, scale, datum, feature_control, datum_anchor_x, datum_anchor_y,
            datum_anchor_z, feature_control_anchor_x, feature_control_anchor_y,
            feature_control_anchor_z, section_plane, section_offset, detail_active, detail_x,
            detail_y, detail_radius, detail_scale, detail_label, bom_rows, balloons);
        bool has_anchor = datum_anchor_valid;
        double anchor_canvas_x = 0.0;
        double anchor_canvas_y = 0.0;

        if (setup.sheet_mode) {
            const double sheet_gap = 16.0;

            auto top_view = build_drawing_view(shape, "top", scale, hidden, center_marks,
                                               dimensions, callouts, ordinate, setup.section);
            auto front_view = build_drawing_view(shape, "front", scale, hidden, center_marks,
                                                 dimensions, callouts, ordinate, setup.section);
            auto side_view = build_drawing_view(shape, "side", scale, hidden, center_marks,
                                                dimensions, callouts, ordinate, setup.section);

            const SheetLayout layout =
                compute_sheet_layout(top_view, front_view, side_view, sheet_gap,
                                     front_view.geom_ymax - front_view.geom_ymin + sheet_gap);
            const DrawingCanvasBounds& canvas = layout.canvas;

            // As in the SVG path: balloons ring the top view.  DXF has no page
            // box to extend, so nothing else follows from their placement.
            auto balloon_list = setup.balloons;
            place_balloons(balloon_list, top_view);

            std::ofstream f(setup.path);
            if (!f.is_open())
                throw std::runtime_error("export_dxf: cannot open file: " + setup.path);

            f << std::fixed << std::setprecision(6);
            f << "  0\nSECTION\n  2\nHEADER\n";
            f << "  9\n$ACADVER\n  1\nAC1009\n";
            f << "  0\nENDSEC\n";
            f << "  0\nSECTION\n  2\nENTITIES\n";

            write_dxf_view(f, top_view, layout.top_dx, layout.top_dy, hidden, center_marks,
                           dimensions, callouts, true, "X", "Y", tolerance_plus, tolerance_minus);
            write_dxf_view(f, front_view, layout.front_dx, layout.front_dy, hidden, center_marks,
                           dimensions, callouts, true, "X", "Z", tolerance_plus, tolerance_minus);
            write_dxf_view(f, side_view, layout.side_dx, layout.side_dy, hidden, center_marks,
                           dimensions, callouts, true, "Y", "Z", tolerance_plus, tolerance_minus);
            if (has_anchor) {
                anchor_canvas_x = setup.anchor_2d.first * scale + layout.top_dx;
                anchor_canvas_y = setup.anchor_2d.second * scale + layout.top_dy;
            }
            write_dxf_gdt_frame(f, canvas, setup.datum, setup.feature_control, has_anchor,
                                anchor_canvas_x, anchor_canvas_y, feature_control_anchor_valid,
                                setup.feature_anchor_canvas_x + layout.top_dx,
                                setup.feature_anchor_canvas_y + layout.top_dy);
            write_dxf_balloons(f, balloon_list, layout.top_dx, layout.top_dy);
            write_dxf_bom_table(f, canvas, setup.bom_rows);
            if (title_block)
                write_dxf_title_block(f, canvas, "sheet", setup.view, true, scale, tolerance_plus,
                                      tolerance_minus);

            f << "  0\nENDSEC\n  0\nEOF\n";
            if (!f.good())
                throw std::runtime_error("export_dxf: write error on file: " + setup.path);
            return;
        }

        auto single_view = build_drawing_view(shape, setup.view, scale, hidden, center_marks,
                                              dimensions, callouts, ordinate, setup.section);
        // Same order as the SVG path: magnify first, then mark the parent.
        DrawingViewData detail_view;
        DetailPlacement detail_at;
        if (setup.detail.active) {
            detail_view = build_detail_view(single_view, setup.detail, scale);
            attach_detail_marker(single_view, setup.detail, scale);
            detail_at = place_detail_view(single_view, detail_view);
        }

        double cx_min = single_view.geom_xmin, cx_max = single_view.geom_xmax;
        double cy_min = single_view.geom_ymin, cy_max = single_view.geom_ymax;
        if (setup.detail.active)
            include_placed_bounds(cx_min, cx_max, cy_min, cy_max, detail_view, detail_at.dx,
                                  detail_at.dy);
        const DrawingCanvasBounds canvas{cx_min, cx_max, cy_min, cy_max};
        auto balloon_list = setup.balloons;
        place_balloons(balloon_list, single_view);

        std::ofstream f(setup.path);
        if (!f.is_open())
            throw std::runtime_error("export_dxf: cannot open file: " + setup.path);

        f << std::fixed << std::setprecision(6);

        // Minimal DXF R12 header.
        f << "  0\nSECTION\n  2\nHEADER\n";
        f << "  9\n$ACADVER\n  1\nAC1009\n"; // AutoCAD R12
        f << "  0\nENDSEC\n";

        // ENTITIES section: one LINE entity per polyline segment.
        f << "  0\nSECTION\n  2\nENTITIES\n";

        write_dxf_view(f, single_view, 0.0, 0.0, hidden, center_marks, dimensions, callouts, false,
                       nullptr, nullptr, tolerance_plus, tolerance_minus);
        if (setup.detail.active) {
            // No dimensions on the close-up — see the SVG path for why.
            write_dxf_view(f, detail_view, detail_at.dx, detail_at.dy, hidden, false, false, false,
                           false);
        }
        if (has_anchor) {
            anchor_canvas_x = setup.anchor_2d.first * scale;
            anchor_canvas_y = setup.anchor_2d.second * scale;
        }
        write_dxf_gdt_frame(f, canvas, setup.datum, setup.feature_control, has_anchor,
                            anchor_canvas_x, anchor_canvas_y, feature_control_anchor_valid,
                            setup.feature_anchor_canvas_x, setup.feature_anchor_canvas_y);
        write_dxf_balloons(f, balloon_list, 0.0, 0.0);
        write_dxf_bom_table(f, canvas, setup.bom_rows);
        if (title_block)
            write_dxf_title_block(f, canvas, "sheet", setup.view, false, scale, tolerance_plus,
                                  tolerance_minus);

        f << "  0\nENDSEC\n  0\nEOF\n";
        if (!f.good())
            throw std::runtime_error("export_dxf: write error on file: " + setup.path);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    } catch (...) {
        throw std::runtime_error("unknown C++ exception in export_dxf");
    }
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

std::unique_ptr<FragmentBuilder> fragment_new() {
    // OCCT Standard_Failure does not derive std::exception; without this guard an
    // escaping OCCT exception would abort the process at the cxx bridge boundary.
    try {
        return std::make_unique<FragmentBuilder>();
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("fragment_new failed: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    } catch (...) {
        throw std::runtime_error("unknown C++ exception in fragment_new");
    }
}

void fragment_add(FragmentBuilder& builder, const OcctShape& shape) {
    try {
        builder.impl->shapes.Append(shape.get());
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

std::unique_ptr<OcctShape> fragment_build(FragmentBuilder& builder) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
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
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// --- path_pattern -----------------------------------------------------------

std::unique_ptr<OcctShape> shape_path_pattern(const OcctShape& shape, const OcctShape& path,
                                              int32_t n) {
    try {
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
        // GCPnts_UniformAbscissa walks the curve and returns parameter values at
        // positions equally spaced by arc-length — so copies are physically
        // equidistant along the path regardless of parameterisation.
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

            TopoDS_Shape copy =
                BRepBuilderAPI_Transform(shape.get(), combined, /*copy=*/true).Shape();
            bb.Add(compound, copy);
        }

        return wrap(compound);
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// --- sweep_guide ------------------------------------------------------------

std::unique_ptr<OcctShape> shape_sweep_guide(const OcctShape& profile, const OcctShape& path,
                                             const OcctShape& guide) {
    try {
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
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

// ---------------------------------------------------------------------------
// Flat cut-file export — export_face_outline
//
// Distinct from export_svg/export_dxf, which draw an *HLR projection* of a 3-D
// shape: a drawing, complete with whatever else is visible from that
// direction.  A cut file is a different deliverable.  A laser or CNC shop
// wants the closed loops of one flat face, at 1:1, with nothing else in the
// file — outer profile plus every hole, and nothing standing in for a curve
// that a controller could cut exactly.
//
// So circular edges become true DXF CIRCLE / ARC entities rather than many
// short chords; only genuinely free-form curves (splines) are approximated,
// to the caller's deflection tolerance.
// ---------------------------------------------------------------------------

// One outline entity in the face's own 2-D plane coordinates.
struct OutlineEntity {
    enum class Kind { Polyline, Circle, Arc } kind = Kind::Polyline;
    // Polyline: the vertices, in order.  Circle/Arc: unused.
    std::vector<gp_Pnt2d> points;
    // Circle/Arc.  Angles are degrees, CCW, as DXF expects.
    gp_Pnt2d center;
    double radius = 0.0;
    double start_angle = 0.0;
    double end_angle = 0.0;
};

// One closed loop: the outer boundary or a hole.
struct OutlineLoop {
    std::vector<OutlineEntity> entities;
    bool is_hole = false;
};

// Locate the single planar face to export.
//
// Accepts a Face directly, or any shape containing exactly one face (a lone
// sketch profile, typically).  More than one face is ambiguous — the user has
// to say which — and the error says so rather than guessing.
static TopoDS_Face outline_target_face(const TopoDS_Shape& shape) {
    if (shape.IsNull())
        throw std::runtime_error("export_outline: shape is empty");

    if (shape.ShapeType() == TopAbs_FACE)
        return TopoDS::Face(shape);

    std::vector<TopoDS_Face> faces;
    for (TopExp_Explorer exp(shape, TopAbs_FACE); exp.More(); exp.Next()) {
        faces.push_back(TopoDS::Face(exp.Current()));
        if (faces.size() > 1)
            break;
    }
    if (faces.empty())
        throw std::runtime_error("export_outline: shape has no faces to export");
    if (faces.size() > 1)
        throw std::runtime_error(
            "export_outline: shape has more than one face — select the one to cut, "
            "e.g. part.faces(:top).first.export_outline(...)");
    return faces.front();
}

// The plane a face lies in, or an error naming what it actually is.
static gp_Pln outline_face_plane(const TopoDS_Face& face) {
    BRepAdaptor_Surface adaptor(face, Standard_True);
    if (adaptor.GetType() != GeomAbs_Plane)
        throw std::runtime_error(
            "export_outline: face is not planar — a cut file needs a flat face");
    return adaptor.Plane();
}

// Project a 3-D point onto the face's plane coordinate system.
static gp_Pnt2d outline_to_2d(const gp_Pln& plane, const gp_Pnt& p) {
    const gp_Pnt origin = plane.Location();
    const gp_Vec delta(origin, p);
    return gp_Pnt2d(delta.Dot(gp_Vec(plane.Position().XDirection())),
                    delta.Dot(gp_Vec(plane.Position().YDirection())));
}

// Normalise an angle in degrees to [0, 360).
static double outline_norm_deg(double deg) {
    double d = std::fmod(deg, 360.0);
    if (d < 0.0)
        d += 360.0;
    return d;
}

// True when `mid` lies on the CCW sweep from `start` to `end` (all degrees).
static bool outline_ccw_contains(double start, double end, double mid) {
    const double sweep = outline_norm_deg(end - start);
    const double offset = outline_norm_deg(mid - start);
    return offset <= sweep;
}

// Convert one edge to an outline entity in plane coordinates.
//
// Lines and circles are emitted exactly; everything else is approximated to
// `deflection`.  Arc direction is decided *empirically* — by sampling the
// edge's own midpoint and checking which way round it lies — rather than by
// reasoning about the circle axis against the plane normal and the edge
// orientation.  Those three signs compose in ways that are easy to get
// backwards and produce an arc bulging the wrong way, which a reader would
// not notice until the part came back from the cutter.
static OutlineEntity outline_edge_entity(const TopoDS_Edge& edge, const gp_Pln& plane,
                                         double deflection) {
    BRepAdaptor_Curve curve(edge);
    const double first = curve.FirstParameter();
    const double last = curve.LastParameter();

    OutlineEntity entity;

    if (curve.GetType() == GeomAbs_Line) {
        entity.kind = OutlineEntity::Kind::Polyline;
        entity.points.push_back(outline_to_2d(plane, curve.Value(first)));
        entity.points.push_back(outline_to_2d(plane, curve.Value(last)));
        return entity;
    }

    if (curve.GetType() == GeomAbs_Circle) {
        const gp_Circ circle = curve.Circle();
        const gp_Pnt2d center = outline_to_2d(plane, circle.Location());
        const double radius = circle.Radius();

        const gp_Pnt2d p_start = outline_to_2d(plane, curve.Value(first));
        const gp_Pnt2d p_end = outline_to_2d(plane, curve.Value(last));
        const gp_Pnt2d p_mid = outline_to_2d(plane, curve.Value((first + last) / 2.0));

        // A closed circular edge is a full circle: one CIRCLE entity, which is
        // what a bolt hole should be in a cut file.
        const bool closed = p_start.Distance(p_end) <= Precision::Confusion() * 10.0 ||
                            std::abs((last - first) - 2.0 * M_PI) < 1.0e-7;
        if (closed) {
            entity.kind = OutlineEntity::Kind::Circle;
            entity.center = center;
            entity.radius = radius;
            return entity;
        }

        auto angle_of = [&center](const gp_Pnt2d& p) {
            return outline_norm_deg(std::atan2(p.Y() - center.Y(), p.X() - center.X()) * 180.0 /
                                    M_PI);
        };
        double a_start = angle_of(p_start);
        double a_end = angle_of(p_end);
        const double a_mid = angle_of(p_mid);

        // DXF arcs always sweep CCW from start to end.  If the edge's own
        // midpoint is not on that sweep, we have the two ends the wrong way
        // round; swapping them describes the identical arc.
        if (!outline_ccw_contains(a_start, a_end, a_mid))
            std::swap(a_start, a_end);

        entity.kind = OutlineEntity::Kind::Arc;
        entity.center = center;
        entity.radius = radius;
        entity.start_angle = a_start;
        entity.end_angle = a_end;
        return entity;
    }

    // Free-form geometry: approximate to the requested deflection.
    entity.kind = OutlineEntity::Kind::Polyline;
    GCPnts_QuasiUniformDeflection sampler(curve, deflection, first, last);
    if (sampler.IsDone() && sampler.NbPoints() >= 2) {
        for (int i = 1; i <= sampler.NbPoints(); ++i)
            entity.points.push_back(outline_to_2d(plane, sampler.Value(i)));
    } else {
        // Fall back to a coarse uniform sampling rather than dropping the edge.
        const int steps = 32;
        for (int i = 0; i <= steps; ++i) {
            const double t = first + (last - first) * (double(i) / double(steps));
            entity.points.push_back(outline_to_2d(plane, curve.Value(t)));
        }
    }
    return entity;
}

// Walk one wire in order and convert every edge.
static OutlineLoop outline_wire_loop(const TopoDS_Wire& wire, const TopoDS_Face& face,
                                     const gp_Pln& plane, double deflection, bool is_hole) {
    OutlineLoop loop;
    loop.is_hole = is_hole;
    for (BRepTools_WireExplorer exp(wire, face); exp.More(); exp.Next())
        loop.entities.push_back(outline_edge_entity(exp.Current(), plane, deflection));
    if (loop.entities.empty())
        throw std::runtime_error("export_outline: a boundary loop produced no geometry");
    return loop;
}

// Extract the outer boundary and every hole from a planar face.
static std::vector<OutlineLoop> outline_loops(const TopoDS_Face& face, const gp_Pln& plane,
                                              double deflection) {
    std::vector<OutlineLoop> loops;
    const TopoDS_Wire outer = BRepTools::OuterWire(face);
    loops.push_back(outline_wire_loop(outer, face, plane, deflection, false));

    for (TopExp_Explorer exp(face, TopAbs_WIRE); exp.More(); exp.Next()) {
        const TopoDS_Wire wire = TopoDS::Wire(exp.Current());
        if (wire.IsSame(outer))
            continue;
        loops.push_back(outline_wire_loop(wire, face, plane, deflection, true));
    }
    return loops;
}

// Bounding box of every loop, used to shift the outline to the origin.
static void outline_bounds(const std::vector<OutlineLoop>& loops, double& xmin, double& ymin,
                           double& xmax, double& ymax) {
    xmin = ymin = std::numeric_limits<double>::max();
    xmax = ymax = std::numeric_limits<double>::lowest();
    auto note = [&](double x, double y) {
        xmin = std::min(xmin, x);
        ymin = std::min(ymin, y);
        xmax = std::max(xmax, x);
        ymax = std::max(ymax, y);
    };
    for (const OutlineLoop& loop : loops) {
        for (const OutlineEntity& e : loop.entities) {
            switch (e.kind) {
            case OutlineEntity::Kind::Polyline:
                for (const gp_Pnt2d& p : e.points)
                    note(p.X(), p.Y());
                break;
            case OutlineEntity::Kind::Circle:
            case OutlineEntity::Kind::Arc:
                // Bound the whole circle: an arc's extreme point may lie
                // between its endpoints, so using the endpoints alone would
                // clip a bulge out of the bounding box.
                note(e.center.X() - e.radius, e.center.Y() - e.radius);
                note(e.center.X() + e.radius, e.center.Y() + e.radius);
                break;
            }
        }
    }
    if (xmin > xmax || ymin > ymax)
        throw std::runtime_error("export_outline: outline has an empty bounding box");
}

static void write_outline_dxf(const std::string& path, const std::vector<OutlineLoop>& loops,
                              double dx, double dy) {
    std::ofstream f(path);
    if (!f.is_open())
        throw std::runtime_error("export_outline: cannot open file: " + path);
    f << std::fixed << std::setprecision(6);

    // DXF R12, the most portable dialect.  $INSUNITS 4 declares millimetres so
    // a controller does not have to guess the scale.
    f << "  0\nSECTION\n  2\nHEADER\n";
    f << "  9\n$ACADVER\n  1\nAC1009\n";
    f << "  9\n$INSUNITS\n 70\n     4\n";
    f << "  0\nENDSEC\n";
    f << "  0\nSECTION\n  2\nENTITIES\n";

    for (const OutlineLoop& loop : loops) {
        // Separate layers so a shop can treat holes differently from the
        // profile (inside cuts before the outside cut, typically).
        const char* layer = loop.is_hole ? "HOLES" : "PROFILE";
        for (const OutlineEntity& e : loop.entities) {
            switch (e.kind) {
            case OutlineEntity::Kind::Polyline:
                for (size_t i = 1; i < e.points.size(); ++i) {
                    f << "  0\nLINE\n  8\n" << layer << "\n";
                    f << " 10\n" << (e.points[i - 1].X() + dx) << "\n";
                    f << " 20\n" << (e.points[i - 1].Y() + dy) << "\n";
                    f << " 30\n0.0\n";
                    f << " 11\n" << (e.points[i].X() + dx) << "\n";
                    f << " 21\n" << (e.points[i].Y() + dy) << "\n";
                    f << " 31\n0.0\n";
                }
                break;
            case OutlineEntity::Kind::Circle:
                f << "  0\nCIRCLE\n  8\n" << layer << "\n";
                f << " 10\n" << (e.center.X() + dx) << "\n";
                f << " 20\n" << (e.center.Y() + dy) << "\n";
                f << " 30\n0.0\n";
                f << " 40\n" << e.radius << "\n";
                break;
            case OutlineEntity::Kind::Arc:
                f << "  0\nARC\n  8\n" << layer << "\n";
                f << " 10\n" << (e.center.X() + dx) << "\n";
                f << " 20\n" << (e.center.Y() + dy) << "\n";
                f << " 30\n0.0\n";
                f << " 40\n" << e.radius << "\n";
                f << " 50\n" << e.start_angle << "\n";
                f << " 51\n" << e.end_angle << "\n";
                break;
            }
        }
    }

    f << "  0\nENDSEC\n  0\nEOF\n";
    if (!f.good())
        throw std::runtime_error("export_outline: write error on file: " + path);
}

static void write_outline_svg(const std::string& path, const std::vector<OutlineLoop>& loops,
                              double dx, double dy, double width, double height) {
    std::ofstream f(path);
    if (!f.is_open())
        throw std::runtime_error("export_outline: cannot open file: " + path);
    f << std::fixed << std::setprecision(6);

    // SVG is Y-down; flip so the drawing matches the DXF and the model.
    auto sx = [&](double x) { return x + dx; };
    auto sy = [&](double y) { return height - (y + dy); };

    f << "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
    f << "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"" << width << "mm\" height=\"" << height
      << "mm\" viewBox=\"0 0 " << width << " " << height << "\">\n";

    for (const OutlineLoop& loop : loops) {
        const char* cls = loop.is_hole ? "holes" : "profile";
        f << "<g class=\"" << cls << "\" fill=\"none\" stroke=\"black\" stroke-width=\"0.1\">\n";
        for (const OutlineEntity& e : loop.entities) {
            switch (e.kind) {
            case OutlineEntity::Kind::Polyline:
                for (size_t i = 1; i < e.points.size(); ++i) {
                    f << "<line x1=\"" << sx(e.points[i - 1].X()) << "\" y1=\""
                      << sy(e.points[i - 1].Y()) << "\" x2=\"" << sx(e.points[i].X()) << "\" y2=\""
                      << sy(e.points[i].Y()) << "\"/>\n";
                }
                break;
            case OutlineEntity::Kind::Circle:
                f << "<circle cx=\"" << sx(e.center.X()) << "\" cy=\"" << sy(e.center.Y())
                  << "\" r=\"" << e.radius << "\"/>\n";
                break;
            case OutlineEntity::Kind::Arc: {
                const double rad = M_PI / 180.0;
                const double x1 = e.center.X() + e.radius * std::cos(e.start_angle * rad);
                const double y1 = e.center.Y() + e.radius * std::sin(e.start_angle * rad);
                const double x2 = e.center.X() + e.radius * std::cos(e.end_angle * rad);
                const double y2 = e.center.Y() + e.radius * std::sin(e.end_angle * rad);
                const double sweep = outline_norm_deg(e.end_angle - e.start_angle);
                const int large = sweep > 180.0 ? 1 : 0;
                // The Y-flip reverses handedness, so a CCW arc in model space
                // is drawn clockwise (sweep-flag 0) in SVG's Y-down frame.
                f << "<path d=\"M " << sx(x1) << " " << sy(y1) << " A " << e.radius << " "
                  << e.radius << " 0 " << large << " 0 " << sx(x2) << " " << sy(y2) << "\"/>\n";
                break;
            }
            }
        }
        f << "</g>\n";
    }
    f << "</svg>\n";
    if (!f.good())
        throw std::runtime_error("export_outline: write error on file: " + path);
}

// Export the closed loops of one planar face at 1:1, as a cut file.
//
// The outline is shifted so its bounding box starts at the origin, which is
// what makes the file directly nestable on a sheet; `format` is "dxf" or
// "svg".  `deflection` bounds the chord error where a free-form curve has to
// be approximated.
void export_face_outline(const OcctShape& shape, rust::Str path, rust::Str format,
                         double deflection) {
    try {
        const std::string path_str(path);
        const std::string fmt(format);
        if (!(deflection > 0.0) || !std::isfinite(deflection))
            throw std::runtime_error("export_outline: deflection must be a positive number");

        const TopoDS_Face face = outline_target_face(shape.get());
        const gp_Pln plane = outline_face_plane(face);
        const std::vector<OutlineLoop> loops = outline_loops(face, plane, deflection);

        double xmin = 0.0, ymin = 0.0, xmax = 0.0, ymax = 0.0;
        outline_bounds(loops, xmin, ymin, xmax, ymax);
        const double dx = -xmin;
        const double dy = -ymin;

        if (fmt == "dxf") {
            write_outline_dxf(path_str, loops, dx, dy);
        } else if (fmt == "svg") {
            write_outline_svg(path_str, loops, dx, dy, xmax - xmin, ymax - ymin);
        } else {
            throw std::runtime_error("export_outline: format must be \"dxf\" or \"svg\"");
        }
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(std::string("OCCT error: ") + e.GetMessageString());
    } catch (const std::exception&) {
        throw;
    }
}

} // namespace rrcad
