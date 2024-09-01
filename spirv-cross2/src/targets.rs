use spirv_cross2_derive::CompilerOptions;
use crate::compile::CompilerOptions;
use crate::error::ContextRooted;
use spirv_cross_sys::{spvc_compiler_options, CompilerBackend};
use crate::sealed::Sealed;

pub struct None;
pub struct Glsl;
pub struct Msl;
pub struct Hlsl;
pub struct Cpp;
pub struct Json;

#[derive(Debug, CompilerOptions)]
pub struct NoOptions;

impl Sealed for None {}
impl Target for None {
    const BACKEND: CompilerBackend = CompilerBackend::None;
}

impl CompilableTarget for Glsl {
    type Options = NoOptions;
}
impl Sealed for Glsl {}
impl Target for Glsl {
    const BACKEND: CompilerBackend = CompilerBackend::Glsl;
}

impl CompilableTarget for Hlsl {
    type Options = NoOptions;
}
impl Sealed for Hlsl {}
impl Target for Hlsl {
    const BACKEND: CompilerBackend = CompilerBackend::Hlsl;
}

impl CompilableTarget for Msl {
    type Options = NoOptions;
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

/// A target that can have compiler outputs.
pub trait CompilableTarget: Target {
    #[allow(private_bounds)]
    type Options: CompilerOptions;
}

/// A compiler backend target.
pub trait Target: Sealed {
    const BACKEND: CompilerBackend;
}
