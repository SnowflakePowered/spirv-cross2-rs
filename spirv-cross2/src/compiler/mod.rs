use crate::error::{ContextRooted, Result, ToContextError};
use crate::handle::Handle;
use crate::ContextRoot;
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_compiler_s, spvc_context_s, spvc_set, VariableId};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub mod buffers;
mod combined_image_samplers;
mod constants;
pub mod hlsl;
pub mod msl;
pub mod resources;
pub mod types;

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
    pub(super) ContextRoot<'a>,
    pub(super) PhantomData<T>,
);

pub struct InterfaceVariableSet<'a>(spvc_set, PhantomData<&'a ()>);

impl<T> ContextRooted for &Compiler<'_, T> {
    #[inline(always)]
    fn context(&self) -> NonNull<spvc_context_s> {
        self.1.ptr()
    }
}

impl<T> ContextRooted for &mut Compiler<'_, T> {
    #[inline(always)]
    fn context(&self) -> NonNull<spvc_context_s> {
        self.1.ptr()
    }
}

/// Holds on to the pointer for a compiler instance,
/// but type erased.
///
/// This is used so that child resources of a compiler track the
/// lifetime of a compiler, or create handles attached with the
/// compiler instance, without needing to refer to the typed
/// output of a compiler.
///
/// The only thing a [`PhantomCompiler`] is able to do is create handles or
/// refer to the root context. It's lifetime
#[derive(Copy, Clone)]
pub(crate) struct PhantomCompiler<'a> {
    pub(crate) ptr: NonNull<spvc_compiler_s>,
    ctx: NonNull<spvc_context_s>,
    _pd: PhantomData<&'a ()>,
}

impl ContextRooted for PhantomCompiler<'_> {
    fn context(&self) -> NonNull<spvc_context_s> {
        self.ctx
    }
}

impl<'a, T> Compiler<'a, T> {
    /// Create a type erased phantom for lifetime tracking purposes.
    pub(crate) fn phantom(&self) -> PhantomCompiler<'a> {
        PhantomCompiler {
            ptr: self.0,
            ctx: self.context(),
            _pd: PhantomData,
        }
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
impl<'a, T> Compiler<'a, T> {
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
    pub fn active_interface_variables(&self) -> Result<InterfaceVariableSet<'a>> {
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
}

#[cfg(test)]
mod test {
    use crate::compiler::{targets, Compiler};
    use crate::error::SpirvCrossError;
    use crate::{Module, SpirvCross};
    const BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn create_compiler() -> Result<(), SpirvCrossError> {
        let mut spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        Ok(())
    }

    #[test]
    pub fn reflect_interface_vars() -> Result<(), SpirvCrossError> {
        let mut spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let mut compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let vars = compiler.active_interface_variables()?;
        assert_eq!(&[13, 9], &vars.reflect().as_slice());

        compiler.set_enabled_interface_variables(vars)?;
        Ok(())
    }
}
