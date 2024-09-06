use crate::sync::ContextRoot;
use crate::{SpirvCrossContext, SpirvCrossError};
use std::borrow::Cow;
use std::ffi::{c_char, CStr, CString};
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
/// If the provenance of the string is a `&str` with lifetime longer than `'a`,
/// then an allocation will occur when passing the string to FFI.
///
/// If the provenance of the string is an owned Rust `String`, then an allocation
/// will occur only if necessary to append a nul byte.
///
/// If the provenance of the string is a `&CStr`, or
/// with lifetime longer than `'a`, then an allocation will not occur
/// when passing the string to FFI.
///
/// Using [C-string literals](https://doc.rust-lang.org/edition-guide/rust-2021/c-string-literals.html)
/// where possible can be used to avoid an allocation.
pub struct ContextStr<'a, T = SpirvCrossContext> {
    pointer: Option<ContextPointer<'a, T>>,
    cow: Cow<'a, str>,
}

// SAFETY: SpirvCrossContext is Send.
// Once created, the ContextStr is immutable, so it is also sync.
// cloning the string doesn't affect the memory, as long as it
// is alive for 'a.
//
// There is no interior mutability of a
unsafe impl<T: Send> Send for ContextStr<'_, T> {}
unsafe impl<T: Send> Sync for ContextStr<'_, T> {}

impl<T> Clone for ContextStr<'_, T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone(),
            cow: self.cow.clone(),
        }
    }
}

pub(crate) enum ContextPointer<'a, T> {
    FromContext {
        // the lifetime of pointer should be 'a.
        pointer: *const c_char,
        context: ContextRoot<'a, T>,
    },
    BorrowedCStr(&'a CStr),
}

impl<T> ContextPointer<'_, T> {
    pub const fn pointer(&self) -> *const c_char {
        match self {
            ContextPointer::FromContext { pointer, .. } => *pointer,
            ContextPointer::BorrowedCStr(cstr) => cstr.as_ptr(),
        }
    }
}

impl<T> Clone for ContextPointer<'_, T> {
    fn clone(&self) -> Self {
        match self {
            ContextPointer::FromContext { pointer, context } => ContextPointer::FromContext {
                pointer: pointer.clone(),
                context: context.clone(),
            },
            ContextPointer::BorrowedCStr(cstr) => ContextPointer::BorrowedCStr(*cstr),
        }
    }
}

impl<'a, T> Display for ContextStr<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cow)
    }
}

impl<'a, T> Debug for ContextStr<'a, T> {
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
            MaybeOwnedCString::Borrowed(ptr) => ptr.pointer(),
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

impl<T> PartialEq for ContextStr<'_, T> {
    fn eq(&self, other: &ContextStr<'_, T>) -> bool {
        self.cow.eq(&other.cow)
    }
}

impl<'a, T> PartialEq<&'a str> for ContextStr<'_, T> {
    fn eq(&self, other: &&'a str) -> bool {
        self.cow.eq(other)
    }
}

impl<'a, T> PartialEq<ContextStr<'_, T>> for &'a str {
    fn eq(&self, other: &ContextStr<'_, T>) -> bool {
        self.eq(&other.cow)
    }
}

impl<T> PartialEq<str> for ContextStr<'_, T> {
    fn eq(&self, other: &str) -> bool {
        self.cow.eq(other)
    }
}

impl<T> PartialEq<ContextStr<'_, T>> for str {
    fn eq(&self, other: &ContextStr<'_, T>) -> bool {
        self.eq(&other.cow)
    }
}

impl<T> PartialEq<ContextStr<'_, T>> for String {
    fn eq(&self, other: &ContextStr<'_, T>) -> bool {
        self.eq(&other.cow)
    }
}
impl<T> PartialEq<String> for ContextStr<'_, T> {
    fn eq(&self, other: &String) -> bool {
        self.cow.eq(other)
    }
}

impl<T> Eq for ContextStr<'_, T> {}

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

impl<'a, T> From<&'a CStr> for ContextStr<'a, T> {
    fn from(value: &'a CStr) -> Self {
        Self::from_cstr(value)
    }
}

/// # Safety
/// Returning `ContextStr<'a>` where `'a` is the lifetime of the
/// [`SpirvCrossContext`](crate::SpirvCrossContext) is only correct if the
/// string is borrow-owned by the context.
///
/// In most cases, the returned lifetime should be the lifetime of the mutable borrow,
/// if returning a string from the [`Compiler`](crate::Compiler).
///
/// [`ContextStr::from_ptr`] takes a context argument, and the context must be
/// the source of provenance for the `ContextStr`.
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
                pointer: Some(ContextPointer::FromContext {
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

    pub(crate) fn from_cstr(cstr: &'a CStr) -> Self {
        Self {
            pointer: Some(ContextPointer::BorrowedCStr(cstr)),
            cow: cstr.to_string_lossy(),
        }
    }

    /// Allocate if necessary, if not then return a pointer to the original cstring.
    ///
    /// The returned pointer will be valid for the lifetime `'a`.
    pub(crate) fn into_cstring_ptr(self) -> Result<MaybeOwnedCString<'a, T>, SpirvCrossError> {
        if let Some(ptr) = &self.pointer {
            // this is either free or very cheap (Rc incr at most)
            Ok(MaybeOwnedCString::Borrowed(ptr.clone()))
        } else {
            let cstring = match self.cow {
                Cow::Borrowed(s) => CString::new(s.to_string()),
                Cow::Owned(s) => CString::new(s),
            };

            match cstring {
                Ok(cstring) => Ok(MaybeOwnedCString::Owned(cstring)),
                Err(e) => {
                    let string = e.into_vec();
                    // SAFETY: This string *came* from UTF-8 as its source was the Cow,
                    // which was preverified UTF-8.
                    let string = unsafe { String::from_utf8_unchecked(string) };
                    Err(SpirvCrossError::InvalidString(string))
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::string::ContextStr;
    use crate::sync::ContextRoot;
    use std::ffi::{c_char, CStr, CString};
    use std::sync::Arc;

    struct LifetimeContext(*mut c_char);
    impl LifetimeContext {
        pub fn new() -> Self {
            let cstring = CString::new(String::from("hello")).unwrap().into_raw();

            Self(cstring)
        }
    }

    impl Drop for LifetimeContext {
        fn drop(&mut self) {
            unsafe {
                drop(CString::from_raw(self.0));
            }
        }
    }

    struct LifetimeTest<'a>(ContextRoot<'a, LifetimeContext>);
    impl<'a> LifetimeTest<'a> {
        pub fn get(&self) -> ContextStr<'a, LifetimeContext> {
            unsafe { ContextStr::from_ptr(self.0.as_ref().0, self.0.clone()) }
        }

        pub fn set(&mut self, cstr: ContextStr<'a, LifetimeContext>) {
            println!("{:p}", cstr.into_cstring_ptr().unwrap().as_ptr())
        }
    }

    #[test]
    fn test_string() {
        // use std::borrow::BorrowMut;
        let lc = LifetimeContext::new();
        let ctx = ContextRoot::RefCounted(Arc::new(lc));

        let mut lt = LifetimeTest(ctx);

        // let mut lt = Rc::new(LifetimeTest(PhantomData));
        let cstr = lt.get();
        lt.set(cstr.clone());

        let original_ptr = cstr.clone().into_cstring_ptr().unwrap().as_ptr();
        drop(lt);

        assert_eq!("hello", cstr.as_ref());
        println!("{}", cstr);

        assert_eq!(original_ptr as usize, cstr.as_ptr() as usize);
        // lt.borrow_mut().set(cstr)
    }

    #[test]
    fn test_string_does_not_allocate() {
        // one past the end
        let mut test = String::with_capacity(6);
        test.push_str("Hello");

        let original_ptr = test.as_ptr() as usize;
        let ctxstr = ContextStr::<LifetimeContext>::from(test);

        let new_ptr = ctxstr.into_cstring_ptr().unwrap().as_ptr();
        assert_eq!(original_ptr, new_ptr as usize);
        // lt.borrow_mut().set(cstr)
    }

    #[test]
    fn test_str_does_allocate() {
        let str = "Hello";
        let original_ptr = str.as_ptr() as usize;
        let ctxstr = ContextStr::<LifetimeContext>::from(str);

        let new_ptr = ctxstr.into_cstring_ptr().unwrap().as_ptr();
        assert_ne!(original_ptr, new_ptr as usize);
        // lt.borrow_mut().set(cstr)
    }

    #[test]
    fn test_cstr_does_not_allocate() {
        // can't use cstring literals until 1.77
        let str = unsafe { CStr::from_ptr(b"Hello\0".as_ptr().cast()) };

        let original_ptr = str.as_ptr() as usize;
        let ctxstr = ContextStr::<LifetimeContext>::from(str);

        let new_ptr = ctxstr.into_cstring_ptr().unwrap().as_ptr();
        assert_eq!(original_ptr, new_ptr as usize);
        // lt.borrow_mut().set(cstr)
    }
}
