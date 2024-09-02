use crate::compile;
use crate::compile::{CompilableTarget, NoOptions};
use crate::sealed::Sealed;
use spirv_cross_sys::CompilerBackend;

pub struct None;
pub struct Glsl;
pub struct Msl;
pub struct Hlsl;
pub struct Cpp;
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

impl CompilableTarget for Cpp {
    type Options = NoOptions;
}

impl Sealed for Cpp {}
impl Target for Cpp {
    const BACKEND: CompilerBackend = CompilerBackend::Cpp;
}

/// Marker trait for a compiler backend target.
pub trait Target: Sealed {
    #[doc(hidden)]
    const BACKEND: CompilerBackend;
}
