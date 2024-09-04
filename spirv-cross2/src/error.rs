use crate::handle::{Handle, Id};
use crate::reflect::DecorationValue;
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_context_s, spvc_result};
use std::ffi::CStr;
use std::ptr::NonNull;

/// Result type for SPIR-V Cross.
pub type Result<T> = std::result::Result<T, SpirvCrossError>;

/// Error type for SPIR-V Cross.
#[derive(Debug, thiserror::Error)]
pub enum SpirvCrossError {
    #[error("The SPIR-V is invalid: {0}.")]
    /// The SPIR-V is invalid.
    InvalidSpirv(String),
    #[error("The SPIR-V operation is unsupported: {0}.")]
    /// The SPIR-V operation is invalid.
    UnsupportedSpirv(String),
    #[error("Allocation failure: {0}.")]
    /// Allocation failure.
    OutOfMemory(String),
    #[error("The argument is invalid: {0}.")]
    /// The argument is invalid.
    InvalidArgument(String),
    #[error("The tag of the handle does not match the compiler instance: {0:?}")]
    /// The handle provided originated from a different compiler instance.
    InvalidHandle(Handle<Box<dyn Id>>),
    #[error("The operation is invalid: {0:?}")]
    /// The requested operation is invalid.
    InvalidOperation(String),
    #[error("The decoration value is invalid for the given decoration: {0:?} = {1}")]
    /// The decoration value invalid for the given decoration.
    ///
    /// This is mostly returned if there is an invalid `OpDecoration Builtin` or `OpDecoration FPRoundingMode`
    /// in the SPIR-V module.
    InvalidDecorationOutput(crate::spirv::Decoration, u32),
    #[error("The decoration value is invalid for the given decoration: {0:?} = {1:?}")]
    /// The decoration value is invalid for the given decoration.
    InvalidDecorationInput(crate::spirv::Decoration, DecorationValue<'static>),
    #[error("The string is invalid: {0:?}")]
    /// The string is invalid.
    ///
    /// Strings must not be nul-terminated, and must be valid UTF-8.
    InvalidString(String),
    #[error("The provided index was out of bounds for the resource: ({row}, {column}).")]
    /// The index is out of bounds when trying to access a constant resource.
    ///
    /// Multiscalar specialization constants are stored in column-major order.
    /// Vectors are always in column 1.
    IndexOutOfBounds {
        /// The vector index or row accessed.
        row: u32,
        /// The column accessed.
        column: u32,
    },
    #[error("An unexpected enum value was found.")]
    /// An unexpected enum value was found.
    InvalidEnum,
}

pub(crate) trait ToContextError {
    fn ok(self, context: impl ContextRooted) -> Result<()>;
}

fn get_last_error(context: NonNull<spvc_context_s>) -> String {
    let cstr = unsafe { CStr::from_ptr(sys::spvc_context_get_last_error_string(context.as_ptr())) };

    cstr.to_string_lossy().to_string()
}

impl ToContextError for spvc_result {
    fn ok(self, context: impl ContextRooted) -> Result<()> {
        match self {
            spvc_result::SPVC_SUCCESS => Ok(()),
            spvc_result::SPVC_ERROR_INVALID_SPIRV => Err(SpirvCrossError::InvalidSpirv(
                get_last_error(context.context()),
            )),
            spvc_result::SPVC_ERROR_UNSUPPORTED_SPIRV => Err(SpirvCrossError::UnsupportedSpirv(
                get_last_error(context.context()),
            )),
            spvc_result::SPVC_ERROR_OUT_OF_MEMORY => Err(SpirvCrossError::OutOfMemory(
                get_last_error(context.context()),
            )),
            spvc_result::SPVC_ERROR_INVALID_ARGUMENT => Err(SpirvCrossError::InvalidArgument(
                get_last_error(context.context()),
            )),
        }
    }
}

pub(crate) use crate::sealed::ContextRooted;
