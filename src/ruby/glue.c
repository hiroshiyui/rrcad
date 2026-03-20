/*
 * glue.c — thin C shims that hide mrb_value from Rust.
 *
 * All functions here deal with mrb_value internally and expose only
 * plain C types (char*, int) across the FFI boundary, avoiding the need
 * for Rust to know anything about mRuby's value representation.
 */

#include <mruby.h>
#include <mruby/compile.h>
#include <mruby/error.h>
#include <mruby/string.h>

/*
 * rrcad_mrb_eval — evaluate Ruby source code and return its inspect string.
 *
 * On success  : returns a pointer to a NUL-terminated C string (owned by
 *               mRuby GC — copy before the next eval or GC cycle).
 *               *error_out is set to NULL.
 * On exception: returns NULL.
 *               *error_out points to a NUL-terminated description string
 *               (also GC-owned).  The pending exception is cleared.
 */
const char *rrcad_mrb_eval(mrb_state *mrb, const char *code,
                            const char **error_out) {
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
