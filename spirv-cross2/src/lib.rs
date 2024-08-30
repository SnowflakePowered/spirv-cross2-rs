use crate::compiler::{Compiler, Target};
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_context_s, SpvId};
use std::borrow::Borrow;

use crate::error::{ContextRooted, SpirvCrossError, ToContextError};

use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, Index};
use std::ptr::NonNull;
use std::rc::Rc;

pub mod compiler;
pub mod error;

/// SPIR-V types and definitions.
pub mod spirv;

pub mod handle;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// The SPIRV-Cross context. All memory allocations originating from
/// this context will have the same lifetime as the context.
#[repr(transparent)]
pub struct SpirvCross(NonNull<spvc_context_s>);

enum ContextRoot<'a> {
    Owned(SpirvCross),
    Borrowed(&'a SpirvCross),
    RefCounted(Rc<SpirvCross>),
}

impl<'a> Borrow<SpirvCross> for ContextRoot<'a> {
    fn borrow(&self) -> &SpirvCross {
        match self {
            ContextRoot::Owned(a) => a,
            ContextRoot::Borrowed(a) => *a,
            ContextRoot::RefCounted(a) => a.deref(),
        }
    }
}

impl<'a> AsRef<SpirvCross> for ContextRoot<'a> {
    fn as_ref(&self) -> &SpirvCross {
        match self {
            ContextRoot::Owned(a) => a,
            ContextRoot::Borrowed(a) => *a,
            ContextRoot::RefCounted(a) => a.deref(),
        }
    }
}

impl ContextRoot<'_> {
    fn ptr(&self) -> NonNull<spvc_context_s> {
        match self {
            ContextRoot::Owned(a) => a.0,
            ContextRoot::Borrowed(a) => a.0,
            ContextRoot::RefCounted(a) => a.0,
        }
    }
}

pub struct CompilerCell<T> {
    context: ManuallyDrop<SpirvCross>,
    instances: Vec<Compiler<'static, T>>,
}

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
        // SAFETY:
        //
        // `SpirvCross::create_compiler` is not mut here, because
        // it only mutates the [allocations](https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross_c.cpp#L343)
        // field, which is never observable from Rust.
        //
        // While `allocations` can reallocate being a `SmallVector<std::unique_ptr>`,
        // the actual pointer returned is pinned to `spvc_context` for the lifetime of `Self`.
        // Even if `allocations` reallocates, the pointer returned will always be valid
        // for the lifetime of `spvc_context`.
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

            Ok(Compiler(
                compiler,
                ContextRoot::Borrowed(&self),
                PhantomData,
            ))
        }
    }

    /// Create a compiler instance from a SPIR-V module.
    ///
    /// The compiler instance created carries with it a refcounted
    /// pointer to the SPIRV-Cross context, and thus has a `'static`
    /// lifetime.
    pub fn create_compiler_refcounted<T: Target>(
        self: &Rc<Self>,
        spirv: Module,
    ) -> error::Result<Compiler<'static, T>> {
        unsafe {
            let mut ir = std::ptr::null_mut();
            sys::spvc_context_parse_spirv(
                self.0.as_ptr(),
                spirv.0.as_ptr(),
                spirv.0.len(),
                &mut ir,
            )
            .ok(&**self)?;

            let mut compiler = std::ptr::null_mut();
            sys::spvc_context_create_compiler(
                self.0.as_ptr(),
                T::BACKEND,
                ir,
                spirv_cross_sys::CaptureMode::TakeOwnership,
                &mut compiler,
            )
            .ok(&**self)?;

            let Some(compiler) = NonNull::new(compiler) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(Compiler(
                compiler,
                ContextRoot::RefCounted(Rc::clone(&self)),
                PhantomData,
            ))
        }
    }

    /// Create a compiler instance from a SPIR-V module.
    ///
    /// This consumes the instance so the resulting compiler instance is static,
    /// and allocations will be dropped with the compiler.
    ///
    /// This allows for instances to be stored without keeping a reference to the
    /// context separately.
    pub fn into_compiler<T: Target>(self, spirv: Module) -> error::Result<Compiler<'static, T>> {
        unsafe {
            let mut ir = std::ptr::null_mut();
            sys::spvc_context_parse_spirv(
                self.0.as_ptr(),
                spirv.0.as_ptr(),
                spirv.0.len(),
                &mut ir,
            )
            .ok(&self)?;

            let mut compiler = std::ptr::null_mut();
            sys::spvc_context_create_compiler(
                self.0.as_ptr(),
                T::BACKEND,
                ir,
                spirv_cross_sys::CaptureMode::TakeOwnership,
                &mut compiler,
            )
            .ok(&self)?;

            let Some(compiler) = NonNull::new(compiler) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(Compiler(compiler, ContextRoot::Owned(self), PhantomData))
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
