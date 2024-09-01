use super::CompilerOptions;
use crate::error::SpirvCrossError;
use crate::error::ToContextError;
use crate::{targets, Compiler, ContextRooted, Module};
use spirv_cross_sys::spvc_compiler_option as opt;
#[derive(Debug, Default, spirv_cross2_derive::CompilerOptions)]
pub struct CompileOptions {
    /// Flip vertex y
    #[option(opt::SPVC_COMPILER_OPTION_FLIP_VERTEX_Y)]
    flip_vertex_y: bool,
    /// Flip vertex y
    #[option(opt::SPVC_COMPILER_OPTION_FORCE_TEMPORARY)]
    force_temporary: bool,
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
