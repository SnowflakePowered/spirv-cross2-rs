use crate::compile::CommonCompileOptions;
pub use spirv_cross_sys::MslConstexprSampler as ConstexprSampler;
pub use spirv_cross_sys::MslResourceBinding2 as ResourceBinding;
pub use spirv_cross_sys::MslSamplerYcbcrConversion as SamplerYcbcrConversion;
pub use spirv_cross_sys::MslShaderInput as ShaderInput;
pub use spirv_cross_sys::MslShaderInterfaceVar2 as ShaderInterfaceVar;
pub use spirv_cross_sys::MslVertexAttribute as VertexAttribute;

use crate::compile::CompilerOptions;
use crate::ContextRooted;
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CompileOptions {
    #[expand]
    pub common: CommonCompileOptions,
}
