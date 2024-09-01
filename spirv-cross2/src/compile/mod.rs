use std::ops::Deref;
use crate::error::{ContextRooted, Result, ToContextError};
use crate::handle::Handle;
use crate::targets::CompilableTarget;
use crate::{error, spirv, Compiler};
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_compiler_options, VariableId};
use std::ptr::NonNull;
pub mod glsl;
pub mod hlsl;
pub mod msl;

pub struct CompiledArtifact<'a, T> {
    compiler: Compiler<'a, T>
}

impl<'a, T> Deref for CompiledArtifact<'a, T> {
    type Target = Compiler<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.compiler
    }
}

/// Cross-compilation related methods.
impl<'a, T: CompilableTarget> Compiler<'a, T> {
    pub fn add_header_line(&mut self, line: &str) -> Result<()> {
        unsafe {
            sys::spvc_compiler_add_header_line(self.ptr.as_ptr(), line.as_ptr().cast()).ok(self)
        }
    }

    pub fn flatten_buffer_block(&mut self, block: VariableId) -> Result<()> {
        unsafe { sys::spvc_compiler_flatten_buffer_block(self.ptr.as_ptr(), block).ok(self) }
    }

    pub fn require_extension(&mut self, ext: &str) -> Result<()> {
        unsafe {
            sys::spvc_compiler_require_extension(self.ptr.as_ptr(), ext.as_ptr().cast()).ok(self)
        }
    }

    pub fn mask_stage_output_by_location(&mut self, location: u32, component: u32) -> Result<()> {
        unsafe {
            sys::spvc_compiler_mask_stage_output_by_location(self.ptr.as_ptr(), location, component)
                .ok(&*self)
        }
    }

    pub fn mask_stage_output_by_builtin(&mut self, builtin: spirv::BuiltIn) -> Result<()> {
        unsafe {
            sys::spvc_compiler_mask_stage_output_by_builtin(self.ptr.as_ptr(), builtin).ok(&*self)
        }
    }

    pub fn variable_is_depth_or_compare(&self, variable: Handle<VariableId>) -> Result<bool> {
        let id = self.yield_id(variable)?;
        unsafe {
            Ok(sys::spvc_compiler_variable_is_depth_or_compare(
                self.ptr.as_ptr(),
                id,
            ))
        }
    }

    /// Apply the set of compiler options to the compiler instance.
    pub fn set_compiler_options(&mut self, options: &T::Options) -> error::Result<()> {
        unsafe {
            let mut handle = std::ptr::null_mut();

            sys::spvc_compiler_create_compiler_options(self.ptr.as_ptr(), &mut handle)
                .ok(&*self)?;

            options.apply(handle, &*self)?;

            sys::spvc_compiler_install_compiler_options(self.ptr.as_ptr(), handle)
                .ok(&*self)?;

            Ok(())
        }
    }

    /// Consume the compilation instance, and compile source code to the
    /// output target.
    pub fn compile(self) -> CompiledArtifact<'a, T> {
        // todo: actually do the compilation.

        CompiledArtifact {
            compiler: self
        }
    }
}

pub(crate) trait CompilerOptions {
    unsafe fn apply(
        &self,
        options: spvc_compiler_options,
        root: impl ContextRooted + Copy,
    ) -> error::Result<()>;
}

#[cfg(test)]
mod test {
    use crate::error::SpirvCrossError;
    use crate::targets;
    use crate::Compiler;
    use crate::{Module, SpirvCross};

    const BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn create_compiler() -> Result<(), SpirvCrossError> {
        let spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        Ok(())
    }

    #[test]
    pub fn reflect_interface_vars() -> Result<(), SpirvCrossError> {
        let spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let mut compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let vars = compiler.active_interface_variables()?;
        assert_eq!(
            &[13, 9],
            &vars
                .to_handles()
                .into_iter()
                .map(|h| h.id())
                .collect::<Vec<_>>()
                .as_slice()
        );

        compiler.set_enabled_interface_variables(vars)?;
        Ok(())
    }
}
