use super::CommonOptions;
use crate::compile::sealed::ApplyCompilerOptions;
use crate::error::ToContextError;
use crate::handle::Handle;
use crate::iter::impl_iterator;
use crate::sealed::Sealed;
use crate::targets::Glsl;
use crate::{error, Compiler, CompilerStr, ContextRooted, PhantomCompiler};
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_compiler_option, spvc_compiler_options, VariableId};
use std::marker::PhantomData;
use std::ops::Range;

impl Sealed for CompilerOptions {}
/// GLSL compiler options.
#[non_exhaustive]
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CompilerOptions {
    /// Compile options common to GLSL, HLSL, and MSL.
    #[expand]
    pub common: CommonOptions,

    /// The GLSL version to output. The default is #version 450.
    #[expand]
    pub version: GlslVersion,

    /// If true, Vulkan GLSL features are used instead of GL-compatible features.
    /// Mostly useful for debugging SPIR-V files.
    #[option(SPVC_COMPILER_OPTION_GLSL_VULKAN_SEMANTICS, false)]
    pub vulkan_semantics: bool,

    /// If true, gl_PerVertex is explicitly redeclared in vertex, geometry and tessellation shaders.
    /// The members of gl_PerVertex is determined by which built-ins are declared by the shader.
    /// This option is ignored in ES versions, as redeclaration in ES is not required, and it depends on a different extension
    /// (EXT_shader_io_blocks) which makes things a bit more fuzzy.
    #[option(SPVC_COMPILER_OPTION_GLSL_SEPARATE_SHADER_OBJECTS, false)]
    pub seperate_shader_objects: bool,

    /// For older desktop GLSL targets than version 420, the
    /// GL_ARB_shading_language_420pack extensions is used to be able to support
    /// layout(binding) on UBOs and samplers.
    /// If disabled on older targets, binding decorations will be stripped.
    ///
    /// The default is true.
    #[option(SPVC_COMPILER_OPTION_GLSL_ENABLE_420PACK_EXTENSION, true)]
    pub enable_420pack_extension: bool,

    /// If true, the backend will assume that InstanceIndex will need to apply
    /// a base instance offset. Set to false if you know you will never use base instance
    /// functionality as it might remove some internal uniforms.
    #[option(SPVC_COMPILER_OPTION_GLSL_SUPPORT_NONZERO_BASE_INSTANCE, true)]
    pub support_nonzero_base_instance: bool,

    /// If true, sets the default float precision in ES targets to highp,
    /// otherwise the default is mediump.
    #[option(SPVC_COMPILER_OPTION_GLSL_ES_DEFAULT_FLOAT_PRECISION_HIGHP, false)]
    pub es_default_float_precision_highp: bool,

    /// If false, sets the default float precision in ES targets to mediump,
    /// otherwise the default is highp.
    #[option(SPVC_COMPILER_OPTION_GLSL_ES_DEFAULT_INT_PRECISION_HIGHP, true)]
    pub es_default_int_precision_highp: bool,

    /// In non-Vulkan GLSL, emit push constant blocks as UBOs rather than plain uniforms.
    #[option(SPVC_COMPILER_OPTION_GLSL_EMIT_PUSH_CONSTANT_AS_UNIFORM_BUFFER, false)]
    pub emit_push_constant_as_uniform_buffer: bool,

    /// Always emit uniform blocks as plain uniforms, regardless of the GLSL version, even when UBOs are supported.
    /// Does not apply to shader storage or push constant blocks.
    #[option(SPVC_COMPILER_OPTION_GLSL_EMIT_UNIFORM_BUFFER_AS_PLAIN_UNIFORMS, false)]
    pub emit_uniform_buffer_as_plain_uniforms: bool,

    /// In GLSL, force use of I/O block flattening, similar to
    /// what happens on legacy GLSL targets for blocks and structs.
    #[option(SPVC_COMPILER_OPTION_GLSL_FORCE_FLATTENED_IO_BLOCKS, false)]
    pub force_flattened_io_blocks: bool,

    /// Loading row-major matrices from UBOs on older AMD Windows OpenGL drivers is problematic.
    /// To load these types correctly, we must generate a wrapper. them in a dummy function which only purpose is to
    /// ensure row_major decoration is actually respected.
    /// This workaround may cause significant performance degeneration on some Android devices.
    #[option(SPVC_COMPILER_OPTION_GLSL_ENABLE_ROW_MAJOR_LOAD_WORKAROUND, true)]
    pub enable_row_major_load_workaround: bool,

    /// If non-zero, controls `layout(num_views = N) in;` in GL_OVR_multiview2.
    #[option(SPVC_COMPILER_OPTION_GLSL_OVR_MULTIVIEW_VIEW_COUNT, 0)]
    pub ovr_multiview_view_count: u32,
}

impl Sealed for GlslVersion {}

/// GLSL language version.
#[non_exhaustive]
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum GlslVersion {
    /// #version 110
    Glsl110,
    /// #version 120
    Glsl120,
    /// #version 130
    Glsl130,
    /// #version 140
    Glsl140,
    /// #version 150
    Glsl150,
    /// #version 330
    Glsl330,
    /// #version 400
    Glsl400,
    /// #version 410
    Glsl410,
    /// #version 420
    Glsl420,
    /// #version 430
    Glsl430,
    /// #version 440
    Glsl440,
    /// #version 450
    #[default]
    Glsl450,
    /// #version 460
    Glsl460,
    /// #version 100 es
    Glsl100Es,
    /// #version 300 es
    Glsl300Es,
    /// #version 310 es
    Glsl310Es,
    /// #version 320 es
    Glsl320Es,
}

impl ApplyCompilerOptions for GlslVersion {
    unsafe fn apply(
        &self,
        options: spvc_compiler_options,
        root: impl ContextRooted + Copy,
    ) -> error::Result<()> {
        let version = match self {
            GlslVersion::Glsl110 => 110,
            GlslVersion::Glsl120 => 120,
            GlslVersion::Glsl130 => 130,
            GlslVersion::Glsl140 => 140,
            GlslVersion::Glsl150 => 150,
            GlslVersion::Glsl330 => 330,
            GlslVersion::Glsl400 => 400,
            GlslVersion::Glsl410 => 410,
            GlslVersion::Glsl420 => 420,
            GlslVersion::Glsl430 => 430,
            GlslVersion::Glsl440 => 440,
            GlslVersion::Glsl450 => 450,
            GlslVersion::Glsl460 => 460,
            GlslVersion::Glsl100Es => 100,
            GlslVersion::Glsl300Es => 300,
            GlslVersion::Glsl310Es => 310,
            GlslVersion::Glsl320Es => 320,
        };

        let es = matches!(
            self,
            GlslVersion::Glsl100Es
                | GlslVersion::Glsl300Es
                | GlslVersion::Glsl310Es
                | GlslVersion::Glsl320Es
        );

        unsafe {
            sys::spvc_compiler_options_set_uint(
                options,
                spvc_compiler_option::SPVC_COMPILER_OPTION_GLSL_VERSION,
                version,
            )
            .ok(root)?;
            sys::spvc_compiler_options_set_bool(
                options,
                spvc_compiler_option::SPVC_COMPILER_OPTION_GLSL_ES,
                es,
            )
            .ok(root)?;
        }

        Ok(())
    }
}

impl Compiler<Glsl> {
    /// Legacy GLSL compatibility method.
    ///
    /// Takes a uniform or push constant variable and flattens it into a `(i|u)vec4 array[N];` array instead.
    /// For this to work, all types in the block must be the same basic type, e.g. mixing `vec2` and `vec4` is fine, but
    /// mixing int and float is not.
    ///
    /// The name of the uniform array will be the same as the interface block name.
    pub fn flatten_buffer_block(
        &mut self,
        block: impl Into<Handle<VariableId>>,
    ) -> error::Result<()> {
        let block = block.into();
        let block = self.yield_id(block)?;

        unsafe { sys::spvc_compiler_flatten_buffer_block(self.ptr.as_ptr(), block).ok(&*self) }
    }

    /// Returns the list of required extensions in a GLSL shader.
    ///
    /// If called after compilation this will contain any other extensions that the compiler
    /// used automatically, in addition to the user specified ones.
    pub fn required_extensions(&self) -> GlslExtensionsIter {
        // SAFETY:
        // It is **not sound** to return 'ctx here, the returned strings
        // are from the compiler instance and can be mutated with require_extension
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross_c.cpp#L874
        unsafe {
            let extension_nums = sys::spvc_compiler_get_num_required_extensions(self.ptr.as_ptr());
            let range = 0..extension_nums;
            GlslExtensionsIter(range, self.phantom(), PhantomData)
        }
    }
}

/// Iterator for required GLSL extensions, created by [`Compiler<Glsl>::required_extensions`].
pub struct GlslExtensionsIter<'a>(
    // 'a is 'compiler.
    Range<usize>,
    // this is strictly speaking an abuse of PhantomCompiler,
    // which should be invariant in 'ctx, but as long as its properly returned in
    // required_extensions we should be safe.
    PhantomCompiler,
    PhantomData<&'a Glsl>,
);

impl_iterator!(GlslExtensionsIter<'c>: CompilerStr<'c> as and_then |s, index: usize| {
    unsafe {
        let extension = sys::spvc_compiler_get_required_extension(s.1.ptr.as_ptr(), index);
        if extension.is_null() {
            if cfg!(debug_assertions) {
                panic!("Unexpected null string returned by `spvc_compiler_get_required_extension`.\
                            The index of `spvc_compiler_get_num_required_extensions` did not match, complain to SPIRV-Cross.")
            };
            None
        } else {
            Some(CompilerStr::from_ptr(extension, s.1.ctx.clone()))
        }
    }
} for <'c> [0]);

#[cfg(test)]
mod test {
    use crate::compile::glsl::CompilerOptions;
    use spirv_cross_sys::spvc_compiler_create_compiler_options;

    use crate::compile::CompilableTarget;
    use crate::error::{SpirvCrossError, ToContextError};
    use crate::targets::Glsl;
    use crate::Compiler;
    use crate::{targets, Module};

    static BASIC_SPV: &[u8] = include_bytes!("../../../basic.spv");

    #[test]
    pub fn glsl_opts() -> Result<(), SpirvCrossError> {
        use crate::compile::sealed::ApplyCompilerOptions;

        let words = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&words));

        let compiler: Compiler<targets::Glsl> = Compiler::new(words)?;
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

        Ok(())
    }

    #[test]
    pub fn required_extensions() -> Result<(), SpirvCrossError> {
        let words = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&words));

        let mut compiler: Compiler<targets::Glsl> = Compiler::new(words)?;

        compiler.require_extension("GL_KHR_my_Extension")?;
        let extensions = compiler.required_extensions();
        assert_eq!(
            &["GL_KHR_my_Extension"],
            extensions.collect::<Vec<_>>().as_slice()
        );

        compiler.require_extension("GL_KHR_my_ExtensionS")?;
        compiler.require_extension("GL_KHR_my_ExtensionS")?;

        let extensions: Vec<_> = compiler.required_extensions().collect();
        assert_eq!(
            &["GL_KHR_my_Extension", "GL_KHR_my_ExtensionS"],
            extensions.as_slice()
        );

        let extensions = compiler.required_extensions();
        let artifact = compiler.compile(&Glsl::options())?;
        let extensions = artifact.required_extensions();

        assert_eq!(
            &["GL_KHR_my_Extension", "GL_KHR_my_ExtensionS"],
            extensions.collect::<Vec<_>>().as_slice()
        );

        Ok(())
    }
}
