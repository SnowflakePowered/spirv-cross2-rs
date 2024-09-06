use crate::compile;
use crate::compile::CompilableTarget;
use crate::sealed::Sealed;
use spirv_cross_sys::CompilerBackend;

/// Reflection only backend, no compilation features
/// enabled.
pub struct None;

/// Compile SPIR-V to GLSL.
#[cfg(feature = "glsl")]
#[cfg_attr(docsrs, doc(cfg(feature = "glsl")))]
pub struct Glsl;

/// Compile SPIR-V to Metal Shading Language (MSL).
#[cfg(feature = "msl")]
#[cfg_attr(docsrs, doc(cfg(feature = "msl")))]
pub struct Msl;

/// Compile SPIR-V to HLSL.
#[cfg(feature = "hlsl")]
#[cfg_attr(docsrs, doc(cfg(feature = "hlsl")))]
pub struct Hlsl;

/// Compile SPIR-V to debuggable C++.
///
/// This backend is deprecated but is included here for completion.
/// See the [SPIRV-Cross docs](https://github.com/KhronosGroup/SPIRV-Cross?tab=readme-ov-file#using-shaders-generated-from-c-backend)
/// for how to debug shaders generated from the C++ backend
#[deprecated = "This backend is deprecated in SPIRV-Cross."]
#[cfg(feature = "cpp")]
#[cfg_attr(docsrs, doc(cfg(feature = "cpp")))]
pub struct Cpp;

/// Compile SPIR-V to a JSON reflection format
#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub struct Json;

impl Sealed for None {}
impl Target for None {
    const BACKEND: CompilerBackend = CompilerBackend::None;
}

#[cfg(feature = "glsl")]
#[cfg_attr(docsrs, doc(cfg(feature = "glsl")))]
mod glsl {
    use super::*;
    impl CompilableTarget for Glsl {
        type Options = compile::glsl::CompilerOptions;
    }
    impl Sealed for Glsl {}
    impl Target for Glsl {
        const BACKEND: CompilerBackend = CompilerBackend::Glsl;
    }
}

#[cfg(feature = "hlsl")]
#[cfg_attr(docsrs, doc(cfg(feature = "hlsl")))]
mod hlsl {
    use super::*;
    impl CompilableTarget for Hlsl {
        type Options = compile::hlsl::CompilerOptions;
    }
    impl Sealed for Hlsl {}
    impl Target for Hlsl {
        const BACKEND: CompilerBackend = CompilerBackend::Hlsl;
    }
}

#[cfg(feature = "msl")]
#[cfg_attr(docsrs, doc(cfg(feature = "msl")))]
mod msl {
    use super::*;
    impl CompilableTarget for Msl {
        type Options = compile::msl::CompilerOptions;
    }
    impl Sealed for Msl {}
    impl Target for Msl {
        const BACKEND: CompilerBackend = CompilerBackend::Msl;
    }
}

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
mod json {
    use super::*;
    impl CompilableTarget for Json {
        type Options = compile::NoOptions;
    }
    impl Sealed for Json {}
    impl Target for Json {
        const BACKEND: CompilerBackend = CompilerBackend::Json;
    }
}

#[cfg(feature = "cpp")]
#[cfg_attr(docsrs, doc(cfg(feature = "cpp")))]
mod cpp {
    use super::*;
    #[allow(deprecated)]
    impl CompilableTarget for Cpp {
        type Options = compile::NoOptions;
    }

    #[allow(deprecated)]
    impl Sealed for Cpp {}

    #[allow(deprecated)]
    impl Target for Cpp {
        const BACKEND: CompilerBackend = CompilerBackend::Cpp;
    }
}

/// Marker trait for a compiler backend target.
pub trait Target: Sealed {
    #[doc(hidden)]
    const BACKEND: CompilerBackend;
}
