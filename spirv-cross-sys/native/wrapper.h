#include "spirv_cross_c.h"
#include <stdint.h>

void spvc_rs_expose_set(spvc_set set, uint32_t* out, size_t* length);

spvc_bool spvc_rs_constant_is_scalar(spvc_constant constant);

uint32_t spvc_rs_constant_get_vecsize(spvc_constant constant);

uint32_t spvc_rs_constant_get_matrix_colsize(spvc_constant constant);