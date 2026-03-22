/*
 * glue.c — thin C shims that hide mrb_value from Rust.
 *
 * All functions here deal with mrb_value internally and expose only
 * plain C types (char*, int, void*) across the FFI boundary, avoiding the
 * need for Rust to know anything about mRuby's value representation.
 */

#include <mruby.h>
#include <mruby/array.h>
#include <mruby/class.h>
#include <mruby/compile.h>
#include <mruby/data.h>
#include <mruby/error.h>
#include <mruby/hash.h>
#include <mruby/string.h>
#include <stdlib.h>
#include <string.h>

/* -------------------------------------------------------------------------
 * rrcad_mrb_eval — evaluate Ruby source code and return its inspect string.
 *
 * On success  : returns a pointer to a NUL-terminated C string (owned by
 *               mRuby GC — copy before the next eval or GC cycle).
 *               *error_out is set to NULL.
 * On exception: returns NULL.
 *               *error_out points to a NUL-terminated description string
 *               (also GC-owned).  The pending exception is cleared.
 * -------------------------------------------------------------------------
 */
const char* rrcad_mrb_eval(mrb_state* mrb, const char* code, const char** error_out) {
    *error_out = NULL;

    mrb_value result = mrb_load_string(mrb, code);

    if (mrb->exc) {
        mrb_value exc = mrb_obj_value(mrb->exc);
        mrb_value msg = mrb_inspect(mrb, exc);
        *error_out = mrb_string_value_cstr(mrb, &msg);
        mrb->exc = NULL;
        return NULL;
    }

    mrb_value inspected = mrb_inspect(mrb, result);
    return mrb_string_value_cstr(mrb, &inspected);
}

/* =========================================================================
 * Native Shape class
 *
 * Memory model:
 *   Each native Shape wraps a heap-allocated `Box<occt::Shape>` (Rust).
 *   The raw pointer is stored directly in the mRuby RData `void *data` slot.
 *   When the GC collects a native Shape it calls `shape_dfree`, which calls
 *   `rrcad_shape_drop` on the Rust side to run `drop(Box::from_raw(ptr))`.
 *
 *   Stub shapes created via `Shape.new(...)` from Ruby have data == NULL;
 *   `shape_dfree` is a no-op for them.
 * =========================================================================
 */

/* Forward-declare every Rust extern that this file calls. */
extern void* rrcad_make_box(double dx, double dy, double dz, const char** error_out);
extern void* rrcad_make_cylinder(double r, double h, const char** error_out);
extern void* rrcad_make_sphere(double r, const char** error_out);
extern void* rrcad_make_cone(double r1, double r2, double h, const char** error_out);
extern void* rrcad_make_torus(double r1, double r2, const char** error_out);
extern void* rrcad_make_wedge(double dx, double dy, double dz, double ltx, const char** error_out);
extern void rrcad_shape_drop(void* ptr);
extern void* rrcad_import_step(const char* path, const char** error_out);
extern void* rrcad_import_stl(const char* path, const char** error_out);
extern void rrcad_shape_export_step(void* ptr, const char* path, const char** error_out);
extern void* rrcad_shape_fuse(void* a, void* b, const char** error_out);
extern void* rrcad_shape_cut(void* a, void* b, const char** error_out);
extern void* rrcad_shape_common(void* a, void* b, const char** error_out);

/* Phase 5 */
extern void* rrcad_shape_mate(void* ptr, void* from_ptr, void* to_ptr, double offset,
                              const char** error_out);
extern void* rrcad_shape_set_color(void* ptr, double r, double g, double b,
                                   const char** error_out);

/* Phase 2 */
extern void* rrcad_shape_translate(void* ptr, double dx, double dy, double dz,
                                   const char** error_out);
extern void* rrcad_shape_rotate(void* ptr, double ax, double ay, double az, double angle_deg,
                                const char** error_out);
extern void* rrcad_shape_scale(void* ptr, double factor, const char** error_out);
extern void* rrcad_shape_scale_xyz(void* ptr, double sx, double sy, double sz,
                                   const char** error_out);
extern void* rrcad_shape_fillet(void* ptr, double radius, const char** error_out);
extern void* rrcad_shape_chamfer(void* ptr, double dist, const char** error_out);
extern void* rrcad_shape_fillet_sel(void* ptr, double radius, const char* selector,
                                    const char** error_out);
extern void* rrcad_shape_chamfer_sel(void* ptr, double dist, const char* selector,
                                     const char** error_out);
extern void* rrcad_shape_mirror(void* ptr, const char* plane, const char** error_out);
extern void* rrcad_make_rect(double w, double h, const char** error_out);
extern void* rrcad_make_circle_face(double r, const char** error_out);
extern void* rrcad_make_polygon(const double* pts, size_t n_pts, const char** error_out);
extern void* rrcad_make_ellipse_face(double rx, double ry, const char** error_out);
extern void* rrcad_make_arc(double r, double start_deg, double end_deg, const char** error_out);
extern void* rrcad_shape_extrude(void* ptr, double height, const char** error_out);
extern void* rrcad_shape_revolve(void* ptr, double angle_deg, const char** error_out);

/* Phase 3 — splines and sweep */
extern void* rrcad_make_spline_2d(const double* pts, size_t n_pts, const char** error_out);
extern void* rrcad_make_spline_3d(const double* pts, size_t n_pts, const char** error_out);
/* Tangent-constrained variants: tangents[4] = {t0x,t0z,t1x,t1z} for 2D,
   tangents[6] = {t0x,t0y,t0z,t1x,t1y,t1z} for 3D. */
extern void* rrcad_make_spline_2d_tan(const double* pts, size_t n_pts, const double* tangents,
                                      const char** error_out);
extern void* rrcad_make_spline_3d_tan(const double* pts, size_t n_pts, const double* tangents,
                                      const char** error_out);
extern void* rrcad_shape_sweep(void* profile, void* path, const char** error_out);
extern void* rrcad_shape_sweep_sections(const void** ptrs, size_t n, void* path,
                                        const char** error_out);

/* Phase 3 — live preview */
extern void rrcad_preview_shape(void* ptr, const char** error_out);

/* Phase 3 — sub-shape selectors */
extern int rrcad_shape_faces_count(void* ptr, const char* selector, const char** error_out);
extern void* rrcad_shape_faces_get(void* ptr, const char* selector, int idx,
                                   const char** error_out);
extern int rrcad_shape_edges_count(void* ptr, const char* selector, const char** error_out);
extern void* rrcad_shape_edges_get(void* ptr, const char* selector, int idx,
                                   const char** error_out);

/* Phase 4 — query / introspection */
extern void rrcad_shape_bounding_box(void* ptr, double* out, const char** error_out);
extern double rrcad_shape_volume(void* ptr, const char** error_out);
extern double rrcad_shape_surface_area(void* ptr, const char** error_out);

/* Phase 4 — 3-D operations */
extern void* rrcad_shape_loft(const void** ptrs, size_t n, int ruled, const char** error_out);
extern void* rrcad_shape_shell(void* ptr, double thickness, const char** error_out);
extern void* rrcad_shape_offset(void* ptr, double distance, const char** error_out);
extern void* rrcad_shape_simplify(void* ptr, double min_feature_size, const char** error_out);
extern void* rrcad_shape_extrude_ex(void* ptr, double height, double twist_deg, double scale,
                                    const char** error_out);

/* Phase 4 — patterns */
extern void* rrcad_shape_linear_pattern(void* ptr, int n, double dx, double dy, double dz,
                                        const char** error_out);
extern void* rrcad_shape_polar_pattern(void* ptr, int n, double angle_deg,
                                       const char** error_out);

/* Phase 4 — vertices selector */
extern int rrcad_shape_vertices_count(void* ptr, const char* selector, const char** error_out);
extern void* rrcad_shape_vertices_get(void* ptr, const char* selector, int idx,
                                      const char** error_out);

/* Phase 4 — additional exports (stl, gltf, glb, obj) */
extern void rrcad_shape_export_stl(void* ptr, const char* path, const char** error_out);
extern void rrcad_shape_export_gltf(void* ptr, const char* path, const char** error_out);
extern void rrcad_shape_export_glb(void* ptr, const char* path, const char** error_out);
extern void rrcad_shape_export_obj(void* ptr, const char* path, const char** error_out);

/* mRuby data type descriptor — name appears in TypeError messages. */
static void shape_dfree(mrb_state* mrb, void* ptr) {
    (void)mrb;
    rrcad_shape_drop(ptr); /* no-op for NULL */
}

static const mrb_data_type shape_type = {"Shape", shape_dfree};

/* Wrap a raw Rust Box pointer in a new mRuby Shape RData value.
 * The Shape class is looked up per-call so multiple concurrent VMs
 * (e.g. parallel test threads) each see their own class pointer. */
static mrb_value shape_from_ptr(mrb_state* mrb, void* ptr) {
    struct RClass* cls = mrb_class_get(mrb, "Shape");
    struct RData* rd = mrb_data_object_alloc(mrb, cls, ptr, &shape_type);
    return mrb_obj_value(rd);
}

/* Extract and type-check the raw pointer from a Shape mrb_value.
 * Raises TypeError if `v` is not a Shape RData object. */
static void* shape_ptr(mrb_state* mrb, mrb_value v) {
    return mrb_data_get_ptr(mrb, v, &shape_type);
}

/* -------------------------------------------------------------------------
 * Helpers
 * -------------------------------------------------------------------------
 */

/* Check that `ptr` is not NULL (i.e. the shape was created natively).
 * Raises RuntimeError if it is — this protects callers from accessing
 * stub shapes that have no backing OCCT object. */
static void require_native_ptr(mrb_state* mrb, void* ptr) {
    if (!ptr) {
        mrb_raise(mrb, E_RUNTIME_ERROR,
                  "Shape has no backing geometry — "
                  "create shapes via box(), cylinder(), or sphere()");
    }
}

/* -------------------------------------------------------------------------
 * Top-level primitive constructors (defined on Kernel)
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_box(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float dx, dy, dz;
    mrb_get_args(mrb, "fff", &dx, &dy, &dz);

    const char* err = NULL;
    void* ptr = rrcad_make_box((double)dx, (double)dy, (double)dz, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_cylinder(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float r, h;
    mrb_get_args(mrb, "ff", &r, &h);

    const char* err = NULL;
    void* ptr = rrcad_make_cylinder((double)r, (double)h, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_sphere(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float r;
    mrb_get_args(mrb, "f", &r);

    const char* err = NULL;
    void* ptr = rrcad_make_sphere((double)r, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_cone(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float r1, r2, h;
    mrb_get_args(mrb, "fff", &r1, &r2, &h);

    const char* err = NULL;
    void* ptr = rrcad_make_cone((double)r1, (double)r2, (double)h, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_torus(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float r1, r2;
    mrb_get_args(mrb, "ff", &r1, &r2);

    const char* err = NULL;
    void* ptr = rrcad_make_torus((double)r1, (double)r2, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_wedge(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float dx, dy, dz, ltx;
    mrb_get_args(mrb, "ffff", &dx, &dy, &dz, &ltx);

    const char* err = NULL;
    void* ptr = rrcad_make_wedge((double)dx, (double)dy, (double)dz, (double)ltx, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

/* -------------------------------------------------------------------------
 * Shape instance methods
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_shape_inspect(mrb_state* mrb, mrb_value self) {
    (void)self;
    return mrb_str_new_cstr(mrb, "#<Shape>");
}

/* .export("path") — dispatches by file extension:
 *   .step / .stp  → STEP AP203
 *   .stl          → ASCII STL
 *   .glb          → binary glTF (GLB)
 *   .gltf         → text glTF
 *   .obj          → Wavefront OBJ
 * Defaults to STEP for any unrecognised extension. */
static mrb_value mrb_rrcad_shape_export(mrb_state* mrb, mrb_value self) {
    const char* path;
    mrb_get_args(mrb, "z", &path);

    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);

    /* Find the last '.' to determine the extension. */
    const char* dot = strrchr(path, '.');
    const char* err = NULL;

    if (dot && (strcasecmp(dot, ".stl") == 0)) {
        rrcad_shape_export_stl(ptr, path, &err);
    } else if (dot && (strcasecmp(dot, ".glb") == 0)) {
        rrcad_shape_export_glb(ptr, path, &err);
    } else if (dot && (strcasecmp(dot, ".gltf") == 0)) {
        rrcad_shape_export_gltf(ptr, path, &err);
    } else if (dot && (strcasecmp(dot, ".obj") == 0)) {
        rrcad_shape_export_obj(ptr, path, &err);
    } else {
        /* Default: STEP (.step, .stp, or unknown extension) */
        rrcad_shape_export_step(ptr, path, &err);
    }
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return self;
}

static mrb_value mrb_rrcad_shape_fuse(mrb_state* mrb, mrb_value self) {
    mrb_value other;
    mrb_get_args(mrb, "o", &other);

    void* a = DATA_PTR(self);
    require_native_ptr(mrb, a);
    void* b = shape_ptr(mrb, other);
    require_native_ptr(mrb, b);

    const char* err = NULL;
    void* result = rrcad_shape_fuse(a, b, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_cut(mrb_state* mrb, mrb_value self) {
    mrb_value other;
    mrb_get_args(mrb, "o", &other);

    void* a = DATA_PTR(self);
    require_native_ptr(mrb, a);
    void* b = shape_ptr(mrb, other);
    require_native_ptr(mrb, b);

    const char* err = NULL;
    void* result = rrcad_shape_cut(a, b, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_common(mrb_state* mrb, mrb_value self) {
    mrb_value other;
    mrb_get_args(mrb, "o", &other);

    void* a = DATA_PTR(self);
    require_native_ptr(mrb, a);
    void* b = shape_ptr(mrb, other);
    require_native_ptr(mrb, b);

    const char* err = NULL;
    void* result = rrcad_shape_common(a, b, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 5: Assembly mating
 * -------------------------------------------------------------------------
 */

/* shape.mate(from_face, to_face, offset=0.0) → Shape
 * Returns a copy of `shape` repositioned so that `from_face` lies flush
 * against `to_face`.  Optional `offset` (Float) leaves a gap or creates
 * interference.  Both face arguments must be planar Shape objects. */
static mrb_value mrb_rrcad_shape_mate(mrb_state* mrb, mrb_value self) {
    mrb_value from_val, to_val;
    mrb_float offset = 0.0;
    mrb_get_args(mrb, "oo|f", &from_val, &to_val, &offset);

    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);

    void* from_ptr = mrb_data_p(from_val) ? DATA_PTR(from_val) : NULL;
    void* to_ptr   = mrb_data_p(to_val)   ? DATA_PTR(to_val)   : NULL;
    if (!from_ptr || !to_ptr)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "mate: from and to must be Shape objects");

    const char* err = NULL;
    void* result = rrcad_shape_mate(ptr, from_ptr, to_ptr, (double)offset, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 5: Color
 * -------------------------------------------------------------------------
 */

/* shape.color(r, g, b) → Shape
 * r/g/b are Float values in [0.0, 1.0] (sRGB).
 * Returns a new Shape with the same geometry and the color tag attached.
 * The color is embedded in GLB / glTF / OBJ output during export. */
static mrb_value mrb_rrcad_shape_color(mrb_state* mrb, mrb_value self) {
    mrb_float r, g, b;
    mrb_get_args(mrb, "fff", &r, &g, &b);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_set_color(ptr, (double)r, (double)g, (double)b, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 2: Transform methods
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_shape_translate(mrb_state* mrb, mrb_value self) {
    mrb_float dx, dy, dz;
    mrb_get_args(mrb, "fff", &dx, &dy, &dz);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_translate(ptr, (double)dx, (double)dy, (double)dz, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_rotate(mrb_state* mrb, mrb_value self) {
    mrb_float ax, ay, az, angle;
    mrb_get_args(mrb, "ffff", &ax, &ay, &az, &angle);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_rotate(ptr, (double)ax, (double)ay, (double)az, (double)angle, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* scale(factor)       — uniform scale
 * scale(sx, sy, sz)  — non-uniform scale (independent axis factors) */
static mrb_value mrb_rrcad_shape_scale(mrb_state* mrb, mrb_value self) {
    mrb_float sx, sy = 0.0, sz = 0.0;
    mrb_int argc = mrb_get_args(mrb, "f|ff", &sx, &sy, &sz);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result;
    if (argc == 1) {
        /* Uniform: use the exact gp_Trsf path (no approximation). */
        result = rrcad_shape_scale(ptr, (double)sx, &err);
    } else if (argc == 3) {
        result = rrcad_shape_scale_xyz(ptr, (double)sx, (double)sy, (double)sz, &err);
    } else {
        mrb_raise(mrb, E_ARGUMENT_ERROR, "scale requires 1 argument (uniform) or 3 (sx, sy, sz)");
        return mrb_nil_value(); /* unreachable */
    }
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* fillet(r)            — round all edges
 * fillet(r, :selector) — round only edges matching selector
 *                        (:all, :vertical, :horizontal) */
static mrb_value mrb_rrcad_shape_fillet(mrb_state* mrb, mrb_value self) {
    mrb_float r;
    mrb_value sel_val = mrb_nil_value();
    mrb_get_args(mrb, "f|o", &r, &sel_val);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result;
    if (mrb_nil_p(sel_val)) {
        result = rrcad_shape_fillet(ptr, (double)r, &err);
    } else {
        if (!mrb_symbol_p(sel_val))
            mrb_raise(mrb, E_TYPE_ERROR, "fillet selector must be a Symbol (:all, :vertical, :horizontal)");
        const char* sel = mrb_sym_name(mrb, mrb_symbol(sel_val));
        result = rrcad_shape_fillet_sel(ptr, (double)r, sel, &err);
    }
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* chamfer(d)            — bevel all edges
 * chamfer(d, :selector) — bevel only edges matching selector */
static mrb_value mrb_rrcad_shape_chamfer(mrb_state* mrb, mrb_value self) {
    mrb_float d;
    mrb_value sel_val = mrb_nil_value();
    mrb_get_args(mrb, "f|o", &d, &sel_val);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result;
    if (mrb_nil_p(sel_val)) {
        result = rrcad_shape_chamfer(ptr, (double)d, &err);
    } else {
        if (!mrb_symbol_p(sel_val))
            mrb_raise(mrb, E_TYPE_ERROR, "chamfer selector must be a Symbol (:all, :vertical, :horizontal)");
        const char* sel = mrb_sym_name(mrb, mrb_symbol(sel_val));
        result = rrcad_shape_chamfer_sel(ptr, (double)d, sel, &err);
    }
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_mirror(mrb_state* mrb, mrb_value self) {
    mrb_sym plane_sym;
    mrb_get_args(mrb, "n", &plane_sym);
    const char* plane = mrb_sym_name(mrb, plane_sym);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_mirror(ptr, plane, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 2: Sketch constructors (top-level)
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_rect(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float w, h;
    mrb_get_args(mrb, "ff", &w, &h);
    const char* err = NULL;
    void* ptr = rrcad_make_rect((double)w, (double)h, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_circle(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float r;
    mrb_get_args(mrb, "f", &r);
    const char* err = NULL;
    void* ptr = rrcad_make_circle_face((double)r, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

/* -------------------------------------------------------------------------
 * Phase 2: Extrude / Revolve
 * -------------------------------------------------------------------------
 */

/* extrude(height, twist_deg: 0, scale: 1.0)
 *
 * When called without keyword arguments this is identical to the Phase 2
 * extrude (uses BRepPrimAPI_MakePrism).  When twist_deg or scale are
 * supplied the extended path (BRepOffsetAPI_ThruSections) is used. */
static mrb_value mrb_rrcad_shape_extrude(mrb_state* mrb, mrb_value self) {
    mrb_float height;
    mrb_value opts = mrb_nil_value();
    mrb_get_args(mrb, "f|H", &height, &opts);

    double twist_deg = 0.0;
    double scale = 1.0;

    if (!mrb_nil_p(opts) && mrb_hash_p(opts)) {
        mrb_value twist_val =
            mrb_hash_fetch(mrb, opts, mrb_symbol_value(mrb_intern_lit(mrb, "twist_deg")),
                           mrb_float_value(mrb, 0.0));
        mrb_value scale_val = mrb_hash_fetch(mrb, opts,
                                              mrb_symbol_value(mrb_intern_lit(mrb, "scale")),
                                              mrb_float_value(mrb, 1.0));
        if (mrb_float_p(twist_val))
            twist_deg = (double)mrb_float(twist_val);
        else if (mrb_integer_p(twist_val))
            twist_deg = (double)mrb_integer(twist_val);
        if (mrb_float_p(scale_val))
            scale = (double)mrb_float(scale_val);
        else if (mrb_integer_p(scale_val))
            scale = (double)mrb_integer(scale_val);
    }

    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    /* rrcad_shape_extrude_ex falls back to MakePrism when twist≈0 and scale≈1 */
    void* result = rrcad_shape_extrude_ex(ptr, (double)height, twist_deg, scale, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 4: 3-D operations — loft, shell, offset
 * -------------------------------------------------------------------------
 */

/* loft(profiles, ruled: false)
 *
 * Top-level Kernel method.  `profiles` is an Array of Shape objects; each
 * must be a Face, Wire, or Vertex (for a pointed cap/base).  Optional
 * `ruled: true` produces a ruled (flat-face) solid instead of a smooth one. */
static mrb_value mrb_rrcad_loft(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value arr;
    mrb_value opts = mrb_nil_value();
    mrb_get_args(mrb, "A|H", &arr, &opts);

    int ruled = 0;
    if (!mrb_nil_p(opts) && mrb_hash_p(opts)) {
        mrb_value v = mrb_hash_fetch(mrb, opts,
                                     mrb_symbol_value(mrb_intern_lit(mrb, "ruled")),
                                     mrb_false_value());
        ruled = mrb_test(v) ? 1 : 0;
    }

    int n = (int)RARRAY_LEN(arr);
    if (n < 2)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "loft requires at least 2 profiles");

    const void** ptrs = (const void**)malloc((size_t)n * sizeof(void*));
    if (!ptrs)
        mrb_raise(mrb, E_RUNTIME_ERROR, "out of memory");

    for (int i = 0; i < n; i++) {
        mrb_value elem = mrb_ary_ref(mrb, arr, i);
        void* p = shape_ptr(mrb, elem); /* type-checks the element */
        require_native_ptr(mrb, p);
        ptrs[i] = p;
    }

    const char* err = NULL;
    void* result = rrcad_shape_loft(ptrs, (size_t)n, ruled, &err);
    free(ptrs);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* .shell(thickness) — hollow out the solid, removing the top face. */
static mrb_value mrb_rrcad_shape_shell(mrb_state* mrb, mrb_value self) {
    mrb_float thickness;
    mrb_get_args(mrb, "f", &thickness);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_shell(ptr, (double)thickness, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* .offset(distance) — inflate (>0) or deflate (<0) the solid uniformly. */
static mrb_value mrb_rrcad_shape_offset(mrb_state* mrb, mrb_value self) {
    mrb_float distance;
    mrb_get_args(mrb, "f", &distance);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_offset(ptr, (double)distance, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* .simplify(min_feature_size) — remove small holes/fillets via defeaturing.
   Faces with area < min_feature_size² are passed to BRepAlgoAPI_Defeaturing. */
static mrb_value mrb_rrcad_shape_simplify(mrb_state* mrb, mrb_value self) {
    mrb_float min_size;
    mrb_get_args(mrb, "f", &min_size);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_simplify(ptr, (double)min_size, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_revolve(mrb_state* mrb, mrb_value self) {
    mrb_float angle = 360.0;
    mrb_get_args(mrb, "|f", &angle);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_revolve(ptr, (double)angle, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 3: Spline constructors and sweep
 * -------------------------------------------------------------------------
 */

/* Extract a flat double array from a Ruby Array of 2-element inner Arrays.
 * Returns a malloc'd buffer of n * stride doubles.  Caller must free().
 * Raises on type errors after freeing the buffer. */
static double* extract_point_array(mrb_state* mrb, mrb_value arr, int stride, int* n_out) {
    int n = (int)RARRAY_LEN(arr);
    *n_out = n;
    double* pts = (double*)malloc((size_t)(n * stride) * sizeof(double));
    if (!pts)
        mrb_raise(mrb, E_RUNTIME_ERROR, "out of memory");

    for (int i = 0; i < n; i++) {
        mrb_value inner = mrb_ary_ref(mrb, arr, i);
        if (!mrb_array_p(inner) || (int)RARRAY_LEN(inner) < stride) {
            free(pts);
            mrb_raisef(mrb, E_ARGUMENT_ERROR,
                       "each point must be an Array of %d numbers (got element %d)", stride, i);
        }
        for (int j = 0; j < stride; j++) {
            mrb_value v = mrb_ary_ref(mrb, inner, j);
            if (mrb_float_p(v)) {
                pts[i * stride + j] = (double)mrb_float(v);
            } else if (mrb_integer_p(v)) {
                pts[i * stride + j] = (double)mrb_integer(v);
            } else {
                free(pts);
                mrb_raise(mrb, E_ARGUMENT_ERROR, "point coordinates must be numbers");
            }
        }
    }
    return pts;
}

static mrb_value mrb_rrcad_spline_2d(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value arr;
    /* mRuby assigns mrb_undef_value() for absent optional keyword args. */
    mrb_value tangents_val = mrb_undef_value();
    mrb_sym tangents_kw = mrb_intern_cstr(mrb, "tangents");
    mrb_kwargs kwargs = {1, 0, &tangents_kw, &tangents_val, NULL};
    mrb_get_args(mrb, "A:", &arr, &kwargs);

    int n = 0;
    double* pts = extract_point_array(mrb, arr, 2, &n);

    const char* err = NULL;
    void* ptr;

    if (!mrb_undef_p(tangents_val)) {
        /* tangents: [[t0x, t0z], [t1x, t1z]] */
        if (!mrb_array_p(tangents_val) || RARRAY_LEN(tangents_val) != 2) {
            free(pts);
            mrb_raise(mrb, E_ARGUMENT_ERROR, "spline_2d tangents: must be [[t0x,t0z],[t1x,t1z]]");
        }
        int tn = 0;
        double* tvecs = extract_point_array(mrb, tangents_val, 2, &tn);
        /* tvecs layout: [t0x, t0z, t1x, t1z] */
        ptr = rrcad_make_spline_2d_tan(pts, (size_t)n, tvecs, &err);
        free(tvecs);
    } else {
        ptr = rrcad_make_spline_2d(pts, (size_t)n, &err);
    }

    free(pts);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_spline_3d(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value arr;
    /* mRuby assigns mrb_undef_value() for absent optional keyword args. */
    mrb_value tangents_val = mrb_undef_value();
    mrb_sym tangents_kw = mrb_intern_cstr(mrb, "tangents");
    mrb_kwargs kwargs = {1, 0, &tangents_kw, &tangents_val, NULL};
    mrb_get_args(mrb, "A:", &arr, &kwargs);

    int n = 0;
    double* pts = extract_point_array(mrb, arr, 3, &n);

    const char* err = NULL;
    void* ptr;

    if (!mrb_undef_p(tangents_val)) {
        /* tangents: [[t0x, t0y, t0z], [t1x, t1y, t1z]] */
        if (!mrb_array_p(tangents_val) || RARRAY_LEN(tangents_val) != 2) {
            free(pts);
            mrb_raise(mrb, E_ARGUMENT_ERROR,
                      "spline_3d tangents: must be [[t0x,t0y,t0z],[t1x,t1y,t1z]]");
        }
        int tn = 0;
        double* tvecs = extract_point_array(mrb, tangents_val, 3, &tn);
        /* tvecs layout: [t0x, t0y, t0z, t1x, t1y, t1z] */
        ptr = rrcad_make_spline_3d_tan(pts, (size_t)n, tvecs, &err);
        free(tvecs);
    } else {
        ptr = rrcad_make_spline_3d(pts, (size_t)n, &err);
    }

    free(pts);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_shape_sweep(mrb_state* mrb, mrb_value self) {
    mrb_value path_val;
    mrb_get_args(mrb, "o", &path_val);

    void* profile = DATA_PTR(self);
    require_native_ptr(mrb, profile);
    void* path = shape_ptr(mrb, path_val);
    require_native_ptr(mrb, path);

    const char* err = NULL;
    void* result = rrcad_shape_sweep(profile, path, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* sweep_sections(path, profiles)
 *
 * Kernel-level method.  Sweeps multiple section profiles along `path` (a Wire
 * from spline_3d), morphing smoothly between them.  The first profile is placed
 * at the spine start, the last at the spine end, and any intermediate profiles
 * at evenly spaced parametric positions.
 *
 * `path`     — Shape (Wire) produced by spline_3d.
 * `profiles` — Array of Shape objects; each must be a Face (e.g. circle, rect),
 *              Wire, or Vertex (pointed cap).  At least 2 required.
 *
 * Example:
 *   path = spline_3d([[-4.5,0,1.5], [-8.54,0,4.8], [-4.0,0,6.3]])
 *   handle = sweep_sections(path, [circle(1.4), circle(0.7), circle(1.4)])
 */
static mrb_value mrb_rrcad_sweep_sections(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value path_val;
    mrb_value arr;
    mrb_get_args(mrb, "oA", &path_val, &arr);

    void* path = shape_ptr(mrb, path_val);
    require_native_ptr(mrb, path);

    int n = (int)RARRAY_LEN(arr);
    if (n < 2)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "sweep_sections requires at least 2 profiles");

    const void** ptrs = (const void**)malloc((size_t)n * sizeof(void*));
    if (!ptrs)
        mrb_raise(mrb, E_RUNTIME_ERROR, "out of memory");

    for (int i = 0; i < n; i++) {
        mrb_value elem = mrb_ary_ref(mrb, arr, i);
        void* p = shape_ptr(mrb, elem);
        require_native_ptr(mrb, p);
        ptrs[i] = p;
    }

    const char* err = NULL;
    void* result = rrcad_shape_sweep_sections(ptrs, (size_t)n, path, &err);
    free(ptrs);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 4: Sketch profiles — polygon, ellipse, arc
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_polygon(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value arr;
    mrb_get_args(mrb, "A", &arr);

    int n = 0;
    double* pts = extract_point_array(mrb, arr, 2, &n);

    const char* err = NULL;
    void* ptr = rrcad_make_polygon(pts, (size_t)n, &err);
    free(pts);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_ellipse(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float rx, ry;
    mrb_get_args(mrb, "ff", &rx, &ry);
    const char* err = NULL;
    void* ptr = rrcad_make_ellipse_face((double)rx, (double)ry, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_arc(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_float r, start_deg, end_deg;
    mrb_get_args(mrb, "fff", &r, &start_deg, &end_deg);
    const char* err = NULL;
    void* ptr = rrcad_make_arc((double)r, (double)start_deg, (double)end_deg, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

/* -------------------------------------------------------------------------
 * Phase 4: Import — import_step / import_stl
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_import_step(mrb_state* mrb, mrb_value self) {
    (void)self;
    const char* path;
    mrb_get_args(mrb, "z", &path);
    const char* err = NULL;
    void* ptr = rrcad_import_step(path, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_import_stl(mrb_state* mrb, mrb_value self) {
    (void)self;
    const char* path;
    mrb_get_args(mrb, "z", &path);
    const char* err = NULL;
    void* ptr = rrcad_import_stl(path, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

/* -------------------------------------------------------------------------
 * Phase 3: Live preview — preview(shape)
 *
 * Tessellates the shape to a temp GLB file and signals WebSocket clients
 * to reload.  A no-op when not in --preview mode.
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_preview(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value shape_val;
    mrb_get_args(mrb, "o", &shape_val);

    void* ptr = shape_ptr(mrb, shape_val);
    if (!ptr)
        return mrb_nil_value(); /* stub shape (no OCCT backing) — silent no-op */

    const char* err = NULL;
    rrcad_preview_shape(ptr, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return mrb_nil_value();
}

/* -------------------------------------------------------------------------
 * Phase 3: Sub-shape selectors — faces and edges
 *
 * Both methods accept a Ruby Symbol selector (:all, :top, :bottom, :side for
 * faces; :all, :vertical, :horizontal for edges) and return an Array of Shape
 * objects, each wrapping one matching sub-shape.
 * -------------------------------------------------------------------------
 */

/* .faces(selector) — selector is a Symbol (:all, :top, :bottom, :side) or a
 * String for direction-based selectors (">Z", "<X", etc.). */
static mrb_value mrb_rrcad_shape_faces(mrb_state* mrb, mrb_value self) {
    mrb_value sel_val;
    mrb_get_args(mrb, "o", &sel_val);

    const char* sel;
    if (mrb_symbol_p(sel_val)) {
        sel = mrb_sym_name(mrb, mrb_symbol(sel_val));
    } else if (mrb_string_p(sel_val)) {
        sel = mrb_str_to_cstr(mrb, sel_val);
    } else {
        mrb_raise(mrb, E_TYPE_ERROR,
                  "faces: selector must be a Symbol (:all/:top/:bottom/:side) "
                  "or a String (\">Z\", \"<X\", etc.)");
    }

    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);

    const char* err = NULL;
    int count = rrcad_shape_faces_count(ptr, sel, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);

    mrb_value result = mrb_ary_new_capa(mrb, count);
    for (int i = 0; i < count; i++) {
        err = NULL;
        void* face_ptr = rrcad_shape_faces_get(ptr, sel, i, &err);
        if (err)
            mrb_raise(mrb, E_RUNTIME_ERROR, err);
        mrb_ary_push(mrb, result, shape_from_ptr(mrb, face_ptr));
    }
    return result;
}

static mrb_value mrb_rrcad_shape_edges(mrb_state* mrb, mrb_value self) {
    mrb_sym sel_sym;
    mrb_get_args(mrb, "n", &sel_sym);
    const char* sel = mrb_sym_name(mrb, sel_sym);

    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);

    const char* err = NULL;
    int count = rrcad_shape_edges_count(ptr, sel, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);

    mrb_value result = mrb_ary_new_capa(mrb, count);
    for (int i = 0; i < count; i++) {
        err = NULL;
        void* edge_ptr = rrcad_shape_edges_get(ptr, sel, i, &err);
        if (err)
            mrb_raise(mrb, E_RUNTIME_ERROR, err);
        mrb_ary_push(mrb, result, shape_from_ptr(mrb, edge_ptr));
    }
    return result;
}

/* -------------------------------------------------------------------------
 * Phase 4: Vertices selector
 *
 * .vertices(:all) — returns an Array of Shape objects, each wrapping one
 * unique vertex.  Uses TopTools_IndexedMapOfShape for deduplication.
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_shape_vertices(mrb_state* mrb, mrb_value self) {
    mrb_sym sel_sym;
    mrb_get_args(mrb, "n", &sel_sym);
    const char* sel = mrb_sym_name(mrb, sel_sym);

    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);

    const char* err = NULL;
    int count = rrcad_shape_vertices_count(ptr, sel, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);

    mrb_value result = mrb_ary_new_capa(mrb, count);
    for (int i = 0; i < count; i++) {
        err = NULL;
        void* vertex_ptr = rrcad_shape_vertices_get(ptr, sel, i, &err);
        if (err)
            mrb_raise(mrb, E_RUNTIME_ERROR, err);
        mrb_ary_push(mrb, result, shape_from_ptr(mrb, vertex_ptr));
    }
    return result;
}

/* -------------------------------------------------------------------------
 * Phase 4: Query / introspection
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_shape_bounding_box(mrb_state* mrb, mrb_value self) {
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);

    double bounds[6];
    const char* err = NULL;
    rrcad_shape_bounding_box(ptr, bounds, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);

    /* Return {x:, y:, z:, dx:, dy:, dz:} where x/y/z is the min corner
     * and dx/dy/dz is the extent in each axis. */
    mrb_value hash = mrb_hash_new(mrb);
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "x")),
                 mrb_float_value(mrb, bounds[0]));
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "y")),
                 mrb_float_value(mrb, bounds[1]));
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "z")),
                 mrb_float_value(mrb, bounds[2]));
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "dx")),
                 mrb_float_value(mrb, bounds[3] - bounds[0]));
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "dy")),
                 mrb_float_value(mrb, bounds[4] - bounds[1]));
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "dz")),
                 mrb_float_value(mrb, bounds[5] - bounds[2]));
    return hash;
}

static mrb_value mrb_rrcad_shape_volume(mrb_state* mrb, mrb_value self) {
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    double vol = rrcad_shape_volume(ptr, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return mrb_float_value(mrb, vol);
}

static mrb_value mrb_rrcad_shape_surface_area(mrb_state* mrb, mrb_value self) {
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    double area = rrcad_shape_surface_area(ptr, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return mrb_float_value(mrb, area);
}

/* -------------------------------------------------------------------------
 * Phase 4: Patterns — linear_pattern / polar_pattern
 *
 * Both are top-level Kernel methods that return a Compound of n copies.
 *
 *   linear_pattern(shape, n, [dx, dy, dz])
 *     — copy i is translated by i * [dx, dy, dz]; i=0 is the original.
 *
 *   polar_pattern(shape, n, angle_deg)
 *     — copy i is rotated by i * (angle_deg / n) degrees around Z; i=0 is
 *       the original.  A total of 360 gives evenly-spaced full-circle copies.
 * -------------------------------------------------------------------------
 */

/* linear_pattern(shape, n, [dx, dy, dz]) */
static mrb_value mrb_rrcad_linear_pattern(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value shape_val, vec_val;
    mrb_int n;
    mrb_get_args(mrb, "oiA", &shape_val, &n, &vec_val);

    void* ptr = shape_ptr(mrb, shape_val);
    require_native_ptr(mrb, ptr);

    if ((int)RARRAY_LEN(vec_val) < 3)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "linear_pattern: vector must have 3 elements [dx,dy,dz]");

    double dx = 0.0, dy = 0.0, dz = 0.0;
    for (int j = 0; j < 3; j++) {
        mrb_value v = mrb_ary_ref(mrb, vec_val, j);
        double d = mrb_float_p(v)   ? (double)mrb_float(v)
                   : mrb_integer_p(v) ? (double)mrb_integer(v)
                                      : 0.0;
        if (j == 0) dx = d;
        else if (j == 1) dy = d;
        else dz = d;
    }

    const char* err = NULL;
    void* result = rrcad_shape_linear_pattern(ptr, (int)n, dx, dy, dz, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* polar_pattern(shape, n, angle_deg) */
static mrb_value mrb_rrcad_polar_pattern(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value shape_val;
    mrb_int n;
    mrb_float angle_deg;
    mrb_get_args(mrb, "oif", &shape_val, &n, &angle_deg);

    void* ptr = shape_ptr(mrb, shape_val);
    require_native_ptr(mrb, ptr);

    const char* err = NULL;
    void* result = rrcad_shape_polar_pattern(ptr, (int)n, (double)angle_deg, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* =========================================================================
 * rrcad_register_shape_class
 *
 * Called from Rust (MrubyVm::new) after the DSL prelude has been evaluated.
 * Registering after the prelude means our native methods shadow the stubs.
 * =========================================================================
 */
void rrcad_register_shape_class(mrb_state* mrb) {
    /* Get (or create) the Shape class and mark its instances as RData. */
    struct RClass* shape_class = mrb_define_class(mrb, "Shape", mrb->object_class);
    MRB_SET_INSTANCE_TT(shape_class, MRB_TT_DATA);

    /* Instance methods — override the prelude stubs. */
    mrb_define_method(mrb, shape_class, "inspect", mrb_rrcad_shape_inspect, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "to_s", mrb_rrcad_shape_inspect, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "export", mrb_rrcad_shape_export, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "fuse", mrb_rrcad_shape_fuse, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "cut", mrb_rrcad_shape_cut, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "common", mrb_rrcad_shape_common, MRB_ARGS_REQ(1));

    /* Phase 5: Assembly mating */
    mrb_define_method(mrb, shape_class, "mate", mrb_rrcad_shape_mate,
                      MRB_ARGS_REQ(2) | MRB_ARGS_OPT(1));

    /* Phase 5: Color */
    mrb_define_method(mrb, shape_class, "color", mrb_rrcad_shape_color, MRB_ARGS_REQ(3));

    /* Phase 2: Transforms */
    mrb_define_method(mrb, shape_class, "translate", mrb_rrcad_shape_translate, MRB_ARGS_REQ(3));
    mrb_define_method(mrb, shape_class, "rotate", mrb_rrcad_shape_rotate, MRB_ARGS_REQ(4));
    mrb_define_method(mrb, shape_class, "scale", mrb_rrcad_shape_scale,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(2));
    mrb_define_method(mrb, shape_class, "fillet", mrb_rrcad_shape_fillet,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1)); /* (r[, :selector]) */
    mrb_define_method(mrb, shape_class, "chamfer", mrb_rrcad_shape_chamfer,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1)); /* (d[, :selector]) */
    mrb_define_method(mrb, shape_class, "mirror", mrb_rrcad_shape_mirror, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "extrude", mrb_rrcad_shape_extrude,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1));
    mrb_define_method(mrb, shape_class, "revolve", mrb_rrcad_shape_revolve, MRB_ARGS_OPT(1));

    /* Top-level primitive constructors — available everywhere via Kernel. */
    mrb_define_method(mrb, mrb->kernel_module, "box", mrb_rrcad_box, MRB_ARGS_REQ(3));
    mrb_define_method(mrb, mrb->kernel_module, "cylinder", mrb_rrcad_cylinder, MRB_ARGS_REQ(2));
    mrb_define_method(mrb, mrb->kernel_module, "sphere", mrb_rrcad_sphere, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "cone", mrb_rrcad_cone, MRB_ARGS_REQ(3));
    mrb_define_method(mrb, mrb->kernel_module, "torus", mrb_rrcad_torus, MRB_ARGS_REQ(2));
    mrb_define_method(mrb, mrb->kernel_module, "wedge", mrb_rrcad_wedge, MRB_ARGS_REQ(4));

    /* Phase 2: Sketch constructors */
    mrb_define_method(mrb, mrb->kernel_module, "rect", mrb_rrcad_rect, MRB_ARGS_REQ(2));
    mrb_define_method(mrb, mrb->kernel_module, "circle", mrb_rrcad_circle, MRB_ARGS_REQ(1));

    /* Phase 4: Sketch profiles */
    mrb_define_method(mrb, mrb->kernel_module, "polygon", mrb_rrcad_polygon, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "ellipse", mrb_rrcad_ellipse, MRB_ARGS_REQ(2));
    mrb_define_method(mrb, mrb->kernel_module, "arc", mrb_rrcad_arc, MRB_ARGS_REQ(3));

    /* Phase 3: Spline constructors and sweep */
    mrb_define_method(mrb, mrb->kernel_module, "spline_2d", mrb_rrcad_spline_2d,
                      MRB_ARGS_REQ(1) | MRB_ARGS_KEY(1, 0)); /* (pts[, tangents:]) */
    mrb_define_method(mrb, mrb->kernel_module, "spline_3d", mrb_rrcad_spline_3d,
                      MRB_ARGS_REQ(1) | MRB_ARGS_KEY(1, 0)); /* (pts[, tangents:]) */
    mrb_define_method(mrb, shape_class, "sweep", mrb_rrcad_shape_sweep, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "sweep_sections", mrb_rrcad_sweep_sections,
                      MRB_ARGS_REQ(2));

    /* Phase 3: Live preview */
    mrb_define_method(mrb, mrb->kernel_module, "preview", mrb_rrcad_preview, MRB_ARGS_REQ(1));

    /* Phase 3: Sub-shape selectors */
    mrb_define_method(mrb, shape_class, "faces", mrb_rrcad_shape_faces, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "edges", mrb_rrcad_shape_edges, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "vertices", mrb_rrcad_shape_vertices, MRB_ARGS_REQ(1));

    /* Phase 4: Import */
    mrb_define_method(mrb, mrb->kernel_module, "import_step", mrb_rrcad_import_step,
                      MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "import_stl", mrb_rrcad_import_stl,
                      MRB_ARGS_REQ(1));

    /* Phase 4: 3-D operations */
    mrb_define_method(mrb, mrb->kernel_module, "loft", mrb_rrcad_loft,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1));
    mrb_define_method(mrb, shape_class, "shell", mrb_rrcad_shape_shell, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "offset", mrb_rrcad_shape_offset, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "simplify", mrb_rrcad_shape_simplify, MRB_ARGS_REQ(1));

    /* Phase 4: Patterns */
    mrb_define_method(mrb, mrb->kernel_module, "linear_pattern", mrb_rrcad_linear_pattern,
                      MRB_ARGS_REQ(3));
    mrb_define_method(mrb, mrb->kernel_module, "polar_pattern", mrb_rrcad_polar_pattern,
                      MRB_ARGS_REQ(3));

    /* Phase 4: Query / introspection */
    mrb_define_method(mrb, shape_class, "bounding_box", mrb_rrcad_shape_bounding_box,
                      MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "volume", mrb_rrcad_shape_volume, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "surface_area", mrb_rrcad_shape_surface_area,
                      MRB_ARGS_NONE());
}
