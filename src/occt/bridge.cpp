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
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>

// --- OCCT: boolean ops ---
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>

// --- OCCT: fillets and chamfers ---
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>

// --- OCCT: transforms ---
#include <BRepBuilderAPI_Transform.hxx>

// --- OCCT: tessellation (required before glTF export) ---
#include <BRepMesh_IncrementalMesh.hxx>

// --- OCCT: STEP export ---
#include <IFSelect_ReturnStatus.hxx>
#include <STEPControl_StepModelType.hxx>
#include <STEPControl_Writer.hxx>

// --- OCCT: STL export ---
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

} // namespace rrcad
