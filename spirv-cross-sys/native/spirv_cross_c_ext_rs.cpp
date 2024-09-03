#include "spirv_cross_c.cpp"
#include <cstdint>
#include <unordered_set>

// hack to get at protected methods of compiler.
// this must not have any fields to maintain ABI, only static dispatch methods.
struct __InternalCompilerHack : Compiler {
    public:
      SPIRVariable& get_variable(VariableID id) {
          return get<SPIRVariable>(id);
      };
};

static_assert(sizeof(__InternalCompilerHack) == sizeof(Compiler),
    "Compiler can not be casted to __InternalCompilerHack" );

/*
 * This is the native entrypoint for spirv-cross2/spirv-cross-sys.
 *
 * spirv_cross_c.cpp is included so that we can write extensions to
 * the SPIRV-Cross C API without interfering with upstream to support
 * the Rust API.
 *
 * Functions here should be namespaced under spvc_rs_ to avoid namespacing
 * issues with the main C API.
 */
extern "C" {

void spvc_rs_expose_set(spvc_set opaque_set, uint32_t* out, size_t* length) {
    if (length != nullptr) {
        *length = opaque_set->set.size();
    }

    if (out == nullptr) {
        return;
    }

    for (auto &id: opaque_set->set) {
       *out = id;
       out++;
    }
}

spvc_bool spvc_rs_constant_is_scalar(spvc_constant constant) {
    return constant->m.columns == 1 && constant->m.c[0].vecsize == 1;
}

uint32_t spvc_rs_constant_get_vecsize(spvc_constant constant) {
    return constant->m.c[0].vecsize;
}

uint32_t spvc_rs_constant_get_matrix_colsize(spvc_constant constant) {
    return constant->m.columns;
}

spvc_result spvc_rs_compiler_variable_get_type(spvc_compiler compiler, spvc_variable_id variable_id, spvc_type_id* out) {
    // Should only throw if an intentionally garbage ID is passed, but the IDs are not type-safe.
    SPVC_BEGIN_SAFE_SCOPE
    {
        SPIRVariable& variable = static_cast<__InternalCompilerHack *>(compiler->compiler.get())->get_variable(variable_id);
        *out = variable.basetype;
        return SPVC_SUCCESS;
    }
    SPVC_END_SAFE_SCOPE(compiler->context, SPVC_ERROR_INVALID_ARGUMENT)
}

spvc_bool spvc_rs_type_is_pointer(spvc_type type) {
    return type->pointer;
}

spvc_bool spvc_rs_type_is_forward_pointer(spvc_type type) {
    return type->forward_pointer;
}

} // extern "C"