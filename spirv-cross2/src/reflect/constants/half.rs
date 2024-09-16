#![cfg(feature = "f16")]
#![cfg_attr(docsrs, doc(cfg(feature = "f16")))]

use crate::reflect::ConstantScalar;
use crate::sealed::Sealed;
use spirv_cross_sys as sys;
use spirv_cross_sys::spvc_constant;

impl Sealed for half::f16 {}
impl ConstantScalar for half::f16 {
    unsafe fn get(constant: spvc_constant, column: u32, row: u32) -> Self {
        let f32 = unsafe { sys::spvc_constant_get_scalar_fp16(constant, column, row) };
        half::f16::from_f32(f32)
    }

    unsafe fn set(constant: spvc_constant, column: u32, row: u32, value: Self) {
        unsafe { sys::spvc_constant_set_scalar_fp16(constant, column, row, value.to_bits()) }
    }
}
