use crate::compile::CommonCompileOptions;
pub use spirv_cross_sys::MslConstexprSampler as ConstexprSampler;
use spirv_cross_sys::MslIndexType;
pub use spirv_cross_sys::MslResourceBinding2 as ResourceBinding;
pub use spirv_cross_sys::MslSamplerYcbcrConversion as SamplerYcbcrConversion;
pub use spirv_cross_sys::MslShaderInput as ShaderInput;
pub use spirv_cross_sys::MslShaderInterfaceVar2 as ShaderInterfaceVar;
pub use spirv_cross_sys::MslVertexAttribute as VertexAttribute;
use std::fmt::{Debug, Formatter};

use crate::compile::CompilerOptions;
use crate::error::ToContextError;
use crate::ContextRooted;
/// MSL compiler options
#[non_exhaustive]
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CompileOptions {
    /// Compile options common to GLSL, HLSL, and MSL.
    #[expand]
    pub common: CommonCompileOptions,

    /// The MSL version
    #[option(SPVC_COMPILER_OPTION_MSL_VERSION)]
    pub version: MslVersion,

    /// Width of 2D Metal textures used as 1D texel buffers.
    #[option(SPVC_COMPILER_OPTION_MSL_TEXEL_BUFFER_TEXTURE_WIDTH, 4096)]
    pub texel_buffer_texture_width: u32,

    /// Index of the swizzle buffer.
    ///
    /// The default is 30.
    #[option(SPVC_COMPILER_OPTION_MSL_SWIZZLE_BUFFER_INDEX, 30)]
    pub swizzle_buffer_index: u32,

    /// Index of the indirect params buffer.
    ///
    /// The default is 29.
    #[option(SPVC_COMPILER_OPTION_MSL_INDIRECT_PARAMS_BUFFER_INDEX, 29)]
    pub indirect_params_buffer_index: u32,

    /// Index of the shader output buffer.
    ///
    /// The default is 28.
    #[option(SPVC_COMPILER_OPTION_MSL_SHADER_OUTPUT_BUFFER_INDEX, 28)]
    pub shader_output_buffer_index: u32,

    /// Index of the shader patch output buffer.
    ///
    /// The default is 27.
    #[option(SPVC_COMPILER_OPTION_MSL_SHADER_PATCH_OUTPUT_BUFFER_INDEX, 27)]
    pub shader_patch_output_buffer_index: u32,

    /// Index of the shader tesselation factor output buffer.
    ///
    /// The default is 26.
    #[option(SPVC_COMPILER_OPTION_MSL_SHADER_TESS_FACTOR_OUTPUT_BUFFER_INDEX, 26)]
    pub shader_tess_factor_output_buffer_index: u32,

    /// Index of the buffer size buffer.
    ///
    /// The default is 25.
    #[option(SPVC_COMPILER_OPTION_MSL_BUFFER_SIZE_BUFFER_INDEX, 25)]
    pub buffer_size_buffer_index: u32,

    /// Index of the view mask buffer.
    ///
    /// The default is 24
    #[option(SPVC_COMPILER_OPTION_MSL_VIEW_MASK_BUFFER_INDEX, 24)]
    pub view_mask_buffer_index: u32,

    /// Index of the dynamic offsets buffer.
    ///
    /// The default is 23
    #[option(SPVC_COMPILER_OPTION_MSL_DYNAMIC_OFFSETS_BUFFER_INDEX, 23)]
    pub dynamic_offsets_buffer_index: u32,

    /// Index of the shader input buffer.
    ///
    /// The default is 22.
    #[option(SPVC_COMPILER_OPTION_MSL_SHADER_INPUT_BUFFER_INDEX, 22)]
    pub shader_input_buffer_index: u32,

    /// Index of the shader index buffer.
    ///
    /// The default is 21.
    #[option(SPVC_COMPILER_OPTION_MSL_SHADER_INDEX_BUFFER_INDEX, 21)]
    pub shader_index_buffer_index: u32,

    /// Index of the shader patch input buffer.
    ///
    /// The default is 20.
    #[option(SPVC_COMPILER_OPTION_MSL_SHADER_PATCH_INPUT_BUFFER_INDEX, 20)]
    pub shader_patch_input_buffer_index: u32,

    /// Index of the input workgroup index buffer.
    ///
    /// The default is 0
    #[option(SPVC_COMPILER_OPTION_MSL_SHADER_INPUT_WORKGROUP_INDEX, 0)]
    pub shader_input_workgroup_index: u32,

    /// Enable `point_size` builtin.
    #[option(SPVC_COMPILER_OPTION_MSL_ENABLE_POINT_SIZE_BUILTIN, true)]
    pub enable_point_size_builtin: bool,

    /// Enable the `FragDepth` builtin.
    ///
    /// Disable if pipeline does not enable depth, as pipeline
    /// creation might otherwise fail.
    #[option(SPVC_COMPILER_OPTION_MSL_ENABLE_FRAG_DEPTH_BUILTIN, true)]
    pub enable_frag_depth_builtin: bool,

    /// Enable the `FragStencilRef` output.
    ///
    /// Disablle if pipeline does not enable  stencil  output,
    /// as pipeline creation might otherwise fail.
    #[option(SPVC_COMPILER_OPTION_MSL_ENABLE_FRAG_STENCIL_REF_BUILTIN, true)]
    pub enable_frag_stencil_ref_builtin: bool,

    ///
    #[option(SPVC_COMPILER_OPTION_MSL_DISABLE_RASTERIZATION, false)]
    pub disable_rasterization: bool,

    /// Writes geometry varyings to a buffer instead of as stage-outputs.
    #[option(SPVC_COMPILER_OPTION_MSL_CAPTURE_OUTPUT_TO_BUFFER, false)]
    pub capture_output_to_buffer: bool,

    /// Works around lack of support for VkImageView component swizzles.
    /// Recent Metal versions do not require this workaround.
    /// This has a massive impact on performance and bloat.
    ///
    /// Do not use this unless you are absolutely forced to.
    ///
    /// To use this feature, the API side must pass down swizzle buffers.
    /// Should only be used by translation layers as a last resort.
    #[option(SPVC_COMPILER_OPTION_MSL_SWIZZLE_TEXTURE_SAMPLES, false)]
    pub swizzle_texture_samples: bool,

    /// Always emit color outputs as 4-component variables.
    ///
    /// In Metal, the fragment shader must emit at least as many components
    /// as the render target format.
    #[option(SPVC_COMPILER_OPTION_MSL_PAD_FRAGMENT_OUTPUT_COMPONENTS, false)]
    pub pad_fragment_output_components: bool,

    /// Use a lower-left tessellation domain.
    #[option(SPVC_COMPILER_OPTION_MSL_TESS_DOMAIN_ORIGIN_LOWER_LEFT, false)]
    pub tess_domain_origin_lower_left: bool,

    /// The plattform to output MSL for. Defaults to macOS.
    #[option(SPVC_COMPILER_OPTION_MSL_PLATFORM, MetalPlatform::MacOS)]
    pub platform: MetalPlatform,

    /// Enable use of Metal argument buffers.
    ///
    /// MSL 2.0 or higher must be used.
    #[option(SPVC_COMPILER_OPTION_MSL_ARGUMENT_BUFFERS, false)]
    pub argument_buffers: bool,

    /// Defines Metal argument buffer tier levels.
    /// Uses same values as Metal `MTLArgumentBuffersTier` enumeration.
    #[option(
        SPVC_COMPILER_OPTION_MSL_ARGUMENT_BUFFERS_TIER,
        ArgumentBuffersTier::Tier1
    )]
    pub argument_buffers_tier: ArgumentBuffersTier,

    /// Requires MSL 2.1, use the native support for texel buffers.
    #[option(SPVC_COMPILER_OPTION_MSL_TEXTURE_BUFFER_NATIVE, false)]
    pub texture_buffer_native: bool,

    /// Enable SPV_KHR_multiview emulation.
    #[option(SPVC_COMPILER_OPTION_MSL_MULTIVIEW, false)]
    pub multiview: bool,

    /// If disabled, don't set [[render_target_array_index]] in multiview shaders.
    ///
    /// Useful for devices which don't support layered rendering.
    ///
    /// Only effective when [`CompileOptions::multiview`] is enabled.
    #[option(SPVC_COMPILER_OPTION_MSL_MULTIVIEW_LAYERED_RENDERING, true)]
    pub multiview_layered_rendering: bool,

    /// The index of the device
    #[option(SPVC_COMPILER_OPTION_MSL_DEVICE_INDEX, 0)]
    pub device_index: u32,

    /// Treat the view index as the device index instead. For multi-GPU rendering.
    #[option(SPVC_COMPILER_OPTION_MSL_VIEW_INDEX_FROM_DEVICE_INDEX, false)]
    pub view_index_from_device_index: bool,

    /// Add  support for `vkCmdDispatchBase()` or similar APIs.
    ///
    /// Offsets the workgroup ID based on a buffer.
    #[option(SPVC_COMPILER_OPTION_MSL_DISPATCH_BASE, false)]
    pub dispatch_base: bool,

    /// Emit Image variables of dimension Dim1D as `texture2d`.
    ///
    /// In Metal, 1D textures do not support all features that 2D textures do.
    ///
    /// Use this option if your code relies on these features.
    #[option(SPVC_COMPILER_OPTION_MSL_TEXTURE_1D_AS_2D, false)]
    pub texture_1d_as_2d: bool,

    /// Ensures vertex and instance indices start at zero.
    ///
    /// This reflects the behavior of HLSL with SV_VertexID and SV_InstanceID.
    #[option(SPVC_COMPILER_OPTION_MSL_ENABLE_BASE_INDEX_ZERO, false)]
    pub enable_base_index_zero: bool,

    /// Use Metal's native frame-buffer fetch API for subpass inputs.
    #[option(SPVC_COMPILER_OPTION_MSL_FRAMEBUFFER_FETCH_SUBPASS, false)]
    pub framebuffer_fetch_subpass: bool,

    /// Enables use of "fma" intrinsic for invariant float math
    #[option(SPVC_COMPILER_OPTION_MSL_INVARIANT_FP_MATH, false)]
    pub invariant_fp_math: bool,

    /// Emulate texturecube_array with texture2d_array for iOS where this type is not available
    #[option(SPVC_COMPILER_OPTION_MSL_EMULATE_CUBEMAP_ARRAY, false)]
    pub emulate_cubemap_array: bool,

    /// Allow user to enable decoration binding
    #[option(SPVC_COMPILER_OPTION_MSL_ENABLE_DECORATION_BINDING, false)]
    pub enable_decoration_binding: bool,

    /// Forces all resources which are part of an argument buffer to be considered active.
    ///
    /// This ensures ABI compatibility between shaders where some resources might be unused,
    /// and would otherwise declare a different ABI.
    #[option(SPVC_COMPILER_OPTION_MSL_FORCE_ACTIVE_ARGUMENT_BUFFER_RESOURCES, false)]
    pub force_active_argument_buffer_resources: bool,

    /// Forces the use of plain arrays, which works around certain driver bugs on certain versions
    /// of Intel Macbooks.
    ///
    /// See https://github.com/KhronosGroup/SPIRV-Cross/issues/1210.
    /// May reduce performance in scenarios where arrays are copied around as value-types.
    #[option(SPVC_COMPILER_OPTION_MSL_FORCE_NATIVE_ARRAYS, false)]
    pub force_native_arrays: bool,

    /// Only selectively enable fragment outputs.
    ///
    /// Useful if pipeline does not enable
    /// fragment output for certain locations, as pipeline creation might otherwise fail.
    #[option(SPVC_COMPILER_OPTION_MSL_ENABLE_FRAG_OUTPUT_MASK, 0xffffffff)]
    pub enable_frag_output_mask: u32,

    /// If a shader writes clip distance, also emit user varyings which
    /// can be read in subsequent stages.
    #[option(SPVC_COMPILER_OPTION_MSL_ENABLE_CLIP_DISTANCE_USER_VARYING, true)]
    pub enable_clip_distance_user_varying: bool,

    /// In a tessellation control shader, assume that more than one patch can be processed in a
    /// single workgroup. This requires changes to the way the InvocationId and PrimitiveId
    /// builtins are processed, but should result in more efficient usage of the GPU.
    #[option(SPVC_COMPILER_OPTION_MSL_MULTI_PATCH_WORKGROUP, false)]
    pub multi_patch_workgroup: bool,

    /// If set, a vertex shader will be compiled as part of a tessellation pipeline.
    /// It will be translated as a compute kernel, so it can use the global invocation ID
    /// to index the output buffer.
    #[option(SPVC_COMPILER_OPTION_MSL_VERTEX_FOR_TESSELLATION, false)]
    pub vertex_for_tessellation: bool,

    /// The type of index in the index buffer, if present. For a compute shader, Metal
    /// requires specifying the indexing at pipeline creation, rather than at draw time
    /// as with graphics pipelines. This means we must create three different pipelines,
    /// for no indexing, 16-bit indices, and 32-bit indices. Each requires different
    /// handling for the gl_VertexIndex builtin. We may as well, then, create three
    /// different shaders for these three scenarios.
    #[option(SPVC_COMPILER_OPTION_MSL_VERTEX_INDEX_TYPE, IndexType::None)]
    pub vertex_index_type: IndexType,

    /// Assume that SubpassData images have multiple layers. Layered input attachments
    /// are addressed relative to the Layer output from the vertex pipeline. This option
    /// has no effect with multiview, since all input attachments are assumed to be layered
    /// and will be addressed using the current ViewIndex.
    #[option(SPVC_COMPILER_OPTION_MSL_ARRAYED_SUBPASS_INPUT, false)]
    pub arrayed_subpass_input: bool,

    /// The required alignment of linear textures of format `MTLPixelFormatR32Uint`.
    ///
    /// This is used to align the row stride for atomic accesses to such images.
    #[option(SPVC_COMPILER_OPTION_MSL_R32UI_LINEAR_TEXTURE_ALIGNMENT, 4)]
    pub r32ui_linear_texture_alignment: u32,

    /// The function constant ID to use for the linear texture alignment.
    ///
    /// On MSL 1.2 or later, you can override the alignment by setting this function constant.
    #[option(SPVC_COMPILER_OPTION_MSL_R32UI_ALIGNMENT_CONSTANT_ID, 65535)]
    pub r32ui_alignment_constant_id: u32,

    /// Whether to use SIMD-group or quadgroup functions to implement group non-uniform
    /// operations. Some GPUs on iOS do not support the SIMD-group functions, only the
    /// quadgroup functions.
    #[option(SPVC_COMPILER_OPTION_MSL_IOS_USE_SIMDGROUP_FUNCTIONS, false)]
    pub ios_use_simdgroup_functions: bool,

    /// If set, the subgroup size will be assumed to be one, and subgroup-related
    /// builtins and operations will be emitted accordingly.
    ///
    /// This mode is intended to be used by MoltenVK on hardware/software configurations
    /// which do not provide sufficient support for subgroups.
    #[option(SPVC_COMPILER_OPTION_MSL_EMULATE_SUBGROUPS, false)]
    pub emulate_subgroups: bool,

    /// If nonzero, a fixed subgroup size to assume. Metal, similarly to VK_EXT_subgroup_size_control,
    /// allows the SIMD-group size (aka thread execution width) to vary depending on
    /// register usage and requirements.
    ///
    /// In certain circumstances--for example, a pipeline
    /// in MoltenVK without VK_PIPELINE_SHADER_STAGE_CREATE_ALLOW_VARYING_SUBGROUP_SIZE_BIT_EXT--
    /// this is undesirable. This fixes the value of the SubgroupSize builtin, instead of
    /// mapping it to the Metal builtin `[[thread_execution_width]]`. If the thread
    /// execution width is reduced, the extra invocations will appear to be inactive.
    ///
    /// If zero, the SubgroupSize will be allowed to vary, and the builtin will be mapped
    /// to the Metal `[[thread_execution_width]]` builtin.
    #[option(SPVC_COMPILER_OPTION_MSL_FIXED_SUBGROUP_SIZE, 0)]
    pub fixed_subgroup_size: u32,

    /// If set, a dummy `[[sample_id]]` input is added to a fragment shader if none is present.
    ///
    /// This will force the shader to run at sample rate, assuming Metal does not optimize
    /// the extra threads away.
    #[option(SPVC_COMPILER_OPTION_MSL_FORCE_SAMPLE_RATE_SHADING, false)]
    pub force_sample_rate_shading: bool,

    /// Specifies whether the iOS target version supports the `[[base_vertex]]`
    /// and `[[base_instance]]` attributes.
    #[option(SPVC_COMPILER_OPTION_MSL_IOS_SUPPORT_BASE_VERTEX_INSTANCE, false)]
    pub ios_support_base_vertex_instance: bool,

    /// Use storage buffers instead of vertex-style attributes for tessellation evaluation
    /// input.
    ///
    /// This may require conversion of inputs in the generated post-tessellation
    /// vertex shader, but allows the use of nested arrays.
    #[option(SPVC_COMPILER_OPTION_MSL_RAW_BUFFER_TESE_INPUT, false)]
    pub raw_buffer_tese_input: bool,

    /// If set, gl_HelperInvocation will be set manually whenever a fragment is discarded.
    /// Some Metal devices have a bug where `simd_is_helper_thread()` does not return true
    /// after a fragment has been discarded.
    ///
    /// This is a workaround that is only expected to be needed
    /// until the bug is fixed in Metal; it is provided as an option to allow disabling it when that occurs.
    #[option(SPVC_COMPILER_OPTION_MSL_MANUAL_HELPER_INVOCATION_UPDATES, true)]
    pub manual_helper_invocation_updates: bool,

    /// If set, extra checks will be emitted in fragment shaders to prevent writes
    /// from discarded fragments. Some Metal devices have a bug where writes to storage resources
    /// from discarded fragment threads continue to occur, despite the fragment being
    /// discarded.
    ///
    /// This is a workaround that is only expected to be needed until the
    /// bug is fixed in Metal; it is provided as an option so it can be enabled
    /// only when the bug is present.
    #[option(SPVC_COMPILER_OPTION_MSL_CHECK_DISCARDED_FRAG_STORES, false)]
    pub check_discarded_frag_stores: bool,

    /// If set, Lod operands to OpImageSample*DrefExplicitLod for 1D and 2D array images
    /// will be implemented using a gradient instead of passing the level operand directly.
    ///
    /// Some Metal devices have a bug where the `level()` argument to `depth2d_array<T>::sample_compare()`
    /// in a fragment shader is biased by some unknown amount, possibly dependent on the
    /// partial derivatives of the texture coordinates.
    ///
    /// This is a workaround that is only
    /// expected to be needed until the bug is fixed in Metal; it is provided as an option
    /// so it can be enabled only when the bug is present.
    #[option(SPVC_COMPILER_OPTION_MSL_SAMPLE_DREF_LOD_ARRAY_AS_GRAD, false)]
    pub sample_dref_lod_array_as_grad: bool,

    /// MSL doesn't guarantee coherence between writes and subsequent reads of read_write textures.
    /// This inserts fences before each read of a read_write texture to ensure coherency.
    /// If you're sure you never rely on this, you can set this to false for a possible performance improvement.
    /// Note: Only Apple's GPU compiler takes advantage of the lack of coherency, so make sure to test on Apple GPUs if you disable this.
    #[option(SPVC_COMPILER_OPTION_MSL_READWRITE_TEXTURE_FENCES, true)]
    pub readwrite_texture_fences: bool,

    /// Metal 3.1 introduced a Metal regression bug which causes infinite recursion during
    /// Metal's analysis of an entry point input structure that is itself recursive. Enabling
    /// this option will replace the recursive input declaration with a alternate variable of
    /// type void*, and then cast to the correct type at the top of the entry point function.
    /// The bug has been reported to Apple, and will hopefully be fixed in future releases.
    #[option(SPVC_COMPILER_OPTION_MSL_REPLACE_RECURSIVE_INPUTS, false)]
    pub replace_recursive_inputs: bool,

    /// If set, manual fixups of gradient vectors for cube texture lookups will be performed.
    /// All released Apple Silicon GPUs to date behave incorrectly when sampling a cube texture
    /// with explicit gradients. They will ignore one of the three partial derivatives based
    /// on the selected major axis, and expect the remaining derivatives to be partially
    /// transformed.
    #[option(SPVC_COMPILER_OPTION_MSL_AGX_MANUAL_CUBE_GRAD_FIXUP, false)]
    pub agx_manual_cube_grad_fixup: bool,

    /// Metal will discard fragments with side effects under certain circumstances prematurely.
    /// Example: CTS test dEQP-VK.fragment_operations.early_fragment.discard_no_early_fragment_tests_depth
    /// Test will render a full screen quad with varying depth [0,1] for each fragment.
    /// Each fragment will do an operation with side effects, modify the depth value and
    /// discard the fragment. The test expects the fragment to be run due to:
    /// https://registry.khronos.org/vulkan/specs/1.0-extensions/html/vkspec.html#fragops-shader-depthreplacement
    /// which states that the fragment shader must be run due to replacing the depth in shader.
    ///
    /// However, Metal may prematurely discards fragments without executing them
    /// (I believe this to be due to a greedy optimization on their end) making the test fail.
    ///
    /// This option enforces fragment execution for such cases where the fragment has operations
    /// with side effects. Provided as an option hoping Metal will fix this issue in the future.
    #[option(
        SPVC_COMPILER_OPTION_MSL_FORCE_FRAGMENT_WITH_SIDE_EFFECTS_EXECUTION,
        false
    )]
    pub force_fragment_with_side_effects_execution: bool,
}

#[derive(Copy, Clone)]
pub struct MslVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Default for MslVersion {
    fn default() -> Self {
        MslVersion::from((1, 2))
    }
}

impl Debug for MslVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MslVersion({}.{}.{})",
            self.major, self.minor, self.patch
        )
    }
}

impl From<MslVersion> for u32 {
    fn from(value: MslVersion) -> Self {
        (value.major * 10000) + (value.minor * 100) + value.patch
    }
}

impl From<(u32, u32)> for MslVersion {
    fn from(value: (u32, u32)) -> Self {
        Self {
            major: value.0,
            minor: value.1,
            patch: 0,
        }
    }
}

impl From<(u32, u32, u32)> for MslVersion {
    fn from(value: (u32, u32, u32)) -> Self {
        Self {
            major: value.0,
            minor: value.1,
            patch: value.2,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum ArgumentBuffersTier {
    Tier1 = 0,
    Tier2 = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum MetalPlatform {
    #[allow(non_camel_case_types)]
    iOS = 0,
    MacOS = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum IndexType {
    None = 0,
    Uint16 = 1,
    Uint32 = 2,
}

impl From<MetalPlatform> for u32 {
    fn from(value: MetalPlatform) -> Self {
        match value {
            MetalPlatform::iOS => 0,
            MetalPlatform::MacOS => 1,
        }
    }
}

impl From<IndexType> for u32 {
    fn from(value: IndexType) -> Self {
        match value {
            IndexType::None => 0,
            IndexType::Uint16 => 1,
            IndexType::Uint32 => 1,
        }
    }
}

impl From<ArgumentBuffersTier> for u32 {
    fn from(value: ArgumentBuffersTier) -> Self {
        match value {
            ArgumentBuffersTier::Tier1 => 0,
            ArgumentBuffersTier::Tier2 => 1,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compile::msl::CompileOptions;
    use crate::compile::CompilerOptions;
    use spirv_cross_sys::spvc_compiler_create_compiler_options;

    use crate::error::{SpirvCrossError, ToContextError};
    use crate::Compiler;
    use crate::{targets, Module, SpirvCross};

    static BASIC_SPV: &[u8] = include_bytes!("../../../basic.spv");

    #[test]
    pub fn msl_opts() -> Result<(), SpirvCrossError> {
        let spv = SpirvCross::new()?;
        let words = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&words));

        let compiler: Compiler<targets::Msl> = spv.create_compiler(words)?;
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
