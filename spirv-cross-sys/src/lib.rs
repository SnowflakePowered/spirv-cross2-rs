#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Raw bindings to the C API of SPIRV-Cross.
//!
//! Types in `PascalCase` can be safely exposed.
//! Types and functions in `snake_case` are all unsafe.
//!
mod bindings;

type spvc_bool = bool;

pub use bindings::*;
use bytemuck::{Pod, Zeroable};
use std::ffi::CStr;
use std::ptr::NonNull;

#[derive(Debug, thiserror::Error)]
pub enum SpirvCrossError {
    #[error("The SPIR-V is invalid: {0}.")]
    InvalidSpirv(String),
    #[error("The SPIR-V operation is unsupported: {0}.")]
    UnsupportedSpirv(String),
    #[error("Allocation failure: {0}.")]
    OutOfMemory(String),
    #[error("The argument is invalid: {0}.")]
    InvalidArgument(String),
}

pub trait ContextRooted {
    fn context(&self) -> NonNull<spvc_context_s>;
}

impl spvc_result {
    fn get_last_error(context: NonNull<spvc_context_s>) -> String {
        let cstr = unsafe { CStr::from_ptr(spvc_context_get_last_error_string(context.as_ptr())) };

        cstr.to_string_lossy().to_string()
    }

    pub fn ok(self, context: impl ContextRooted) -> Result<(), SpirvCrossError> {
        match self {
            spvc_result::SPVC_SUCCESS => Ok(()),
            spvc_result::SPVC_ERROR_INVALID_SPIRV => Err(SpirvCrossError::InvalidSpirv(
                Self::get_last_error(context.context()),
            )),
            spvc_result::SPVC_ERROR_UNSUPPORTED_SPIRV => Err(SpirvCrossError::UnsupportedSpirv(
                Self::get_last_error(context.context()),
            )),
            spvc_result::SPVC_ERROR_OUT_OF_MEMORY => Err(SpirvCrossError::OutOfMemory(
                Self::get_last_error(context.context()),
            )),
            spvc_result::SPVC_ERROR_INVALID_ARGUMENT => Err(SpirvCrossError::InvalidArgument(
                Self::get_last_error(context.context()),
            )),
        }
    }
}

unsafe impl Zeroable for SpvId {}
unsafe impl Pod for SpvId {}

impl From<u32> for SpvId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

macro_rules! from_u32 {
    ($($id:ty)*) => {
        $(impl From<u32> for $id {
            fn from(value: u32) -> Self {
                Self(From::from(value))
            }
         })*
    };
}

from_u32! {
    VariableId ConstantId
}
