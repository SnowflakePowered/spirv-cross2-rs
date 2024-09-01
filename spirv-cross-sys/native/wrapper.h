#include "spirv_cross_c.h"
#include <stdint.h>

void spvc_rs_expose_set(spvc_set set, uint32_t* out, size_t* length);

/* This shold be wrapped in SPVC_BEGIN_SAFE_SCOPE but alas */
spvc_result spvc_compiler_set_entry_point_safe(spvc_compiler compiler, const char *name, SpvExecutionModel model);
