#![cfg_attr(docsrs, feature(doc_cfg, doc_cfg_hide))]
#![forbid(missing_docs)]

//! Safe and sound Rust bindings to [SPIRV-Cross](https://github.com/KhronosGroup/SPIRV-Cross).
//!
//! All backends exposed by the SPIRV-Cross C API are fully supported, including
//!
//! * [GLSL](targets::Glsl)
//! * [HLSL](targets::Hlsl)
//! * [MSL](targets::Msl)
//! * [JSON](targets::Json)
//! * [C++](targets::Cpp)
//! * [Reflection Only](targets::None)
//!
//! The API provided is roughly similar to the SPIRV-Cross [`Compiler`](https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross.hpp) C++ API,
//! with some inspiration from [naga](https://docs.rs/naga/latest/naga/index.html). A best effort has been
//! made to ensure that these bindings are sound, and that mutations occur strictly within Rust's
//! borrow rules.
//!
//! ## Context
//! The entry point to the library is [`SpirvCrossContext`], which owns all foreign allocations.
//! Hence, all wrapper structs have a lifetime parameter that refers to the lifetime of the context.
//!
//! [`Compiler`] instances can share a context, in which case the context must outlive all associated
//! objects, or it can take ownership of a context and have a `'static` lifetime, where all associated
//! objects will be dropped only when the compiler instance is dropped.
//!
//! ## Strings
//! Methods on [`Compiler`] return and accept [`ContextStr`] instead of a normal string type. A
//! [`ContextStr`] may or may not be owned by the context, or may come from Rust. Rust string types
//! can be coerced automatically to [`ContextStr`] as an input, and [`ContextStr`] can easily be copied
//! to a Rust string type.
//!
//! If a returned [`ContextStr`] is owned by the context and is immutable,
//! it will share the lifetime of the context. Some functions return _short lived_ strings which
//! are owned by the underlying compiler instance. These strings can be modified by `set_` functions,
//! thus they only have a lifetime corresponding to the lifetime of the immutable borrow of the [`Compiler`]
//! that produced them. References to these short-lived strings can not be alive before calling a
//! mutating function.
//!
//! ## Usage
//! Here is an example of using the API to do some reflection and compile to GLSL.
//!
//! Note the `'static` lifetime of the artifact, as the context is owned by the compiler.
//!
//! ```
//! use spirv_cross2::compile::{CompilableTarget, CompiledArtifact};
//! use spirv_cross2::{Module, SpirvCrossContext, SpirvCrossError};
//! use spirv_cross2::compile::glsl::GlslVersion;
//! use spirv_cross2::reflect::{DecorationValue, ResourceType};
//! use spirv_cross2::spirv;
//! use spirv_cross2::targets::Glsl;
//!
//! fn compile_spirv(words: &[u32]) -> Result<CompiledArtifact<'static, Glsl>, SpirvCrossError> {
//!     let module = Module::from_words(words);
//!     let context = SpirvCrossContext::new()?;
//!
//!     let mut compiler = context.into_compiler::<Glsl>(module)?;
//!
//!     let resources = compiler.shader_resources()?;
//!
//!     for resource in resources.resources_for_type(ResourceType::SampledImage)? {
//!         let Some(DecorationValue::Literal(set)) =
//!                 compiler.decoration(resource.id,  spirv::Decoration::DescriptorSet)? else {
//!             continue;
//!         };
//!         let Some(DecorationValue::Literal(binding)) =
//!             compiler.decoration(resource.id,  spirv::Decoration::Binding)? else {
//!             continue;
//!         };
//!
//!         println!("Image {} at set = {}, binding = {}", resource.name, set, binding);
//!
//!         // Modify the decoration to prepare it for GLSL.
//!         compiler.set_decoration(resource.id, spirv::Decoration::DescriptorSet,
//!                 DecorationValue::unset())?;
//!
//!         // Some arbitrary remapping if we want.
//!         compiler.set_decoration(resource.id, spirv::Decoration::Binding,
//!             Some(set * 16 + binding))?;
//!     }
//!
//!     let mut options = Glsl::options();
//!     options.version = GlslVersion::Glsl300Es;
//!
//!     compiler.compile(&options)
//! }
//! ```
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_compiler_s, spvc_context_s, SpvId};
use std::borrow::Borrow;

use crate::error::{ToContextError};

use crate::sealed::{ContextRooted, Sealed};
use crate::targets::Target;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::rc::Rc;

/// Compilation of SPIR-V to a textual format.
pub mod compile;

/// SPIR-V types and definitions.
pub mod spirv;

/// Handles to SPIR-V IDs from reflection.
pub mod handle;

/// SPIR-V reflection helpers and types.
pub mod reflect;

/// Compiler output targets.
pub mod targets;

/// Error handling traits and support.
pub(crate) mod error;

/// String helpers
pub(crate) mod string;

pub(crate) mod sealed {
    use std::ptr::NonNull;
    use spirv_cross_sys::spvc_context_s;

    pub trait Sealed {}

    pub trait ContextRooted {
        fn context(&self) -> NonNull<spvc_context_s>;
    }
}

pub use crate::error::SpirvCrossError;
pub use crate::string::ContextStr;

/// The SPIRV-Cross context. All memory allocations originating from
/// this context will have the same lifetime as the context.
#[repr(transparent)]
pub struct SpirvCrossContext(NonNull<spvc_context_s>);

enum ContextRoot<'a> {
    Owned(SpirvCrossContext),
    Borrowed(&'a SpirvCrossContext),
    RefCounted(Rc<SpirvCrossContext>),
}

impl<'a> Borrow<SpirvCrossContext> for ContextRoot<'a> {
    fn borrow(&self) -> &SpirvCrossContext {
        match self {
            ContextRoot::Owned(a) => a,
            ContextRoot::Borrowed(a) => a,
            ContextRoot::RefCounted(a) => a.deref(),
        }
    }
}

impl<'a> AsRef<SpirvCrossContext> for ContextRoot<'a> {
    fn as_ref(&self) -> &SpirvCrossContext {
        match self {
            ContextRoot::Owned(a) => a,
            ContextRoot::Borrowed(a) => a,
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

/// A SPIR-V Module represented as SPIR-V words.
pub struct Module<'a>(&'a [SpvId]);

impl<'a> Module<'a> {
    /// Create a new `Module` from SPIR-V words.
    pub fn from_words(words: &'a [u32]) -> Self {
        Module(bytemuck::must_cast_slice(words))
    }
}

impl SpirvCrossContext {
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
                spirv_cross_sys::spvc_capture_mode::TakeOwnership,
                &mut compiler,
            )
            .ok(self)?;

            let Some(compiler) = NonNull::new(compiler) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(Compiler::new_from_raw(
                compiler,
                ContextRoot::Borrowed(self),
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
                spirv_cross_sys::spvc_capture_mode::TakeOwnership,
                &mut compiler,
            )
            .ok(&**self)?;

            let Some(compiler) = NonNull::new(compiler) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(Compiler::new_from_raw(
                compiler,
                ContextRoot::RefCounted(Rc::clone(self)),
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
                spirv_cross_sys::spvc_capture_mode::TakeOwnership,
                &mut compiler,
            )
            .ok(&self)?;

            let Some(compiler) = NonNull::new(compiler) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(Compiler::new_from_raw(compiler, ContextRoot::Owned(self)))
        }
    }
}

impl Drop for SpirvCrossContext {
    fn drop(&mut self) {
        unsafe { sys::spvc_context_destroy(self.0.as_ptr()) }
    }
}

impl ContextRooted for &SpirvCrossContext {
    fn context(&self) -> NonNull<spvc_context_s> {
        self.0
    }
}

/// Helper trait to detach objects with lifetimes attached to
/// a compiler or context.
pub trait ToStatic: Sealed {
    /// The static type to return.
    type Static<'a>
    where
        'a: 'static;

    /// Clone the object into an instance with `'static` lifetime.
    fn to_static(&self) -> Self::Static<'static>;
}

#[cfg(test)]
mod test {
    use crate::SpirvCrossContext;

    #[test]
    pub fn init_context_test() {
        SpirvCrossContext::new().unwrap();
    }
}

/// An instance of a SPIRV-Cross compiler.
///
/// Depending on the target, different methods will be
/// available.
///
/// Once compiled into a [`CompiledArtifact`](compile::CompiledArtifact),
/// reflection methods will still remain available, but the instance will be frozen,
/// and no more mutation will be available.
pub struct Compiler<'a, T> {
    pub(crate) ptr: NonNull<spvc_compiler_s>,
    ctx: ContextRoot<'a>,
    _pd: PhantomData<T>,
}

impl<T> Compiler<'_, T> {
    /// Create a new compiler instance.
    ///
    /// The pointer to the `spvc_compiler_s` must have the same lifetime as the context root.
    pub(crate) unsafe fn new_from_raw(
        ptr: NonNull<spvc_compiler_s>,
        ctx: ContextRoot,
    ) -> Compiler<T> {
        Compiler {
            ptr,
            ctx,
            _pd: PhantomData,
        }
    }
}

impl<T> ContextRooted for &Compiler<'_, T> {
    #[inline(always)]
    fn context(&self) -> NonNull<spvc_context_s> {
        self.ctx.ptr()
    }
}

impl<T> ContextRooted for &mut Compiler<'_, T> {
    #[inline(always)]
    fn context(&self) -> NonNull<spvc_context_s> {
        self.ctx.ptr()
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
/// refer to the root context. It's lifetime should be the same as the lifetime
/// of the compiler.
#[derive(Copy, Clone)]
pub(crate) struct PhantomCompiler<'a> {
    pub(crate) ptr: NonNull<spvc_compiler_s>,
    ctx: NonNull<spvc_context_s>,
    _pd: PhantomData<&'a ()>,
}

impl ContextRooted for PhantomCompiler<'_> {
    #[inline(always)]
    fn context(&self) -> NonNull<spvc_context_s> {
        self.ctx
    }
}

impl<'a, T> Compiler<'a, T> {
    /// Create a type erased phantom for lifetime tracking purposes.
    ///
    /// This function is unsafe because a [`PhantomCompiler`] can be used to
    /// **safely** create handles originating from the compiler.
    pub(crate) unsafe fn phantom(&self) -> PhantomCompiler<'a> {
        PhantomCompiler {
            ptr: self.ptr,
            ctx: self.context(),
            _pd: PhantomData,
        }
    }
}
