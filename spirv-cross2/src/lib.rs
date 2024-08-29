use crate::compiler::{Compiler, Target};
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_context_s, ContextRooted, SpirvCrossError, SpvId};

use std::marker::PhantomData;
use std::ptr::NonNull;
pub mod compiler;
mod error;
pub mod spirv;

/// The SPIRV-Cross context. All memory allocations originating from
/// this context will have the same lifetime as the context.
pub struct SpirvCross(NonNull<spvc_context_s>);

pub struct Module<'a>(&'a [SpvId]);

impl<'a> Module<'a> {
    pub fn from_words(words: &'a [u32]) -> Self {
        const {
            assert!(std::mem::size_of::<u32>() == std::mem::size_of::<SpvId>());
        }

        Module(bytemuck::cast_slice(words))
    }
}

impl SpirvCross {
    /// Initialize a new SPIRV-Cross context.
    pub fn new() -> error::Result<Self> {
        unsafe {
            let mut context = std::ptr::null_mut();
            let result = sys::spvc_context_create(&mut context);

            if result != sys::spvc_result::SPVC_SUCCESS {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            }

            let Some(context) = NonNull::new(context) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(Self(context))
        }
    }

    /// Create a compiler instance from a SPIR-V module.
    pub fn create_compiler<T: Target>(&self, spirv: Module) -> error::Result<Compiler<T>> {
        unsafe {
            let mut ir = std::ptr::null_mut();
            sys::spvc_context_parse_spirv(
                self.0.as_ptr(),
                spirv.0.as_ptr(),
                spirv.0.len(),
                &mut ir,
            )
            .ok(self)?;

            let mut compiler = std::ptr::null_mut();
            sys::spvc_context_create_compiler(
                self.0.as_ptr(),
                T::BACKEND,
                ir,
                spirv_cross_sys::CaptureMode::TakeOwnership,
                &mut compiler,
            )
            .ok(self)?;

            let Some(compiler) = NonNull::new(compiler) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(Compiler(compiler, &self, PhantomData))
        }
    }
}

impl Drop for SpirvCross {
    fn drop(&mut self) {
        unsafe { sys::spvc_context_destroy(self.0.as_ptr()) }
    }
}

impl ContextRooted for &SpirvCross {
    fn context(&self) -> NonNull<spvc_context_s> {
        self.0
    }
}

trait ToStatic {
    type Static<'a>
    where
        'a: 'static;
    fn to_static(&self) -> Self::Static<'static>;
}

#[cfg(test)]
mod test {
    use crate::SpirvCross;

    #[test]
    pub fn init_context_test() {
        SpirvCross::new().unwrap();
    }
}
