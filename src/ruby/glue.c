/*
 * glue.c — thin C shims that hide mrb_value from Rust.
 *
 * All functions here deal with mrb_value internally and expose only
 * plain C types (char*, int, void*) across the FFI boundary, avoiding the
 * need for Rust to know anything about mRuby's value representation.
 */

#include <math.h>
#include <mruby.h>
#include <mruby/array.h>
#include <mruby/class.h>
#include <mruby/compile.h>
#include <mruby/data.h>
#include <mruby/error.h>
#include <mruby/hash.h>
#include <mruby/range.h>
#include <mruby/string.h>
#include <mruby/throw.h>
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
extern void rrcad_script_write(const char* text, const char** error_out);
extern int rrcad_require_begin(const char* arg, const char** code_out, const char** name_out,
                               const char** error_out);
extern void rrcad_require_end(void);
extern void* rrcad_import_step(const char* path, const char** error_out);
extern void* rrcad_import_stl(const char* path, const char** error_out);
extern void rrcad_shape_export_step(void* ptr, const char* path, const char** error_out);
extern void* rrcad_shape_fuse(void* a, void* b, const char** error_out);
extern void* rrcad_shape_cut(void* a, void* b, const char** error_out);
extern void* rrcad_shape_common(void* a, void* b, const char** error_out);

/* Phase 5 */
extern void* rrcad_shape_mate(void* ptr, void* from_ptr, void* to_ptr, double offset,
                              const char** error_out);
extern void* rrcad_shape_set_color(void* ptr, double r, double g, double b, const char** error_out);
extern void rrcad_shape_name_face(void* ptr, const char* name, const char* selector,
                                  const char** error_out);
extern void rrcad_shape_name_edge(void* ptr, const char* name, const char* selector,
                                  const char** error_out);
extern void rrcad_shape_datum(void* ptr, const char* name, void* datum_ptr, const char** error_out);
extern void* rrcad_shape_ref(void* ptr, const char* name, const char** error_out);
extern void rrcad_shape_gdt_apply(void* ptr, const char* standard, const char* datum_label,
                                  const char* datum_selector, int datum_anchor_valid,
                                  double datum_anchor_x, double datum_anchor_y,
                                  double datum_anchor_z, const char* feature_control_text,
                                  const char* feature_control_selector,
                                  int feature_control_anchor_valid, double feature_control_anchor_x,
                                  double feature_control_anchor_y, double feature_control_anchor_z,
                                  const char* feature_control_datums,
                                  const char* feature_control_modifiers, const char** error_out);

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
extern void* rrcad_shape_fillet_var(void* ptr, double r1, double r2, const char** error_out);
extern void* rrcad_shape_fillet_var_sel(void* ptr, double r1, double r2, const char* selector,
                                        const char** error_out);
extern void* rrcad_shape_mirror(void* ptr, const char* plane, const char** error_out);
extern void* rrcad_make_rect(double w, double h, const char** error_out);
extern void* rrcad_make_circle_face(double r, const char** error_out);
extern void* rrcad_make_polygon(const double* pts, size_t n_pts, const char** error_out);
extern void* rrcad_make_profile_2d(const double* pts, size_t n_pts, const int* counts,
                                   const int* kinds, size_t n_segments, const char** error_out);
extern void* rrcad_make_ellipse_face(double rx, double ry, const char** error_out);
extern void* rrcad_make_arc(double r, double start_deg, double end_deg, const char** error_out);
extern void* rrcad_shape_extrude(void* ptr, double height, const char** error_out);
extern void* rrcad_shape_revolve(void* ptr, double angle_deg, const char** error_out);
extern const char* rrcad_shape_feature_graph(void* ptr, const char** error_out);
extern void* rrcad_shape_rebuild(void* ptr, const char** error_out);

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
extern void* rrcad_shape_polar_pattern(void* ptr, int n, double angle_deg, const char** error_out);

/* Phase 4 — vertices selector */
extern int rrcad_shape_vertices_count(void* ptr, const char* selector, const char** error_out);
extern void* rrcad_shape_vertices_get(void* ptr, const char* selector, int idx,
                                      const char** error_out);

/* Phase 4 — additional exports (stl, gltf, glb, obj) */
/* The tessellating exporters take the mesh deflection, plus a flag saying
 * whether the script asked for one at all — see shape_export_meshed! in
 * native_io.rs for why "unset" is not folded into the value. */
extern void rrcad_shape_export_stl(void* ptr, const char* path, int has_deflection,
                                   double linear_deflection, const char** error_out);
extern void rrcad_shape_export_gltf(void* ptr, const char* path, int has_deflection,
                                    double linear_deflection, const char** error_out);
extern void rrcad_shape_export_glb(void* ptr, const char* path, int has_deflection,
                                   double linear_deflection, const char** error_out);
extern void rrcad_shape_export_obj(void* ptr, const char* path, int has_deflection,
                                   double linear_deflection, const char** error_out);
extern void rrcad_shape_export_3mf(void* ptr, const char* path, int has_deflection,
                                   double linear_deflection, const char** error_out);

/* Phase 8 Tier 4 — SVG / DXF 2-D drawing output */
extern void rrcad_shape_export_svg(
    void* ptr, const char* path, const char* view, double scale, int hidden, int center_marks,
    int dimensions, int title_block, int callouts, const char* datum, int datum_anchor_valid,
    double datum_anchor_x, double datum_anchor_y, double datum_anchor_z,
    const char* feature_control, int feature_control_anchor_valid, double feature_control_anchor_x,
    double feature_control_anchor_y, double feature_control_anchor_z, double tolerance_plus,
    double tolerance_minus, const char* section_plane, double section_offset, int detail_active,
    double detail_x, double detail_y, double detail_radius, double detail_scale,
    const char* detail_label, int ordinate, const char* bom_rows, const char* balloons,
    const char** error_out);
extern void rrcad_shape_export_dxf(
    void* ptr, const char* path, const char* view, double scale, int hidden, int center_marks,
    int dimensions, int title_block, int callouts, const char* datum, int datum_anchor_valid,
    double datum_anchor_x, double datum_anchor_y, double datum_anchor_z,
    const char* feature_control, int feature_control_anchor_valid, double feature_control_anchor_x,
    double feature_control_anchor_y, double feature_control_anchor_z, double tolerance_plus,
    double tolerance_minus, const char* section_plane, double section_offset, int detail_active,
    double detail_x, double detail_y, double detail_radius, double detail_scale,
    const char* detail_label, int ordinate, const char* bom_rows, const char* balloons,
    const char** error_out);
extern void rrcad_shape_export_outline(void* ptr, const char* path, const char* format,
                                       double deflection, const char** error_out);

/* Phase 7 — Bézier patch and sewing */
extern void* rrcad_make_bezier_patch(const double* pts, size_t n_pts, const char** error_out);
extern void* rrcad_sew(const void** ptrs, size_t n, double tolerance, const char** error_out);

/* Phase 7 Tier 1 — asymmetric chamfer, offset_2d, grid_pattern, fuse_all, cut_all */
extern void* rrcad_shape_chamfer_asym(void* ptr, double d1, double d2, const char** error_out);
extern void* rrcad_shape_chamfer_asym_sel(void* ptr, double d1, double d2, const char* selector,
                                          const char** error_out);
extern void* rrcad_shape_offset_2d(void* ptr, double distance, const char** error_out);
extern void* rrcad_shape_grid_pattern(void* ptr, int nx, int ny, double dx, double dy,
                                      const char** error_out);
extern void* rrcad_shape_fuse_all(const void** ptrs, size_t n, const char** error_out);
extern void* rrcad_shape_cut_all(void* base, const void** ptrs, size_t n, const char** error_out);

/* Phase 7 Tier 2 — validation & introspection */
extern const char* rrcad_shape_type_name(void* ptr, const char** error_out);
extern void rrcad_shape_centroid(void* ptr, double* out, const char** error_out);
extern void rrcad_shape_face_normal(void* ptr, double* out, const char** error_out);
extern void rrcad_shape_cylinder_axis(void* ptr, double* out, const char** error_out);
extern int rrcad_shape_is_closed(void* ptr, const char** error_out);
extern int rrcad_shape_is_manifold(void* ptr, const char** error_out);
extern const char* rrcad_shape_validate(void* ptr, const char** error_out);
extern const char* rrcad_shape_history(void* ptr, const char** error_out);

/* Phase 7 Tier 3 — surface modeling */
extern void* rrcad_ruled_surface(void* a, void* b, const char** error_out);
extern void* rrcad_fill_surface(void* ptr, const char** error_out);
extern void* rrcad_shape_slice(void* ptr, const char* plane, double offset, const char** error_out);

/* Phase 8 Tier 1 — Core Part Design */
extern void* rrcad_shape_pad(void* body, void* face, void* sketch, double height,
                             const char** error_out);
extern void* rrcad_shape_pocket(void* body, void* face, void* sketch, double depth,
                                const char** error_out);
extern void* rrcad_shape_fillet_wire(void* ptr, double radius, const char** error_out);
extern void* rrcad_datum_plane(double ox, double oy, double oz, double nx, double ny, double nz,
                               double xx, double xy, double xz, const char** error_out);

/* Phase 8 Tier 3 — Inspection & clearance */
extern double rrcad_shape_distance_to(void* a, void* b, const char** error_out);
extern void rrcad_shape_inertia(void* ptr, double* out, const char** error_out);
extern double rrcad_shape_min_thickness(void* ptr, const char** error_out);

/* Phase 8 Tier 2 — Manufacturing features */
extern void* rrcad_shape_extrude_draft(void* ptr, double height, double draft_deg,
                                       const char** error_out);
extern void* rrcad_make_helix(double radius, double pitch, double height, const char** error_out);

/* Phase 8 Tier 5 — Advanced composition */
extern void* rrcad_shape_fragment(const void** ptrs, size_t n, const char** error_out);
extern void* rrcad_shape_convex_hull(void* ptr, const char** error_out);
extern void* rrcad_shape_thicken(void* ptr, double thickness, const char** error_out);
extern void* rrcad_shape_path_pattern(void* ptr, void* path_ptr, int n, const char** error_out);
extern void* rrcad_shape_sweep_guide(void* ptr, void* path_ptr, void* guide_ptr,
                                     const char** error_out);

static const char* shape_name_from_value(mrb_state* mrb, mrb_value v);
static void* shape_ptr(mrb_state* mrb, mrb_value v);
static void require_native_ptr(mrb_state* mrb, void* ptr);
static void* checked_shape_ptr(mrb_state* mrb, mrb_value v);
static void raise_if_err(mrb_state* mrb, const char* err);
static double value_to_double(mrb_state* mrb, mrb_value v) {
    if (!mrb_obj_is_kind_of(mrb, v, mrb_class_get(mrb, "Numeric"))) {
        mrb_raisef(mrb, E_TYPE_ERROR, "%Y cannot be converted to Float", v);
    }
    return (double)mrb_as_float(mrb, mrb_funcall(mrb, v, "to_f", 0));
}

static int datum_anchor_from_shape_value(mrb_state* mrb, void* body_ptr, mrb_value value,
                                         double out[3], const char** error_out) {
    if (mrb_nil_p(value))
        return 0;

    if (mrb_data_p(value)) {
        void* datum_ptr = checked_shape_ptr(mrb, value);
        rrcad_shape_centroid(datum_ptr, out, error_out);
        return *error_out == NULL;
    }

    const char* selector = shape_name_from_value(mrb, value);
    if (!selector) {
        *error_out = "datum face expects a Shape, Symbol, or String selector";
        return 0;
    }

    int count = rrcad_shape_faces_count(body_ptr, selector, error_out);
    if (*error_out)
        return 0;
    if (count < 1) {
        *error_out = "datum face selector matched no faces";
        return 0;
    }

    void* face_ptr = rrcad_shape_faces_get(body_ptr, selector, 0, error_out);
    if (*error_out || !face_ptr)
        return 0;
    rrcad_shape_centroid(face_ptr, out, error_out);
    return *error_out == NULL;
}

static mrb_value append_selector_list(mrb_state* mrb, mrb_value text, mrb_value list) {
    if (mrb_nil_p(list))
        return text;

    if (mrb_array_p(list)) {
        mrb_int len = RARRAY_LEN(list);
        for (mrb_int i = 0; i < len; i++) {
            mrb_value item = mrb_ary_ref(mrb, list, i);
            const char* s = shape_name_from_value(mrb, item);
            if (!s)
                mrb_raise(mrb, E_TYPE_ERROR, "feature_control datums must be Symbols or Strings");
            if (RSTRING_LEN(text) > 0)
                mrb_str_cat_cstr(mrb, text, " | ");
            mrb_str_cat_cstr(mrb, text, s);
        }
        return text;
    }

    const char* s = shape_name_from_value(mrb, list);
    if (!s)
        mrb_raise(mrb, E_TYPE_ERROR,
                  "feature_control datums must be a Symbol, String, or Array thereof");
    if (RSTRING_LEN(text) > 0)
        mrb_str_cat_cstr(mrb, text, " | ");
    mrb_str_cat_cstr(mrb, text, s);
    return text;
}

static mrb_value append_joined_list(mrb_state* mrb, mrb_value text, mrb_value list,
                                    const char* what) {
    if (mrb_nil_p(list))
        return text;

    if (mrb_array_p(list)) {
        mrb_int len = RARRAY_LEN(list);
        for (mrb_int i = 0; i < len; i++) {
            mrb_value item = mrb_ary_ref(mrb, list, i);
            const char* s = shape_name_from_value(mrb, item);
            if (!s)
                mrb_raise(mrb, E_TYPE_ERROR, what);
            if (RSTRING_LEN(text) > 0)
                mrb_str_cat_cstr(mrb, text, ",");
            mrb_str_cat_cstr(mrb, text, s);
        }
        return text;
    }

    const char* s = shape_name_from_value(mrb, list);
    if (!s)
        mrb_raise(mrb, E_TYPE_ERROR, what);
    if (RSTRING_LEN(text) > 0)
        mrb_str_cat_cstr(mrb, text, ",");
    mrb_str_cat_cstr(mrb, text, s);
    return text;
}

/* mRuby data type descriptor — name appears in TypeError messages. */
static void shape_dfree(mrb_state* mrb, void* ptr) {
    (void)mrb;
    rrcad_shape_drop(ptr); /* no-op for NULL */
}

static const mrb_data_type shape_type = {"Shape", shape_dfree};

/* Wrap a raw Rust Box pointer in a new mRuby Shape RData value.
 * The Shape class is looked up per-call so multiple concurrent VMs
 * (e.g. parallel test threads) each see their own class pointer.
 *
 * Note: if mrb_class_get or mrb_data_object_alloc raises (e.g. OOM),
 * the Rust Box<Shape> behind `ptr` leaks — dfree never runs for it.
 * This is a memory leak only, not UB; accepted as-is. */
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

/* Raw pointer of the *receiver*, guaranteed to be backed by native geometry. */
static void* self_ptr(mrb_state* mrb, mrb_value self) {
    void* ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);
    return ptr;
}

/* Raw pointer of a Shape *argument*: type-checked (TypeError when `v` is not a
 * Shape) and guaranteed to be backed by native geometry. */
static void* checked_shape_ptr(mrb_state* mrb, mrb_value v) {
    void* ptr = shape_ptr(mrb, v);
    require_native_ptr(mrb, ptr);
    return ptr;
}

/* Turn a Rust-side error string into a Ruby RuntimeError.  Every native call
 * below funnels its `const char* err` through this; a NULL means success.
 * Note this never returns when `err` is set — mrb_raise longjmps. */
static void raise_if_err(mrb_state* mrb, const char* err) {
    if (err)
        mrb_raise(mrb, E_RUNTIME_ERROR, err);
}

/* hash[:key], or `dflt` when the key is absent. */
static mrb_value opt_fetch(mrb_state* mrb, mrb_value hash, const char* key, mrb_value dflt) {
    return mrb_hash_fetch(mrb, hash, mrb_symbol_value(mrb_intern_cstr(mrb, key)), dflt);
}

/* hash[:key] as a C boolean, defaulting to false. */
static int opt_flag(mrb_state* mrb, mrb_value hash, const char* key) {
    return mrb_test(opt_fetch(mrb, hash, key, mrb_false_value())) ? 1 : 0;
}

/* hash[:key] as a double, defaulting to `dflt`. */
static double opt_double(mrb_state* mrb, mrb_value hash, const char* key, double dflt) {
    return value_to_double(mrb, opt_fetch(mrb, hash, key, mrb_float_value(mrb, dflt)));
}

/* Build a 3-element Ruby Array from `v[0..3]`. */
static mrb_value float_ary3(mrb_state* mrb, const double* v) {
    mrb_value ary = mrb_ary_new_capa(mrb, 3);
    for (int i = 0; i < 3; i++)
        mrb_ary_push(mrb, ary, mrb_float_value(mrb, (mrb_float)v[i]));
    return ary;
}

/* Collect a Ruby Array of Shape objects into a malloc'd C array of raw
 * pointers, for the native entry points that take `(const void** ptrs, size_t n)`.
 *
 *   `min_count` / `min_err` — reject arrays that are too short.
 *   `what`                  — method name used in the per-element TypeError.
 *   `*n_out`                — receives the element count.
 *
 * Returns a buffer the caller must free().  `mrb_data_check_get_ptr` is
 * non-raising (NULL for a wrong type or a stub shape), so the buffer is always
 * freed before raising and can never leak. */
static const void** collect_shape_ptrs(mrb_state* mrb, mrb_value arr, int min_count,
                                       const char* min_err, const char* what, int* n_out) {
    int n = (int)RARRAY_LEN(arr);
    if (n < min_count)
        mrb_raise(mrb, E_ARGUMENT_ERROR, min_err);

    const void** ptrs = (const void**)malloc((size_t)n * sizeof(void*));
    if (!ptrs)
        mrb_raise(mrb, E_RUNTIME_ERROR, "out of memory");

    for (int i = 0; i < n; i++) {
        mrb_value elem = mrb_ary_ref(mrb, arr, (mrb_int)i);
        void* p = mrb_data_check_get_ptr(mrb, elem, &shape_type);
        if (!p) {
            free(ptrs);
            mrb_raisef(mrb, E_TYPE_ERROR, "%s: element %d is not a valid Shape", what, i);
        }
        ptrs[i] = p;
    }

    *n_out = n;
    return ptrs;
}

/* Split a NUL-terminated, "\n"-separated report into an Array of Strings.
 * An empty report yields an empty Array. */
static mrb_value split_lines(mrb_state* mrb, const char* report) {
    mrb_value ary = mrb_ary_new(mrb);
    const char* p = report;
    while (*p) {
        const char* nl = strchr(p, '\n');
        size_t len = nl ? (size_t)(nl - p) : strlen(p);
        mrb_ary_push(mrb, ary, mrb_str_new(mrb, p, (mrb_int)len));
        p += len;
        if (*p == '\n')
            p++;
    }
    return ary;
}

/* -------------------------------------------------------------------------
 * Top-level primitive constructors (defined on Kernel)
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_box(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value dx_val, dy_val, dz_val;
    mrb_get_args(mrb, "ooo", &dx_val, &dy_val, &dz_val);
    double dx = value_to_double(mrb, dx_val);
    double dy = value_to_double(mrb, dy_val);
    double dz = value_to_double(mrb, dz_val);

    const char* err = NULL;
    void* ptr = rrcad_make_box(dx, dy, dz, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_cylinder(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value r_val, h_val;
    mrb_get_args(mrb, "oo", &r_val, &h_val);
    double r = value_to_double(mrb, r_val);
    double h = value_to_double(mrb, h_val);

    const char* err = NULL;
    void* ptr = rrcad_make_cylinder(r, h, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_sphere(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value r_val;
    mrb_get_args(mrb, "o", &r_val);
    double r = value_to_double(mrb, r_val);

    const char* err = NULL;
    void* ptr = rrcad_make_sphere(r, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_cone(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value r1_val, r2_val, h_val;
    mrb_get_args(mrb, "ooo", &r1_val, &r2_val, &h_val);
    double r1 = value_to_double(mrb, r1_val);
    double r2 = value_to_double(mrb, r2_val);
    double h = value_to_double(mrb, h_val);

    const char* err = NULL;
    void* ptr = rrcad_make_cone(r1, r2, h, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_torus(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value r1_val, r2_val;
    mrb_get_args(mrb, "oo", &r1_val, &r2_val);
    double r1 = value_to_double(mrb, r1_val);
    double r2 = value_to_double(mrb, r2_val);

    const char* err = NULL;
    void* ptr = rrcad_make_torus(r1, r2, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_wedge(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value dx_val, dy_val, dz_val, ltx_val;
    mrb_get_args(mrb, "oooo", &dx_val, &dy_val, &dz_val, &ltx_val);
    double dx = value_to_double(mrb, dx_val);
    double dy = value_to_double(mrb, dy_val);
    double dz = value_to_double(mrb, dz_val);
    double ltx = value_to_double(mrb, ltx_val);

    const char* err = NULL;
    void* ptr = rrcad_make_wedge(dx, dy, dz, ltx, &err);
    raise_if_err(mrb, err);
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

/* .export("path" [, view: :top|:front|:side, scale: 1.0, hidden: false, center_marks: false,
 * dimensions: false, title_block: false, callouts: false, datum: nil, feature_control: nil,
 * tolerance: 0.0]) — dispatches by file extension: .step / .stp  → STEP AP203 .stl          → ASCII
 * STL .glb          → binary glTF (GLB) .gltf         → text glTF .obj          → Wavefront OBJ
 *   .3mf          → 3MF (units, colour, one object per solid)
 *   .svg          → SVG 2-D drawing (HLR projection)
 *   .dxf          → DXF R12 2-D drawing (HLR projection)
 * Defaults to STEP for any unrecognised extension.
 *
 * The optional `view:` keyword (Symbol) is used only by SVG and DXF:
 *   :top   (default) — looking down −Z axis
 *   :front           — looking along −Y axis
 *   :side            — looking along +X axis
 *
 * The optional `section:` keyword (SVG / DXF only) turns the drawing into a
 * section view.  Accepts a plane name (:xy, :xz, :yz) or a Hash
 * { plane: :xz, offset: 5.0 }; the offset (default 0.0) is measured along the
 * plane's normal.  Material in front of the plane is removed and the exposed
 * cut faces are drawn with 45° hatching. */
static mrb_value mrb_rrcad_shape_export(mrb_state* mrb, mrb_value self) {
    const char* path;
    mrb_value opts = mrb_nil_value();
    mrb_get_args(mrb, "z|H", &path, &opts);

    void* ptr = self_ptr(mrb, self);

    /* Extract optional view: keyword (default "top") and drawing scale. */
    const char* view = "top";
    mrb_value view_buf = mrb_nil_value();
    double scale = 1.0;
    int hidden = 0;
    int center_marks = 0;
    int dimensions = 0;
    int title_block = 0;
    int callouts = 0;
    const char* datum = "";
    mrb_value datum_buf = mrb_nil_value();
    int datum_anchor_valid = 0;
    double datum_anchor[3] = {0.0, 0.0, 0.0};
    const char* feature_control = "";
    mrb_value feature_control_buf = mrb_nil_value();
    int feature_control_anchor_valid = 0;
    double feature_control_anchor[3] = {0.0, 0.0, 0.0};
    double tolerance_plus = 0.0;
    double tolerance_minus = 0.0;
    /* Section view: an empty plane name means "no section" (plain projection). */
    const char* section_plane = "";
    mrb_value section_plane_buf = mrb_nil_value();
    /* NAN = "cut through the middle" (the exporter resolves it against the
     * shape's bounding box); an explicit offset: overrides it. */
    double section_offset = (double)NAN;
    /* Detail view: a magnified close-up of one circular region of the drawing.
     * Inactive unless the caller passed detail:. */
    int detail_active = 0;
    double detail_x = 0.0;
    double detail_y = 0.0;
    double detail_radius = 0.0;
    double detail_scale = 2.0;
    const char* detail_label = "A";
    mrb_value detail_label_buf = mrb_nil_value();
    /* Ordinate dimensions for the part's located features. */
    int ordinate = 0;
    /* Parts list and balloon callouts.  These carry per-component data, so the
     * assembly layer builds them; a bare Shape has nothing to put in them.
     * Both are delimited records — see bridge.h for the format. */
    const char* bom_rows = "";
    mrb_value bom_rows_buf = mrb_nil_value();
    const char* balloons = "";
    mrb_value balloons_buf = mrb_nil_value();
    /* Mesh tessellation quality, used only by the mesh formats. Absent means
     * "use the default", which is not the same as any value the caller could
     * pass, so it travels as its own flag. */
    int has_deflection = 0;
    double linear_deflection = 0.0;
    if (!mrb_nil_p(opts) && mrb_hash_p(opts)) {
        mrb_value vv = opt_fetch(mrb, opts, "view", mrb_nil_value());
        if (mrb_symbol_p(vv)) {
            /* mruby packs short symbols into a single shared scratch buffer, so
             * mrb_sym_name's result must be copied before any later symbol is
             * decoded (e.g. the section plane below) overwrites it. */
            view_buf = mrb_str_new_cstr(mrb, mrb_sym_name(mrb, mrb_symbol(vv)));
            view = mrb_string_value_cstr(mrb, &view_buf);
        }
        scale = opt_double(mrb, opts, "scale", 1.0);
        /* `deflection:` is accepted alongside `linear_deflection:` because
         * export_outline already spells it the short way, and an unrecognised
         * key here would be silently ignored — which is the failure this
         * option is being fixed from. */
        mrb_value defl_v = opt_fetch(mrb, opts, "linear_deflection", mrb_nil_value());
        if (mrb_nil_p(defl_v))
            defl_v = opt_fetch(mrb, opts, "deflection", mrb_nil_value());
        if (!mrb_nil_p(defl_v)) {
            linear_deflection = value_to_double(mrb, defl_v);
            has_deflection = 1;
        }
        hidden = opt_flag(mrb, opts, "hidden");
        center_marks = opt_flag(mrb, opts, "center_marks");
        dimensions = opt_flag(mrb, opts, "dimensions");
        title_block = opt_flag(mrb, opts, "title_block");
        callouts = opt_flag(mrb, opts, "callouts");
        ordinate = opt_flag(mrb, opts, "ordinate");
        mrb_value bom_v = opt_fetch(mrb, opts, "bom_rows", mrb_nil_value());
        if (!mrb_nil_p(bom_v)) {
            if (!mrb_string_p(bom_v))
                mrb_raise(mrb, E_TYPE_ERROR, "bom_rows expects a String of delimited records");
            bom_rows_buf = bom_v;
            bom_rows = mrb_string_value_cstr(mrb, &bom_rows_buf);
        }
        mrb_value balloons_v = opt_fetch(mrb, opts, "balloons", mrb_nil_value());
        if (!mrb_nil_p(balloons_v)) {
            if (!mrb_string_p(balloons_v))
                mrb_raise(mrb, E_TYPE_ERROR, "balloons expects a String of delimited records");
            balloons_buf = balloons_v;
            balloons = mrb_string_value_cstr(mrb, &balloons_buf);
        }
        mrb_value datum_v = opt_fetch(mrb, opts, "datum", mrb_nil_value());
        if (!mrb_nil_p(datum_v)) {
            if (mrb_hash_p(datum_v)) {
                mrb_value label_v = opt_fetch(mrb, datum_v, "label", mrb_nil_value());
                if (mrb_nil_p(label_v)) {
                    label_v = opt_fetch(mrb, datum_v, "name", mrb_nil_value());
                }
                mrb_value face_v = opt_fetch(mrb, datum_v, "face", mrb_nil_value());
                if (mrb_nil_p(face_v)) {
                    face_v = opt_fetch(mrb, datum_v, "selector", mrb_nil_value());
                }
                const char* label = shape_name_from_value(mrb, label_v);
                if (!label)
                    mrb_raise(mrb, E_TYPE_ERROR,
                              "datum hash expects label:/name: as a Symbol or String");
                datum_buf = mrb_str_new_cstr(mrb, label);
                datum = mrb_string_value_cstr(mrb, &datum_buf);
                if (!mrb_nil_p(face_v)) {
                    const char* anchor_err = NULL;
                    if (!datum_anchor_from_shape_value(mrb, ptr, face_v, datum_anchor,
                                                       &anchor_err)) {
                        mrb_raise(mrb, E_RUNTIME_ERROR,
                                  anchor_err ? anchor_err : "failed to resolve datum face");
                    }
                    datum_anchor_valid = 1;
                }
            } else {
                const char* label = shape_name_from_value(mrb, datum_v);
                if (!label)
                    mrb_raise(mrb, E_TYPE_ERROR,
                              "datum expects a Symbol, String, or Hash with label:/face:");
                datum_buf = mrb_str_new_cstr(mrb, label);
                datum = mrb_string_value_cstr(mrb, &datum_buf);
            }
        }
        mrb_value fc_v = opt_fetch(mrb, opts, "feature_control", mrb_nil_value());
        if (!mrb_nil_p(fc_v)) {
            if (mrb_hash_p(fc_v)) {
                mrb_value text_v = opt_fetch(mrb, fc_v, "text", mrb_nil_value());
                if (mrb_nil_p(text_v)) {
                    text_v = opt_fetch(mrb, fc_v, "frame", mrb_nil_value());
                }
                if (mrb_nil_p(text_v)) {
                    text_v = opt_fetch(mrb, fc_v, "value", mrb_nil_value());
                }
                const char* text = shape_name_from_value(mrb, text_v);
                if (!text)
                    mrb_raise(
                        mrb, E_TYPE_ERROR,
                        "feature_control hash expects text:/frame:/value: as a Symbol or String");
                feature_control_buf = mrb_str_new_cstr(mrb, text);
                mrb_value fc_text = feature_control_buf;
                mrb_value datums_v = opt_fetch(mrb, fc_v, "datums", mrb_nil_value());
                if (mrb_nil_p(datums_v)) {
                    datums_v = opt_fetch(mrb, fc_v, "datum", mrb_nil_value());
                }
                fc_text = append_selector_list(mrb, fc_text, datums_v);
                feature_control = mrb_string_value_cstr(mrb, &fc_text);

                mrb_value face_v = opt_fetch(mrb, fc_v, "face", mrb_nil_value());
                if (mrb_nil_p(face_v)) {
                    face_v = opt_fetch(mrb, fc_v, "selector", mrb_nil_value());
                }
                if (!mrb_nil_p(face_v)) {
                    const char* anchor_err = NULL;
                    if (!datum_anchor_from_shape_value(mrb, ptr, face_v, feature_control_anchor,
                                                       &anchor_err)) {
                        mrb_raise(mrb, E_RUNTIME_ERROR,
                                  anchor_err ? anchor_err
                                             : "failed to resolve feature_control face");
                    }
                    feature_control_anchor_valid = 1;
                }
            } else if (mrb_symbol_p(fc_v)) {
                /* Copied for the same reason as `view` above. */
                feature_control_buf = mrb_str_new_cstr(mrb, mrb_sym_name(mrb, mrb_symbol(fc_v)));
                feature_control = mrb_string_value_cstr(mrb, &feature_control_buf);
            } else {
                feature_control = mrb_string_value_cstr(mrb, &fc_v);
            }
        }
        mrb_value tv2 = opt_fetch(mrb, opts, "tolerance", mrb_float_value(mrb, 0.0));
        if (mrb_hash_p(tv2)) {
            tolerance_plus = opt_double(mrb, tv2, "plus", 0.0);
            tolerance_minus = opt_double(mrb, tv2, "minus", 0.0);
        } else if (!mrb_nil_p(tv2)) {
            double tol = value_to_double(mrb, tv2);
            tolerance_plus = tol;
            tolerance_minus = tol;
        }
        /* section: :xz                        — cut on the XZ plane at y = 0
         * section: { plane: :xz, offset: 5 }  — cut on the XZ plane at y = 5
         * Only .svg and .dxf use this; other formats ignore it. */
        mrb_value section_v = opt_fetch(mrb, opts, "section", mrb_nil_value());
        if (!mrb_nil_p(section_v)) {
            mrb_value plane_v = section_v;
            if (mrb_hash_p(section_v)) {
                plane_v = opt_fetch(mrb, section_v, "plane", mrb_nil_value());
                /* No explicit offset: NAN asks the exporter for the shape's
                 * mid-plane rather than the origin (see bridge.cpp). */
                section_offset = opt_double(mrb, section_v, "offset", (double)NAN);
            }
            const char* plane = shape_name_from_value(mrb, plane_v);
            if (!plane)
                mrb_raise(mrb, E_TYPE_ERROR,
                          "section expects :xy/:xz/:yz as a Symbol or String, or a Hash with "
                          "plane:/offset:");
            section_plane_buf = mrb_str_new_cstr(mrb, plane);
            section_plane = mrb_string_value_cstr(mrb, &section_plane_buf);
        }
        /* detail: { at: [x, y], radius: 6, scale: 4, label: "A" }
         * `at:` is stated on the view's own drawing plane — top uses [X, Y],
         * front [X, Z], side [Y, Z] — so it reads in model units regardless of
         * the drawing scale.  Only .svg and .dxf use this. */
        mrb_value detail_v = opt_fetch(mrb, opts, "detail", mrb_nil_value());
        if (!mrb_nil_p(detail_v)) {
            if (!mrb_hash_p(detail_v))
                mrb_raise(mrb, E_TYPE_ERROR,
                          "detail expects a Hash with at:/radius: (and optional scale:/label:)");
            mrb_value at_v = opt_fetch(mrb, detail_v, "at", mrb_nil_value());
            if (!mrb_array_p(at_v) || RARRAY_LEN(at_v) != 2)
                mrb_raise(mrb, E_ARGUMENT_ERROR,
                          "detail at: expects a 2-element [x, y] point on the view's plane");
            detail_x = value_to_double(mrb, mrb_ary_ref(mrb, at_v, 0));
            detail_y = value_to_double(mrb, mrb_ary_ref(mrb, at_v, 1));
            mrb_value radius_v = opt_fetch(mrb, detail_v, "radius", mrb_nil_value());
            if (mrb_nil_p(radius_v))
                radius_v = opt_fetch(mrb, detail_v, "r", mrb_nil_value());
            if (mrb_nil_p(radius_v))
                mrb_raise(mrb, E_ARGUMENT_ERROR, "detail needs a radius: (or r:)");
            detail_radius = value_to_double(mrb, radius_v);
            detail_scale = opt_double(mrb, detail_v, "scale", 2.0);
            mrb_value dl_v = opt_fetch(mrb, detail_v, "label", mrb_nil_value());
            if (!mrb_nil_p(dl_v)) {
                const char* label = shape_name_from_value(mrb, dl_v);
                if (!label)
                    mrb_raise(mrb, E_TYPE_ERROR, "detail label: expects a Symbol or String");
                /* Copied for the same reason as `view` above. */
                detail_label_buf = mrb_str_new_cstr(mrb, label);
                detail_label = mrb_string_value_cstr(mrb, &detail_label_buf);
            }
            detail_active = 1;
        }
    }

    /* Find the last '.' to determine the extension. */
    const char* dot = strrchr(path, '.');
    const char* err = NULL;

    if (dot && (strcasecmp(dot, ".stl") == 0)) {
        rrcad_shape_export_stl(ptr, path, has_deflection, linear_deflection, &err);
    } else if (dot && (strcasecmp(dot, ".glb") == 0)) {
        rrcad_shape_export_glb(ptr, path, has_deflection, linear_deflection, &err);
    } else if (dot && (strcasecmp(dot, ".gltf") == 0)) {
        rrcad_shape_export_gltf(ptr, path, has_deflection, linear_deflection, &err);
    } else if (dot && (strcasecmp(dot, ".obj") == 0)) {
        rrcad_shape_export_obj(ptr, path, has_deflection, linear_deflection, &err);
    } else if (dot && (strcasecmp(dot, ".3mf") == 0)) {
        rrcad_shape_export_3mf(ptr, path, has_deflection, linear_deflection, &err);
    } else if (dot && (strcasecmp(dot, ".svg") == 0)) {
        rrcad_shape_export_svg(
            ptr, path, view, scale, hidden, center_marks, dimensions, title_block, callouts, datum,
            datum_anchor_valid, datum_anchor[0], datum_anchor[1], datum_anchor[2], feature_control,
            feature_control_anchor_valid, feature_control_anchor[0], feature_control_anchor[1],
            feature_control_anchor[2], tolerance_plus, tolerance_minus, section_plane,
            section_offset, detail_active, detail_x, detail_y, detail_radius, detail_scale,
            detail_label, ordinate, bom_rows, balloons, &err);
    } else if (dot && (strcasecmp(dot, ".dxf") == 0)) {
        rrcad_shape_export_dxf(
            ptr, path, view, scale, hidden, center_marks, dimensions, title_block, callouts, datum,
            datum_anchor_valid, datum_anchor[0], datum_anchor[1], datum_anchor[2], feature_control,
            feature_control_anchor_valid, feature_control_anchor[0], feature_control_anchor[1],
            feature_control_anchor[2], tolerance_plus, tolerance_minus, section_plane,
            section_offset, detail_active, detail_x, detail_y, detail_radius, detail_scale,
            detail_label, ordinate, bom_rows, balloons, &err);
    } else {
        /* Default: STEP (.step, .stp, or unknown extension) */
        rrcad_shape_export_step(ptr, path, &err);
    }
    raise_if_err(mrb, err);
    return self;
}

static mrb_value mrb_rrcad_shape_fuse(mrb_state* mrb, mrb_value self) {
    mrb_value other;
    mrb_get_args(mrb, "o", &other);

    void* a = self_ptr(mrb, self);
    void* b = checked_shape_ptr(mrb, other);

    const char* err = NULL;
    void* result = rrcad_shape_fuse(a, b, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_cut(mrb_state* mrb, mrb_value self) {
    mrb_value other;
    mrb_get_args(mrb, "o", &other);

    void* a = self_ptr(mrb, self);
    void* b = checked_shape_ptr(mrb, other);

    const char* err = NULL;
    void* result = rrcad_shape_cut(a, b, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_common(mrb_state* mrb, mrb_value self) {
    mrb_value other;
    mrb_get_args(mrb, "o", &other);

    void* a = self_ptr(mrb, self);
    void* b = checked_shape_ptr(mrb, other);

    const char* err = NULL;
    void* result = rrcad_shape_common(a, b, &err);
    raise_if_err(mrb, err);
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
    mrb_value offset_val = mrb_float_value(mrb, 0.0);
    mrb_get_args(mrb, "oo|o", &from_val, &to_val, &offset_val);
    double offset = value_to_double(mrb, offset_val);

    void* ptr = self_ptr(mrb, self);

    void* from_ptr = mrb_data_p(from_val) ? DATA_PTR(from_val) : NULL;
    void* to_ptr = mrb_data_p(to_val) ? DATA_PTR(to_val) : NULL;
    if (!from_ptr || !to_ptr)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "mate: from and to must be Shape objects");

    const char* err = NULL;
    void* result = rrcad_shape_mate(ptr, from_ptr, to_ptr, offset, &err);
    raise_if_err(mrb, err);
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
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_set_color(ptr, (double)r, (double)g, (double)b, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

static const char* shape_name_from_value(mrb_state* mrb, mrb_value v) {
    if (mrb_symbol_p(v))
        return mrb_sym_name(mrb, mrb_symbol(v));
    if (mrb_string_p(v))
        return mrb_str_to_cstr(mrb, v);
    return NULL;
}

static mrb_value mrb_rrcad_shape_name_face(mrb_state* mrb, mrb_value self) {
    mrb_value name_val, selector_val;
    mrb_get_args(mrb, "oo", &name_val, &selector_val);

    const char* name = shape_name_from_value(mrb, name_val);
    const char* selector = shape_name_from_value(mrb, selector_val);
    if (!name || !selector)
        mrb_raise(mrb, E_TYPE_ERROR,
                  "name_face expects a Symbol or String name and a Symbol or String selector");

    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    rrcad_shape_name_face(ptr, name, selector, &err);
    raise_if_err(mrb, err);
    return self;
}

static mrb_value mrb_rrcad_shape_name_edge(mrb_state* mrb, mrb_value self) {
    mrb_value name_val, selector_val;
    mrb_get_args(mrb, "oo", &name_val, &selector_val);

    const char* name = shape_name_from_value(mrb, name_val);
    const char* selector = shape_name_from_value(mrb, selector_val);
    if (!name || !selector)
        mrb_raise(mrb, E_TYPE_ERROR,
                  "name_edge expects a Symbol or String name and a Symbol or String selector");

    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    rrcad_shape_name_edge(ptr, name, selector, &err);
    raise_if_err(mrb, err);
    return self;
}

static mrb_value mrb_rrcad_shape_datum(mrb_state* mrb, mrb_value self) {
    mrb_value name_val, datum_val;
    mrb_get_args(mrb, "oo", &name_val, &datum_val);

    const char* name = shape_name_from_value(mrb, name_val);
    if (!name)
        mrb_raise(mrb, E_TYPE_ERROR, "datum expects a Symbol or String name");

    void* ptr = self_ptr(mrb, self);
    void* datum_ptr = checked_shape_ptr(mrb, datum_val);

    const char* err = NULL;
    rrcad_shape_datum(ptr, name, datum_ptr, &err);
    raise_if_err(mrb, err);
    return self;
}

static mrb_value mrb_rrcad_shape_ref(mrb_state* mrb, mrb_value self) {
    mrb_value name_val;
    mrb_get_args(mrb, "o", &name_val);

    const char* name = shape_name_from_value(mrb, name_val);
    if (!name)
        mrb_raise(mrb, E_TYPE_ERROR, "ref expects a Symbol or String name");

    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_ref(ptr, name, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* --- gdt_apply argument-parsing helpers ---------------------------------- */

/* Fetch hash[key] (symbol key), returning nil when absent. */
static mrb_value gdt_fetch(mrb_state* mrb, mrb_value hash, const char* key) {
    return opt_fetch(mrb, hash, key, mrb_nil_value());
}

/* Fetch hash[key], falling back to hash[alt] when key is absent.
 * `alt` may be NULL for a single-key lookup. */
static mrb_value gdt_fetch2(mrb_state* mrb, mrb_value hash, const char* key, const char* alt) {
    mrb_value v = gdt_fetch(mrb, hash, key);
    if (mrb_nil_p(v) && alt)
        v = gdt_fetch(mrb, hash, alt);
    return v;
}

/* Convert a required Symbol/String value into a C string backed by *buf
 * (kept alive by the caller's stack slot). Raises TypeError with `type_err`
 * if the value is nil or not name-like. */
static const char* gdt_required_name(mrb_state* mrb, mrb_value v, mrb_value* buf,
                                     const char* type_err) {
    const char* s = shape_name_from_value(mrb, v);
    if (!s)
        mrb_raise(mrb, E_TYPE_ERROR, type_err);
    *buf = mrb_str_new_cstr(mrb, s);
    return mrb_string_value_cstr(mrb, buf);
}

/* Resolve the selector:/face: entry of a GD&T datum / feature-control hash.
 * A Symbol/String selector is copied into *selector_buf / *selector_out; in
 * every non-nil case the anchor point is resolved via
 * datum_anchor_from_shape_value. Returns 1 when an anchor was resolved,
 * 0 when the hash has no selector. Raises on type or resolution errors. */
static int gdt_resolve_selector(mrb_state* mrb, void* ptr, mrb_value hash,
                                const char* selector_type_err, const char* resolve_err,
                                const char** selector_out, mrb_value* selector_buf,
                                double anchor[3]) {
    mrb_value selector_v = gdt_fetch2(mrb, hash, "selector", "face");
    if (mrb_nil_p(selector_v))
        return 0;
    if (!mrb_data_p(selector_v)) {
        /* Named selector: keep the name for the Rust side as well. */
        *selector_out = gdt_required_name(mrb, selector_v, selector_buf, selector_type_err);
    }
    const char* anchor_err = NULL;
    if (!datum_anchor_from_shape_value(mrb, ptr, selector_v, anchor, &anchor_err))
        mrb_raise(mrb, E_RUNTIME_ERROR, anchor_err ? anchor_err : resolve_err);
    return 1;
}

static mrb_value mrb_rrcad_shape_gdt_apply(mrb_state* mrb, mrb_value self) {
    mrb_value spec_v;
    mrb_get_args(mrb, "H", &spec_v);

    void* ptr = self_ptr(mrb, self);

    mrb_value standard_v = gdt_fetch(mrb, spec_v, "standard");
    if (mrb_nil_p(standard_v))
        standard_v = mrb_symbol_value(mrb_intern_lit(mrb, "asme"));
    const char* standard = shape_name_from_value(mrb, standard_v);
    if (!standard)
        mrb_raise(mrb, E_TYPE_ERROR, "gdt_apply standard must be a Symbol or String");

    const char* datum_label = "";
    mrb_value datum_label_buf = mrb_nil_value();
    const char* datum_selector = "";
    mrb_value datum_selector_buf = mrb_nil_value();
    int datum_anchor_valid = 0;
    double datum_anchor[3] = {0.0, 0.0, 0.0};

    mrb_value datum_v = gdt_fetch(mrb, spec_v, "datum");
    if (!mrb_nil_p(datum_v)) {
        if (!mrb_hash_p(datum_v)) {
            mrb_raise(mrb, E_TYPE_ERROR,
                      "gdt_apply datum expects a hash with label:/face:/selector:");
        }
        datum_label =
            gdt_required_name(mrb, gdt_fetch2(mrb, datum_v, "label", "name"), &datum_label_buf,
                              "gdt_apply datum hash expects label:/name: as a Symbol or String");
        datum_anchor_valid = gdt_resolve_selector(
            mrb, ptr, datum_v, "gdt_apply datum selector expects a Symbol, String, or Shape",
            "failed to resolve gdt datum face", &datum_selector, &datum_selector_buf, datum_anchor);
    }

    const char* feature_control_text = "";
    mrb_value feature_control_text_buf = mrb_nil_value();
    const char* feature_control_selector = "";
    mrb_value feature_control_selector_buf = mrb_nil_value();
    int feature_control_anchor_valid = 0;
    double feature_control_anchor[3] = {0.0, 0.0, 0.0};
    mrb_value feature_control_datums_buf = mrb_str_new_cstr(mrb, "");
    mrb_value feature_control_modifiers_buf = mrb_str_new_cstr(mrb, "");

    mrb_value fc_v = gdt_fetch(mrb, spec_v, "feature_control");
    if (!mrb_nil_p(fc_v)) {
        if (!mrb_hash_p(fc_v)) {
            mrb_raise(mrb, E_TYPE_ERROR,
                      "gdt_apply feature_control expects a hash with text:/frame:/value:");
        }
        mrb_value text_v = gdt_fetch2(mrb, fc_v, "text", "frame");
        if (mrb_nil_p(text_v))
            text_v = gdt_fetch(mrb, fc_v, "value");
        feature_control_text = gdt_required_name(
            mrb, text_v, &feature_control_text_buf,
            "gdt_apply feature_control hash expects text:/frame:/value: as a Symbol or String");
        feature_control_anchor_valid = gdt_resolve_selector(
            mrb, ptr, fc_v, "gdt_apply feature_control selector expects a Symbol, String, or Shape",
            "failed to resolve gdt feature_control face", &feature_control_selector,
            &feature_control_selector_buf, feature_control_anchor);

        mrb_value datums_v = gdt_fetch2(mrb, fc_v, "datums", "datum");
        if (!mrb_nil_p(datums_v))
            feature_control_datums_buf =
                append_joined_list(mrb, feature_control_datums_buf, datums_v,
                                   "gdt_apply feature_control datums must be Symbols or Strings");

        mrb_value modifiers_v = gdt_fetch(mrb, fc_v, "modifiers");
        if (!mrb_nil_p(modifiers_v))
            feature_control_modifiers_buf = append_joined_list(
                mrb, feature_control_modifiers_buf, modifiers_v,
                "gdt_apply feature_control modifiers must be Symbols or Strings");
    }

    const char* feature_control_datums =
        RSTRING_LEN(feature_control_datums_buf) > 0
            ? mrb_string_value_cstr(mrb, &feature_control_datums_buf)
            : "";
    const char* feature_control_modifiers =
        RSTRING_LEN(feature_control_modifiers_buf) > 0
            ? mrb_string_value_cstr(mrb, &feature_control_modifiers_buf)
            : "";
    const char* err = NULL;
    rrcad_shape_gdt_apply(
        ptr, standard, datum_label, datum_selector, datum_anchor_valid, datum_anchor[0],
        datum_anchor[1], datum_anchor[2], feature_control_text, feature_control_selector,
        feature_control_anchor_valid, feature_control_anchor[0], feature_control_anchor[1],
        feature_control_anchor[2], feature_control_datums, feature_control_modifiers, &err);
    raise_if_err(mrb, err);
    return self;
}

/* -------------------------------------------------------------------------
 * Phase 2: Transform methods
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_shape_translate(mrb_state* mrb, mrb_value self) {
    mrb_value dx_val, dy_val, dz_val;
    mrb_get_args(mrb, "ooo", &dx_val, &dy_val, &dz_val);
    double dx = value_to_double(mrb, dx_val);
    double dy = value_to_double(mrb, dy_val);
    double dz = value_to_double(mrb, dz_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_translate(ptr, dx, dy, dz, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_rotate(mrb_state* mrb, mrb_value self) {
    mrb_value ax_val, ay_val, az_val, angle_val;
    mrb_get_args(mrb, "oooo", &ax_val, &ay_val, &az_val, &angle_val);
    double ax = value_to_double(mrb, ax_val);
    double ay = value_to_double(mrb, ay_val);
    double az = value_to_double(mrb, az_val);
    double angle = value_to_double(mrb, angle_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_rotate(ptr, ax, ay, az, angle, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* scale(factor)       — uniform scale
 * scale(sx, sy, sz)  — non-uniform scale (independent axis factors) */
static mrb_value mrb_rrcad_shape_scale(mrb_state* mrb, mrb_value self) {
    mrb_value sx_val, sy_val = mrb_nil_value(), sz_val = mrb_nil_value();
    mrb_int argc = mrb_get_args(mrb, "o|oo", &sx_val, &sy_val, &sz_val);
    double sx = value_to_double(mrb, sx_val);
    double sy = 0.0, sz = 0.0;
    if (argc == 3) {
        sy = value_to_double(mrb, sy_val);
        sz = value_to_double(mrb, sz_val);
    }
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result;
    if (argc == 1) {
        /* Uniform: use the exact gp_Trsf path (no approximation). */
        result = rrcad_shape_scale(ptr, sx, &err);
    } else if (argc == 3) {
        result = rrcad_shape_scale_xyz(ptr, sx, sy, sz, &err);
    } else {
        mrb_raise(mrb, E_ARGUMENT_ERROR, "scale requires 1 argument (uniform) or 3 (sx, sy, sz)");
        return mrb_nil_value(); /* unreachable */
    }
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* fillet(r)             — constant radius, all edges
 * fillet(r, :selector)  — constant radius, matching edges
 * fillet(r1..r2)        — variable radius (r1→r2 along each edge), all edges
 * fillet(r1..r2, :sel)  — variable radius, matching edges
 *
 * A Ruby Range (r1..r2) selects BRepFilletAPI_MakeFillet::Add(r1, r2, edge),
 * which transitions the fillet radius smoothly along each edge. */
static mrb_value mrb_rrcad_shape_fillet(mrb_state* mrb, mrb_value self) {
    mrb_value radius_val;
    mrb_value sel_val = mrb_nil_value();
    mrb_get_args(mrb, "o|o", &radius_val, &sel_val);

    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result;

    if (mrb_range_p(radius_val)) {
        /* Variable-radius form: fillet(r1..r2) or fillet(r1..r2, :selector) */
        mrb_value beg = mrb_range_beg(mrb, radius_val);
        mrb_value end = mrb_range_end(mrb, radius_val);
        double r1 = value_to_double(mrb, beg);
        double r2 = value_to_double(mrb, end);

        if (mrb_nil_p(sel_val)) {
            result = rrcad_shape_fillet_var(ptr, r1, r2, &err);
        } else {
            if (!mrb_symbol_p(sel_val))
                mrb_raise(mrb, E_TYPE_ERROR,
                          "fillet selector must be a Symbol (:all, :vertical, :horizontal)");
            const char* sel = mrb_sym_name(mrb, mrb_symbol(sel_val));
            result = rrcad_shape_fillet_var_sel(ptr, r1, r2, sel, &err);
        }
    } else {
        /* Constant-radius form: fillet(r) or fillet(r, :selector) */
        double r = value_to_double(mrb, radius_val);
        if (mrb_nil_p(sel_val)) {
            result = rrcad_shape_fillet(ptr, r, &err);
        } else {
            if (!mrb_symbol_p(sel_val))
                mrb_raise(mrb, E_TYPE_ERROR,
                          "fillet selector must be a Symbol (:all, :vertical, :horizontal)");
            const char* sel = mrb_sym_name(mrb, mrb_symbol(sel_val));
            result = rrcad_shape_fillet_sel(ptr, r, sel, &err);
        }
    }

    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* chamfer(d)            — bevel all edges
 * chamfer(d, :selector) — bevel only edges matching selector */
static mrb_value mrb_rrcad_shape_chamfer(mrb_state* mrb, mrb_value self) {
    mrb_value d_val;
    mrb_value sel_val = mrb_nil_value();
    mrb_get_args(mrb, "o|o", &d_val, &sel_val);
    double d = value_to_double(mrb, d_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result;
    if (mrb_nil_p(sel_val)) {
        result = rrcad_shape_chamfer(ptr, d, &err);
    } else {
        if (!mrb_symbol_p(sel_val))
            mrb_raise(mrb, E_TYPE_ERROR,
                      "chamfer selector must be a Symbol (:all, :vertical, :horizontal)");
        const char* sel = mrb_sym_name(mrb, mrb_symbol(sel_val));
        result = rrcad_shape_chamfer_sel(ptr, (double)d, sel, &err);
    }
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* chamfer(d1, d2 [, :selector]) — asymmetric chamfer.
 * Two distance arguments distinguish this from the symmetric form.
 * An optional Symbol selector (:all / :vertical / :horizontal) restricts edges. */
static mrb_value mrb_rrcad_shape_chamfer_asym(mrb_state* mrb, mrb_value self) {
    mrb_value d1_val, d2_val;
    mrb_value sel_val = mrb_nil_value();
    mrb_get_args(mrb, "oo|o", &d1_val, &d2_val, &sel_val);
    double d1 = value_to_double(mrb, d1_val);
    double d2 = value_to_double(mrb, d2_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result;
    if (mrb_nil_p(sel_val)) {
        result = rrcad_shape_chamfer_asym(ptr, d1, d2, &err);
    } else {
        if (!mrb_symbol_p(sel_val))
            mrb_raise(mrb, E_TYPE_ERROR,
                      "chamfer selector must be a Symbol (:all, :vertical, :horizontal)");
        const char* sel = mrb_sym_name(mrb, mrb_symbol(sel_val));
        result = rrcad_shape_chamfer_asym_sel(ptr, d1, d2, sel, &err);
    }
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_mirror(mrb_state* mrb, mrb_value self) {
    mrb_sym plane_sym;
    mrb_get_args(mrb, "n", &plane_sym);
    const char* plane = mrb_sym_name(mrb, plane_sym);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_mirror(ptr, plane, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 2: Sketch constructors (top-level)
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_rect(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value w_val, h_val;
    mrb_get_args(mrb, "oo", &w_val, &h_val);
    double w = value_to_double(mrb, w_val);
    double h = value_to_double(mrb, h_val);
    const char* err = NULL;
    void* ptr = rrcad_make_rect(w, h, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_circle(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value r_val;
    mrb_get_args(mrb, "o", &r_val);
    double r = value_to_double(mrb, r_val);
    const char* err = NULL;
    void* ptr = rrcad_make_circle_face(r, &err);
    raise_if_err(mrb, err);
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
    mrb_value height_val;
    mrb_value opts = mrb_nil_value();
    mrb_get_args(mrb, "o|H", &height_val, &opts);
    double height = value_to_double(mrb, height_val);

    double twist_deg = 0.0;
    double scale = 1.0;
    double draft_deg = 0.0;

    if (!mrb_nil_p(opts) && mrb_hash_p(opts)) {
        twist_deg = opt_double(mrb, opts, "twist_deg", 0.0);
        scale = opt_double(mrb, opts, "scale", 1.0);
        draft_deg = opt_double(mrb, opts, "draft", 0.0);
    }

    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;

    /* draft: keyword routes to BRepOffsetAPI_DraftAngle path. */
    if (draft_deg != 0.0) {
        void* result = rrcad_shape_extrude_draft(ptr, height, draft_deg, &err);
        raise_if_err(mrb, err);
        return shape_from_ptr(mrb, result);
    }

    /* rrcad_shape_extrude_ex falls back to MakePrism when twist≈0 and scale≈1 */
    void* result = rrcad_shape_extrude_ex(ptr, height, twist_deg, scale, &err);
    raise_if_err(mrb, err);
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
        ruled = opt_flag(mrb, opts, "ruled");
    }

    int n = 0;
    const void** ptrs =
        collect_shape_ptrs(mrb, arr, 2, "loft requires at least 2 profiles", "loft", &n);

    const char* err = NULL;
    void* result = rrcad_shape_loft(ptrs, (size_t)n, ruled, &err);
    free(ptrs);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* .shell(thickness) — hollow out the solid, removing the top face. */
static mrb_value mrb_rrcad_shape_shell(mrb_state* mrb, mrb_value self) {
    mrb_value thickness_val;
    mrb_get_args(mrb, "o", &thickness_val);
    double thickness = value_to_double(mrb, thickness_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_shell(ptr, thickness, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* .offset(distance) — inflate (>0) or deflate (<0) the solid uniformly. */
static mrb_value mrb_rrcad_shape_offset(mrb_state* mrb, mrb_value self) {
    mrb_value distance_val;
    mrb_get_args(mrb, "o", &distance_val);
    double distance = value_to_double(mrb, distance_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_offset(ptr, distance, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* .offset_2d(distance) — offset a Wire or Face inward (<0) or outward (>0) in its own plane. */
static mrb_value mrb_rrcad_shape_offset_2d(mrb_state* mrb, mrb_value self) {
    mrb_value distance_val;
    mrb_get_args(mrb, "o", &distance_val);
    double distance = value_to_double(mrb, distance_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_offset_2d(ptr, distance, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* .simplify(min_feature_size) — remove small holes/fillets via defeaturing.
   Faces with area < min_feature_size² are passed to BRepAlgoAPI_Defeaturing. */
static mrb_value mrb_rrcad_shape_simplify(mrb_state* mrb, mrb_value self) {
    mrb_value min_size_val;
    mrb_get_args(mrb, "o", &min_size_val);
    double min_size = value_to_double(mrb, min_size_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_simplify(ptr, min_size, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_revolve(mrb_state* mrb, mrb_value self) {
    mrb_value angle_val = mrb_float_value(mrb, 360.0);
    mrb_get_args(mrb, "|o", &angle_val);
    double angle = value_to_double(mrb, angle_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_revolve(ptr, angle, &err);
    raise_if_err(mrb, err);
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
            pts[i * stride + j] = value_to_double(mrb, v);
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
    raise_if_err(mrb, err);
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
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

/* shape.sweep(path)
 * shape.sweep(path, guide: guide_wire)
 *
 * With no guide: keyword, calls the standard BRepOffsetAPI_MakePipe sweep.
 * With guide: keyword, uses BRepOffsetAPI_MakePipeShell in auxiliary-spine mode
 * so the profile orientation tracks the guide wire at every point along path.
 */
static mrb_value mrb_rrcad_shape_sweep(mrb_state* mrb, mrb_value self) {
    mrb_value path_val;
    mrb_value opts = mrb_nil_value();
    mrb_get_args(mrb, "o|H", &path_val, &opts);

    void* profile = self_ptr(mrb, self);
    void* path = checked_shape_ptr(mrb, path_val);

    /* Check for optional guide: keyword. */
    mrb_value guide_val = mrb_nil_value();
    if (!mrb_nil_p(opts) && mrb_hash_p(opts)) {
        mrb_sym guide_key = mrb_intern_lit(mrb, "guide");
        guide_val = mrb_hash_fetch(mrb, opts, mrb_symbol_value(guide_key), mrb_nil_value());
    }

    const char* err = NULL;
    void* result;
    if (!mrb_nil_p(guide_val)) {
        /* Guided sweep: auxiliary spine mode. */
        void* guide = checked_shape_ptr(mrb, guide_val);
        result = rrcad_shape_sweep_guide(profile, path, guide, &err);
    } else {
        /* Plain sweep. */
        result = rrcad_shape_sweep(profile, path, &err);
    }
    raise_if_err(mrb, err);
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

    void* path = checked_shape_ptr(mrb, path_val);

    int n = 0;
    const void** ptrs = collect_shape_ptrs(
        mrb, arr, 2, "sweep_sections requires at least 2 profiles", "sweep_sections", &n);

    const char* err = NULL;
    void* result = rrcad_shape_sweep_sections(ptrs, (size_t)n, path, &err);
    free(ptrs);
    raise_if_err(mrb, err);
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
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

/* __rrcad_profile_2d(points, counts, kinds)
 *
 * Internal primitive behind sketches that mix straight and spline segments.
 * `points` is an Array of [x, y] pairs, `counts` how many of them each segment
 * owns, and `kinds` 0 for a straight run or 1 for an interpolated spline. The
 * sketcher flattens its segment list into this form; scripts use `sketch`. */
static mrb_value mrb_rrcad_profile_2d(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value pts_arr, counts_arr, kinds_arr;
    mrb_get_args(mrb, "AAA", &pts_arr, &counts_arr, &kinds_arr);

    int n_segments = (int)RARRAY_LEN(counts_arr);
    if (n_segments != (int)RARRAY_LEN(kinds_arr))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "profile_2d: counts and kinds must have equal length");
    if (n_segments < 1)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "profile_2d: need at least one segment");

    int n = 0;
    double* pts = extract_point_array(mrb, pts_arr, 2, &n);

    int* counts = (int*)malloc((size_t)n_segments * sizeof(int));
    int* kinds = (int*)malloc((size_t)n_segments * sizeof(int));
    if (!counts || !kinds) {
        free(pts);
        free(counts);
        free(kinds);
        mrb_raise(mrb, E_RUNTIME_ERROR, "out of memory");
    }

    for (int i = 0; i < n_segments; i++) {
        counts[i] = (int)mrb_as_int(mrb, mrb_ary_ref(mrb, counts_arr, i));
        kinds[i] = (int)mrb_as_int(mrb, mrb_ary_ref(mrb, kinds_arr, i));
    }

    const char* err = NULL;
    void* ptr = rrcad_make_profile_2d(pts, (size_t)n, counts, kinds, (size_t)n_segments, &err);
    free(pts);
    free(counts);
    free(kinds);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_ellipse(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value rx_val, ry_val;
    mrb_get_args(mrb, "oo", &rx_val, &ry_val);
    double rx = value_to_double(mrb, rx_val);
    double ry = value_to_double(mrb, ry_val);
    const char* err = NULL;
    void* ptr = rrcad_make_ellipse_face(rx, ry, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_arc(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value r_val, start_val, end_val;
    mrb_get_args(mrb, "ooo", &r_val, &start_val, &end_val);
    double r = value_to_double(mrb, r_val);
    double start_deg = value_to_double(mrb, start_val);
    double end_deg = value_to_double(mrb, end_val);
    const char* err = NULL;
    void* ptr = rrcad_make_arc(r, start_deg, end_deg, &err);
    raise_if_err(mrb, err);
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
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_import_stl(mrb_state* mrb, mrb_value self) {
    (void)self;
    const char* path;
    mrb_get_args(mrb, "z", &path);
    const char* err = NULL;
    void* ptr = rrcad_import_stl(path, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

/* -------------------------------------------------------------------------
 * Phase 3: Live preview — preview(shape)
 *
 * Tessellates the shape to a temp GLB file and signals WebSocket clients
 * to reload.  A no-op when not in --preview mode.
 * -------------------------------------------------------------------------
 */

/* Write an already-formatted string to the script output sink.
 *
 * The prelude's puts / print / p do all formatting in Ruby and pass the exact
 * bytes here.  mruby strings may contain embedded NULs, so the length is
 * checked against strlen and a NUL-bearing string is rejected rather than
 * silently truncated at the C boundary. */
static mrb_value mrb_rrcad_write(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value text;
    mrb_get_args(mrb, "S", &text);

    const char* bytes = RSTRING_PTR(text);
    const mrb_int len = RSTRING_LEN(text);
    if (len > 0 && (mrb_int)strlen(bytes) != len)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "output text must not contain a NUL byte");

    const char* err = NULL;
    rrcad_script_write(mrb_string_value_cstr(mrb, &text), &err);
    raise_if_err(mrb, err);
    return mrb_nil_value();
}

/* require_relative(path) — evaluate another script file once.
 *
 * The Rust side resolves the path and reads the source; evaluation has to
 * happen here because it needs the mrb_state. rrcad_require_begin pushes the
 * include stack and rrcad_require_end pops it, so the two must be paired even
 * when the required file raises — otherwise a later require would resolve
 * against a stale directory. mrb_load_string_cxt carries the real filename so
 * a syntax error names the file it is in rather than "(eval)".
 *
 * Returns true when the file was evaluated, false when it had already been
 * loaded (matching Ruby's require semantics, and what makes a require cycle
 * terminate rather than recurse).
 */
static mrb_value mrb_rrcad_require_relative(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value path;
    mrb_get_args(mrb, "S", &path);

    const char* bytes = RSTRING_PTR(path);
    const mrb_int len = RSTRING_LEN(path);
    if (len > 0 && (mrb_int)strlen(bytes) != len)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "require_relative path must not contain a NUL byte");

    const char* code = NULL;
    const char* filename = NULL;
    const char* err = NULL;
    const int status =
        rrcad_require_begin(mrb_string_value_cstr(mrb, &path), &code, &filename, &err);
    if (status < 0) {
        raise_if_err(mrb, err);
        /* Defensive: a negative status always sets err, but never fall through
         * into evaluation with a NULL code pointer. */
        mrb_raise(mrb, E_RUNTIME_ERROR, "require_relative failed");
    }
    if (status == 0)
        return mrb_false_value();

    mrbc_context* cxt = mrbc_context_new(mrb);
    if (cxt == NULL) {
        rrcad_require_end();
        mrb_raise(mrb, E_RUNTIME_ERROR, "require_relative: out of memory");
    }
    mrbc_filename(mrb, cxt, filename);

    /* mRuby propagates exceptions with longjmp, so a `raise` inside the
     * required file would jump straight past the cleanup below and leave the
     * include stack pushed — after which the *next* require would resolve
     * against the failed file's directory instead of this one's.  Catching it
     * here with our own jmpbuf is what makes the stack unwind with the C
     * stack.  (Only visible when the two directories differ, which is why the
     * regression test puts the raising file in a subdirectory.) */
    struct mrb_jmpbuf* prev_jmp = mrb->jmp;
    struct mrb_jmpbuf c_jmp;
    mrb_value raised = mrb_nil_value();
    mrb_bool did_raise = FALSE;

    MRB_TRY(&c_jmp) {
        mrb->jmp = &c_jmp;
        mrb_load_string_cxt(mrb, code, cxt);
        mrb->jmp = prev_jmp;
        /* A parse error does not longjmp; it returns with mrb->exc set. */
        if (mrb->exc) {
            raised = mrb_obj_value(mrb->exc);
            mrb->exc = NULL;
            did_raise = TRUE;
        }
    }
    MRB_CATCH(&c_jmp) {
        mrb->jmp = prev_jmp;
        raised = mrb_obj_value(mrb->exc);
        mrb->exc = NULL;
        did_raise = TRUE;
    }
    MRB_END_EXC(&c_jmp);

    mrbc_context_free(mrb, cxt);
    rrcad_require_end();

    if (did_raise)
        mrb_exc_raise(mrb, raised);
    return mrb_true_value();
}

/* face.export_outline(path, format, deflection) — flat cut-file export.
 *
 * Distinct from export("x.dxf"), which draws an HLR projection of the whole
 * shape.  This writes only the closed loops of one planar face at 1:1.
 * The prelude picks the format from the file extension and supplies the
 * default deflection, so this primitive takes both explicitly.
 */
static mrb_value mrb_rrcad_shape_export_outline(mrb_state* mrb, mrb_value self) {
    const char* path;
    const char* format;
    mrb_float deflection;
    mrb_get_args(mrb, "zzf", &path, &format, &deflection);

    void* ptr = shape_ptr(mrb, self);
    const char* err = NULL;
    rrcad_shape_export_outline(ptr, path, format, (double)deflection, &err);
    raise_if_err(mrb, err);
    return self;
}

static mrb_value mrb_rrcad_preview(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value shape_val;
    mrb_get_args(mrb, "o", &shape_val);

    void* ptr = shape_ptr(mrb, shape_val);
    if (!ptr)
        return mrb_nil_value(); /* stub shape (no OCCT backing) — silent no-op */

    const char* err = NULL;
    rrcad_preview_shape(ptr, &err);
    raise_if_err(mrb, err);
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

    void* ptr = self_ptr(mrb, self);

    const char* err = NULL;
    int count = rrcad_shape_faces_count(ptr, sel, &err);
    raise_if_err(mrb, err);

    mrb_value result = mrb_ary_new_capa(mrb, count);
    for (int i = 0; i < count; i++) {
        err = NULL;
        void* face_ptr = rrcad_shape_faces_get(ptr, sel, i, &err);
        raise_if_err(mrb, err);
        mrb_ary_push(mrb, result, shape_from_ptr(mrb, face_ptr));
    }
    return result;
}

static mrb_value mrb_rrcad_shape_edges(mrb_state* mrb, mrb_value self) {
    mrb_sym sel_sym;
    mrb_get_args(mrb, "n", &sel_sym);
    const char* sel = mrb_sym_name(mrb, sel_sym);

    void* ptr = self_ptr(mrb, self);

    const char* err = NULL;
    int count = rrcad_shape_edges_count(ptr, sel, &err);
    raise_if_err(mrb, err);

    mrb_value result = mrb_ary_new_capa(mrb, count);
    for (int i = 0; i < count; i++) {
        err = NULL;
        void* edge_ptr = rrcad_shape_edges_get(ptr, sel, i, &err);
        raise_if_err(mrb, err);
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

    void* ptr = self_ptr(mrb, self);

    const char* err = NULL;
    int count = rrcad_shape_vertices_count(ptr, sel, &err);
    raise_if_err(mrb, err);

    mrb_value result = mrb_ary_new_capa(mrb, count);
    for (int i = 0; i < count; i++) {
        err = NULL;
        void* vertex_ptr = rrcad_shape_vertices_get(ptr, sel, i, &err);
        raise_if_err(mrb, err);
        mrb_ary_push(mrb, result, shape_from_ptr(mrb, vertex_ptr));
    }
    return result;
}

/* -------------------------------------------------------------------------
 * Phase 4: Query / introspection
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_shape_bounding_box(mrb_state* mrb, mrb_value self) {
    void* ptr = self_ptr(mrb, self);

    double bounds[6];
    const char* err = NULL;
    rrcad_shape_bounding_box(ptr, bounds, &err);
    raise_if_err(mrb, err);

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
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    double vol = rrcad_shape_volume(ptr, &err);
    raise_if_err(mrb, err);
    return mrb_float_value(mrb, vol);
}

static mrb_value mrb_rrcad_shape_surface_area(mrb_state* mrb, mrb_value self) {
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    double area = rrcad_shape_surface_area(ptr, &err);
    raise_if_err(mrb, err);
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

    void* ptr = checked_shape_ptr(mrb, shape_val);

    if ((int)RARRAY_LEN(vec_val) < 3)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "linear_pattern: vector must have 3 elements [dx,dy,dz]");

    double dx = 0.0, dy = 0.0, dz = 0.0;
    for (int j = 0; j < 3; j++) {
        mrb_value v = mrb_ary_ref(mrb, vec_val, j);
        double d = value_to_double(mrb, v);
        if (j == 0)
            dx = d;
        else if (j == 1)
            dy = d;
        else
            dz = d;
    }

    const char* err = NULL;
    void* result = rrcad_shape_linear_pattern(ptr, (int)n, dx, dy, dz, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* polar_pattern(shape, n, angle_deg) */
static mrb_value mrb_rrcad_polar_pattern(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value shape_val;
    mrb_int n;
    mrb_value angle_val;
    mrb_get_args(mrb, "oio", &shape_val, &n, &angle_val);
    double angle_deg = value_to_double(mrb, angle_val);

    void* ptr = checked_shape_ptr(mrb, shape_val);

    const char* err = NULL;
    void* result = rrcad_shape_polar_pattern(ptr, (int)n, angle_deg, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 7 Tier 1: grid_pattern / fuse_all / cut_all
 * -------------------------------------------------------------------------
 */

/* grid_pattern(shape, nx, ny, dx, dy)
 *
 * Creates nx * ny translated copies of `shape` arranged in a 2-D grid.
 * Copy (i, j) is at position (i*dx, j*dy, 0).
 * Implemented as two nested linear_pattern calls in Rust — no new C++.
 *
 *   bolts = grid_pattern(bolt, 4, 3, 20, 20)
 */
static mrb_value mrb_rrcad_grid_pattern(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value shape_val;
    mrb_int nx, ny;
    mrb_value dx_val, dy_val;
    mrb_get_args(mrb, "oiioo", &shape_val, &nx, &ny, &dx_val, &dy_val);
    double dx = value_to_double(mrb, dx_val);
    double dy = value_to_double(mrb, dy_val);

    void* ptr = checked_shape_ptr(mrb, shape_val);

    const char* err = NULL;
    void* result = rrcad_shape_grid_pattern(ptr, (int)nx, (int)ny, dx, dy, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* fuse_all([shape1, shape2, ...])
 *
 * Fold-left union of all shapes in the array.  Requires at least 2 shapes.
 *
 *   merged = fuse_all([a, b, c, d])
 */
static mrb_value mrb_rrcad_fuse_all(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value arr;
    mrb_get_args(mrb, "A", &arr);

    int n = 0;
    const void** ptrs =
        collect_shape_ptrs(mrb, arr, 2, "fuse_all requires at least 2 shapes", "fuse_all", &n);

    const char* err = NULL;
    void* result = rrcad_shape_fuse_all(ptrs, (size_t)n, &err);
    free(ptrs);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* cut_all(base, [tool1, tool2, ...])
 *
 * Subtract each tool shape from `base` in sequence.  Requires at least 1 tool.
 *
 *   result = cut_all(block, [hole_a, hole_b, slot])
 */
static mrb_value mrb_rrcad_cut_all(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value base_val, arr;
    mrb_get_args(mrb, "oA", &base_val, &arr);

    void* base = checked_shape_ptr(mrb, base_val);

    int n = 0;
    const void** ptrs =
        collect_shape_ptrs(mrb, arr, 1, "cut_all requires at least 1 tool", "cut_all", &n);

    const char* err = NULL;
    void* result = rrcad_shape_cut_all(base, ptrs, (size_t)n, &err);
    free(ptrs);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 7: bezier_patch / sew
 * -------------------------------------------------------------------------
 */

/* bezier_patch(pts)
 *
 * Kernel-level method.  `pts` is an Array of exactly 16 control points, each
 * a [x, y, z] sub-array.  Returns a Shape wrapping a Bézier face.
 *
 * Example (one quadrant of the teapot rim):
 *   face = bezier_patch([
 *     [1.4, 0.0, 2.25], [1.3375, 0.0, 2.38125], [1.4375, 0.0, 2.38125], [1.5, 0.0, 2.25],
 *     ...
 *   ])
 */
static mrb_value mrb_rrcad_bezier_patch(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value arr;
    mrb_get_args(mrb, "A", &arr);

    if ((int)RARRAY_LEN(arr) != 16)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "bezier_patch requires exactly 16 control points");

    /* Extract 16 × [x,y,z] into a flat 48-double buffer. */
    double pts[48];
    for (int i = 0; i < 16; i++) {
        mrb_value pt = mrb_ary_ref(mrb, arr, i);
        if (!mrb_array_p(pt) || (int)RARRAY_LEN(pt) < 3)
            mrb_raise(mrb, E_ARGUMENT_ERROR, "each bezier_patch control point must be [x, y, z]");
        for (int j = 0; j < 3; j++) {
            mrb_value v = mrb_ary_ref(mrb, pt, j);
            pts[i * 3 + j] = value_to_double(mrb, v);
        }
    }

    const char* err = NULL;
    void* ptr = rrcad_make_bezier_patch(pts, 48, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, ptr);
}

/* sew(faces, tolerance: 1e-4)
 *
 * Kernel-level method.  `faces` is an Array of Shape objects (Faces or Shells)
 * to be sewn into a closed Shell / Solid.  The optional `tolerance:` keyword
 * controls how close shared edges must be to be sewn together.
 *
 * Example:
 *   teapot = sew(patches, tolerance: 1e-3).scale(3.0)
 */
static mrb_value mrb_rrcad_sew(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value arr;
    mrb_float tol = 1e-4;
    mrb_value opts = mrb_nil_value();
    mrb_get_args(mrb, "A|H", &arr, &opts);

    if (!mrb_nil_p(opts) && mrb_hash_p(opts)) {
        tol = opt_double(mrb, opts, "tolerance", 1e-4);
    }

    int n = 0;
    const void** ptrs = collect_shape_ptrs(mrb, arr, 1, "sew requires at least 1 shape", "sew", &n);

    const char* err = NULL;
    void* result = rrcad_sew(ptrs, (size_t)n, (double)tol, &err);
    free(ptrs);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* -------------------------------------------------------------------------
 * Phase 7 Tier 2: shape_type / centroid / closed? / manifold? / validate
 * -------------------------------------------------------------------------
 */

/* shape.shape_type  → :solid, :shell, :face, :wire, :edge, :vertex, ... */
static mrb_value mrb_rrcad_shape_type(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    const char* err = NULL;
    const char* type_str = rrcad_shape_type_name(ptr, &err);
    raise_if_err(mrb, err);
    return mrb_symbol_value(mrb_intern_cstr(mrb, type_str));
}

/* shape.centroid  → [x, y, z] */
static mrb_value mrb_rrcad_shape_centroid(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    double out[3] = {0.0, 0.0, 0.0};
    const char* err = NULL;
    rrcad_shape_centroid(ptr, out, &err);
    raise_if_err(mrb, err);

    return float_ary3(mrb, out);
}

/* face.normal  → [nx, ny, nz] outward unit normal of a planar face */
static mrb_value mrb_rrcad_shape_face_normal(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    double out[3] = {0.0, 0.0, 0.0};
    const char* err = NULL;
    rrcad_shape_face_normal(ptr, out, &err);
    raise_if_err(mrb, err);

    return float_ary3(mrb, out);
}

/* face.cylinder_axis  → {origin: [x,y,z], axis: [ax,ay,az], radius: r}
 * Errors when the shape is not a cylindrical face. */
static mrb_value mrb_rrcad_shape_cylinder_axis(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    double out[7] = {0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0};
    const char* err = NULL;
    rrcad_shape_cylinder_axis(ptr, out, &err);
    raise_if_err(mrb, err);

    mrb_value origin = float_ary3(mrb, &out[0]);
    mrb_value axis = float_ary3(mrb, &out[3]);

    mrb_value hash = mrb_hash_new(mrb);
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "origin")), origin);
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "axis")), axis);
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "radius")),
                 mrb_float_value(mrb, (mrb_float)out[6]));
    return hash;
}

/* shape.closed?  → true/false */
static mrb_value mrb_rrcad_shape_closed(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    const char* err = NULL;
    int result = rrcad_shape_is_closed(ptr, &err);
    raise_if_err(mrb, err);
    return mrb_bool_value(result != 0);
}

/* shape.manifold?  → true/false */
static mrb_value mrb_rrcad_shape_manifold(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    const char* err = NULL;
    int result = rrcad_shape_is_manifold(ptr, &err);
    raise_if_err(mrb, err);
    return mrb_bool_value(result != 0);
}

/* shape.validate  → :ok  or  ["error1", "error2", ...] */
static mrb_value mrb_rrcad_shape_validate(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    const char* err = NULL;
    const char* report = rrcad_shape_validate(ptr, &err);
    raise_if_err(mrb, err);

    /* "ok" → :ok; anything else → split on "\n" and return Array of strings */
    if (strcmp(report, "ok") == 0)
        return mrb_symbol_value(mrb_intern_lit(mrb, "ok"));

    return split_lines(mrb, report);
}

/* shape.history  → ["op(...)", "op(...)", ...] */
static mrb_value mrb_rrcad_shape_history(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    const char* err = NULL;
    const char* report = rrcad_shape_history(ptr, &err);
    raise_if_err(mrb, err);

    return split_lines(mrb, report);
}

static mrb_value mrb_rrcad_shape_feature_graph_line(mrb_state* mrb, const char* line, size_t len) {
    const char* tab1 = memchr(line, '\t', len);
    if (!tab1)
        return mrb_nil_value();
    const char* rest1 = tab1 + 1;
    size_t len1 = (size_t)(len - (rest1 - line));
    const char* tab2 = memchr(rest1, '\t', len1);
    if (!tab2)
        return mrb_nil_value();
    const char* rest2 = tab2 + 1;
    size_t len2 = (size_t)(len - (rest2 - line));
    const char* tab3 = memchr(rest2, '\t', len2);
    if (!tab3)
        return mrb_nil_value();
    const char* rest3 = tab3 + 1;

    mrb_value parents = mrb_ary_new(mrb);
    if (tab2 > rest1) {
        const char* p = rest1;
        while (p < tab2) {
            const char* comma = memchr(p, ',', (size_t)(tab2 - p));
            const char* end = comma ? comma : tab2;
            if (end > p) {
                char buf[32];
                size_t n = (size_t)(end - p);
                if (n >= sizeof(buf))
                    n = sizeof(buf) - 1;
                memcpy(buf, p, n);
                buf[n] = '\0';
                mrb_ary_push(mrb, parents, mrb_fixnum_value((mrb_int)strtoll(buf, NULL, 10)));
            }
            if (!comma)
                break;
            p = comma + 1;
        }
    }

    char id_buf[32];
    size_t id_len = (size_t)(tab1 - line);
    if (id_len >= sizeof(id_buf))
        id_len = sizeof(id_buf) - 1;
    memcpy(id_buf, line, id_len);
    id_buf[id_len] = '\0';

    mrb_value hash = mrb_hash_new(mrb);
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "id")),
                 mrb_fixnum_value((mrb_int)strtoll(id_buf, NULL, 10)));
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "parents")), parents);
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "label")),
                 mrb_str_new(mrb, rest2, (mrb_int)(tab3 - rest2)));
    mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_lit(mrb, "entry")),
                 mrb_str_new(mrb, rest3, (mrb_int)(len - (size_t)(rest3 - line))));
    return hash;
}

/* shape.feature_graph  → [{id:, parents:, label:, entry:}, ...] */
static mrb_value mrb_rrcad_shape_feature_graph(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    const char* err = NULL;
    const char* report = rrcad_shape_feature_graph(ptr, &err);
    raise_if_err(mrb, err);

    mrb_value ary = mrb_ary_new(mrb);
    if (report[0] == '\0')
        return ary;

    const char* p = report;
    while (*p) {
        const char* nl = strchr(p, '\n');
        size_t len = nl ? (size_t)(nl - p) : strlen(p);
        mrb_value entry = mrb_rrcad_shape_feature_graph_line(mrb, p, len);
        if (!mrb_nil_p(entry))
            mrb_ary_push(mrb, ary, entry);
        p += len;
        if (*p == '\n')
            p++;
    }
    return ary;
}

static mrb_value mrb_rrcad_shape_rebuild(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    const char* err = NULL;
    void* rebuilt = rrcad_shape_rebuild(ptr, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, rebuilt);
}

/* -------------------------------------------------------------------------
 * Phase 7 Tier 3: ruled_surface / fill_surface / slice
 * -------------------------------------------------------------------------
 */

/* ruled_surface(wire_a, wire_b) — Kernel-level method.
 * Creates a ruled surface (TopoDS_Shell) connecting wire_a to wire_b.
 * Both arguments must be Wire shapes produced by spline_2d/spline_3d/arc/etc.
 */
static mrb_value mrb_rrcad_ruled_surface(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value a, b;
    mrb_get_args(mrb, "oo", &a, &b);

    void* pa = checked_shape_ptr(mrb, a);
    void* pb = checked_shape_ptr(mrb, b);

    const char* err = NULL;
    void* result = rrcad_ruled_surface(pa, pb, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* fill_surface(boundary_wire) — Kernel-level method.
 * Fills the enclosed area of a closed Wire with a smooth NURBS surface.
 * The wire's edges are used as C0 boundary constraints for BRepFill_Filling.
 */
static mrb_value mrb_rrcad_fill_surface(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value v;
    mrb_get_args(mrb, "o", &v);

    void* ptr = checked_shape_ptr(mrb, v);

    const char* err = NULL;
    void* result = rrcad_fill_surface(ptr, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* shape.slice(plane: :xy, z: 5.0)  — instance method.
 *
 * Slices self with an axis-aligned plane and returns the cross-section as a
 * compound of edges/wires.  The plane is specified with keyword arguments:
 *
 *   solid.slice(plane: :xy, z: 5.0)   # XY plane at z=5
 *   solid.slice(plane: :xz, y: 2.0)   # XZ plane at y=2
 *   solid.slice(plane: :yz, x: 1.0)   # YZ plane at x=1
 */
static mrb_value mrb_rrcad_shape_slice(mrb_state* mrb, mrb_value self) {
    void* ptr = checked_shape_ptr(mrb, self);

    mrb_value kwargs = mrb_nil_value();
    mrb_get_args(mrb, "H", &kwargs);

    /* Require plane: keyword */
    mrb_value plane_sym = opt_fetch(mrb, kwargs, "plane", mrb_nil_value());
    if (mrb_nil_p(plane_sym))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "slice requires plane: :xy, :xz, or :yz");

    if (!mrb_symbol_p(plane_sym))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "slice: plane must be a Symbol (:xy, :xz, or :yz)");

    const char* pname = mrb_sym_name(mrb, mrb_symbol(plane_sym));

    /* Determine the offset keyword from the plane name */
    mrb_sym offset_key;
    if (strcmp(pname, "xy") == 0)
        offset_key = mrb_intern_lit(mrb, "z");
    else if (strcmp(pname, "xz") == 0)
        offset_key = mrb_intern_lit(mrb, "y");
    else if (strcmp(pname, "yz") == 0)
        offset_key = mrb_intern_lit(mrb, "x");
    else
        mrb_raise(mrb, E_ARGUMENT_ERROR, "slice: plane must be :xy, :xz, or :yz");

    mrb_value offset_val =
        mrb_hash_fetch(mrb, kwargs, mrb_symbol_value(offset_key), mrb_float_value(mrb, 0.0));
    double offset = value_to_double(mrb, offset_val);

    const char* err = NULL;
    void* result = rrcad_shape_slice(ptr, pname, offset, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* =========================================================================
 * Phase 8 Tier 1: Core Part Design
 * =========================================================================
 */

/* Helper: resolve a face selector (Symbol → first matching face, Shape → itself).
 * Returns a raw Shape pointer (NOT a newly-allocated Shape — may be borrowed
 * from `body_ptr`).  Raises a Ruby exception on failure. */
static void* resolve_face(mrb_state* mrb, void* body_ptr, mrb_value sel_val) {
    if (mrb_data_p(sel_val)) {
        /* Already a native Shape — use it directly. */
        return DATA_PTR(sel_val);
    }

    /* Accept a Symbol (:top, :bottom, :side, :all, ">Z", etc.) */
    const char* sel = NULL;
    if (mrb_symbol_p(sel_val)) {
        sel = mrb_sym_name(mrb, mrb_symbol(sel_val));
    } else if (mrb_string_p(sel_val)) {
        sel = mrb_str_to_cstr(mrb, sel_val);
    } else {
        mrb_raise(mrb, E_TYPE_ERROR, "face selector must be a Symbol, String, or Shape");
    }

    const char* err = NULL;
    int count = rrcad_shape_faces_count(body_ptr, sel, &err);
    raise_if_err(mrb, err);
    if (count < 1)
        mrb_raisef(mrb, E_RUNTIME_ERROR, "no faces matching '%s'", sel);

    void* face_ptr = rrcad_shape_faces_get(body_ptr, sel, 0, &err);
    raise_if_err(mrb, err);
    return face_ptr;
}

/* .pad(face_sel, height:) { sketch } — extrude sketch on face, fuse with self. */
static mrb_value mrb_rrcad_shape_pad(mrb_state* mrb, mrb_value self) {
    void* body_ptr = self_ptr(mrb, self);

    mrb_value face_sel;
    mrb_value kwargs = mrb_nil_value();
    mrb_value block = mrb_nil_value();
    mrb_get_args(mrb, "o|H&", &face_sel, &kwargs, &block);

    /* Extract height: keyword (required). */
    if (mrb_nil_p(kwargs))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "pad requires height: keyword");
    mrb_value height_val = opt_fetch(mrb, kwargs, "height", mrb_nil_value());
    if (mrb_nil_p(height_val))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "pad requires height: keyword");
    double height = value_to_double(mrb, height_val);

    /* Evaluate the block to obtain the sketch shape. */
    if (mrb_nil_p(block))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "pad requires a block { sketch }");
    mrb_value sketch_val = mrb_yield(mrb, block, mrb_nil_value());
    if (mrb->exc)
        return mrb_nil_value(); /* let the exception propagate */
    void* sketch_ptr = checked_shape_ptr(mrb, sketch_val);

    /* Resolve face selector. */
    void* face_ptr = resolve_face(mrb, body_ptr, face_sel);
    require_native_ptr(mrb, face_ptr);

    const char* err = NULL;
    void* result = rrcad_shape_pad(body_ptr, face_ptr, sketch_ptr, height, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* .pocket(face_sel, depth:) { sketch } — extrude sketch inward, cut from self. */
static mrb_value mrb_rrcad_shape_pocket(mrb_state* mrb, mrb_value self) {
    void* body_ptr = self_ptr(mrb, self);

    mrb_value face_sel;
    mrb_value kwargs = mrb_nil_value();
    mrb_value block = mrb_nil_value();
    mrb_get_args(mrb, "o|H&", &face_sel, &kwargs, &block);

    /* Extract depth: keyword (required). */
    if (mrb_nil_p(kwargs))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "pocket requires depth: keyword");
    mrb_value depth_val = opt_fetch(mrb, kwargs, "depth", mrb_nil_value());
    if (mrb_nil_p(depth_val))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "pocket requires depth: keyword");
    double depth = value_to_double(mrb, depth_val);

    /* Evaluate the block to obtain the sketch shape. */
    if (mrb_nil_p(block))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "pocket requires a block { sketch }");
    mrb_value sketch_val = mrb_yield(mrb, block, mrb_nil_value());
    if (mrb->exc)
        return mrb_nil_value(); /* let the exception propagate */
    void* sketch_ptr = checked_shape_ptr(mrb, sketch_val);

    /* Resolve face selector. */
    void* face_ptr = resolve_face(mrb, body_ptr, face_sel);
    require_native_ptr(mrb, face_ptr);

    const char* err = NULL;
    void* result = rrcad_shape_pocket(body_ptr, face_ptr, sketch_ptr, depth, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* .fillet_wire(radius) — fillet all corners of a 2D Wire or Face profile. */
static mrb_value mrb_rrcad_shape_fillet_wire(mrb_state* mrb, mrb_value self) {
    void* ptr = self_ptr(mrb, self);

    mrb_value radius_val;
    mrb_get_args(mrb, "o", &radius_val);
    double radius = value_to_double(mrb, radius_val);

    const char* err = NULL;
    void* result = rrcad_shape_fillet_wire(ptr, radius, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* datum_plane(origin: [ox, oy, oz], normal: [nx, ny, nz], x_dir: [xx, xy, xz])
 * Construct a finite reference plane (Face) for use in pad/pocket or cross-sections. */
static mrb_value mrb_rrcad_datum_plane(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value kwargs = mrb_nil_value();
    mrb_get_args(mrb, "H", &kwargs);

    /* Helper: extract a [x, y, z] array from a keyword. */
    mrb_sym origin_key = mrb_intern_lit(mrb, "origin");
    mrb_sym normal_key = mrb_intern_lit(mrb, "normal");
    mrb_sym x_dir_key = mrb_intern_lit(mrb, "x_dir");

    mrb_value orig_val = mrb_hash_fetch(mrb, kwargs, mrb_symbol_value(origin_key), mrb_nil_value());
    mrb_value norm_val = mrb_hash_fetch(mrb, kwargs, mrb_symbol_value(normal_key), mrb_nil_value());
    mrb_value xdir_val = mrb_hash_fetch(mrb, kwargs, mrb_symbol_value(x_dir_key), mrb_nil_value());

    if (mrb_nil_p(orig_val) || mrb_nil_p(norm_val) || mrb_nil_p(xdir_val))
        mrb_raise(mrb, E_ARGUMENT_ERROR,
                  "datum_plane requires origin:, normal:, and x_dir: arrays");

    /* Extract three floats from a Ruby array. */
#define EXTRACT3(arr, a, b, c)                                                                     \
    do {                                                                                           \
        mrb_value _a = mrb_ary_ref(mrb, (arr), 0);                                                 \
        mrb_value _b = mrb_ary_ref(mrb, (arr), 1);                                                 \
        mrb_value _c = mrb_ary_ref(mrb, (arr), 2);                                                 \
        (a) = value_to_double(mrb, _a);                                                            \
        (b) = value_to_double(mrb, _b);                                                            \
        (c) = value_to_double(mrb, _c);                                                            \
    } while (0)

    mrb_float ox, oy, oz, nx, ny, nz, xx, xy, xz;
    EXTRACT3(orig_val, ox, oy, oz);
    EXTRACT3(norm_val, nx, ny, nz);
    EXTRACT3(xdir_val, xx, xy, xz);
#undef EXTRACT3

    const char* err = NULL;
    void* result = rrcad_datum_plane((double)ox, (double)oy, (double)oz, (double)nx, (double)ny,
                                     (double)nz, (double)xx, (double)xy, (double)xz, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* =========================================================================
 * Phase 8 Tier 3: Inspection & clearance
 * =========================================================================
 */

/* shape.distance_to(other) → Float
 * Minimum distance between two shapes.  Returns 0.0 when they overlap. */
static mrb_value mrb_rrcad_shape_distance_to(mrb_state* mrb, mrb_value self) {
    mrb_value other;
    mrb_get_args(mrb, "o", &other);
    void* a_ptr = self_ptr(mrb, self);
    void* b_ptr = DATA_PTR(other);
    if (!b_ptr)
        mrb_raise(mrb, E_TYPE_ERROR, "distance_to: argument must be a Shape");
    const char* err = NULL;
    double dist = rrcad_shape_distance_to(a_ptr, b_ptr, &err);
    raise_if_err(mrb, err);
    return mrb_float_value(mrb, dist);
}

/* shape.inertia → {ixx:, iyy:, izz:, ixy:, ixz:, iyz:}
 * Inertia tensor about the centre of mass in the world frame. */
static mrb_value mrb_rrcad_shape_inertia(mrb_state* mrb, mrb_value self) {
    void* ptr = self_ptr(mrb, self);
    double buf[6] = {0};
    const char* err = NULL;
    rrcad_shape_inertia(ptr, buf, &err);
    raise_if_err(mrb, err);

    mrb_value hash = mrb_hash_new(mrb);
    static const char* keys[] = {"ixx", "iyy", "izz", "ixy", "ixz", "iyz"};
    for (int i = 0; i < 6; i++) {
        mrb_hash_set(mrb, hash, mrb_symbol_value(mrb_intern_cstr(mrb, keys[i])),
                     mrb_float_value(mrb, buf[i]));
    }
    return hash;
}

/* shape.min_thickness → Float
 * Minimum wall thickness of a solid or shell. */
static mrb_value mrb_rrcad_shape_min_thickness(mrb_state* mrb, mrb_value self) {
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    double t = rrcad_shape_min_thickness(ptr, &err);
    raise_if_err(mrb, err);
    return mrb_float_value(mrb, t);
}

/* =========================================================================
 * Phase 8 Tier 2: Manufacturing features
 * =========================================================================
 */

/* helix(radius:, pitch:, height:)
 *
 * Top-level Kernel method.  Returns a Wire suitable for use as a sweep path
 * for thread profiles.  radius/pitch/height must be positive floats. */
static mrb_value mrb_rrcad_helix(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value kwargs = mrb_nil_value();
    mrb_get_args(mrb, "H", &kwargs);

    mrb_sym radius_key = mrb_intern_lit(mrb, "radius");
    mrb_sym pitch_key = mrb_intern_lit(mrb, "pitch");
    mrb_sym height_key = mrb_intern_lit(mrb, "height");

    mrb_value r_val = mrb_hash_fetch(mrb, kwargs, mrb_symbol_value(radius_key), mrb_nil_value());
    mrb_value p_val = mrb_hash_fetch(mrb, kwargs, mrb_symbol_value(pitch_key), mrb_nil_value());
    mrb_value h_val = mrb_hash_fetch(mrb, kwargs, mrb_symbol_value(height_key), mrb_nil_value());

    if (mrb_nil_p(r_val) || mrb_nil_p(p_val) || mrb_nil_p(h_val))
        mrb_raise(mrb, E_ARGUMENT_ERROR, "helix requires radius:, pitch:, and height:");

    double radius = value_to_double(mrb, r_val);
    double pitch = value_to_double(mrb, p_val);
    double height = value_to_double(mrb, h_val);

    const char* err = NULL;
    void* result = rrcad_make_helix(radius, pitch, height, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* =========================================================================
 * Phase 8 Tier 5: Advanced composition
 *
 *   fragment([a, b, c])               → Compound (all non-overlapping pieces)
 *   shape.convex_hull                 → Shape
 *   path_pattern(shape, path, n)      → Compound (n copies along path)
 *   shape.sweep(path)                 → Shape  (existing)
 *   shape.sweep(path, guide: guide)   → Shape  (guided sweep, new)
 * =========================================================================
 */

/* fragment([a, b, c]) — top-level Kernel method */
static mrb_value mrb_rrcad_fragment(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value ary;
    mrb_get_args(mrb, "A", &ary);

    mrb_int n = RARRAY_LEN(ary);
    if (n < 1)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "fragment: requires at least 1 shape");

    /* Collect shape pointers into a temporary C array. */
    const void** ptrs = (const void**)mrb_malloc(mrb, (size_t)n * sizeof(void*));
    for (mrb_int i = 0; i < n; i++) {
        mrb_value elem = mrb_ary_ref(mrb, ary, i);
        void* p = checked_shape_ptr(mrb, elem);
        ptrs[i] = p;
    }

    const char* err = NULL;
    void* result = rrcad_shape_fragment(ptrs, (size_t)n, &err);
    mrb_free(mrb, ptrs);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* .thicken(thickness) — give a Face or Shell a wall, making it a solid. */
static mrb_value mrb_rrcad_shape_thicken(mrb_state* mrb, mrb_value self) {
    mrb_value thickness_val;
    mrb_get_args(mrb, "o", &thickness_val);
    double thickness = value_to_double(mrb, thickness_val);
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_thicken(ptr, thickness, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* shape.convex_hull */
static mrb_value mrb_rrcad_shape_convex_hull(mrb_state* mrb, mrb_value self) {
    void* ptr = self_ptr(mrb, self);
    const char* err = NULL;
    void* result = rrcad_shape_convex_hull(ptr, &err);
    raise_if_err(mrb, err);
    return shape_from_ptr(mrb, result);
}

/* path_pattern(shape, path, n) — top-level Kernel method */
static mrb_value mrb_rrcad_path_pattern(mrb_state* mrb, mrb_value self) {
    (void)self;
    mrb_value shape_val, path_val;
    mrb_int n;
    mrb_get_args(mrb, "ooi", &shape_val, &path_val, &n);

    void* ptr = shape_ptr(mrb, shape_val);
    void* path_ptr = shape_ptr(mrb, path_val);
    require_native_ptr(mrb, ptr);
    require_native_ptr(mrb, path_ptr);

    /* Guard against integer overflow: mrb_int is 64-bit but the C++ side
     * expects a plain int (32-bit).  A caller passing n > INT_MAX would
     * silently wrap to a negative value, causing undefined behaviour in the
     * C++ loop.  Reject out-of-range values with a clear Ruby error. */
    if (n < 1 || n > INT_MAX)
        mrb_raise(mrb, E_ARGUMENT_ERROR, "path_pattern count must be between 1 and INT_MAX");

    const char* err = NULL;
    void* result = rrcad_shape_path_pattern(ptr, path_ptr, (int)n, &err);
    raise_if_err(mrb, err);
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
    mrb_define_method(mrb, shape_class, "export", mrb_rrcad_shape_export,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1));
    /* Flat cut-file export from a planar face (see mrb_rrcad_shape_export_outline). */
    mrb_define_method(mrb, shape_class, "__rrcad_export_outline", mrb_rrcad_shape_export_outline,
                      MRB_ARGS_REQ(3));
    mrb_define_method(mrb, shape_class, "fuse", mrb_rrcad_shape_fuse, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "cut", mrb_rrcad_shape_cut, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "common", mrb_rrcad_shape_common, MRB_ARGS_REQ(1));

    /* Phase 5: Assembly mating */
    mrb_define_method(mrb, shape_class, "mate", mrb_rrcad_shape_mate,
                      MRB_ARGS_REQ(2) | MRB_ARGS_OPT(1));

    /* Phase 5: Color */
    mrb_define_method(mrb, shape_class, "color", mrb_rrcad_shape_color, MRB_ARGS_REQ(3));
    mrb_define_method(mrb, shape_class, "name_face", mrb_rrcad_shape_name_face, MRB_ARGS_REQ(2));
    mrb_define_method(mrb, shape_class, "name_edge", mrb_rrcad_shape_name_edge, MRB_ARGS_REQ(2));
    mrb_define_method(mrb, shape_class, "datum", mrb_rrcad_shape_datum, MRB_ARGS_REQ(2));
    mrb_define_method(mrb, shape_class, "ref", mrb_rrcad_shape_ref, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "gdt_apply", mrb_rrcad_shape_gdt_apply, MRB_ARGS_REQ(1));

    /* Phase 2: Transforms */
    mrb_define_method(mrb, shape_class, "translate", mrb_rrcad_shape_translate, MRB_ARGS_REQ(3));
    mrb_define_method(mrb, shape_class, "rotate", mrb_rrcad_shape_rotate, MRB_ARGS_REQ(4));
    mrb_define_method(mrb, shape_class, "scale", mrb_rrcad_shape_scale,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(2));
    mrb_define_method(mrb, shape_class, "fillet", mrb_rrcad_shape_fillet,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1)); /* (r[, :selector]) */
    mrb_define_method(mrb, shape_class, "chamfer", mrb_rrcad_shape_chamfer,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1)); /* (d[, :selector]) */
    mrb_define_method(mrb, shape_class, "chamfer_asym", mrb_rrcad_shape_chamfer_asym,
                      MRB_ARGS_REQ(2) | MRB_ARGS_OPT(1)); /* (d1, d2[, :selector]) */
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
    mrb_define_method(mrb, mrb->kernel_module, "__rrcad_profile_2d", mrb_rrcad_profile_2d,
                      MRB_ARGS_REQ(3));
    mrb_define_method(mrb, mrb->kernel_module, "ellipse", mrb_rrcad_ellipse, MRB_ARGS_REQ(2));
    mrb_define_method(mrb, mrb->kernel_module, "arc", mrb_rrcad_arc, MRB_ARGS_REQ(3));

    /* Phase 3: Spline constructors and sweep */
    mrb_define_method(mrb, mrb->kernel_module, "spline_2d", mrb_rrcad_spline_2d,
                      MRB_ARGS_REQ(1) | MRB_ARGS_KEY(1, 0)); /* (pts[, tangents:]) */
    mrb_define_method(mrb, mrb->kernel_module, "spline_3d", mrb_rrcad_spline_3d,
                      MRB_ARGS_REQ(1) | MRB_ARGS_KEY(1, 0)); /* (pts[, tangents:]) */
    mrb_define_method(mrb, shape_class, "sweep", mrb_rrcad_shape_sweep,
                      MRB_ARGS_REQ(1) | MRB_ARGS_KEY(1, 0)); /* (path[, guide:]) */
    mrb_define_method(mrb, mrb->kernel_module, "sweep_sections", mrb_rrcad_sweep_sections,
                      MRB_ARGS_REQ(2));

    /* Phase 3: Live preview */
    mrb_define_method(mrb, mrb->kernel_module, "preview", mrb_rrcad_preview, MRB_ARGS_REQ(1));
    /* Output primitive behind the prelude's puts / print / p. */
    mrb_define_method(mrb, mrb->kernel_module, "__rrcad_write", mrb_rrcad_write, MRB_ARGS_REQ(1));
    /* Multi-file projects.  Disabled unless a base directory was set, and
     * undefined outright by the MCP security prelude. */
    mrb_define_method(mrb, mrb->kernel_module, "require_relative", mrb_rrcad_require_relative,
                      MRB_ARGS_REQ(1));

    /* Phase 3: Sub-shape selectors */
    mrb_define_method(mrb, shape_class, "faces", mrb_rrcad_shape_faces, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "edges", mrb_rrcad_shape_edges, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "vertices", mrb_rrcad_shape_vertices, MRB_ARGS_REQ(1));

    /* Phase 4: Import */
    mrb_define_method(mrb, mrb->kernel_module, "import_step", mrb_rrcad_import_step,
                      MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "import_stl", mrb_rrcad_import_stl, MRB_ARGS_REQ(1));

    /* Phase 4: 3-D operations */
    mrb_define_method(mrb, mrb->kernel_module, "loft", mrb_rrcad_loft,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1));
    mrb_define_method(mrb, shape_class, "shell", mrb_rrcad_shape_shell, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "offset", mrb_rrcad_shape_offset, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "offset_2d", mrb_rrcad_shape_offset_2d, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "simplify", mrb_rrcad_shape_simplify, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "thicken", mrb_rrcad_shape_thicken, MRB_ARGS_REQ(1));

    /* Phase 4: Patterns */
    mrb_define_method(mrb, mrb->kernel_module, "linear_pattern", mrb_rrcad_linear_pattern,
                      MRB_ARGS_REQ(3));
    mrb_define_method(mrb, mrb->kernel_module, "polar_pattern", mrb_rrcad_polar_pattern,
                      MRB_ARGS_REQ(3));

    /* Phase 7 Tier 1: grid_pattern / fuse_all / cut_all */
    mrb_define_method(mrb, mrb->kernel_module, "grid_pattern", mrb_rrcad_grid_pattern,
                      MRB_ARGS_REQ(5));
    mrb_define_method(mrb, mrb->kernel_module, "fuse_all", mrb_rrcad_fuse_all, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "cut_all", mrb_rrcad_cut_all, MRB_ARGS_REQ(2));

    /* Phase 4: Query / introspection */
    mrb_define_method(mrb, shape_class, "bounding_box", mrb_rrcad_shape_bounding_box,
                      MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "volume", mrb_rrcad_shape_volume, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "surface_area", mrb_rrcad_shape_surface_area,
                      MRB_ARGS_NONE());

    /* Phase 7 Tier 2: validation & introspection */
    mrb_define_method(mrb, shape_class, "shape_type", mrb_rrcad_shape_type, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "centroid", mrb_rrcad_shape_centroid, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "normal", mrb_rrcad_shape_face_normal, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "cylinder_axis", mrb_rrcad_shape_cylinder_axis,
                      MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "closed?", mrb_rrcad_shape_closed, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "manifold?", mrb_rrcad_shape_manifold, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "validate", mrb_rrcad_shape_validate, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "history", mrb_rrcad_shape_history, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "feature_graph", mrb_rrcad_shape_feature_graph,
                      MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "rebuild", mrb_rrcad_shape_rebuild, MRB_ARGS_NONE());

    /* Phase 7: Bézier patch and sewing */
    mrb_define_method(mrb, mrb->kernel_module, "bezier_patch", mrb_rrcad_bezier_patch,
                      MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "sew", mrb_rrcad_sew,
                      MRB_ARGS_REQ(1) | MRB_ARGS_OPT(1));

    /* Phase 7 Tier 3: surface modeling */
    mrb_define_method(mrb, mrb->kernel_module, "ruled_surface", mrb_rrcad_ruled_surface,
                      MRB_ARGS_REQ(2));
    mrb_define_method(mrb, mrb->kernel_module, "fill_surface", mrb_rrcad_fill_surface,
                      MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "slice", mrb_rrcad_shape_slice,
                      MRB_ARGS_KEY(2, 0)); /* plane:, z:/y:/x: */

    /* Phase 8 Tier 1: Core Part Design */
    mrb_define_method(mrb, shape_class, "pad", mrb_rrcad_shape_pad,
                      MRB_ARGS_REQ(1) | MRB_ARGS_KEY(1, 0) |
                          MRB_ARGS_BLOCK()); /* (sel, height:) {} */
    mrb_define_method(mrb, shape_class, "pocket", mrb_rrcad_shape_pocket,
                      MRB_ARGS_REQ(1) | MRB_ARGS_KEY(1, 0) |
                          MRB_ARGS_BLOCK()); /* (sel, depth:) {} */
    mrb_define_method(mrb, shape_class, "fillet_wire", mrb_rrcad_shape_fillet_wire,
                      MRB_ARGS_REQ(1));
    mrb_define_method(mrb, mrb->kernel_module, "datum_plane", mrb_rrcad_datum_plane,
                      MRB_ARGS_KEY(3, 0)); /* origin:, normal:, x_dir: */

    /* Phase 8 Tier 2: Manufacturing features */
    mrb_define_method(mrb, mrb->kernel_module, "helix", mrb_rrcad_helix,
                      MRB_ARGS_KEY(3, 0)); /* radius:, pitch:, height: */

    /* Phase 8 Tier 3: Inspection & clearance */
    mrb_define_method(mrb, shape_class, "distance_to", mrb_rrcad_shape_distance_to,
                      MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "inertia", mrb_rrcad_shape_inertia, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "min_thickness", mrb_rrcad_shape_min_thickness,
                      MRB_ARGS_NONE());

    /* Phase 8 Tier 5: Advanced composition */
    mrb_define_method(mrb, mrb->kernel_module, "fragment", mrb_rrcad_fragment, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "convex_hull", mrb_rrcad_shape_convex_hull,
                      MRB_ARGS_NONE());
    mrb_define_method(mrb, mrb->kernel_module, "path_pattern", mrb_rrcad_path_pattern,
                      MRB_ARGS_REQ(3));
}
