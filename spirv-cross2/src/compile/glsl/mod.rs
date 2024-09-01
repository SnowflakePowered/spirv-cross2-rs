use super::{CommonCompileOptions, CompilerOptions};
use crate::error::ToContextError;
use crate::{targets, Compiler, ContextRooted, Module};
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_compiler_option, spvc_compiler_options};
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CompileOptions {
    // common options
    #[expand]
    /// Compile options common to GLSL, HLSL, and MSL.
    pub common: CommonCompileOptions,

    #[expand]
    /// The GLSL version to output. The default is #version 450.
    pub version: GlslVersion,
}

/// GLSL language version.
#[derive(Debug)]
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

impl Default for GlslVersion {
    fn default() -> Self {
        GlslVersion::Glsl450
    }
}

impl CompilerOptions for GlslVersion {
    unsafe fn apply(
        &self,
        options: spvc_compiler_options,
        root: impl ContextRooted + Copy,
    ) -> crate::error::Result<()> {
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

#[cfg(test)]
mod test {
    use crate::compile::glsl::CompileOptions;
    use crate::compile::CompilerOptions;
    use spirv_cross_sys::spvc_compiler_create_compiler_options;

    use crate::error::{SpirvCrossError, ToContextError};
    use crate::Compiler;
    use crate::{targets, Module, SpirvCross};

    static BASIC_SPV: &[u8] = include_bytes!("../../../basic.spv");

    #[test]
    pub fn glsl_opts() -> Result<(), SpirvCrossError> {
        let spv = SpirvCross::new()?;
        let words = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&words));

        let compiler: Compiler<targets::Glsl> = spv.create_compiler(words)?;
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
