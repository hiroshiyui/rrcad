/*
 * glue.c — thin C shims that hide mrb_value from Rust.
 *
 * All functions here deal with mrb_value internally and expose only
 * plain C types (char*, int, void*) across the FFI boundary, avoiding the
 * need for Rust to know anything about mRuby's value representation.
 */

#include <mruby.h>
#include <mruby/class.h>
#include <mruby/compile.h>
#include <mruby/data.h>
#include <mruby/error.h>
#include <mruby/string.h>

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
const char *rrcad_mrb_eval(mrb_state *mrb, const char *code,
                            const char **error_out)
{
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
extern void *rrcad_make_box(double dx, double dy, double dz,
                            const char **error_out);
extern void *rrcad_make_cylinder(double r, double h, const char **error_out);
extern void *rrcad_make_sphere(double r, const char **error_out);
extern void  rrcad_shape_drop(void *ptr);
extern void  rrcad_shape_export_step(void *ptr, const char *path,
                                     const char **error_out);
extern void *rrcad_shape_fuse(void *a, void *b, const char **error_out);
extern void *rrcad_shape_cut(void *a, void *b, const char **error_out);
extern void *rrcad_shape_common(void *a, void *b, const char **error_out);

/* mRuby data type descriptor — name appears in TypeError messages. */
static void shape_dfree(mrb_state *mrb, void *ptr)
{
    (void)mrb;
    rrcad_shape_drop(ptr); /* no-op for NULL */
}

static const mrb_data_type shape_type = {"Shape", shape_dfree};

/* Wrap a raw Rust Box pointer in a new mRuby Shape RData value.
 * The Shape class is looked up per-call so multiple concurrent VMs
 * (e.g. parallel test threads) each see their own class pointer. */
static mrb_value shape_from_ptr(mrb_state *mrb, void *ptr)
{
    struct RClass *cls = mrb_class_get(mrb, "Shape");
    struct RData *rd = mrb_data_object_alloc(mrb, cls, ptr, &shape_type);
    return mrb_obj_value(rd);
}

/* Extract and type-check the raw pointer from a Shape mrb_value.
 * Raises TypeError if `v` is not a Shape RData object. */
static void *shape_ptr(mrb_state *mrb, mrb_value v)
{
    return mrb_data_get_ptr(mrb, v, &shape_type);
}

/* -------------------------------------------------------------------------
 * Helpers
 * -------------------------------------------------------------------------
 */

/* Check that `ptr` is not NULL (i.e. the shape was created natively).
 * Raises RuntimeError if it is — this protects callers from accessing
 * stub shapes that have no backing OCCT object. */
static void require_native_ptr(mrb_state *mrb, void *ptr)
{
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

static mrb_value mrb_rrcad_box(mrb_state *mrb, mrb_value self)
{
    (void)self;
    mrb_float dx, dy, dz;
    mrb_get_args(mrb, "fff", &dx, &dy, &dz);

    const char *err = NULL;
    void *ptr = rrcad_make_box((double)dx, (double)dy, (double)dz, &err);
    if (err) mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_cylinder(mrb_state *mrb, mrb_value self)
{
    (void)self;
    mrb_float r, h;
    mrb_get_args(mrb, "ff", &r, &h);

    const char *err = NULL;
    void *ptr = rrcad_make_cylinder((double)r, (double)h, &err);
    if (err) mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

static mrb_value mrb_rrcad_sphere(mrb_state *mrb, mrb_value self)
{
    (void)self;
    mrb_float r;
    mrb_get_args(mrb, "f", &r);

    const char *err = NULL;
    void *ptr = rrcad_make_sphere((double)r, &err);
    if (err) mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, ptr);
}

/* -------------------------------------------------------------------------
 * Shape instance methods
 * -------------------------------------------------------------------------
 */

static mrb_value mrb_rrcad_shape_inspect(mrb_state *mrb, mrb_value self)
{
    (void)self;
    return mrb_str_new_cstr(mrb, "#<Shape>");
}

static mrb_value mrb_rrcad_shape_export(mrb_state *mrb, mrb_value self)
{
    const char *path;
    mrb_get_args(mrb, "z", &path);

    void *ptr = DATA_PTR(self);
    require_native_ptr(mrb, ptr);

    const char *err = NULL;
    rrcad_shape_export_step(ptr, path, &err);
    if (err) mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return self;
}

static mrb_value mrb_rrcad_shape_fuse(mrb_state *mrb, mrb_value self)
{
    mrb_value other;
    mrb_get_args(mrb, "o", &other);

    void *a = DATA_PTR(self);
    require_native_ptr(mrb, a);
    void *b = shape_ptr(mrb, other);
    require_native_ptr(mrb, b);

    const char *err = NULL;
    void *result = rrcad_shape_fuse(a, b, &err);
    if (err) mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_cut(mrb_state *mrb, mrb_value self)
{
    mrb_value other;
    mrb_get_args(mrb, "o", &other);

    void *a = DATA_PTR(self);
    require_native_ptr(mrb, a);
    void *b = shape_ptr(mrb, other);
    require_native_ptr(mrb, b);

    const char *err = NULL;
    void *result = rrcad_shape_cut(a, b, &err);
    if (err) mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

static mrb_value mrb_rrcad_shape_common(mrb_state *mrb, mrb_value self)
{
    mrb_value other;
    mrb_get_args(mrb, "o", &other);

    void *a = DATA_PTR(self);
    require_native_ptr(mrb, a);
    void *b = shape_ptr(mrb, other);
    require_native_ptr(mrb, b);

    const char *err = NULL;
    void *result = rrcad_shape_common(a, b, &err);
    if (err) mrb_raise(mrb, E_RUNTIME_ERROR, err);
    return shape_from_ptr(mrb, result);
}

/* =========================================================================
 * rrcad_register_shape_class
 *
 * Called from Rust (MrubyVm::new) after the DSL prelude has been evaluated.
 * Registering after the prelude means our native methods shadow the stubs.
 * =========================================================================
 */
void rrcad_register_shape_class(mrb_state *mrb)
{
    /* Get (or create) the Shape class and mark its instances as RData. */
    struct RClass *shape_class = mrb_define_class(mrb, "Shape", mrb->object_class);
    MRB_SET_INSTANCE_TT(shape_class, MRB_TT_DATA);

    /* Instance methods — override the prelude stubs. */
    mrb_define_method(mrb, shape_class, "inspect",
                      mrb_rrcad_shape_inspect, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "to_s",
                      mrb_rrcad_shape_inspect, MRB_ARGS_NONE());
    mrb_define_method(mrb, shape_class, "export",
                      mrb_rrcad_shape_export, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "fuse",
                      mrb_rrcad_shape_fuse, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "cut",
                      mrb_rrcad_shape_cut, MRB_ARGS_REQ(1));
    mrb_define_method(mrb, shape_class, "common",
                      mrb_rrcad_shape_common, MRB_ARGS_REQ(1));

    /* Top-level primitive constructors — available everywhere via Kernel. */
    mrb_define_method(mrb, mrb->kernel_module, "box",
                      mrb_rrcad_box, MRB_ARGS_REQ(3));
    mrb_define_method(mrb, mrb->kernel_module, "cylinder",
                      mrb_rrcad_cylinder, MRB_ARGS_REQ(2));
    mrb_define_method(mrb, mrb->kernel_module, "sphere",
                      mrb_rrcad_sphere, MRB_ARGS_REQ(1));
}
