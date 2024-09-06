use crate::SpirvCrossContext;
use spirv_cross_sys::{spvc_compiler_s, spvc_context_s};
use std::borrow::Borrow;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::Arc;
use crate::handle::{PointerForComparison, PointerOnlyForComparison};

/// The root lifetime of a SPIRV-Cross context.
///
/// There are mainly two lifetimes to worry about in the entire crate,
/// the context lifetime (`'ctx`), and the compiler lifetime, (unnamed, `'_`).
///
/// The context lifetime must outlive every compiler. That is, every compiler-lifetimed value
/// has lifetime at least 'ctx, **for drop purposes**. In qcell terminology, the drop-owner for
/// every value is `SpirvCrossContext`. This is because the lifetime of the compiler is rooted
/// at the lifetime of the context.
///
/// However, particularly strings, can be borrow-owned by either the context, or the compiler.
/// Values that are borrow-owned by the context are moved into [`spvc_context_s::allocations`](https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross_c.cpp#L115).
/// Note that compiler instances are borrow-owned by the context, which is why the compiler needs to carry
/// a reference in the form of a borrow or Rc to the context to maintain its liveness. It can not **own**
/// a context, because that would lead to a self-referential struct; a compiler can not be borrow-owned
/// by itself.
///
/// Values that are borrow-owned by the compiler are those that do not get copied into a buffer, and
/// can be mutated by `set` functions. These need to ensure that the lifetime of the value returned
/// matches the lifetime of the immutable borrow of the compiler.
pub enum ContextRoot<'a, T = SpirvCrossContext> {
    Borrowed(&'a T),
    RefCounted(Arc<T>),
}

impl<'a, T> ContextRoot<'a, T> {
    /// Create a new drop guard.
    pub(crate) fn drop_guard(&self) -> ContextDropGuard<'a, T> {
        ContextDropGuard(self.clone())
    }
}

/// A newtype over [`ContextRoot`] that ensures it's only used to prolong
/// the lifetime of the context
pub(crate) struct ContextDropGuard<'a, T=SpirvCrossContext>(ContextRoot<'a, T>);

impl<'a, T> Clone for ContextRoot<'a, T> {
    fn clone(&self) -> Self {
        match self {
            &ContextRoot::Borrowed(a) => ContextRoot::Borrowed(a),
            ContextRoot::RefCounted(rc) => ContextRoot::RefCounted(Arc::clone(rc)),
        }
    }
}

impl<'a, T> Borrow<T> for ContextRoot<'a, T> {
    fn borrow(&self) -> &T {
        match self {
            ContextRoot::Borrowed(a) => a,
            ContextRoot::RefCounted(a) => a.deref(),
        }
    }
}

impl<'a, T> AsRef<T> for ContextRoot<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            ContextRoot::Borrowed(a) => a,
            ContextRoot::RefCounted(a) => a.deref(),
        }
    }
}

impl ContextRoot<'_, SpirvCrossContext> {
    pub fn ptr(&self) -> NonNull<spvc_context_s> {
        match self {
            ContextRoot::Borrowed(a) => a.0,
            ContextRoot::RefCounted(a) => a.0,
        }
    }
}

pub trait WithContext<'ctx, Ptr = spvc_compiler_s, Ctx = SpirvCrossContext> {
    fn new(root: ContextRoot<'ctx, Ctx>, ptr: Ptr) -> Self;
    fn with_context<T, F>(&self, accessor: F) -> T
    where
        F: for<'a> FnOnce(&'a ContextRoot<'a, Ctx>) -> T;
    fn with_ptr<T, F>(&self, accessor: F) -> T
    where
        F: for<'a> FnOnce(&'a NonNull<Ptr>) -> T;

    fn context_mut(&mut self) -> &ContextRoot<'ctx, Ctx>;

    /// Get a reference to the pointer when
    /// mutable access is allowed.
    ///
    /// Since this borrows the struct mutably
    fn ptr_mut(&mut self) -> &NonNull<Ptr>;

    fn tag(&self) -> PointerOnlyForComparison<Ptr>;
}

pub struct UnsendContext<'ctx, Ptr=spvc_compiler_s, Ctx=SpirvCrossContext> {
    root: ContextRoot<'ctx, Ctx>,
    ptr: Ptr,
}

impl<Ptr: Clone, Ctx> Clone for UnsendContext<'_, Ptr, Ctx> {
    fn clone(&self) -> Self {
        UnsendContext {
            root: self.root.clone(),
            ptr: self.ptr.clone(),
        }
    }
}

impl<'ctx, Ctx, Ptr> WithContext<'ctx, Ptr, Ctx> for UnsendContext<'ctx, Ptr, Ctx> {
    fn new(root: ContextRoot<'ctx, Ctx>, ptr: Ptr) -> Self {
        Self { root, ptr }
    }

    fn with_context<T, F>(&self, accessor: F) -> T
    where
        F: for<'a> FnOnce(&'a ContextRoot<'a, Ctx>) -> T,
    {
        accessor(&self.root)
    }

    fn with_ptr<T, F>(&self, accessor: F) -> T
    where
        F: for<'a> FnOnce(&'a NonNull<Ptr>) -> T,
    {
        accessor(&self.ptr)
    }

    fn context_mut(&mut self) -> &ContextRoot<'ctx, Ctx> {
        &self.root
    }

    fn ptr_mut(&mut self) -> &NonNull<Ptr> {
        &self.ptr
    }

    fn tag(&self) -> PointerOnlyForComparison<Ptr> {
        PointerOnlyForComparison(*self.ptr)
    }
}

pub struct SynchronizedContext<'ctx, Ptr=spvc_compiler_s, Ctx=SpirvCrossContext> {
    root: Arc<parking_lot::Mutex<ContextRoot<'ctx, Ctx>>>,
    ptr: Ptr,
}

impl<Ptr: Clone, Ctx> Clone for SynchronizedContext<'_, Ptr, Ctx> {
    fn clone(&self) -> Self {
        SynchronizedContext {
            root: Arc::clone(&self.root),
            ptr: self.ptr.clone(),
        }
    }
}
impl<'ctx, Ctx, Ptr> WithContext<'ctx, Ptr, Ctx> for SynchronizedContext<'ctx, Ptr, Ctx> {
    fn new(root: ContextRoot<'ctx, Ctx>, ptr: NonNull<Ptr>) -> Self {
        Self {
            root: Arc::new(parking_lot::const_mutex(root)),
            ptr,
        }
    }

    fn with_context<T, F>(&self, accessor: F) -> T
    where
        F: for<'a> FnOnce(&'a ContextRoot<'a, Ctx>) -> T,
    {
        let lock = self.root.lock();
        let res = accessor(lock.deref());
        drop(lock);
        res
    }

    fn with_ptr<T, F>(&self, accessor: F) -> T
    where
        F: for<'a> FnOnce(&'a NonNull<Ptr>) -> T,
    {
        // IMPORTANT: context must be locked for safety, otherwise
        // there is no synchronization of the interior allocator.
        let ctx_lock = self.root.lock();
        let res = accessor(&self.ptr);
        drop(ctx_lock);
        res
    }

    fn context_mut(&mut self) -> &ContextRoot<'ctx, Ctx> {
        &*self.root.get_mut()
    }

    fn ptr_mut(&mut self) -> &NonNull<Ptr> {
        &self.ptr
    }

    fn tag(&self) -> PointerOnlyForComparison<Ptr> {
        PointerOnlyForComparison(*self.ptr)
    }
}
