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

impl Default for MslVertexAttribute {
    fn default() -> Self {
        // This should be non_exhaustive.
        // waiting for https://github.com/rust-lang/rust-bindgen/pull/2866
        let mut attr = MslVertexAttribute {
            location: 0,
            msl_buffer: 0,
            msl_offset: 0,
            msl_stride: 0,
            per_instance: false,
            format: MslShaderVariableFormat::Other,
            builtin: SpvBuiltIn::Position,
        };

        unsafe {
            bindings::spvc_msl_vertex_attribute_init(&mut attr);
        }

        attr
    }
}

impl Default for MslShaderInterfaceVar {
    fn default() -> Self {
        // This should be non_exhaustive.
        // waiting for https://github.com/rust-lang/rust-bindgen/pull/2866
        let mut attr = MslShaderInterfaceVar {
            location: 0,
            format: MslShaderVariableFormat::Other,
            builtin: SpvBuiltIn::Position,
            vecsize: 0,
        };

        unsafe {
            bindings::spvc_msl_shader_interface_var_init(&mut attr);
        }

        attr
    }
}

impl Default for MslShaderInterfaceVar2 {
    fn default() -> Self {
        // This should be non_exhaustive.
        // waiting for https://github.com/rust-lang/rust-bindgen/pull/2866
        let mut attr = MslShaderInterfaceVar2 {
            location: 0,
            format: MslShaderVariableFormat::Other,
            builtin: SpvBuiltIn::Position,
            vecsize: 0,
            rate: MslShaderVariableRate::PerVertex,
        };

        unsafe {
            bindings::spvc_msl_shader_interface_var_init_2(&mut attr);
        }

        attr
    }
}

impl Default for MslResourceBinding {
    fn default() -> Self {
        let mut binding = MslResourceBinding {
            stage: SpvExecutionModel::Vertex,
            desc_set: 0,
            binding: 0,
            msl_buffer: 0,
            msl_texture: 0,
            msl_sampler: 0,
        };

        unsafe {
            bindings::spvc_msl_resource_binding_init(&mut binding);

            binding
        }
    }
}

impl Default for MslResourceBinding2 {
    fn default() -> Self {
        let mut binding = MslResourceBinding2 {
            stage: SpvExecutionModel::Vertex,
            desc_set: 0,
            binding: 0,
            count: 0,
            msl_buffer: 0,
            msl_texture: 0,
            msl_sampler: 0,
        };

        unsafe {
            bindings::spvc_msl_resource_binding_init_2(&mut binding);
        }
        binding
    }
}

impl Default for MslConstexprSampler {
    fn default() -> Self {
        let mut sampler = MslConstexprSampler {
            coord: MslSamplerCoord::Normalized,
            min_filter: MslSamplerFilter::Nearest,
            mag_filter: MslSamplerFilter::Nearest,
            mip_filter: MslSamplerMipFilter::None,
            s_address: MslSamplerAddress::ClampToZero,
            t_address: MslSamplerAddress::ClampToZero,
            r_address: MslSamplerAddress::ClampToZero,
            compare_func: MslSamplerCompareFunc::Never,
            border_color: MslSamplerBorderColor::TransparentBlack,
            lod_clamp_min: 0.0,
            lod_clamp_max: 0.0,
            max_anisotropy: 0,
            compare_enable: false,
            lod_clamp_enable: false,
            anisotropy_enable: false,
        };

        unsafe { bindings::spvc_msl_constexpr_sampler_init(&mut sampler) }

        sampler
    }
}

impl Default for MslSamplerYcbcrConversion {
    fn default() -> Self {
        let mut conversion = MslSamplerYcbcrConversion {
            planes: 0,
            resolution: MslFormatResolution::FormatResolution444,
            chroma_filter: MslSamplerFilter::Nearest,
            x_chroma_offset: MslChromaLocation::CositedEven,
            y_chroma_offset: MslChromaLocation::CositedEven,
            swizzle: [
                MslComponentSwizzle::Zero,
                MslComponentSwizzle::Zero,
                MslComponentSwizzle::Zero,
                MslComponentSwizzle::Zero,
            ],
            ycbcr_model: MslSamplerYcbcrModelConversion::RgbIdentity,
            ycbcr_range: MslSamplerYcbcrRange::ItuFull,
            bpc: 0,
        };

        unsafe { bindings::spvc_msl_sampler_ycbcr_conversion_init(&mut conversion) }

        conversion
    }
}

impl Default for HlslResourceBinding {
    fn default() -> Self {
        // This should be non_exhaustive.
        // waiting for https://github.com/rust-lang/rust-bindgen/pull/2866
        let mut binding = HlslResourceBinding {
            stage: SpvExecutionModel::Vertex,
            desc_set: 0,
            binding: 0,
            cbv: HlslResourceBindingMapping {
                register_space: 0,
                register_binding: 0,
            },
            uav: HlslResourceBindingMapping {
                register_space: 0,
                register_binding: 0,
            },
            srv: HlslResourceBindingMapping {
                register_space: 0,
                register_binding: 0,
            },
            sampler: HlslResourceBindingMapping {
                register_space: 0,
                register_binding: 0,
            },
        };

        unsafe {
            bindings::spvc_hlsl_resource_binding_init(&mut binding);
        }

        binding
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
