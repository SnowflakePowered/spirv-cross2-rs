use crate::compile;
use crate::compile::{CompilableTarget, NoOptions};
use crate::sealed::Sealed;
use spirv_cross_sys::CompilerBackend;

/// Reflection only backend, no compilation features
/// enabled.
pub struct None;
/// Compile SPIR-V to GLSL.
pub struct Glsl;
/// Compile SPIR-V to Metal Shading Language (MSL).
pub struct Msl;
/// Compile SPIR-V to HLSL.
pub struct Hlsl;
/// Compile SPIR-V to debuggable C++.
///
/// This backend is deprecated but is included here for completion.
/// See the [SPIRV-Cross docs](https://github.com/KhronosGroup/SPIRV-Cross?tab=readme-ov-file#using-shaders-generated-from-c-backend)
/// for how to debug shaders generated from the C++ backend
#[deprecated = "This backend is deprecated in SPIRV-Cross."]
pub struct Cpp;
/// Compile SPIR-V to a JSON reflection format
pub struct Json;

impl Sealed for None {}
impl Target for None {
    const BACKEND: CompilerBackend = CompilerBackend::None;
}

impl CompilableTarget for Glsl {
    type Options = compile::glsl::CompileOptions;
}
impl Sealed for Glsl {}
impl Target for Glsl {
    const BACKEND: CompilerBackend = CompilerBackend::Glsl;
}

impl CompilableTarget for Hlsl {
    type Options = compile::hlsl::CompileOptions;
}
impl Sealed for Hlsl {}
impl Target for Hlsl {
    const BACKEND: CompilerBackend = CompilerBackend::Hlsl;
}

impl CompilableTarget for Msl {
    type Options = compile::msl::CompileOptions;
}
impl Sealed for Msl {}
impl Target for Msl {
    const BACKEND: CompilerBackend = CompilerBackend::Msl;
}

impl CompilableTarget for Json {
    type Options = NoOptions;
}
impl Sealed for Json {}
impl Target for Json {
    const BACKEND: CompilerBackend = CompilerBackend::Json;
}

#[allow(deprecated)]
impl CompilableTarget for Cpp {
    type Options = NoOptions;
}

#[allow(deprecated)]
impl Sealed for Cpp {}

#[allow(deprecated)]
impl Target for Cpp {
    const BACKEND: CompilerBackend = CompilerBackend::Cpp;
}

/// Marker trait for a compiler backend target.
pub trait Target: Sealed {
    #[doc(hidden)]
    const BACKEND: CompilerBackend;
}
