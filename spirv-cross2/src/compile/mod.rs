use crate::error::{Result, ToContextError};
use crate::sealed::Sealed;
use crate::targets::Target;
use crate::{error, Compiler, ContextRooted, ContextStr};
use spirv_cross_sys as sys;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

/// GLSL compile options.
pub mod glsl;

/// HLSL compile options.
pub mod hlsl;

/// MSL compile options.
pub mod msl;

impl Sealed for CommonOptions {}

/// Compile options common to all backends.
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CommonOptions {
    // common options
    /// Debug option to always emit temporary variables for all expressions.
    #[option(SPVC_COMPILER_OPTION_FORCE_TEMPORARY, false)]
    pub force_temporary: bool,

    /// Flattens multidimensional arrays, e.g. `float foo[a][b][c]` into single-dimensional arrays,
    /// e.g. `float foo[a * b * c]`.
    /// This function does not change the actual type of any object.
    /// Only the generated code, including declarations of interface variables
    /// are changed to be single array dimension.
    #[option(SPVC_COMPILER_OPTION_FLATTEN_MULTIDIMENSIONAL_ARRAYS, false)]
    pub flatten_multidimensional_arrays: bool,

    /// In vertex-like shaders, inverts gl_Position.y or equivalent.
    #[option(SPVC_COMPILER_OPTION_FLIP_VERTEX_Y, false)]
    pub flip_vertex_y: bool,

    /// GLSL: In vertex-like shaders, rewrite [0, w] depth (Vulkan/D3D style) to [-w, w] depth (GL style).
    /// MSL: In vertex-like shaders, rewrite [-w, w] depth (GL style) to [0, w] depth.
    /// HLSL: In vertex-like shaders, rewrite [-w, w] depth (GL style) to [0, w] depth.
    #[option(SPVC_COMPILER_OPTION_FIXUP_DEPTH_CONVENTION, false)]
    pub fixup_clipspace: bool,

    /// Emit OpLine directives if present in the module.
    /// May not correspond exactly to original source, but should be a good approximation.
    #[option(SPVC_COMPILER_OPTION_EMIT_LINE_DIRECTIVES, false)]
    pub emit_line_directives: bool,

    /// On some targets (WebGPU), uninitialized variables are banned.
    /// If this is enabled, all variables (temporaries, Private, Function)
    /// which would otherwise be uninitialized will now be initialized to 0 instead.
    #[option(SPVC_COMPILER_OPTION_FORCE_ZERO_INITIALIZED_VARIABLES, false)]
    pub force_zero_initialized_variables: bool,

    /// In cases where readonly/writeonly decoration are not used at all,
    /// we try to deduce which qualifier(s) we should actually used, since actually emitting
    /// read-write decoration is very rare, and older glslang/HLSL compilers tend to just emit readwrite as a matter of fact.
    /// The default (true) is to enable automatic deduction for these cases, but if you trust the decorations set
    /// by the SPIR-V, it's recommended to set this to false.
    #[option(SPVC_COMPILER_OPTION_ENABLE_STORAGE_IMAGE_QUALIFIER_DEDUCTION, true)]
    pub enable_storage_image_qualifier_deduction: bool,

    /// For opcodes where we have to perform explicit additional nan checks, very ugly code is generated.
    /// If we opt-in, ignore these requirements.
    /// In opcodes like NClamp/NMin/NMax and FP compare, ignore NaN behavior.
    /// Use FClamp/FMin/FMax semantics for clamps and lets implementation choose ordered or unordered
    /// compares.
    #[option(SPVC_COMPILER_OPTION_RELAX_NAN_CHECKS, false)]
    pub relax_nan_checks: bool,
}

/// The output of a SPIRV-Cross compilation.
///
/// [`CompiledArtifact`] implements [`Display`] with the
/// value of the compiled source code, which can be copied
/// to detach it from the lifetime '`a`.
///
/// If the [`Compiler`] instance is static, the source
/// will also be static.
///
/// Reflection is still available, but the [`Compiler`]
/// instance can no longer be mutated once compiled.
pub struct CompiledArtifact<'a, T> {
    compiler: Compiler<'a, T>,
    source: ContextStr<'a>,
}

impl<T> AsRef<str> for CompiledArtifact<'_, T> {
    fn as_ref(&self) -> &str {
        self.source.as_ref()
    }
}

impl<T> Display for CompiledArtifact<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.source, f)
    }
}

impl<'a, T> Deref for CompiledArtifact<'a, T> {
    type Target = Compiler<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.compiler
    }
}

/// Cross-compilation related methods.
impl<'a, T: CompilableTarget> Compiler<'a, T> {
    /// Adds a line in valid header position.
    ///
    /// For example, in the GLSL backend this would be right after #version.
    ///
    /// This is useful for enabling custom extensions which are outside the scope of SPIRV-Cross.
    /// This can be combined with variable remapping.
    ///
    /// A new-line will be added after this line.
    ///
    /// While this function is a more generic way of adding arbitrary text to the header
    /// of an output file, [`Compiler::require_extension`] should be used when adding extensions since it will
    /// avoid creating collisions with SPIRV-Cross generated extensions.
    pub fn add_header_line<'str>(&mut self, line: impl Into<ContextStr<'str>>) -> Result<()> {
        let line = line.into();
        let cstring = line.into_cstring_ptr()?;
        unsafe { sys::spvc_compiler_add_header_line(self.ptr.as_ptr(), cstring.as_ptr()).ok(self) }
    }

    /// Adds an extension which is required to run this shader, e.g.
    /// `require_extension("GL_KHR_my_extension");`
    pub fn require_extension<'str>(&mut self, ext: impl Into<ContextStr<'str>>) -> Result<()> {
        let ext = ext.into();
        let cstring = ext.into_cstring_ptr()?;

        unsafe {
            sys::spvc_compiler_require_extension(self.ptr.as_ptr(), cstring.as_ptr().cast())
                .ok(self)
        }
    }

    /// Apply the set of compiler options to the compiler instance.
    fn set_compiler_options(&mut self, options: &T::Options) -> error::Result<()> {
        use crate::compile::sealed::ApplyCompilerOptions;
        unsafe {
            let mut handle = std::ptr::null_mut();

            sys::spvc_compiler_create_compiler_options(self.ptr.as_ptr(), &mut handle)
                .ok(&*self)?;

            options.apply(handle, &*self)?;

            sys::spvc_compiler_install_compiler_options(self.ptr.as_ptr(), handle).ok(&*self)?;

            Ok(())
        }
    }

    /// Consume the compilation instance, and compile source code to the
    /// output target.
    pub fn compile(mut self, options: &T::Options) -> error::Result<CompiledArtifact<'a, T>> {
        self.set_compiler_options(options)?;

        unsafe {
            let mut src = std::ptr::null();
            sys::spvc_compiler_compile(self.ptr.as_ptr(), &mut src).ok(&self)?;

            // SAFETY: 'a is OK to return here
            // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_cross_c.cpp#L1782
            let src = ContextStr::from_ptr(src, self.ctx.clone());
            Ok(CompiledArtifact {
                compiler: self,
                source: src,
            })
        }
    }
}

/// Marker trait for compiler options.
pub trait CompilerOptions: Default + sealed::ApplyCompilerOptions {}

pub(crate) mod sealed {
    use crate::error;
    use crate::error::ContextRooted;
    use crate::sealed::Sealed;
    use spirv_cross_sys::spvc_compiler_options;

    pub trait ApplyCompilerOptions: Sealed {
        #[doc(hidden)]
        unsafe fn apply(
            &self,
            options: spvc_compiler_options,
            root: impl ContextRooted + Copy,
        ) -> error::Result<()>;
    }
}

#[cfg(test)]
mod test {
    use crate::error::SpirvCrossError;
    use crate::targets;
    use crate::Compiler;
    use crate::{Module, SpirvCrossContext};

    const BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn create_compiler() -> Result<(), SpirvCrossError> {
        let spv = SpirvCrossContext::new()?;
        let vec = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&vec));

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        Ok(())
    }
}

impl Sealed for NoOptions {}

/// No compilation options.
///
/// Used for compiler backends that take no options.
#[derive(Debug, Default, spirv_cross2_derive::CompilerOptions)]
pub struct NoOptions;

/// Marker trait for a compiler target that can have compiler outputs.
pub trait CompilableTarget: Target {
    /// The options that this target accepts.
    type Options: CompilerOptions;

    /// Create a new instance of compiler options for this target.
    fn options() -> Self::Options {
        Self::Options::default()
    }
}
