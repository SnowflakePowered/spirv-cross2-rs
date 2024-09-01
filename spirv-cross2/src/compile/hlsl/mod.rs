use crate::compile::CommonCompileOptions;
use crate::targets::Hlsl;
use crate::Compiler;
pub use spirv_cross_sys::HlslBindingFlagBits as BindingFlags;
pub use spirv_cross_sys::HlslBindingFlags;
pub use spirv_cross_sys::HlslResourceBinding as ResourceBinding;
pub use spirv_cross_sys::HlslResourceBindingMapping as ResourceBindingMapping;
pub use spirv_cross_sys::HlslRootConstants as RootConstants;
// SPVC_PUBLIC_API spvc_result spvc_compiler_hlsl_set_root_constants_layout(spvc_compiler compiler,
// const spvc_hlsl_root_constants *constant_info,
// size_t count);
// SPVC_PUBLIC_API spvc_result spvc_compiler_hlsl_add_vertex_attribute_remap(spvc_compiler compiler,
// const spvc_hlsl_vertex_attribute_remap *remap,
// size_t remaps);
// SPVC_PUBLIC_API spvc_variable_id spvc_compiler_hlsl_remap_num_workgroups_builtin(spvc_compiler compiler);
//
// SPVC_PUBLIC_API spvc_result spvc_compiler_hlsl_set_resource_binding_flags(spvc_compiler compiler,
// spvc_hlsl_binding_flags flags);
//
// SPVC_PUBLIC_API spvc_result spvc_compiler_hlsl_add_resource_binding(spvc_compiler compiler,
// const spvc_hlsl_resource_binding *binding);
// SPVC_PUBLIC_API spvc_bool spvc_compiler_hlsl_is_resource_used(spvc_compiler compiler,
// SpvExecutionModel model,
// unsigned set,
// unsigned binding);
use crate::compile::CompilerOptions;
use crate::ContextRooted;
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CompileOptions {
    #[expand]
    pub common: CommonCompileOptions,
}

impl<'a> Compiler<'a, Hlsl> {}
