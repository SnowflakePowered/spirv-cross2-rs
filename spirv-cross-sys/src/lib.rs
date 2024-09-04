#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Raw bindings to the C API of SPIRV-Cross.
//!
//! Incorrect use of `_init` functions can cause undefined behaviour.
//!
//! Always go through `MaybeUninit` for anything that sets an enum,
//! then check for `u32::MAX`.
//!
//! `spvc_rs` functions are unstable and are meant for consumption by [spirv-cross2](https://crates.io/crates/spirv-cross2)
//! only.

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

impl Default for MslConstexprSampler {
    fn default() -> Self {
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_msl.hpp#L216
        MslConstexprSampler {
            coord: MslSamplerCoord::Normalized,
            min_filter: MslSamplerFilter::Nearest,
            mag_filter: MslSamplerFilter::Nearest,
            mip_filter: MslSamplerMipFilter::None,
            s_address: MslSamplerAddress::ClampToEdge,
            t_address: MslSamplerAddress::ClampToEdge,
            r_address: MslSamplerAddress::ClampToEdge,
            compare_func: MslSamplerCompareFunc::Never,
            border_color: MslSamplerBorderColor::TransparentBlack,
            lod_clamp_min: 0.0,
            lod_clamp_max: 1000.0,
            max_anisotropy: 1,
            compare_enable: false,
            lod_clamp_enable: false,
            anisotropy_enable: false,
        }
    }
}

impl Default for MslSamplerYcbcrConversion {
    fn default() -> Self {
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_msl.hpp#L230
        MslSamplerYcbcrConversion {
            planes: 0,
            resolution: MslFormatResolution::FormatResolution444,
            chroma_filter: MslSamplerFilter::Nearest,
            x_chroma_offset: MslChromaLocation::CositedEven,
            y_chroma_offset: MslChromaLocation::CositedEven,
            swizzle: [
                MslComponentSwizzle::Identity,
                MslComponentSwizzle::Identity,
                MslComponentSwizzle::Identity,
                MslComponentSwizzle::Identity,
            ],
            ycbcr_model: MslSamplerYcbcrModelConversion::RgbIdentity,
            ycbcr_range: MslSamplerYcbcrRange::ItuFull,
            bpc: 8,
        }
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
