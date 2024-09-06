use spirv_cross_sys as sys;
use spirv_cross_sys::spvc_context_s;
use std::ptr::NonNull;
use std::sync::Arc;

use crate::error::{ContextRooted, ToContextError};
use crate::targets::Target;
use crate::{error, Compiler, Module, PhantomCompiler, SpirvCrossError};

/// The SPIRV-Cross context. All memory allocations originating from
/// this context will have the same lifetime as the context.
///
/// This acts as an interior-mutability cell to a vector within the SPIRV-Cross
/// C API that keeps track of all allocations.
///
/// A [`Compiler`] must have unique ownership of a `CrossAllocationCell`
/// in order to safely be `Send`. If multiple compilers have access to the
/// same cell, then there could be multiple mutable references to the underlying
/// cell.
///
/// To extend the lifetime, use [`CrossAllocationCell::drop_guard`], which returns
/// a dropguard that can't do anything except extend the lifetime.
#[repr(transparent)]
pub(crate) struct CrossAllocationCell(Arc<CrossAllocationCellInner>);

pub(crate) struct CrossAllocationCellInner(NonNull<spvc_context_s>);

/// A neutered `CrossAllocationCell` that is used to extend the lifetime
/// of a foreign allocation.
#[repr(transparent)]
pub(crate) struct AllocationDropGuard<T = CrossAllocationCellInner>(Arc<T>);

impl<T> Clone for AllocationDropGuard<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl CrossAllocationCell {
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

            Ok(Self(Arc::new(CrossAllocationCellInner(context))))
        }
    }

    /// Create a compiler instance from a SPIR-V module.
    ///
    /// This consumes the instance so the resulting compiler instance is static,
    /// and allocations will be dropped with the compiler.
    ///
    /// This allows for instances to be stored without keeping a reference to the
    /// context separately.
    pub(crate) fn into_compiler<T: Target>(self, spirv: Module) -> error::Result<Compiler<T>> {
        unsafe {
            let mut ir = std::ptr::null_mut();
            sys::spvc_context_parse_spirv(
                self.0 .0.as_ptr(),
                spirv.0.as_ptr(),
                spirv.0.len(),
                &mut ir,
            )
            .ok(&self)?;

            let mut compiler = std::ptr::null_mut();
            sys::spvc_context_create_compiler(
                self.0 .0.as_ptr(),
                T::BACKEND,
                ir,
                spirv_cross_sys::spvc_capture_mode::TakeOwnership,
                &mut compiler,
            )
            .ok(&self)?;

            let Some(compiler) = NonNull::new(compiler) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(Compiler::new_from_raw(compiler, self))
        }
    }

    /// Get a pointer to the context.
    ///
    /// SAFETY: must have unique ownership.
    pub unsafe fn as_ptr(&self) -> NonNull<spvc_context_s> {
        self.0 .0
    }

    /// Produce a drop guard for the allocation cell.
    pub fn drop_guard(&self) -> AllocationDropGuard {
        AllocationDropGuard(Arc::clone(&self.0))
    }
}

impl Drop for CrossAllocationCellInner {
    fn drop(&mut self) {
        unsafe { sys::spvc_context_destroy(self.0.as_ptr()) }
    }
}

impl ContextRooted for &CrossAllocationCell {
    fn context(&self) -> NonNull<spvc_context_s> {
        unsafe { self.as_ptr() }
    }
}

impl<T> ContextRooted for &Compiler<T> {
    fn context(&self) -> NonNull<spvc_context_s> {
        unsafe { self.ctx.as_ptr() }
    }
}

impl ContextRooted for &PhantomCompiler {
    #[inline(always)]
    fn context(&self) -> NonNull<spvc_context_s> {
        self.ctx.0 .0
    }
}
