use crate::compile::CommonCompileOptions;
use crate::targets::Hlsl;
use crate::Compiler;
use spirv_cross_sys as sys;

pub use spirv_cross_sys::HlslBindingFlagBits as BindingFlags;
pub use spirv_cross_sys::HlslBindingFlags;
pub use spirv_cross_sys::HlslResourceBinding as ResourceBinding;
pub use spirv_cross_sys::HlslResourceBindingMapping as ResourceBindingMapping;
pub use spirv_cross_sys::HlslRootConstants as RootConstants;
use spirv_cross_sys::{spvc_compiler_option, spvc_compiler_options};

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
use crate::error::ToContextError;
use crate::ContextRooted;

/// HLSL compiler options
#[non_exhaustive]
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CompileOptions {
    /// Compile options common to GLSL, HLSL, and MSL.
    #[expand]
    pub common: CommonCompileOptions,

    /// The HLSL shader model version to output. The default is SM 3.0
    #[option(
        SPVC_COMPILER_OPTION_HLSL_SHADER_MODEL,
        HlslShaderModel::ShaderModel3_0
    )]
    pub shader_model: HlslShaderModel,

    /// Allows the PointSize builtin in SM 4.0+, and ignores it,
    /// as PointSize is not supported in SM 4+.
    #[option(SPVC_COMPILER_OPTION_HLSL_POINT_SIZE_COMPAT, false)]
    pub point_size_compat: bool,

    /// Allows the PointCoord builtin, returns float2(0.5, 0.5),
    /// as PointCoord is not supported in HLSL.
    #[option(SPVC_COMPILER_OPTION_HLSL_POINT_COORD_COMPAT, false)]
    pub point_coord_compat: bool,

    /// If true, the backend will assume that VertexIndex and InstanceIndex will need to apply
    /// a base offset, and you will need to fill in a cbuffer with offsets.
    /// Set to false if you know you will never use base instance or base vertex
    /// functionality as it might remove an internal cbuffer.
    #[option(
        SPVC_COMPILER_OPTION_HLSL_SUPPORT_NONZERO_BASE_VERTEX_BASE_INSTANCE,
        false
    )]
    pub support_nonzero_base_vertex_base_instance: bool,

    /// Forces a storage buffer to always be declared as UAV, even if the readonly decoration is used.
    /// By default, a readonly storage buffer will be declared as ByteAddressBuffer (SRV) instead.
    /// Alternatively, use set_hlsl_force_storage_buffer_as_uav to specify individually.
    #[option(SPVC_COMPILER_OPTION_HLSL_FORCE_STORAGE_BUFFER_AS_UAV, false)]
    pub force_storage_buffer_as_uav: bool,

    /// Forces any storage image type marked as NonWritable to be considered an SRV instead.
    /// For this to work with function call parameters, NonWritable must be considered to be part of the type system
    /// so that NonWritable image arguments are also translated to Texture rather than RWTexture.
    #[option(SPVC_COMPILER_OPTION_HLSL_NONWRITABLE_UAV_TEXTURE_AS_SRV, false)]
    pub nonwritable_uav_texture_as_srv: bool,

    /// If matrices are used as IO variables, flatten the attribute declaration to use
    /// `TEXCOORD{N,N+1,N+2,...}` rather than `TEXCOORDN_{0,1,2,3}`.
    /// If `add_vertex_attribute_remap` is used and this feature is used,
    /// the semantic name will be queried once per active location.
    #[option(SPVC_COMPILER_OPTION_HLSL_FLATTEN_MATRIX_VERTEX_INPUT_SEMANTICS, false)]
    pub flatten_matrix_vertex_input_semantics: bool,

    /// Enables native 16-bit types. Needs SM 6.2.
    /// Uses half/int16_t/uint16_t instead of min16* types.
    /// Also adds support for 16-bit load-store from (RW)ByteAddressBuffer.
    #[option(SPVC_COMPILER_OPTION_HLSL_ENABLE_16BIT_TYPES, false)]
    pub enable_16bit_types: bool,

    /// Rather than emitting main() for the entry point, use the name in SPIR-V.
    #[option(SPVC_COMPILER_OPTION_HLSL_USE_ENTRY_POINT_NAME, false)]
    pub use_entry_point_name: bool,
    // todo: preserve_structured_buffers
}

/// HLSL Shader model.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum HlslShaderModel {
    /// Shader Model 3.0 (Direct3D 9.0c).
    ///
    /// This is the lowest supported shader model.
    ShaderModel3_0,
    /// Shader Model 4.0 (Direct3D 10.0).
    ///
    /// Level 9.x feature levels are not explicitly supported.
    ShaderModel4_0,
    /// Shader Model 4.1 (Direct3D 10.1).
    ShaderModel4_1,
    /// Shader Model 5.0 (Direct3D 11/11.1)
    ShaderModel5_0,
    /// Shader Model 5.1 (Direct3D 12).
    ShaderModel5_1,
    /// Shader Model 6.0 (Direct3D 12)
    ShaderModel6_0,
    /// Shader Model 6.1 (Direct3D 12)
    ShaderModel6_1,
    /// Shader Model 6.2 (Direct3D 12)
    ShaderModel6_2,
    /// Shader Model 6.3 (Direct3D 12)
    ShaderModel6_3,
    /// Shader Model 6.4 (Direct3D 12)
    ShaderModel6_4,
    /// Shader Model 6.5 (Direct3D 12)
    ShaderModel6_5,
    /// Shader Model 6.6 (Direct3D 12)
    ShaderModel6_6,
    /// Shader Model 6.7 (Direct3D 12)
    ShaderModel6_7,
    /// Shader Model 6.8 (Direct3D 12)
    ShaderModel6_8,
}

impl From<HlslShaderModel> for u32 {
    fn from(value: HlslShaderModel) -> Self {
        match value {
            HlslShaderModel::ShaderModel3_0 => 30,
            HlslShaderModel::ShaderModel4_0 => 40,
            HlslShaderModel::ShaderModel4_1 => 41,
            HlslShaderModel::ShaderModel5_0 => 50,
            HlslShaderModel::ShaderModel5_1 => 51,
            HlslShaderModel::ShaderModel6_0 => 60,
            HlslShaderModel::ShaderModel6_1 => 61,
            HlslShaderModel::ShaderModel6_2 => 62,
            HlslShaderModel::ShaderModel6_3 => 63,
            HlslShaderModel::ShaderModel6_4 => 64,
            HlslShaderModel::ShaderModel6_5 => 65,
            HlslShaderModel::ShaderModel6_6 => 66,
            HlslShaderModel::ShaderModel6_7 => 67,
            HlslShaderModel::ShaderModel6_8 => 68,
        }
    }
}

impl Default for HlslShaderModel {
    fn default() -> Self {
        HlslShaderModel::ShaderModel3_0
    }
}

impl<'a> Compiler<'a, Hlsl> {}

#[cfg(test)]
mod test {
    use crate::compile::hlsl::CompileOptions;
    use crate::compile::CompilerOptions;
    use spirv_cross_sys::spvc_compiler_create_compiler_options;

    use crate::error::{SpirvCrossError, ToContextError};
    use crate::Compiler;
    use crate::{targets, Module, SpirvCross};

    static BASIC_SPV: &[u8] = include_bytes!("../../../basic.spv");

    #[test]
    pub fn hlsl_opts() -> Result<(), SpirvCrossError> {
        let spv = SpirvCross::new()?;
        let words = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&words));

        let compiler: Compiler<targets::Hlsl> = spv.create_compiler(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        let mut opts_ptr = std::ptr::null_mut();

        unsafe {
            spvc_compiler_create_compiler_options(compiler.ptr.as_ptr(), &mut opts_ptr)
                .ok(&compiler)?;
        }

        // println!("{:#?}", resources);
        let opts = CompileOptions::default();
        unsafe {
            opts.apply(opts_ptr, &compiler)?;
        }

        // match ty.inner {
        //     TypeInner::Struct(ty) => {
        //         compiler.get_type(ty.members[0].id)?;
        //     }
        //     TypeInner::Vector { .. } => {}
        //     _ => {}
        // }
        Ok(())
    }
}
