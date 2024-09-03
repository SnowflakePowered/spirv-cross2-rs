use crate::{ContextRoot, SpirvCrossContext};
use std::borrow::Cow;
use std::ffi::{c_char, CStr, CString, NulError};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

/// An immutable wrapper around a valid UTF-8 string whose memory contents
/// may or may not be originating from a [`SpirvCrossContext`](crate::SpirvCrossContext)
/// context.
///
/// In most cases, users of this library do not need to worry about
/// constructing a [`ContextStr`]. All functions that take strings
/// take `impl Into<ContextStr<'_>>`, which converts automatically from
/// [`&str`](str) and [`String`](String).
///
/// [`ContextStr`] also implements [`Deref`](Deref) for [`&str`](str),
/// so all immutable `str` methods are available.
///
/// # Allocation Behaviour
/// If the string originated from FFI and is a valid nul-terminated
/// C string, then the pointer to the string will be saved,
/// such that when being read by FFI there are no extra allocations
/// required.
///
/// If the provenance of the string is an owned Rust `String`, or
/// a `&str` with lifetime longer than `'a`, then an allocation will
/// occur when passing the string to FFI.
///
/// # Safety
/// Returning `ContextStr<'a>` where `'a` is the lifetime of the
/// [`SpirvCrossContext`](crate::SpirvCrossContext) is **almost always incorrect**.
///
/// The only exception is if the name is explicitly owned by the context,
/// and can not be modified by a `set_`. function.
///
/// In most cases, the returned lifetime should be the lifetime of the mutable borrow,
/// if returning a string from the [`Compiler`].
///
/// [`ContextStr::from_ptr`] takes a context argument, and the context must be
/// the source of provenance for the `ContextStr`.
pub struct ContextStr<'a, T = SpirvCrossContext> {
    pointer: Option<ContextPointer<'a, T>>,
    cow: Cow<'a, str>,
}

impl<T> Clone for ContextStr<'_, T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone(),
            cow: self.cow.clone(),
        }
    }
}

pub(crate) struct ContextPointer<'a, T> {
    // the lifetime of pointer should be 'a.
    pointer: *const c_char,
    context: ContextRoot<'a, T>,
}

impl<T> Clone for ContextPointer<'_, T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone(),
            context: self.context.clone(),
        }
    }
}

impl<'a> Display for ContextStr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cow)
    }
}

impl<'a> Debug for ContextStr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cow)
    }
}

pub(crate) enum MaybeOwnedCString<'a, T = SpirvCrossContext> {
    Owned(CString),
    Borrowed(ContextPointer<'a, T>),
}

impl<T> MaybeOwnedCString<'_, T> {
    /// Get a pointer to the C string.
    ///
    /// The pointer will be valid for the lifetime of `self`.
    pub fn as_ptr(&self) -> *const c_char {
        match self {
            MaybeOwnedCString::Owned(c) => c.as_ptr(),
            MaybeOwnedCString::Borrowed(ptr) => ptr.pointer,
        }
    }
}

impl<T> AsRef<str> for ContextStr<'_, T> {
    fn as_ref(&self) -> &str {
        self.cow.as_ref()
    }
}

impl<T> Deref for ContextStr<'_, T> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.cow.deref()
    }
}

impl<T> From<String> for ContextStr<'_, T> {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl<'a, T> From<&'a str> for ContextStr<'a, T> {
    fn from(value: &'a str) -> Self {
        Self::from_str(value)
    }
}

impl<'a, T> ContextStr<'a, T> {
    /// Wraps a raw C string with a safe C string wrapper.
    ///
    /// If the raw C string is valid UTF-8, a pointer to the string will be
    /// kept around, which can be passed back to C at zero cost.
    ///
    /// # Safety
    ///
    /// * The memory pointed to by `ptr` must contain a valid nul terminator at the
    ///   end of the string.
    ///
    /// * `ptr` must be valid for reads of bytes up to and including the nul terminator.
    ///   This means in particular:
    ///
    ///     * The entire memory range of this `CStr` must be contained within a single allocated object!
    ///     * `ptr` must be non-null even for a zero-length cstr.
    ///
    /// * The memory referenced by the returned `CStr` must not be mutated for
    ///   the duration of lifetime `'a`.
    ///
    /// * The nul terminator must be within `isize::MAX` from `ptr`
    ///
    /// * The memory pointed to by `ptr` must be valid for the duration of the lifetime `'a`.
    ///
    ///  * THe provenance of `context.ptr` must be a superset of `ptr`.
    /// # Caveat
    ///
    /// The lifetime for the returned slice is inferred from its usage. To prevent accidental misuse,
    /// it's suggested to tie the lifetime to whichever source lifetime is safe in the context,
    /// such as by providing a helper function taking the lifetime of a host value for the slice,
    /// or by explicit annotation.
    pub(crate) unsafe fn from_ptr<'b>(
        ptr: *const c_char,
        context: ContextRoot<'a, T>,
    ) -> ContextStr<'b, T>
    where
        'a: 'b,
    {
        let cstr = CStr::from_ptr(ptr);
        let maybe = cstr.to_string_lossy();
        if matches!(&maybe, &Cow::Borrowed(_)) {
            Self {
                pointer: Some(ContextPointer {
                    pointer: ptr,
                    context,
                }),
                cow: maybe,
            }
        } else {
            Self {
                pointer: None,
                cow: maybe,
            }
        }
    }

    /// Wrap a Rust `&str`.
    ///
    /// This will allocate when exposing to C.
    pub(crate) fn from_str(str: &'a str) -> Self {
        Self {
            pointer: None,
            cow: Cow::Borrowed(str),
        }
    }

    /// Wrap a Rust `String`.
    ///
    /// This will allocate when exposing to C.
    pub(crate) fn from_string(str: String) -> Self {
        Self {
            pointer: None,
            cow: Cow::Owned(str),
        }
    }

    /// Allocate if necessary, if not then return a pointer to the original cstring.
    ///
    /// The returned pointer will be valid for the lifetime `'a`.
    pub(crate) fn to_cstring_ptr(&self) -> Result<MaybeOwnedCString<'a, T>, NulError> {
        if let Some(ptr) = &self.pointer {
            Ok(MaybeOwnedCString::Borrowed(ptr.clone()))
        } else {
            let cstring = CString::new(self.cow.to_string())?;
            Ok(MaybeOwnedCString::Owned(cstring))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::string::ContextStr;
    use crate::ContextRoot;
    use std::ffi::CString;
    use std::marker::PhantomData;
    use std::rc::Rc;

    struct LifetimeTest<'a>(PhantomData<&'a ()>);
    impl<'a> LifetimeTest<'a> {
        pub fn get(self: &Rc<Self>) -> ContextStr<'a, LifetimeTest> {
            let cstring = CString::new(String::from("hello"))
                .unwrap()
                .into_raw()
                .cast_const();

            unsafe { ContextStr::from_ptr(cstring, ContextRoot::RefCounted(Rc::clone(&self))) }
        }

        pub fn set(&mut self, cstr: ContextStr<'a, LifetimeTest>) {
            println!("{:p}", cstr.to_cstring_ptr().unwrap().as_ptr())
        }
    }

    #[test]
    fn test_string() {
        // use std::borrow::BorrowMut;
        // let mut lt = Rc::new(LifetimeTest(PhantomData));
        // let cstr = lt.get();
        // lt.borrow_mut().set(cstr)
    }
}
