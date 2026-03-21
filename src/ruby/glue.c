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

/* Phase 2 */
extern void* rrcad_shape_translate(void* ptr, double dx, double dy, double dz,
                                   const char** error_out);
extern void* rrcad_shape_rotate(void* ptr, double ax, double ay, double az, double angle_deg,
                                const char** error_out);
extern void* rrcad_shape_scale(void* ptr, double factor, const char** error_out);
extern void* rrcad_shape_fillet(void* ptr, double radius, const char** error_out);
extern void* rrcad_shape_chamfer(void* ptr, double dist, const char** error_out);
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
extern void* rrcad_shape_sweep(void* profile, void* path, const char** error_out);

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

static mrb_value mrb_rrcad_shape_export(mrb_state* mrb, mrb_value self) {
    const char* path;
    mrb_get_args(mrb, "z", &path);

    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);

    const char* err = NULL;
    rrcad_shape_export_step(ptr, path, &err);
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

static mrb_value mrb_rrcad_shape_scale(mrb_state* mrb, mrb_value self) {
    mrb_float factor;
    mrb_get_args(mrb, "f", &factor);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_scale(ptr, (double)factor, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_fillet(mrb_state* mrb, mrb_value self) {
    mrb_float r;
    mrb_value opts = mrb_nil_value();
    mrb_get_args(mrb, "f|o", &r, &opts);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_fillet(ptr, (double)r, &err);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_chamfer(mrb_state* mrb, mrb_value self) {
    mrb_float d;
    mrb_value opts = mrb_nil_value();
    mrb_get_args(mrb, "f|o", &d, &opts);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_chamfer(ptr, (double)d, &err);
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

static mrb_value mrb_rrcad_shape_extrude(mrb_state* mrb, mrb_value self) {
    mrb_float height;
    mrb_get_args(mrb, "f", &height);
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    const char* err = NULL;
    void* result = rrcad_shape_extrude(ptr, (double)height, &err);
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
    mrb_get_args(mrb, "A", &arr);

    int n = 0;
    double* pts = extract_point_array(mrb, arr, 2, &n);

    const char* err = NULL;
    void* ptr = rrcad_make_spline_2d(pts, (size_t)n, &err);
    free(pts);
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_spline_3d(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value arr;
    mrb_get_args(mrb, "A", &arr);

    int n = 0;
    double* pts = extract_point_array(mrb, arr, 3, &n);

    const char* err = NULL;
    void* ptr = rrcad_make_spline_3d(pts, (size_t)n, &err);
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

static mrb_value mrb_rrcad_shape_faces(mrb_state* mrb, mrb_value self) {
    mrb_sym sel_sym;
    mrb_get_args(mrb, "n", &sel_sym);
    const char* sel = mrb_sym_name(mrb, sel_sym);

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

    /* Phase 2: Transforms */
    mrb_define_method(mrb, shape_class, "translate", mrb_rrcad_shape_translate, MRB_ARGS_REQ(3));
    mrb_define_method(mrb, shape_class, "rotate", mrb_rrcad_shape_rotate, MRB_ARGS_REQ(4));
    mrb_define_method(mrb, shape_class, "scale", mrb_rrcad_shape_scale, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "fillet", mrb_rrcad_shape_fillet,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1));
    mrb_define_method(mrb, shape_class, "chamfer", mrb_rrcad_shape_chamfer,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1));
    mrb_define_method(mrb, shape_class, "mirror", mrb_rrcad_shape_mirror, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "extrude", mrb_rrcad_shape_extrude, MRB_ARGS_REQ(1));
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
    mrb_define_method(mrb, mrb->kernel_module, "spline_2d", mrb_rrcad_spline_2d, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "spline_3d", mrb_rrcad_spline_3d, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "sweep", mrb_rrcad_shape_sweep, MRB_ARGS_REQ(1));

    /* Phase 3: Live preview */
    mrb_define_method(mrb, mrb->kernel_module, "preview", mrb_rrcad_preview, MRB_ARGS_REQ(1));

    /* Phase 3: Sub-shape selectors */
    mrb_define_method(mrb, shape_class, "faces", mrb_rrcad_shape_faces, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "edges", mrb_rrcad_shape_edges, MRB_ARGS_REQ(1));

    /* Phase 4: Import */
    mrb_define_method(mrb, mrb->kernel_module, "import_step", mrb_rrcad_import_step,
                      MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "import_stl", mrb_rrcad_import_stl,
                      MRB_ARGS_REQ(1));

    /* Phase 4: Query / introspection */
    mrb_define_method(mrb, shape_class, "bounding_box", mrb_rrcad_shape_bounding_box,
                      MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "volume", mrb_rrcad_shape_volume, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "surface_area", mrb_rrcad_shape_surface_area,
                      MRB_ARGS_NONE());
}
