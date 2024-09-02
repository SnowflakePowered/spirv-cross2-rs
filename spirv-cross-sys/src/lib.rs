#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Raw bindings to the C API of SPIRV-Cross.
//!
//! Types in `PascalCase` can be safely exposed.
//! Types and functions in `snake_case` are all unsafe.
//!
mod bindings;


// Because SPIRV-Cross's C API is C89, we don't have stdint,
// but the C++ API uses sized int types.
//
// Since it always links to sized int types, it is safe to
// define it as fixed sized rather than wobbly. Particularly,
// a SPIR-V word is `uint32_t`, so it is nice to deal with `u32`
// rather than c_uint.
//
// On esoteric systems, we don't expect SPIRV-Cross to be usable
// anyways.
mod ctypes {
    pub type spvc_bool = bool;
    pub type c_char = std::os::raw::c_char;

    pub type c_void = ::std::os::raw::c_void;
    pub type c_uint = u32;
    pub type c_schar = i8;
    pub type c_uchar = u8;
    pub type c_short = i16;
    pub type c_ushort = u16;
    pub type c_int = i32;
    pub type c_longlong = i64;
    pub type c_ulonglong = u64;
}

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
