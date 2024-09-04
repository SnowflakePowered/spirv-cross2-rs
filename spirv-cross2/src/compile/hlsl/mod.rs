use crate::compile::{CommonOptions, CompiledArtifact};
use crate::targets::Hlsl;
use crate::{error, spirv, Compiler};
use bitflags::bitflags;

use spirv_cross_sys::HlslResourceBinding;
use spirv_cross_sys::HlslResourceBindingMapping;

/// Specifies an HLSL root constant layout.
pub use spirv_cross_sys::HlslRootConstants as RootConstants;

use crate::error::ToContextError;
use crate::handle::{Handle, VariableId};
use crate::sealed::Sealed;
use crate::string::ContextStr;
use crate::ContextRooted;
use spirv_cross_sys as sys;
use spirv_cross_sys::{HlslBindingFlagBits, HlslBindingFlags, HlslVertexAttributeRemap};

bitflags! {
    /// Controls how resource bindings are declared in the output HLSL.
    ///
    /// For finer control, decorations may be removed from specific resources instead.
    pub struct BindingFlags: u32 {
        /// No auto binding of resources.
        const AUTO_NONE = HlslBindingFlagBits::SPVC_HLSL_BINDING_AUTO_NONE_BIT.0 as u32;
        /// Push constant (root constant) resources will be declared as CBVs (b-space) without a `register()` declaration.
        ///
        /// A register will be automatically assigned by the D3D compiler, but must therefore be reflected in D3D-land.
        /// Push constants do not normally have a `DecorationBinding` set, but if they do, this can be used to ignore it.
        const AUTO_PUSH_CONSTANT = HlslBindingFlagBits::SPVC_HLSL_BINDING_AUTO_PUSH_CONSTANT_BIT.0 as u32;
        /// cbuffer resources will be declared as CBVs (b-space) without a `register()` declaration.
        ///
        /// A register will be automatically assigned, but must be reflected in D3D-land.
        const AUTO_CBV = HlslBindingFlagBits::SPVC_HLSL_BINDING_AUTO_CBV_BIT.0 as u32;
        /// All SRVs (t-space) will be declared without a `register()` declaration.
        const AUTO_SRV = HlslBindingFlagBits::SPVC_HLSL_BINDING_AUTO_SRV_BIT.0 as u32;
        /// All UAVs (u-space) will be declared without a `register()` declaration.
        const AUTO_UAV = HlslBindingFlagBits::SPVC_HLSL_BINDING_AUTO_UAV_BIT.0 as u32;
        /// All samplers (s-space) will be declared without a `register()` declaration.
        const AUTO_SAMPLER = HlslBindingFlagBits::SPVC_HLSL_BINDING_AUTO_SAMPLER_BIT.0 as u32;
        /// No resources will be declared with `register()`.
        const AUTO_ALL = HlslBindingFlagBits::SPVC_HLSL_BINDING_AUTO_ALL.0 as u32;
    }
}

impl Sealed for CompilerOptions {}
/// HLSL compiler options
#[non_exhaustive]
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CompilerOptions {
    /// Compile options common to GLSL, HLSL, and MSL.
    #[expand]
    pub common: CommonOptions,

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
    ///
    /// Set to false if you know you will never use base instance or base vertex
    /// functionality as it might remove an internal cbuffer.
    #[option(
        SPVC_COMPILER_OPTION_HLSL_SUPPORT_NONZERO_BASE_VERTEX_BASE_INSTANCE,
        false
    )]
    pub support_nonzero_base_vertex_base_instance: bool,

    /// Forces a storage buffer to always be declared as UAV, even if the readonly decoration is used.
    /// By default, a readonly storage buffer will be declared as ByteAddressBuffer (SRV) instead.
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

    /// Preserve (RW)StructuredBuffer types if the input source was HLSL.
    ///
    /// This relies on UserTypeGOOGLE to encode the buffer type either as `structuredbuffer or `rwstructuredbuffer
    /// whereas the type can be extended with an optional subtype, e.g. `structuredbuffer:int`.
    #[option(SPVC_COMPILER_OPTION_HLSL_PRESERVE_STRUCTURED_BUFFERS, false)]
    pub preserve_structured_buffers: bool,
}

/// HLSL Shader model.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
#[derive(Default)]
pub enum HlslShaderModel {
    /// Shader Model 3.0 (Direct3D 9.0c).
    ///
    /// This is the lowest supported shader model.
    #[default]
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

/// Pipeline binding information for a resource.
///
/// Used to map a SPIR-V resource to an HLSL buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ResourceBinding {
    /// A resource with a qualified layout.
    ///
    /// i.e. `layout(set = 0, binding = 1)` in GLSL.
    Qualified {
        /// The descriptor set of the qualified layout.
        set: u32,
        /// The binding number of the qualified layout.
        binding: u32,
    },
    /// The push constant buffer.
    PushConstant,
}

impl ResourceBinding {
    /// Create a resource binding for a qualified SPIR-V layout
    /// specifier.
    pub const fn from_qualified(set: u32, binding: u32) -> Self {
        Self::Qualified { set, binding }
    }

    const fn descriptor_set(&self) -> u32 {
        const PUSH_CONSTANT_DESCRIPTOR_SET: u32 = !0;
        match self {
            ResourceBinding::Qualified { set, .. } => *set,
            ResourceBinding::PushConstant => PUSH_CONSTANT_DESCRIPTOR_SET,
        }
    }

    const fn binding(&self) -> u32 {
        const PUSH_CONSTANT_BINDING: u32 = 0;
        match self {
            ResourceBinding::Qualified { binding, .. } => *binding,
            ResourceBinding::PushConstant => PUSH_CONSTANT_BINDING,
        }
    }
}

/// The HLSL buffer to bind a resource to.
///
/// A single SPIR-V binding may bind to multiple
/// registers for multiple resource types.
///
/// For example, for combined image samplers, both the `srv` and `sampler`
/// bindings will be considered.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindTarget {
    /// Bind to a constant buffer view register.
    pub cbv: Option<RegisterBinding>,
    /// Bind to an unordered access view register.
    pub uav: Option<RegisterBinding>,
    /// Bind to a shader resource view register.
    pub srv: Option<RegisterBinding>,
    /// Bind to a sampler register.
    pub sampler: Option<RegisterBinding>,
}

/// An HLSL register location.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RegisterBinding {
    /// The register for this binding.
    pub register: u32,
    /// The address space index for this binding.
    ///
    /// This is ignored on Shader Model 5.0 and below.
    pub space: u32,
}

impl From<RegisterBinding> for HlslResourceBindingMapping {
    #[inline(always)]
    fn from(value: RegisterBinding) -> Self {
        HlslResourceBindingMapping {
            register_space: value.space,
            register_binding: value.register,
        }
    }
}

/// HLSL specific APIs.
impl Compiler<'_, Hlsl> {
    /// Add a resource binding to the HLSL compilation.
    ///
    /// Register bindings are set based on whether the HLSL resource is a
    /// CBV, UAV, SRV or Sampler.
    ///
    /// A single binding in SPIR-V might contain multiple
    /// resource types, e.g. `COMBINED_IMAGE_SAMPLER`, and SRV/Sampler bindings will be used respectively.
    ///
    /// On SM 5.0 and lower, `register_space` is ignored.
    ///
    /// For deeper control of push constants, [`Compiler<Hlsl>::set_root_constant_layout`] can be used instead.
    ///
    /// If resource bindings are provided, [`CompiledArtifact<Hlsl>::is_resource_used`] will return true if
    /// the set/binding combination was used by the HLSL code.
    pub fn add_resource_binding(
        &mut self,
        stage: spirv::ExecutionModel,
        binding: ResourceBinding,
        bind_target: &BindTarget,
    ) -> error::Result<()> {
        const DEFAULT_BINDING: HlslResourceBindingMapping = HlslResourceBindingMapping {
            register_space: 0,
            register_binding: 0,
        };

        let hlsl_resource_binding = HlslResourceBinding {
            stage,
            desc_set: binding.descriptor_set(),
            binding: binding.binding(),
            cbv: bind_target.cbv.map_or(DEFAULT_BINDING, From::from),
            uav: bind_target.uav.map_or(DEFAULT_BINDING, From::from),
            srv: bind_target.srv.map_or(DEFAULT_BINDING, From::from),
            sampler: bind_target.sampler.map_or(DEFAULT_BINDING, From::from),
        };

        unsafe {
            sys::spvc_compiler_hlsl_add_resource_binding(self.ptr.as_ptr(), &hlsl_resource_binding)
                .ok(&*self)
        }
    }

    /// Compiles and remap vertex attribute at specific locations to a fixed semantic.
    ///
    /// The default is `TEXCOORD#` where # denotes location.
    /// Matrices are unrolled to vectors with notation `${SEMANTIC}_#`, where # denotes row.
    /// `$SEMANTIC` is either `TEXCOORD#` or a semantic name specified here.
    pub fn remap_vertex_attribute<'str>(
        &mut self,
        location: u32,
        semantic: impl Into<ContextStr<'str>>,
    ) -> error::Result<()> {
        let str = semantic.into();
        let semantic = str.into_cstring_ptr()?;

        let remap = HlslVertexAttributeRemap {
            location,
            semantic: semantic.as_ptr(),
        };

        unsafe {
            sys::spvc_compiler_hlsl_add_vertex_attribute_remap(self.ptr.as_ptr(), &remap, 1)
                .ok(&*self)
        }
    }

    /// Optionally specify a custom root constant layout.
    ///
    /// Push constants ranges will be split up according to the
    /// layout specified.
    pub fn set_root_constant_layout(
        &mut self,
        constant_info: &[RootConstants],
    ) -> error::Result<()> {
        unsafe {
            sys::spvc_compiler_hlsl_set_root_constants_layout(
                self.ptr.as_ptr(),
                constant_info.as_ptr(),
                constant_info.len(),
            )
            .ok(&*self)
        }
    }

    /// Controls how resource bindings are declared in the output HLSL.
    pub fn set_resource_binding_flags(&mut self, flags: BindingFlags) -> error::Result<()> {
        unsafe {
            sys::spvc_compiler_hlsl_set_resource_binding_flags(
                self.ptr.as_ptr(),
                HlslBindingFlags(flags.bits()),
            )
            .ok(&*self)
        }
    }

    /// This is a special HLSL workaround for the NumWorkGroups builtin.
    /// This does not exist in HLSL, so the calling application must create a dummy cbuffer in
    /// which the application will store this builtin.
    ///
    /// The cbuffer layout will be:
    ///
    /// ```hlsl
    ///  cbuffer SPIRV_Cross_NumWorkgroups : register(b#, space#) {
    ///     uint3 SPIRV_Cross_NumWorkgroups_count;
    /// };
    /// ```
    ///
    /// This must be called before [`Compiler::compile`] if the `NumWorkgroups` builtin is used,
    /// or compilation will fail.
    ///
    /// The function returns None if NumWorkGroups builtin is not statically used in the shader
    /// from the current entry point.
    ///
    /// If Some, returns the variable ID of a cbuffer which corresponds to
    /// the cbuffer declared above.
    ///
    /// By default, no binding or descriptor set decoration is set,
    /// so the calling application should declare explicit bindings on this ID before calling
    /// [`Compiler::compile`].
    pub fn remap_num_workgroups_builtin(&mut self) -> Option<Handle<VariableId>> {
        unsafe {
            let id = sys::spvc_compiler_hlsl_remap_num_workgroups_builtin(self.ptr.as_ptr());
            self.create_handle_if_not_zero(id)
        }
    }

    /// Mask a stage output by location.
    ///
    /// If a shader output is active in this stage, but inactive in a subsequent stage,
    /// this can be signalled here. This can be used to work around certain cross-stage matching problems
    /// which plagues HLSL in certain scenarios.
    ///
    /// An output which matches one of these will not be emitted in stage output interfaces, but rather treated as a private
    /// variable.
    ///
    /// This option is only meaningful for MSL and HLSL, since GLSL matches by location directly.
    ///
    pub fn mask_stage_output_by_location(
        &mut self,
        location: u32,
        component: u32,
    ) -> crate::error::Result<()> {
        unsafe {
            sys::spvc_compiler_mask_stage_output_by_location(self.ptr.as_ptr(), location, component)
                .ok(&*self)
        }
    }

    /// Mask a stage output by builtin. Masking builtins only takes effect if the builtin in question is part of the stage output interface.
    ///
    /// If a shader output is active in this stage, but inactive in a subsequent stage,
    /// this can be signalled here. This can be used to work around certain cross-stage matching problems
    /// which plagues HLSL in certain scenarios.
    ///
    /// An output which matches one of these will not be emitted in stage output interfaces, but rather treated as a private
    /// variable.
    ///
    /// This option is only meaningful for MSL and HLSL, since GLSL matches by location directly.
    /// Masking builtins only takes effect if the builtin in question is part of the stage output interface.
    pub fn mask_stage_output_by_builtin(
        &mut self,
        builtin: spirv::BuiltIn,
    ) -> crate::error::Result<()> {
        unsafe {
            sys::spvc_compiler_mask_stage_output_by_builtin(self.ptr.as_ptr(), builtin).ok(&*self)
        }
    }
}

impl CompiledArtifact<'_, Hlsl> {
    /// Returns whether the set/binding combination provided in [`Compiler<Hlsl>::add_resource_binding`]
    /// was used.
    pub fn is_resource_used(&self, model: spirv::ExecutionModel, binding: ResourceBinding) -> bool {
        unsafe {
            sys::spvc_compiler_hlsl_is_resource_used(
                self.compiler.ptr.as_ptr(),
                model,
                binding.descriptor_set(),
                binding.binding(),
            )
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compile::hlsl::CompilerOptions;
    use spirv_cross_sys::spvc_compiler_create_compiler_options;

    use crate::compile::sealed::ApplyCompilerOptions;
    use crate::error::{SpirvCrossError, ToContextError};
    use crate::Compiler;
    use crate::{targets, Module, SpirvCrossContext};

    static BASIC_SPV: &[u8] = include_bytes!("../../../basic.spv");

    #[test]
    pub fn hlsl_opts() -> Result<(), SpirvCrossError> {
        let spv = SpirvCrossContext::new()?;
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
        let opts = CompilerOptions::default();
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
