use crate::error::Result;
use crate::SpirvCross;
use spirv_cross_sys::{
    spvc_compiler_s, spvc_context_create_compiler, spvc_context_parse_spirv, spvc_context_s,
    spvc_resources_s, spvc_set, spvc_set_s, ContextRooted, SpirvCrossError, SpvId, VariableId,
};
use std::marker::PhantomData;
use std::ptr::NonNull;

use spirv_cross_sys as sys;

pub mod hlsl;
pub mod msl;
pub mod resources;
mod types;

pub mod targets {
    use crate::compiler::Target;
    use spirv_cross_sys::CompilerBackend;

    pub struct None;
    pub struct Glsl;
    pub struct Msl;
    pub struct Hlsl;
    pub struct Json;

    impl Target for None {
        const BACKEND: CompilerBackend = CompilerBackend::None;
    }
    impl Target for Glsl {
        const BACKEND: CompilerBackend = CompilerBackend::Glsl;
    }
}

pub trait Target {
    const BACKEND: sys::CompilerBackend;
}

pub struct Compiler<'a, T>(
    pub(super) NonNull<spvc_compiler_s>,
    pub(super) &'a SpirvCross,
    pub(super) PhantomData<T>,
);

pub struct InterfaceVariableSet<'a>(spvc_set, PhantomData<&'a ()>);
pub struct ShaderResources<'a>(NonNull<spvc_resources_s>, &'a SpirvCross);

impl<T> ContextRooted for &Compiler<'_, T> {
    #[inline(always)]
    fn context(&self) -> NonNull<spvc_context_s> {
        self.1 .0
    }
}

impl<T> ContextRooted for &mut Compiler<'_, T> {
    #[inline(always)]
    fn context(&self) -> NonNull<spvc_context_s> {
        self.1 .0
    }
}

impl<T> Compiler<'_, T> {
    pub fn add_header_line(&mut self, line: &str) -> Result<()> {
        unsafe {
            sys::spvc_compiler_add_header_line(self.0.as_ptr(), line.as_ptr().cast()).ok(self)
        }
    }

    pub fn flatten_buffer_block(&mut self, block: VariableId) -> Result<()> {
        unsafe { sys::spvc_compiler_flatten_buffer_block(self.0.as_ptr(), block).ok(self) }
    }

    pub fn require_extension(&mut self, ext: &str) -> Result<()> {
        unsafe {
            sys::spvc_compiler_require_extension(self.0.as_ptr(), ext.as_ptr().cast()).ok(self)
        }
    }
}

// reflection
impl<T> Compiler<'_, T> {
    /// Returns a set of all global variables which are statically accessed
    /// by the control flow graph from the current entry point.
    /// Only variables which change the interface for a shader are returned, that is,
    /// variables with storage class of Input, Output, Uniform, UniformConstant, PushConstant and AtomicCounter
    /// storage classes are returned.
    ///
    /// To use the returned set as the filter for which variables are used during compilation,
    /// this set can be moved to set_enabled_interface_variables().
    ///
    /// The return object is opaque to Rust, but its contents inspected by using [`InterfaceVariableSet::reflect`].
    /// There is no way to modify the contents or use your own `InterfaceVariableSet`.
    pub fn active_interface_variables(&self) -> Result<InterfaceVariableSet> {
        unsafe {
            let mut set = std::ptr::null();
            sys::spvc_compiler_get_active_interface_variables(self.0.as_ptr(), &mut set)
                .ok(self)?;

            Ok(InterfaceVariableSet(set, PhantomData))
        }
    }

    /// Sets the interface variables which are used during compilation.
    /// By default, all variables are used.
    /// Once set, [`Compiler::compile`] will only consider the set in active_variables.
    pub fn set_enabled_interface_variables(&mut self, set: InterfaceVariableSet) -> Result<()> {
        unsafe {
            sys::spvc_compiler_set_enabled_interface_variables(self.0.as_ptr(), set.0).ok(self)?;
            Ok(())
        }
    }

    /// Query shader resources, use ids with reflection interface to modify or query binding points, etc.
    pub fn shader_resources(&self) -> Result<ShaderResources> {
        unsafe {
            let mut resources = std::ptr::null_mut();
            sys::spvc_compiler_create_shader_resources(self.0.as_ptr(), &mut resources).ok(self)?;

            let Some(resources) = NonNull::new(resources) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(ShaderResources(resources, self.1))
        }
    }

    /// Query shader resources, but only return the variables which are part of active_variables.
    /// E.g.: get_shader_resources(get_active_variables()) to only return the variables which are statically
    /// accessed.
    pub fn shader_resources_for_active_variables(
        &self,
        set: InterfaceVariableSet,
    ) -> Result<ShaderResources> {
        unsafe {
            let mut resources = std::ptr::null_mut();
            sys::spvc_compiler_create_shader_resources_for_active_variables(
                self.0.as_ptr(),
                &mut resources,
                set.0,
            )
            .ok(self)?;

            let Some(resources) = NonNull::new(resources) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(ShaderResources(resources, self.1))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::{targets, Compiler};
    use crate::{Module, SpirvCross};
    use spirv_cross_sys::SpirvCrossError;

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

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let vars = compiler.active_interface_variables()?;
        assert_eq!(&[13, 9], &vars.reflect().as_slice());
        Ok(())
    }
}
