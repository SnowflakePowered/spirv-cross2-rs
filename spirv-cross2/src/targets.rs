use spirv_cross_sys::CompilerBackend;

pub struct None;
pub struct Glsl;
pub struct Msl;
pub struct Hlsl;
pub struct Cpp;
pub struct Json;

impl Target for None {
    const BACKEND: CompilerBackend = CompilerBackend::None;
}

impl CompilableTarget for Glsl {}
impl Target for Glsl {
    const BACKEND: CompilerBackend = CompilerBackend::Glsl;
}

impl CompilableTarget for Hlsl {}
impl Target for Hlsl {
    const BACKEND: CompilerBackend = CompilerBackend::Hlsl;
}

impl CompilableTarget for Msl {}
impl Target for Msl {
    const BACKEND: CompilerBackend = CompilerBackend::Msl;
}

impl CompilableTarget for Json {}
impl Target for Json {
    const BACKEND: CompilerBackend = CompilerBackend::Json;
}

impl CompilableTarget for Cpp {}
impl Target for Cpp {
    const BACKEND: CompilerBackend = CompilerBackend::Cpp;
}

/// A target that can have compiler outputs.
pub trait CompilableTarget: Target {}

/// A compiler backend target.
pub trait Target {
    const BACKEND: CompilerBackend;
}
