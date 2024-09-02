# Contributing

`spirv-cross2` links the C API of [SPIRV-Cross](https://github.com/KhronosGroup/SPIRV-Cross), and should not need any
special handling outside of rerunning `bindgen` if the `SPVC_C_API_VERSION_MAJOR` is 0.

The following `-sys` types should be manually marked `#[non_exhaustive]`.

- `MslVertexAttribute`
- `MslShaderInterfaceVar`
- `MslShaderInterfaceVar2`
- `MslResourceBinding`
- `MslResourceBinding2`
- `MslConstexprSampler`
- `MslSamplerYcbcrConversion`
- `HlslResourceBinding`


The following types are re-exported, so any change to them should also result in a semver-breaking
version bump (although this is unlikely, as all the structs are essentially fixed).

```rust
pub mod spirv {
    pub use spirv_cross_sys::SpvBuiltIn as BuiltIn;
    pub use spirv_cross_sys::SpvCapability as Capability;
    pub use spirv_cross_sys::SpvDecoration as Decoration;
    pub use spirv_cross_sys::SpvDim as Dim;
    pub use spirv_cross_sys::SpvExecutionMode as ExecutionMode;
    pub use spirv_cross_sys::SpvExecutionModel as ExecutionModel;
    pub use spirv_cross_sys::SpvFPDenormMode as FPDenormMode;
    pub use spirv_cross_sys::SpvFPFastMathModeMask as FPFastMathModeMask;
    pub use spirv_cross_sys::SpvFPFastMathModeShift as FPFastMathModeShift;
    pub use spirv_cross_sys::SpvFPOperationMode as FPOperationMode;
    pub use spirv_cross_sys::SpvFPRoundingMode as FPRoundingMode;
    pub use spirv_cross_sys::SpvImageFormat as ImageFormat;
    pub use spirv_cross_sys::SpvStorageClass as StorageClass;
}

pub mod handle {
    pub use spirv_cross_sys::{ConstantId, TypeId, VariableId};
}

pub mod buffers {
    pub use spirv_cross_sys::BufferRange;
}

pub mod resources {
    pub use spirv_cross_sys::{BuiltinResourceType, ResourceType};
}

pub mod hlsl {
    pub use spirv_cross_sys::HlslResourceBinding as ResourceBinding;
    pub use spirv_cross_sys::HlslResourceBindingMapping as ResourceBindingMapping;
    pub use spirv_cross_sys::HlslRootConstants as RootConstants;
}

pub mod msl {
    pub use spirv_cross_sys::MslChromaLocation as ChromaLocation;
    pub use spirv_cross_sys::MslComponentSwizzle as ComponentSwizzle;
    pub use spirv_cross_sys::MslConstexprSampler as ConstexprSampler;
    pub use spirv_cross_sys::MslFormatResolution as FormatResolution;
    pub use spirv_cross_sys::MslResourceBinding2 as ResourceBinding;
    pub use spirv_cross_sys::MslSamplerAddress as SamplerAddress;
    pub use spirv_cross_sys::MslSamplerBorderColor as SamplerBorderColor;
    pub use spirv_cross_sys::MslSamplerCompareFunc as SamplerCompareFunc;
    pub use spirv_cross_sys::MslSamplerCoord as SamplerCoord;
    pub use spirv_cross_sys::MslSamplerFilter as SamplerFilter;
    pub use spirv_cross_sys::MslSamplerMipFilter as SamplerMipFilter;
    pub use spirv_cross_sys::MslSamplerYcbcrConversion as SamplerYcbcrConversion;
    pub use spirv_cross_sys::MslSamplerYcbcrModelConversion as SamplerYcbcrModelConversion;
    pub use spirv_cross_sys::MslSamplerYcbcrRange as SamplerYcbcrRange;
    pub use spirv_cross_sys::MslShaderInput as ShaderInput;
    pub use spirv_cross_sys::MslShaderInterfaceVar2 as ShaderInterfaceVar;
    pub use spirv_cross_sys::MslShaderVariableFormat as ShaderVariableFormat;
    pub use spirv_cross_sys::MslShaderVariableRate as ShaderVariableRate;
}
```

If (and likely it will be MSL) an API gets deprecated, best practice is to create an `From` impl for the deprecated input
struct to the new struct, replace it with an `Into` arg on the old API such that it is not semver breaking, push a release,
then push a semver breaking release with just the new struct such that the old struct is not accessible.

## Options derive
`spirv-cross2-derive` is used to derive options structs.

It relies on two traits
```rust 
pub mod compile {
    /// Marker trait for compiler options.
    pub trait CompilerOptions: Default + sealed::ApplyCompilerOptions {}

    pub(crate) mod sealed {
        use crate::error;
        use crate::error::ContextRooted;
        use crate::sealed::Sealed;
        use spirv_cross_sys::spvc_compiler_options;

        pub trait ApplyCompilerOptions: Sealed {
            unsafe fn apply(
                &self,
                options: spvc_compiler_options,
                root: impl ContextRooted + Copy,
            ) -> error::Result<()>;
        }
    }
}
```

where `ApplyCompilerOptions` is the actual trait derived by `spirv_cross2_derive::CompilerOptions` but is hidden so the `apply`
function is not part of the public API.

There are two attributes registered, `#[option]` and `#[expand]`

```rust
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CommonOptions {
    /// Debug option to always emit temporary variables for all expressions.
    #[option(SPVC_COMPILER_OPTION_FORCE_TEMPORARY, false)]
    pub force_temporary: bool,
}

#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
#[non_exhaustive]
pub struct CompileOptions {
    /// Compile options common to GLSL, HLSL, and MSL.
    #[expand]
    pub common: CommonOptions,
    
    /// If true, Vulkan GLSL features are used instead of GL-compatible features.
    /// Mostly useful for debugging SPIR-V files.
    #[option(SPVC_COMPILER_OPTION_GLSL_VULKAN_SEMANTICS, false)]
    pub vulkan_semantics: bool,
}
```

If `#[expand]` is marked on a field, that field must also implement `ApplyCompilerOptions`. `ApplyCompilerOptions` will be recursively
called on all fields marked `#[expand]`, so this can be used to implement custom logic on `apply` such as with `GlslVersion`.

`#[option]` takes an ident that an associated const of `spirv-cross-sys::spvc_compiler_option`, and a default option. If a 
default is not provided, `Default::default()` will be considered the default.

All option fields must also implement `Default`. It is recommended that option structs are marked as `#[non_exhaustive]`, and
that users get an instance via the `Default` impl.