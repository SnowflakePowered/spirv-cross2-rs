use super::CompilerOptions;
use crate::error::SpirvCrossError;
use crate::error::ToContextError;
use crate::{targets, Compiler, ContextRooted, Module};
#[derive(Debug, spirv_cross2_derive::CompilerOptions)]
pub struct CompileOptions {
    // common options
    /// Debug option to always emit temporary variables for all expressions.
    #[option(SPVC_COMPILER_OPTION_FORCE_TEMPORARY, false)]
    pub force_temporary: bool,

    /// Flattens multidimensional arrays, e.g. float foo[a][b][c] into single-dimensional arrays,
    /// e.g. float foo[a * b * c].
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
