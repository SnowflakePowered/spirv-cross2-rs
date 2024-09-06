use crate::error::SpirvCrossError;
use crate::{error, Compiler};
use spirv_cross_sys::spvc_compiler_s;
use std::fmt::{Debug, Formatter};
use std::ptr::NonNull;

use crate::sealed::Sealed;

/// A SPIR-V ID to a specialization constant.
pub use spirv_cross_sys::ConstantId;

/// A SPIR-V ID to a type.
pub use spirv_cross_sys::TypeId;

/// A SPIR-V ID to a variable.
pub use spirv_cross_sys::VariableId;
use crate::sync::WithContext;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct PointerOnlyForComparison<T>(NonNull<T>);

// SAFETY: pointer is only for comparison.
unsafe impl<T> Send for PointerOnlyForComparison<T> {}
unsafe impl<T> Sync for PointerOnlyForComparison<T> {}

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

/// A reference to an ID referring to an item in the compiler instance.
///
/// The usage of `Handle<T>` ensures that item IDs can not be forged from
/// a different compiler instance or from a `u32`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Handle<T> {
    id: T,
    tag: PointerOnlyForComparison<spvc_compiler_s>,
}

impl<T: Id> Handle<T> {
    /// Return the `u32` part of the Id.
    ///
    /// Note that [`Handle<T>`] **can not** implement [`Id`]
    /// for safety reasons. Getting an `impl Id` out of a
    /// [`Handle<T>`] requires using [`Compiler::yield_id`].
    pub fn id(&self) -> u32 {
        self.id.id()
    }
}

/// Trait for SPIRV-Cross ID types.
pub trait Id: Sealed + Debug + Send + Sync + 'static {
    /// Return the `u32` part of the Id.
    fn id(&self) -> u32;
}

impl Sealed for TypeId {}
impl Id for TypeId {
    #[inline(always)]
    fn id(&self) -> u32 {
        self.0 .0
    }
}

impl Sealed for VariableId {}
impl Id for VariableId {
    #[inline(always)]
    fn id(&self) -> u32 {
        self.0 .0
    }
}

impl Sealed for ConstantId {}
impl Id for ConstantId {
    #[inline(always)]
    fn id(&self) -> u32 {
        self.0 .0
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
impl<T, L: WithContext> Compiler<'_, T, L> {
    #[inline(always)]
    /// Create a handle for the given ID tagged with this compiler instance.
    ///
    /// # Safety
    /// When creating a handle, the ID must be valid for the compilation.
    pub unsafe fn create_handle<I>(&self, id: I) -> Handle<I> {
        Handle {
            id,
            tag: PointerOnlyForComparison(self.ptr),
        }
    }

    #[inline(always)]
    /// Create a handle for the given ID tagged with this compiler instance,
    /// if the provided ID is not zero.
    ///
    /// # Safety
    /// When creating a handle, the ID must be valid for the compilation.
    pub unsafe fn create_handle_if_not_zero<I: Id>(&self, id: I) -> Option<Handle<I>> {
        let raw = id.id();
        if raw == 0 {
            return None;
        }
        Some(Handle {
            id,
            tag: PointerOnlyForComparison(self.ptr),
        })
    }

    /// Returns whether the given handle is valid for this compiler instance.
    pub fn handle_is_valid<I>(&self, handle: &Handle<I>) -> bool {
        handle.tag == self.lock.tag()
    }

    /// Yield the value of the handle, if it originated from the same context,
    /// otherwise return [`SpirvCrossError::InvalidHandle`].
    pub fn yield_id<I: Id>(&self, handle: Handle<I>) -> error::Result<I> {
        if self.handle_is_valid(&handle) {
            Ok(handle.id)
        } else {
            Err(SpirvCrossError::InvalidHandle(handle.erase_type()))
        }
    }
}

impl<'ctx, T, Lock: WithContext> Compiler<'ctx, T, Lock> {
    /// Create a type erased phantom for lifetime tracking purposes.
    ///
    /// This function is unsafe because a [`PhantomCompiler`] can be used to
    /// **safely** create handles originating from the compiler.
    pub(crate) unsafe fn phantom(&self) -> PhantomCompiler<'ctx> {
        PhantomCompiler { tag: self.lock.tag() }
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
            tag: self.tag,
        }
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
/// The only thing a [`PhantomCompiler`] is able to do is create handles.
///
/// It's lifetime should be the same as the lifetime
/// of the **context**, or **shorter**, but at least the lifetime of the compiler.
#[derive(Clone)]
pub(crate) struct PhantomCompiler<'ctx> {
    tag: PointerOnlyForComparison<spvc_compiler_s>
}