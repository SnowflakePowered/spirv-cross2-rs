#include "spirv_cross_c.h"
#include <stdint.h>

void spvc_rs_expose_set(spvc_set set, uint32_t* out, size_t* length);

/* This shold be wrapped in SPVC_BEGIN_SAFE_SCOPE but alas */
spvc_result spvc_rs_compiler_set_entry_point_safe(spvc_compiler compiler, const char *name, SpvExecutionModel model);

spvc_bool spvc_rs_constant_is_scalar(spvc_constant constant);

uint32_t spvc_rs_constant_get_vecsize(spvc_constant constant);

uint32_t spvc_rs_constant_get_matrix_colsize(spvc_constant constant);