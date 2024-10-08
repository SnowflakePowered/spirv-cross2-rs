use crate::compile::{CommonOptions, CompiledArtifact};
use spirv_cross_sys as sys;

/// An MSL `constexpr` inlined sampler.
pub use spirv_cross_sys::MslConstexprSampler as ConstexprSampler;

/// Sampler addressing mode for inlined samplers.
pub use spirv_cross_sys::MslSamplerAddress as SamplerAddress;
/// Sampler border color for inlined samplers.
pub use spirv_cross_sys::MslSamplerBorderColor as SamplerBorderColor;
/// Sampler compare function for inline samplers.
pub use spirv_cross_sys::MslSamplerCompareFunc as SamplerCompareFunc;
/// Whether texture coordinates are normalized or not for inline samplers.
pub use spirv_cross_sys::MslSamplerCoord as SamplerCoord;
/// Mag and min filter mode for inline samplers.
pub use spirv_cross_sys::MslSamplerFilter as SamplerFilter;
/// Mip filtering mode for inline samplers.
pub use spirv_cross_sys::MslSamplerMipFilter as SamplerMipFilter;

/// Y′CbCr chroma location.
pub use spirv_cross_sys::MslChromaLocation as YcbcrChromaLocation;
/// Y′CbCr conversion component swizzle.
pub use spirv_cross_sys::MslComponentSwizzle as YcbcrComponentSwizzle;
/// Y′CbCr conversion format resolution.
pub use spirv_cross_sys::MslFormatResolution as YcbcrFormatResolution;
/// Y′CbCr model conversion options for inline samplers.
pub use spirv_cross_sys::MslSamplerYcbcrConversion as SamplerYcbcrConversion;
/// Format to convert when converting Y′CbCr.
pub use spirv_cross_sys::MslSamplerYcbcrModelConversion as YcbcrTargetFormat;

/// Conversion range for Y′CbCr.
pub use spirv_cross_sys::MslSamplerYcbcrRange as YcbcrConversionRange;

/// Indicates the format of a shader interface variable. Currently limited to specifying
/// if the input is an 8-bit unsigned integer, 16-bit unsigned integer, or
/// some other format.
pub use spirv_cross_sys::MslShaderVariableFormat as ShaderVariableFormat;

/// Indicates the rate at which a variable changes value, one of: per-vertex,
/// per-primitive, or per-patch.
pub use spirv_cross_sys::MslShaderVariableRate as ShaderVariableRate;

/// Maximum number of argument buffers supported.
pub const MAX_ARGUMENT_BUFFERS: u32 = 8;

use crate::error::ToContextError;
use crate::handle::{Handle, VariableId};
use crate::sealed::Sealed;
use crate::string::CompilerStr;
use crate::targets::Msl;
use crate::{error, Compiler, ContextRooted};
use spirv_cross_sys::{MslResourceBinding2, MslShaderInterfaceVar2, SpvBuiltIn, SpvExecutionModel};
use std::fmt::{Debug, Formatter};
use std::num::NonZeroU32;

impl Sealed for CompilerOptions {}
/// MSL compiler options
#[non_exhaustive]
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CompilerOptions {
    /// Compile options common to GLSL, HLSL, and MSL.
    #[expand]
    pub common: CommonOptions,

    /// The MSL version to compile to.
    ///
    /// Defaults to MSL 1.2.
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

    /// If disabled, don't set `[[render_target_array_index]]` in multiview shaders.
    ///
    /// Useful for devices which don't support layered rendering.
    ///
    /// Only effective when [`CompilerOptions::multiview`] is enabled.
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
    /// See <https://github.com/KhronosGroup/SPIRV-Cross/issues/1210>.
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
    /// Test will render a full screen quad with varying depth `[0,1]` for each fragment.
    /// Each fragment will do an operation with side effects, modify the depth value and
    /// discard the fragment. The test expects the fragment to be run due to:
    /// <https://registry.khronos.org/vulkan/specs/1.0-extensions/html/vkspec.html#fragops-shader-depthreplacement>
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

/// The version of Metal Shading Language to compile to.
///
/// Defaults to MSL 1.2.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MslVersion {
    /// The major version of MSL.
    pub major: u32,
    /// The minor version of MSL.
    pub minor: u32,
    /// The patch version of MSL.
    pub patch: u32,
}

impl MslVersion {
    /// Create a new `MslVersion`.
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
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

/// When using Metal argument buffers, indicates the Metal argument buffer tier level supported by the Metal platform.
///
/// Tier capabilities based on recommendations from Apple engineering.
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum ArgumentBuffersTier {
    /// Tier1 supports writable images on macOS, but not on iOS.
    Tier1 = 0,
    /// Tier2 supports writable images on macOS and iOS, and higher resource count limits.
    Tier2 = 1,
}

/// The platform that the Metal runtime will be on.
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum MetalPlatform {
    #[allow(non_camel_case_types)]
    /// iOS (mobile and iPad)
    iOS = 0,
    /// macOS (Desktop)
    MacOS = 1,
}

/// The type of index in the index buffer.
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum IndexType {
    /// No index
    None = 0,
    /// Uint16 indices.
    Uint16 = 1,
    /// Uint32 indices.
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

/// Buffers that need to be provided to the MSL shader.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct BufferRequirements {
    /// Whether an auxiliary swizzle buffer is needed by the shader.
    pub needs_swizzle_buffer: bool,
    /// Whether a buffer containing `STORAGE_BUFFER` buffer sizes to support OpArrayLength
    /// is needed by the shader.
    pub needs_buffer_size_buffer: bool,
    /// Whether an output buffer is needed by the shader.
    pub needs_output_buffer: bool,
    /// Whether a patch output buffer is needed by the shader.
    pub needs_patch_output_buffer: bool,
    /// Whether an input threadgroup buffer is needed by the shader.
    pub needs_input_threadgroup_buffer: bool,
}

/// Pipeline binding information for a resource.
///
/// Used to map a SPIR-V resource to an MSL buffer.
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
    PushConstantBuffer,
    /// The swizzle buffer, at the given index.
    SwizzleBuffer(u32),
    /// The buffer binding for buffer size
    /// buffers to support `OpArrayLength`.
    BufferSizeBuffer(u32),
    /// The argument buffer, at the given index.
    ///
    /// This buffer binding should be kept as small as possible as all automatic bindings for buffers
    /// will start at `max(ResourceBinding::ArgumentBuffer) + 1`.
    ArgumentBuffer(u32),
}

impl ResourceBinding {
    /// Create a resource binding for a qualified SPIR-V layout
    /// specifier.
    pub const fn from_qualified(set: u32, binding: u32) -> Self {
        ResourceBinding::Qualified { set, binding }
    }

    const fn descriptor_set(&self) -> u32 {
        const PUSH_CONSTANT_DESCRIPTOR_SET: u32 = !0;
        match self {
            ResourceBinding::Qualified { set, .. }
            | ResourceBinding::SwizzleBuffer(set)
            | ResourceBinding::BufferSizeBuffer(set)
            | ResourceBinding::ArgumentBuffer(set) => *set,
            ResourceBinding::PushConstantBuffer => PUSH_CONSTANT_DESCRIPTOR_SET,
        }
    }

    const fn binding(&self) -> u32 {
        const PUSH_CONSTANT_BINDING: u32 = 0;
        const SWIZZLE_BUFFER_BINDING: u32 = !1;
        const BUFFER_SIZE_BUFFER_BINDING: u32 = !2;
        const ARGUMENT_BUFFER_BINDING: u32 = !3;

        match self {
            ResourceBinding::Qualified { binding, .. } => *binding,
            ResourceBinding::PushConstantBuffer => PUSH_CONSTANT_BINDING,
            ResourceBinding::SwizzleBuffer(_) => SWIZZLE_BUFFER_BINDING,
            ResourceBinding::BufferSizeBuffer(_) => BUFFER_SIZE_BUFFER_BINDING,
            ResourceBinding::ArgumentBuffer(_) => ARGUMENT_BUFFER_BINDING,
        }
    }
}

/// The MSL target to bind a resource to.
///
/// A single SPIR-V binding may bind to multiple
/// registers for multiple resource types.
///
/// The count field indicates the number of resources consumed by this binding, if the binding
/// represents an array of resources.
///
/// If the resource array is a run-time-sized array, which are legal in GLSL or SPIR-V, this value
/// will be used to declare the array size in MSL, which does not support run-time-sized arrays.
///
/// If using MSL 2.0 argument buffers (for iOS only) the resource is not a storage image,
/// the binding reference we remap to will become an `[[id(N)]]` attribute within
/// the argument buffer index specified in
/// [`ResourceBinding::ArgumentBuffer`].
///
/// For resources which are bound in the "classic" MSL 1.0 way or discrete descriptors, the remap will
/// become a `[[buffer(N)]]`, `[[texture(N)]]` or `[[sampler(N)]]` depending on the resource types used.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindTarget {
    /// The buffer index to bind to, if applicable.
    pub buffer: u32,
    /// The texture index to bind to, if applicable.
    pub texture: u32,
    /// The sampler index to bind to, if applicable.
    pub sampler: u32,
    /// The number of resources consumed by this binding,
    /// if the binding is an array of resources.
    pub count: Option<NonZeroU32>,
}

/// Defines MSL characteristics of a shader interface variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderInterfaceVariable {
    /// The builtin for the variable, if any.
    pub builtin: Option<spirv::BuiltIn>,
    /// The `vecsize` for this variable, if applicable.
    ///
    /// If `vecsize` is Some, it must be greater than or equal to the `vecsize` declared in the shader,
    /// or behavior in the generated shader is undefined.
    pub vecsize: Option<NonZeroU32>,
    /// The format of a shader interface variable.
    pub format: ShaderVariableFormat,
    /// Indicates the rate at which a variable changes value.
    pub rate: ShaderVariableRate,
}

impl ShaderInterfaceVariable {
    /// We need to be maybeuninit, because None builtin is represented by i32::MAX,
    /// which is invalid in Rust. I don't want to expose it just for this, so we'll just
    /// do some magic.
    #[must_use]
    fn to_raw(&self, location: u32) -> MslShaderInterfaceVar2 {
        let mut base = MslShaderInterfaceVar2 {
            location,
            format: self.format,
            builtin: SpvBuiltIn::Position,
            vecsize: self.vecsize.map_or(0, NonZeroU32::get),
            rate: self.rate,
        };

        if let Some(builtin) = self.builtin {
            // happy path, we can just set the builtin.
            base.builtin = SpvBuiltIn(builtin as u32 as i32);
        } else {
            base.builtin = SpvBuiltIn(i32::MAX);
        }

        base
    }
}

/// MSL specific APIs.
impl Compiler<Msl> {
    /// Get whether the vertex shader requires rasterization to be disabled.
    pub fn is_rasterization_disabled(&self) -> bool {
        unsafe { sys::spvc_compiler_msl_is_rasterization_disabled(self.ptr.as_ptr()) }
    }

    /// Get information such as required buffers for the MSL shader
    pub fn buffer_requirements(&self) -> BufferRequirements {
        unsafe {
            let needs_swizzle_buffer =
                sys::spvc_compiler_msl_needs_swizzle_buffer(self.ptr.as_ptr());
            let needs_buffer_size_buffer =
                sys::spvc_compiler_msl_needs_buffer_size_buffer(self.ptr.as_ptr());
            let needs_output_buffer = sys::spvc_compiler_msl_needs_output_buffer(self.ptr.as_ptr());
            let needs_patch_output_buffer =
                sys::spvc_compiler_msl_needs_patch_output_buffer(self.ptr.as_ptr());
            let needs_input_threadgroup_buffer =
                sys::spvc_compiler_msl_needs_input_threadgroup_mem(self.ptr.as_ptr());

            BufferRequirements {
                needs_swizzle_buffer,
                needs_buffer_size_buffer,
                needs_output_buffer,
                needs_patch_output_buffer,
                needs_input_threadgroup_buffer,
            }
        }
    }

    /// Add a shader interface variable description used to fix up shader input variables.
    ///
    /// If shader inputs are provided, [`CompiledArtifact::is_shader_input_used`] will return true after
    /// calling [`Compiler::compile`] if the location were used by the MSL code.
    ///
    /// Note: this covers the functionality implemented by the SPIR-V Cross
    /// C API `spvc_compiler_msl_add_vertex_attribute`.
    pub fn add_shader_input(
        &mut self,
        location: u32,
        variable: &ShaderInterfaceVariable,
    ) -> error::Result<()> {
        let variable = variable.to_raw(location);
        unsafe {
            sys::spvc_compiler_msl_add_shader_input_2(self.ptr.as_ptr(), &variable).ok(&*self)
        }
    }

    /// Add a shader interface variable description used to fix up shader output variables.
    ///
    /// If shader outputs are provided, [`CompiledArtifact::is_shader_input_used`] will return true after
    /// calling [`Compiler::compile`] if the location were used by the MSL code.
    ///
    /// Note: this covers the functionality implemented by the SPIR-V Cross
    /// C API `spvc_compiler_msl_add_vertex_attribute`.
    pub fn add_shader_output(
        &mut self,
        location: u32,
        variable: &ShaderInterfaceVariable,
    ) -> error::Result<()> {
        let variable = variable.to_raw(location);
        unsafe {
            sys::spvc_compiler_msl_add_shader_output_2(self.ptr.as_ptr(), &variable).ok(&*self)
        }
    }

    /// Add a resource binding to indicate the MSL buffer, texture or sampler index to use for a
    /// particular resource.
    ///
    /// If resource bindings are provided,
    /// [`CompiledArtifact<Msl>::is_resource_used`] will return true after [`Compiler::compile`] if
    /// the set/binding combination was used by the MSL code.
    pub fn add_resource_binding(
        &mut self,
        stage: spirv::ExecutionModel,
        binding: ResourceBinding,
        bind_target: &BindTarget,
    ) -> error::Result<()> {
        let binding = MslResourceBinding2 {
            stage: SpvExecutionModel(stage as u32 as i32),
            desc_set: binding.descriptor_set(),
            binding: binding.binding(),
            count: bind_target.count.map_or(0, NonZeroU32::get),
            msl_buffer: bind_target.buffer,
            msl_texture: bind_target.texture,
            msl_sampler: bind_target.sampler,
        };
        unsafe {
            sys::spvc_compiler_msl_add_resource_binding_2(self.ptr.as_ptr(), &binding).ok(&*self)
        }
    }

    /// When using MSL argument buffers, we can force "classic" MSL 1.0 binding schemes for certain descriptor sets.
    /// This corresponds to VK_KHR_push_descriptor in Vulkan.
    pub fn add_discrete_descriptor_set(&mut self, desc_set: u32) -> error::Result<()> {
        unsafe {
            sys::spvc_compiler_msl_add_discrete_descriptor_set(self.ptr.as_ptr(), desc_set)
                .ok(&*self)
        }
    }

    /// This function marks a resource as using a dynamic offset
    /// (`VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC` or `VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC`).
    ///
    /// `desc_set` and `binding` are the SPIR-V descriptor set and binding of a buffer resource
    /// in this shader.
    ///
    /// `index` is the index within the dynamic offset buffer to use.
    ///
    /// This function only has any effect if argument buffers are enabled.
    /// If so, the buffer will have its address adjusted at the beginning of the shader with
    /// an offset taken from the dynamic offset buffer.
    pub fn add_dynamic_buffer(
        &mut self,
        desc_set: u32,
        binding: u32,
        index: u32,
    ) -> error::Result<()> {
        unsafe {
            sys::spvc_compiler_msl_add_dynamic_buffer(self.ptr.as_ptr(), desc_set, binding, index)
                .ok(&*self)
        }
    }

    /// This function marks a resource an inline uniform block
    /// (`VK_DESCRIPTOR_TYPE_INLINE_UNIFORM_BLOCK_EXT`)
    ///
    /// `desc_set` and `binding` are the SPIR-V descriptor set and binding of a buffer resource
    /// in this shader.
    ///
    /// This function only has any effect if argument buffers are enabled.
    /// If so, the buffer block will be directly embedded into the argument
    /// buffer, instead of being referenced indirectly via pointer.
    pub fn add_inline_uniform_block(&mut self, desc_set: u32, binding: u32) -> error::Result<()> {
        unsafe {
            sys::spvc_compiler_msl_add_inline_uniform_block(self.ptr.as_ptr(), desc_set, binding)
                .ok(&*self)
        }
    }

    /// If an argument buffer is large enough, it may need to be in the device storage space rather than
    /// constant. Opt-in to this behavior here on a per-set basis.
    pub fn set_argument_buffer_device_address_space(
        &mut self,
        desc_set: u32,
        device_address: bool,
    ) -> error::Result<()> {
        unsafe {
            sys::spvc_compiler_msl_set_argument_buffer_device_address_space(
                self.ptr.as_ptr(),
                desc_set,
                device_address,
            )
            .ok(&*self)
        }
    }

    /// Remap a sampler with ID to a constexpr sampler.
    /// Older iOS targets must use constexpr samplers in certain cases (PCF),
    /// so a static sampler must be used.
    ///
    /// The sampler will not consume a binding, but be declared in the entry point as a constexpr sampler.
    /// This can be used on both combined image/samplers (sampler2D) or standalone samplers.
    /// The remapped sampler must not be an array of samplers.
    ///
    /// Prefer [`Compiler<Msl>::remap_constexpr_sampler_by_binding`] unless you're also doing reflection anyways.
    pub fn remap_constexpr_sampler(
        &mut self,
        variable: impl Into<Handle<VariableId>>,
        sampler: &ConstexprSampler,
        ycbcr: Option<&SamplerYcbcrConversion>,
    ) -> error::Result<()> {
        let variable = variable.into();
        let id = self.yield_id(variable)?;
        if let Some(ycbcr) = ycbcr {
            unsafe {
                sys::spvc_compiler_msl_remap_constexpr_sampler_ycbcr(
                    self.ptr.as_ptr(),
                    id,
                    sampler,
                    ycbcr,
                )
                .ok(&*self)
            }
        } else {
            unsafe {
                sys::spvc_compiler_msl_remap_constexpr_sampler(self.ptr.as_ptr(), id, sampler)
                    .ok(&*self)
            }
        }
    }

    /// Remap a sampler with set/binding, to a constexpr sampler.
    /// Older iOS targets must use constexpr samplers in certain cases (PCF),
    /// so a static sampler must be used.
    ///
    /// The sampler will not consume a binding, but be declared in the entry point as a constexpr sampler.
    /// This can be used on both combined image/samplers (sampler2D) or standalone samplers.
    /// The remapped sampler must not be an array of samplers.
    ///
    /// Remaps based on ID take priority over set/binding remaps.
    pub fn remap_constexpr_sampler_by_binding(
        &mut self,
        desc_set: u32,
        binding: u32,
        sampler: &ConstexprSampler,
        ycbcr: Option<&SamplerYcbcrConversion>,
    ) -> error::Result<()> {
        if let Some(ycbcr) = ycbcr {
            unsafe {
                sys::spvc_compiler_msl_remap_constexpr_sampler_by_binding_ycbcr(
                    self.ptr.as_ptr(),
                    desc_set,
                    binding,
                    sampler,
                    ycbcr,
                )
                .ok(&*self)
            }
        } else {
            unsafe {
                sys::spvc_compiler_msl_remap_constexpr_sampler_by_binding(
                    self.ptr.as_ptr(),
                    desc_set,
                    binding,
                    sampler,
                )
                .ok(&*self)
            }
        }
    }

    /// If using [`CompilerOptions::pad_fragment_output_components`], override the number of components we expect
    /// to use for a particular location. The default is 4 if number of components is not overridden.
    pub fn set_fragment_output_components(
        &mut self,
        location: u32,
        components: u32,
    ) -> error::Result<()> {
        unsafe {
            sys::spvc_compiler_msl_set_fragment_output_components(
                self.ptr.as_ptr(),
                location,
                components,
            )
            .ok(&*self)
        }
    }

    /// Set the suffix for combined image samplers.
    pub fn set_combined_sampler_suffix<'str>(
        &mut self,
        str: impl Into<CompilerStr<'str>>,
    ) -> error::Result<()> {
        unsafe {
            let str = str.into();

            let suffix = str.into_cstring_ptr()?;

            sys::spvc_compiler_msl_set_combined_sampler_suffix(self.ptr.as_ptr(), suffix.as_ptr())
                .ok(&*self)
        }
    }

    /// Get the suffix for combined image samplers.
    pub fn combined_sampler_suffix(&self) -> CompilerStr {
        unsafe {
            let suffix = sys::spvc_compiler_msl_get_combined_sampler_suffix(self.ptr.as_ptr());
            CompilerStr::from_ptr(suffix, self.ctx.drop_guard())
        }
    }

    /// Mask a stage output by location.
    ///
    /// If a shader output is active in this stage, but inactive in a subsequent stage,
    /// this can be signalled here. This can be used to work around certain cross-stage matching problems
    /// which plagues MSL in certain scenarios.
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
    /// which plagues MSL in certain scenarios.
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
            sys::spvc_compiler_mask_stage_output_by_builtin(
                self.ptr.as_ptr(),
                SpvBuiltIn(builtin as u32 as i32),
            )
            .ok(&*self)
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[non_exhaustive]
/// The tier of automatic resource binding.
///
/// Note that tertiary and quaternary bindings are not accessible via
/// the SPIR-V Cross C API.
pub enum AutomaticResourceBindingTier {
    #[default]
    /// The primary automatic resource binding.
    Primary,

    /// Should only be used for combined image samplers, in which case the
    /// sampler's binding is returned instead.
    ///
    /// Also used for the auxillary image atomic buffer.
    Secondary,
}

impl CompiledArtifact<Msl> {
    /// Returns whether the set/binding combination provided in [`Compiler<Msl>::add_resource_binding`]
    /// was used.
    pub fn is_resource_used(&self, model: spirv::ExecutionModel, binding: ResourceBinding) -> bool {
        unsafe {
            sys::spvc_compiler_msl_is_resource_used(
                self.compiler.ptr.as_ptr(),
                SpvExecutionModel(model as u32 as i32),
                binding.descriptor_set(),
                binding.binding(),
            )
        }
    }

    /// Returns whether the location provided in [`Compiler<Msl>::add_shader_input`]
    /// was used.
    pub fn is_shader_input_used(&self, location: u32) -> bool {
        unsafe { sys::spvc_compiler_msl_is_shader_input_used(self.compiler.ptr.as_ptr(), location) }
    }

    /// Returns whether the location provided in [`Compiler<Msl>::add_shader_output`]
    /// was used.
    pub fn is_shader_output_used(&self, location: u32) -> bool {
        unsafe {
            sys::spvc_compiler_msl_is_shader_output_used(self.compiler.ptr.as_ptr(), location)
        }
    }

    /// For a variable resource ID, report the automatically assigned resource index.
    ///
    /// If the descriptor set was part of an argument buffer, report the `[[id(N)]]`,
    /// or `[[buffer/texture/sampler]]` binding for other resources.
    ///
    /// If the resource was a combined image sampler, report the image binding for [`AutomaticResourceBindingTier::Primary`],
    /// or the sampler half for [`AutomaticResourceBindingTier::Secondary`].
    ///
    /// If no binding exists, None is returned.
    pub fn automatic_resource_binding(
        &self,
        handle: impl Into<Handle<VariableId>>,
        tier: AutomaticResourceBindingTier,
    ) -> error::Result<Option<u32>> {
        let handle = handle.into();
        let id = self.yield_id(handle)?;

        let res = match tier {
            AutomaticResourceBindingTier::Primary => unsafe {
                sys::spvc_compiler_msl_get_automatic_resource_binding(self.ptr.as_ptr(), id)
            },
            AutomaticResourceBindingTier::Secondary => unsafe {
                sys::spvc_compiler_msl_get_automatic_resource_binding_secondary(
                    self.ptr.as_ptr(),
                    id,
                )
            },
        };

        if res == u32::MAX {
            Ok(None)
        } else {
            Ok(Some(res))
        }
    }

    /// Query if a variable ID was used as a depth resource.
    ///
    /// This is meaningful for MSL since descriptor types depend on this knowledge.
    /// Cases which return true:
    /// - Images which are declared with depth = 1 image type.
    /// - Samplers which are statically used at least once with Dref opcodes.
    /// - Images which are statically used at least once with Dref opcodes.
    pub fn variable_is_depth_or_compare(
        &self,
        variable: impl Into<Handle<VariableId>>,
    ) -> error::Result<bool> {
        let variable = variable.into();
        let id = self.yield_id(variable)?;
        unsafe {
            Ok(sys::spvc_compiler_variable_is_depth_or_compare(
                self.ptr.as_ptr(),
                id,
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compile::msl::CompilerOptions;
    use spirv_cross_sys::spvc_compiler_create_compiler_options;

    use crate::compile::sealed::ApplyCompilerOptions;
    use crate::error::{SpirvCrossError, ToContextError};
    use crate::Compiler;
    use crate::{targets, Module};

    static BASIC_SPV: &[u8] = include_bytes!("../../../basic.spv");

    #[test]
    pub fn msl_opts() -> Result<(), SpirvCrossError> {
        let words = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&words));

        let compiler: Compiler<targets::Msl> = Compiler::new(words)?;
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
