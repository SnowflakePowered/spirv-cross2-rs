use crate::compiler::{Compiler, PhantomCompiler};
use crate::error;
use crate::error::SpirvCrossError;
use spirv_cross_sys::{spvc_compiler_s, ConstantId, SpvId, TypeId, VariableId};
use std::fmt::{Debug, Formatter};
use std::ptr::NonNull;

#[derive(Copy, Clone)]
struct PointerOnlyForComparison<T>(NonNull<T>);

impl<T> PartialEq for PointerOnlyForComparison<T> {
    fn eq(&self, other: &Self) -> bool {
        other.0.as_ptr() == self.0.as_ptr()
    }
}

impl<T> Eq for PointerOnlyForComparison<T> {}

impl<T> Debug for PointerOnlyForComparison<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Truncate the tag, we don't really care about the upper 32 bytes.
        // - Chop off ignored 16 bits
        // - Low 2 bits are always 0, so we can ignore that too.
        // - Either the low or high 32 bits remaining are good enough to show uniqueness.
        write!(
            f,
            "Tag({:x})",
            (((self.0.as_ptr() as usize) << 16) >> 18) as u32
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Handle<T> {
    id: T,
    tag: PointerOnlyForComparison<spvc_compiler_s>,
}

/// Trait for SPIRV-Cross Ids.
pub trait Id: Debug + 'static {
    /// Return the `u32` part of the Id.
    fn id(&self) -> u32;
}

impl Id for TypeId {
    #[inline(always)]
    fn id(&self) -> u32 {
        self.0 .0
    }
}
impl Id for VariableId {
    #[inline(always)]
    fn id(&self) -> u32 {
        self.0 .0
    }
}
impl Id for ConstantId {
    #[inline(always)]
    fn id(&self) -> u32 {
        self.0 .0
    }
}

impl Id for SpvId {
    #[inline(always)]
    fn id(&self) -> u32 {
        self.0
    }
}

impl<T: Id> Id for Handle<T> {
    #[inline(always)]
    fn id(&self) -> u32 {
        self.id.id()
    }
}

impl<T: Id> Handle<T> {
    /// Erase the type of the handle, this is useful for errors
    /// but is otherwise useless.
    #[cold]
    fn erase_type(self) -> Handle<Box<dyn Id>> {
        Handle {
            id: Box::new(self.id) as Box<dyn Id>,
            tag: self.tag,
        }
    }
}

/// APIs for comparing handles
impl<T> Compiler<'_, T> {
    #[inline(always)]
    /// Create a handle for the given ID tagged with this compiler instance.
    ///
    /// # Safety
    /// When creating a handle, the ID must be valid for the compilation.
    pub unsafe fn create_handle<I>(&self, id: I) -> Handle<I> {
        Handle {
            id,
            tag: PointerOnlyForComparison(self.0),
        }
    }

    /// Returns whether the given handle is valid for this compiler instance.
    pub fn handle_is_valid<I>(&self, handle: &Handle<I>) -> bool {
        handle.tag == PointerOnlyForComparison(self.0)
    }

    /// Yield the value of the handle, if it originated from the same context,
    /// otherwise return [`SpirvCrossError::InvalidHandle`](crate::error::SpirvCrossError::InvalidHandle).
    pub fn yield_id<I: Id>(&self, handle: Handle<I>) -> error::Result<I> {
        if self.handle_is_valid(&handle) {
            Ok(handle.id)
        } else {
            Err(SpirvCrossError::InvalidHandle(handle.erase_type()))
        }
    }
}

impl PhantomCompiler<'_> {
    /// Internal method for creating a handle
    ///
    /// This is not marked unsafe, because it is only ever used internally
    /// for handles valid for a compiler instance, i.e. we never smuggle
    /// an invalid handle. Marking it unsafe would make it too noisy to
    /// audit actually unsafe code.
    ///
    /// This is not necessarily the case for the public API.
    #[inline(always)]
    pub(crate) fn create_handle<I>(&self, id: I) -> Handle<I> {
        Handle {
            id,
            tag: PointerOnlyForComparison(self.ptr),
        }
    }
}
