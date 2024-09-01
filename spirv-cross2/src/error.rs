use crate::compiler::decorations::DecorationValue;
use crate::handle::{Handle, Id};
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_context_s, spvc_result};
use std::ffi::CStr;
use std::ptr::NonNull;

pub type Result<T> = std::result::Result<T, SpirvCrossError>;

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
    InvalidOperation(String),
    #[error("The decoration value is invalid for the given decoration: {0:?} = {1}")]
    InvalidDecorationOutput(crate::spirv::Decoration, u32),
    #[error("The decoration value is invalid for the given decoration: {0:?} = {1:?}")]
    InvalidDecorationInput(crate::spirv::Decoration, DecorationValue<'static>),
    #[error("The name is invalid: {0:?}")]
    InvalidName(String),
    #[error("The provided index was out of bounds for the resource: ({row}, {column}).")]
    /// The index is out of bounds
    IndexOutOfBounds { row: u32, column: u32 },
}

pub(crate) trait ContextRooted {
    fn context(&self) -> NonNull<spvc_context_s>;
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
