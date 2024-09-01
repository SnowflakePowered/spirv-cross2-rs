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
pub use bytemuck::{Pod, Zeroable};
pub use num_traits::{FromPrimitive, ToPrimitive};

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
    TypeId VariableId ConstantId
}
