use std::borrow::Cow;
use std::ffi::{c_char, CStr, CString, NulError};
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
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
/// The only exception is if the name is explicitly allocated into the context,
/// and can not be modified by a `set_`. function.
///
/// In most cases, the returned lifetime should be the lifetime of the mutable borrow.
#[derive(Clone)]
pub struct ContextStr<'a> {
    pointer: Option<*const c_char>,
    cow: Cow<'a, str>,
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

pub(crate) enum MaybeOwnedCString<'a> {
    Owned(CString),
    Borrowed {
        ptr: *const c_char,
        // invariant to be safe.
        _pd: PhantomData<fn(&'a ()) -> &'a ()>,
    },
}

impl MaybeOwnedCString<'_> {
    /// Get a pointer to the C string.
    ///
    /// The pointer will be valid for the lifetime of `self`.
    pub fn as_ptr(&self) -> *const c_char {
        match self {
            MaybeOwnedCString::Owned(c) => c.as_ptr(),
            MaybeOwnedCString::Borrowed { ptr, .. } => *ptr,
        }
    }
}

impl AsRef<str> for ContextStr<'_> {
    fn as_ref(&self) -> &str {
        self.cow.as_ref()
    }
}

impl Deref for ContextStr<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.cow.deref()
    }
}

impl From<String> for ContextStr<'_> {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl<'a> From<&'a str> for ContextStr<'a> {
    fn from(value: &'a str) -> Self {
        Self::from_str(value)
    }
}

impl<'a> From<&'a CStr> for ContextStr<'a> {
    fn from(value: &'a CStr) -> Self {
        // This is OK as long as the lifetime of the cstr is alive for the
        // lifetime of the ContextStr
        unsafe { Self::from_cstr(value) }
    }
}

impl<'a> ContextStr<'a> {
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
    /// # Caveat
    ///
    /// The lifetime for the returned slice is inferred from its usage. To prevent accidental misuse,
    /// it's suggested to tie the lifetime to whichever source lifetime is safe in the context,
    /// such as by providing a helper function taking the lifetime of a host value for the slice,
    /// or by explicit annotation.
    pub(crate) unsafe fn from_ptr<'b>(ptr: *const c_char) -> ContextStr<'b>
    where
        'a: 'b,
    {
        let cstr = CStr::from_ptr(ptr);
        let maybe = cstr.to_string_lossy();
        if matches!(&maybe, &Cow::Borrowed(_)) {
            Self {
                pointer: Some(ptr),
                cow: maybe,
            }
        } else {
            Self {
                pointer: None,
                cow: maybe,
            }
        }
    }

    /// Wraps a `&CStr`.
    ///
    /// # Caveat
    ///
    /// The lifetime for the returned slice is inferred from its usage. To prevent accidental misuse,
    /// it's suggested to tie the lifetime to whichever source lifetime is safe in the context,
    /// such as by providing a helper function taking the lifetime of a host value for the slice,
    /// or by explicit annotation.
    pub(crate) unsafe fn from_cstr(cstr: &'a CStr) -> Self {
        Self {
            pointer: Some(cstr.as_ptr()),
            cow: cstr.to_string_lossy(),
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
    pub(crate) fn to_cstring_ptr(&self) -> Result<MaybeOwnedCString<'a>, NulError> {
        if let Some(ptr) = self.pointer {
            Ok(MaybeOwnedCString::Borrowed {
                ptr,
                _pd: PhantomData,
            })
        } else {
            let cstring = CString::new(self.cow.to_string())?;
            Ok(MaybeOwnedCString::Owned(cstring))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::string::ContextStr;
    use std::ffi::CString;
    use std::marker::PhantomData;

    struct LifetimeTest<'a>(PhantomData<&'a ()>);
    impl<'a> LifetimeTest<'a> {
        pub fn get(&self) -> ContextStr<'a> {
            let cstring = CString::new(String::from("hello"))
                .unwrap()
                .into_raw()
                .cast_const();

            unsafe { ContextStr::from_ptr(cstring) }
        }

        pub fn set(&mut self, cstr: ContextStr) {
            println!("{:p}", cstr.to_cstring_ptr().unwrap().as_ptr())
        }
    }

    #[test]
    fn test_string() {
        let mut lt = LifetimeTest(PhantomData);
        let cstr = lt.get();
        lt.set(cstr)
    }
}
