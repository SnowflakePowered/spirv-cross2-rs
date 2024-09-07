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
//! ## Strings
//! Methods on [`Compiler`] return and accept [`CompilerStr`] instead of a normal string type. A
//! [`CompilerStr`] may or may not be owned by the compiler, or may come from Rust. Rust string types
//! can be coerced automatically to [`CompilerStr`] as an input, and [`CompilerStr`] can easily be copied
//! to a Rust string type.
//!
//! If a returned [`CompilerStr`] is backed by immutable memory, it will have a `'static` lifetime.
//!
//! If instead the underlying string data could possibly be modified by `set_` functions,
//! they will only have a lifetime corresponding to the lifetime of the immutable borrow of the [`Compiler`]
//! that produced them. References to these short-lived strings can not be alive before calling a
//! mutating function.
//!
//! Strings will automatically allocate as needed when passed to FFI. Rust [`String`] and [`&str`](str)
//! may allocate to create a nul-terminated string. Strings coming from FFI will not reallocate,
//! and the pointer will be passed directly back. Rust [`&CStr`](std::ffi::CStr) will not reallocate.
//!
//! If you are just passing in a string constant using a [C-string literal](https://doc.rust-lang.org/edition-guide/rust-2021/c-string-literals.html)
//! will be the most efficient. Otherwise, it is always better to work with Rust [`String`] and [`&str`](str),
//! if you are dynamically building up a string. In particular, [`String`] will not reallocate if
//! there is enough capacity to append a nul byte before being passed to FFI.
//!
//! ## Handles
//! All reflected SPIR-V IDs are returned as [`Handle<T>`](handle::Handle), where the `u32` ID part can
//! be retrieved with [`Handle::id`](handle::Handle::id). Handles are tagged with the pointer of the
//! compiler instance they came from, and are required to ensure safety such that reflection queries
//! aren't made between different SPIR-V modules.
//!
//! Any function that takes or returns SPIR-V handles in the SPIRV-Cross API has been wrapped to accept
//! [`Handle<T>`](handle::Handle) in this crate.
//!
//! Handles can be unsafely forged with [`Compiler::create_handle`], but there are very few if any
//! situations where this would be needed.
//!
//! ## Features
//! By default, the `glsl`, `hlsl`, and `msl` features are enabled by default. The `cpp` and `json` targets can be enabled
//! in Cargo.toml
//!
//! ```toml
//! [dependencies]
//! spirv-cross2 = { features = ["cpp", "json"] }
//! ```
//!
//! SPIRV-Cross will only be built with support for enabled targets. If you want to only perform reflection and shrink the binary size,
//! you can disable all but the `None` target.
//!
//! ```toml
//! [dependencies]
//! spirv-cross2 = { default-features = false }
//! ```
//!
//! To enable all features, including `f16` and vector constant support, use the `full` feature.
//!
//! ```toml
//! [dependencies]
//! spirv-cross2 = { features = ["full"] }
//! ```
//!
//! ### `f16` and vector specialization constants support
//! When querying specialization constants, spirv-cross2 includes optional support for `f16` via [half](https://crates.io/crates/half) and `Vec2`, `Vec3`, `Vec4`, and `Mat4` types
//! via [gfx-maths](https://crates.io/crates/gfx-maths).
//!
//! ```toml
//! [dependencies]
//! spirv-cross2 = { features = ["f16", "half"] }
//! ```
//!
//! ## Usage
//! Here is an example of using the API to do some reflection and compile to GLSL.
//!
//! ```
//! use spirv_cross2::compile::{CompilableTarget, CompiledArtifact};
//! use spirv_cross2::{Compiler, Module, SpirvCrossError};
//! use spirv_cross2::compile::glsl::GlslVersion;
//! use spirv_cross2::reflect::{DecorationValue, ResourceType};
//! use spirv_cross2::spirv;
//! use spirv_cross2::targets::Glsl;
//!
//! fn compile_spirv(words: &[u32]) -> Result<CompiledArtifact<Glsl>, SpirvCrossError> {
//!     let module = Module::from_words(words);
//!
//!     let mut compiler = Compiler::<Glsl>::new(module)?;
//!
//!     let resources = compiler.shader_resources()?;
//!
//!     for resource in resources.resources_for_type(ResourceType::SampledImage)? {
//!         let Some(DecorationValue::Literal(set)) =
//!                 compiler.decoration(resource.id, spirv::Decoration::DescriptorSet)? else {
//!             continue;
//!         };
//!         let Some(DecorationValue::Literal(binding)) =
//!             compiler.decoration(resource.id, spirv::Decoration::Binding)? else {
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
//!
use spirv_cross_sys::{spvc_compiler_s, SpvId};

use crate::cell::{AllocationDropGuard, CrossAllocationCell};
use crate::sealed::{ContextRooted, Sealed};
use crate::targets::Target;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// Compilation of SPIR-V to a textual format.
pub mod compile;

/// Handles to SPIR-V IDs from reflection.
pub mod handle;

/// SPIR-V reflection helpers and types.
pub mod reflect;

/// Compiler output targets.
pub mod targets;

/// Error handling traits and support.
mod error;

/// Cell helpers
mod cell;

/// String helpers
mod string;

/// Iteratator
mod iter;

/// SPIR-V types and definitions.
pub mod spirv {
    pub use spirv::BuiltIn;
    pub use spirv::Capability;
    pub use spirv::Decoration;
    pub use spirv::Dim;
    pub use spirv::ExecutionMode;
    pub use spirv::ExecutionModel;
    pub use spirv::FPRoundingMode;
    pub use spirv::ImageFormat;
    pub use spirv::StorageClass;
}

pub(crate) mod sealed {
    use spirv_cross_sys::spvc_context_s;
    use std::ptr::NonNull;

    pub trait Sealed {}

    pub trait ContextRooted {
        fn context(&self) -> NonNull<spvc_context_s>;
    }
}

pub use crate::error::SpirvCrossError;
pub use crate::string::CompilerStr;

/// A SPIR-V Module represented as SPIR-V words.
pub struct Module<'a>(&'a [SpvId]);

impl<'a> Module<'a> {
    /// Create a new `Module` from SPIR-V words.
    pub fn from_words(words: &'a [u32]) -> Self {
        Module(bytemuck::must_cast_slice(words))
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

/// An instance of a SPIRV-Cross compiler.
///
/// Depending on the target, different methods will be
/// available.
///
/// Once compiled into a [`CompiledArtifact`](compile::CompiledArtifact),
/// reflection methods will still remain available, but the instance will be frozen,
/// and no more mutation will be available.
pub struct Compiler<T> {
    pub(crate) ptr: NonNull<spvc_compiler_s>,
    ctx: CrossAllocationCell,
    _pd: PhantomData<T>,
}

impl<T: Target> Compiler<T> {
    /// Create a compiler instance from a SPIR-V module.
    pub fn new(spirv: Module) -> error::Result<Compiler<T>> {
        let allocs = CrossAllocationCell::new()?;
        allocs.into_compiler(spirv)
    }

    /// Create a new compiler instance.
    ///
    /// The pointer to the `spvc_compiler_s` must have the same lifetime as the context root.
    pub(crate) unsafe fn new_from_raw(
        ptr: NonNull<spvc_compiler_s>,
        ctx: CrossAllocationCell,
    ) -> Compiler<T> {
        Compiler {
            ptr,
            ctx,
            _pd: PhantomData,
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
/// The only thing a [`PhantomCompiler`] is able to do is create handles or
/// refer to the root context. It's lifetime should be the same as the lifetime
/// of the **context**, or **shorter**, but at least the lifetime of the compiler.
///
/// Anything that holds a PhantomCompiler effectively has static lifetime,
/// if and only if it points to an allocation that originates from the context.
///
/// Because it holds an `AllocationDropGuard`, the compiler instance will always be live.
#[derive(Clone)]
pub(crate) struct PhantomCompiler {
    pub(crate) ptr: NonNull<spvc_compiler_s>,
    ctx: AllocationDropGuard,
}

impl<T> Compiler<T> {
    /// Create a type erased phantom for lifetime tracking purposes.
    ///
    /// This function is unsafe because a [`PhantomCompiler`] can be used to
    /// **safely** create handles originating from the compiler.
    pub(crate) unsafe fn phantom(&self) -> PhantomCompiler {
        PhantomCompiler {
            ptr: self.ptr,
            ctx: self.ctx.drop_guard(),
        }
    }
}

unsafe impl<T: Send> Send for Compiler<T> {}
