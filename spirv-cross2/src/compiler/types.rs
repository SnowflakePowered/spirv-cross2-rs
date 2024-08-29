use crate::compiler::Compiler;
use crate::error;
use spirv_cross_sys::{spvc_type, BaseType, SpirvCrossError, SpvId, TypeId};
use std::borrow::Cow;
use std::ffi::CStr;

use spirv_cross_sys as sys;

#[derive(Debug)]
#[repr(u8)]
pub enum ScalarKind {
    Int = 0,
    Uint = 1,
    Float = 2,
    Bool = 3,
}

#[derive(Debug)]
#[repr(u8)]
pub enum BitWidth {
    /// 1 bit
    Bit = 1,
    /// 8 bits
    Byte = 8,
    /// 16 bits
    HalfWord = 16,
    /// 32 bits
    Word = 32,
    /// 64 bits
    DoubleWord = 64,
}

#[derive(Debug)]
pub struct Scalar {
    /// How the value’s bits are to be interpreted.
    pub kind: ScalarKind,
    /// The size of the value in bits.
    pub width: BitWidth,
}

impl TryFrom<BaseType> for Scalar {
    type Error = SpirvCrossError;

    fn try_from(value: BaseType) -> Result<Self, Self::Error> {
        Ok(match value {
            BaseType::Boolean => Scalar {
                kind: ScalarKind::Bool,
                width: BitWidth::Bit,
            },
            BaseType::Int8 => Scalar {
                kind: ScalarKind::Int,
                width: BitWidth::Byte,
            },
            BaseType::Int16 => Scalar {
                kind: ScalarKind::Int,
                width: BitWidth::HalfWord,
            },
            BaseType::Int32 => Scalar {
                kind: ScalarKind::Int,
                width: BitWidth::Word,
            },
            BaseType::Int64 => Scalar {
                kind: ScalarKind::Int,
                width: BitWidth::DoubleWord,
            },
            BaseType::Uint8 => Scalar {
                kind: ScalarKind::Uint,
                width: BitWidth::Byte,
            },
            BaseType::Uint16 => Scalar {
                kind: ScalarKind::Uint,
                width: BitWidth::HalfWord,
            },
            BaseType::Uint32 => Scalar {
                kind: ScalarKind::Uint,
                width: BitWidth::Word,
            },
            BaseType::Uint64 => Scalar {
                kind: ScalarKind::Uint,
                width: BitWidth::DoubleWord,
            },
            BaseType::Fp16 => Scalar {
                kind: ScalarKind::Float,
                width: BitWidth::HalfWord,
            },
            BaseType::Fp32 => Scalar {
                kind: ScalarKind::Float,
                width: BitWidth::Word,
            },
            BaseType::Fp64 => Scalar {
                kind: ScalarKind::Float,
                width: BitWidth::DoubleWord,
            },

            _ => {
                return Err(SpirvCrossError::InvalidArgument(String::from(
                    "Invalid base type used to instantiate a scalar",
                )))
            }
        })
    }
}

#[derive(Debug)]
pub struct Type<'a> {
    name: Option<Cow<'a, str>>,
    id: TypeId,
    inner: TypeInner<'a>,
}

#[derive(Debug)]
pub struct StructMember<'a> {
    pub name: Option<Cow<'a, str>>,
    pub id: TypeId,
    pub index: usize,
    pub offset: u32,
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub id: TypeId,
    pub size: usize,
    pub members: Vec<StructMember<'a>>,
}

#[derive(Debug)]
pub enum TypeInner<'a> {
    Unknown,
    Void,
    Struct(Struct<'a>),
    Scalar(Scalar),
    Vector {
        size: u32,
        scalar: Scalar,
    },
    Matrix {
        columns: u32,
        rows: u32,
        scalar: Scalar,
    },
}

#[derive(Debug)]
struct RawTypeData {
    base_ty: BaseType,
    bit_width: u32,
    vec_size: u32,
    columns: u32,
    storage_class: crate::spirv::StorageClass,
    array_dims: Vec<SpvId>,
    array_is_literal: Vec<bool>,
}
/// Reflection of SPIR-V types.
impl<T> Compiler<'_, T> {
    fn process_struct(&self, struct_ty_id: TypeId) -> error::Result<Struct> {
        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.0.as_ptr(), struct_ty_id);
            let base_ty = sys::spvc_type_get_basetype(ty);
            assert_eq!(base_ty, BaseType::Struct);

            let mut struct_size = 0;
            sys::spvc_compiler_get_declared_struct_size(self.0.as_ptr(), ty, &mut struct_size)
                .ok(self)?;

            let member_type_len = sys::spvc_type_get_num_member_types(ty);
            let mut members = Vec::with_capacity(member_type_len as usize);
            for i in 0..member_type_len {
                let id = sys::spvc_type_get_member_type(ty, i);
                let name = CStr::from_ptr(sys::spvc_compiler_get_member_name(
                    self.0.as_ptr(),
                    struct_ty_id,
                    i,
                ))
                .to_string_lossy();

                let name = if name.is_empty() { None } else { Some(name) };

                let mut offset = 0;
                sys::spvc_compiler_type_struct_member_offset(self.0.as_ptr(), ty, i, &mut offset)
                    .ok(self)?;

                members.push(StructMember {
                    name,
                    id,
                    offset,
                    index: i as usize,
                })
            }

            Ok(Struct {
                id: struct_ty_id,
                size: struct_size,
                members,
            })
        }
    }

    fn process_vector(&self, id: TypeId, vec_width: u32) -> error::Result<TypeInner> {
        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.0.as_ptr(), id);
            let base_ty = sys::spvc_type_get_basetype(ty);
            Ok(TypeInner::Vector {
                size: vec_width,
                scalar: base_ty.try_into()?,
            })
        }
    }

    fn process_matrix(&self, id: TypeId, rows: u32, columns: u32) -> error::Result<TypeInner> {
        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.0.as_ptr(), id);
            let base_ty = sys::spvc_type_get_basetype(ty);
            Ok(TypeInner::Matrix {
                rows,
                columns,
                scalar: base_ty.try_into()?,
            })
        }
    }
    pub fn get_type(&self, id: TypeId) -> error::Result<Type> {
        // let raw = read_from_ptr::<br::ScType>(type_ptr);
        // let member_types = read_into_vec_from_ptr(raw.member_types, raw.member_types_size);
        // let array = read_into_vec_from_ptr(raw.array, raw.array_size);
        // let array_size_literal = read_into_vec_from_ptr(raw.array_size_literal, raw.array_size);
        // let image = raw.image;
        // let result = Type::from_raw(raw.type_, raw.vecsize, raw.columns, member_types, array, array_size_literal, image)?;

        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.0.as_ptr(), id);
            let base_ty = sys::spvc_type_get_basetype(ty);
            let name = CStr::from_ptr(sys::spvc_compiler_get_name(self.0.as_ptr(), id.0))
                .to_string_lossy();
            let name = if name.is_empty() { None } else { Some(name) };

            let array_dim_len = sys::spvc_type_get_num_array_dimensions(ty);
            if array_dim_len != 0 {
                panic!("need to handle array")
            }

            let vec_size = sys::spvc_type_get_vector_size(ty);
            let columns = sys::spvc_type_get_columns(ty);

            let mut prep = None;
            if vec_size > 1 && columns == 1 {
                prep = Some(self.process_vector(id, vec_size)?);
            }

            if vec_size > 1 && columns > 1 {
                prep = Some(self.process_matrix(id, vec_size, columns)?);
            }

            let inner = match base_ty {
                BaseType::Struct => {
                    let ty = self.process_struct(id)?;
                    eprintln!("{:?}", ty);
                    TypeInner::Struct(ty)
                }
                BaseType::Image | BaseType::SampledImage => {
                    todo!("image")
                }
                BaseType::Sampler => {
                    todo!("image")
                }
                BaseType::Boolean
                | BaseType::Int8
                | BaseType::Uint8
                | BaseType::Int16
                | BaseType::Uint16
                | BaseType::Int32
                | BaseType::Uint32
                | BaseType::Int64
                | BaseType::Uint64
                | BaseType::Fp16
                | BaseType::Fp32
                | BaseType::Fp64 => {
                    if let Some(prep) = prep {
                        prep
                    } else {
                        TypeInner::Scalar(base_ty.try_into()?)
                    }
                }

                BaseType::Unknown => TypeInner::Unknown,
                BaseType::Void => TypeInner::Void,

                BaseType::AtomicCounter => {
                    panic!("atomics")
                }

                _ => panic!("unhandled"), // BaseType::Image => {}
                                          // BaseType::SampledImage => {}
                                          // BaseType::Sampler => {}
                                          // BaseType::AccelerationStructure => {}
            };

            let bit_width = sys::spvc_type_get_bit_width(ty);

            let storage_class = sys::spvc_type_get_storage_class(ty);

            let array_dim_len = sys::spvc_type_get_num_array_dimensions(ty);

            let mut array_dims = Vec::with_capacity(array_dim_len as usize);
            for i in 0..array_dim_len {
                array_dims.push(sys::spvc_type_get_array_dimension(ty, i))
            }

            let mut array_is_literal = Vec::with_capacity(array_dim_len as usize);
            for i in 0..array_dim_len {
                array_is_literal.push(sys::spvc_type_array_dimension_is_literal(ty, i))
            }

            let raw = RawTypeData {
                base_ty,
                bit_width,
                vec_size,
                columns,
                storage_class,
                array_dims,
                array_is_literal,
            };
            eprintln!("{raw:#?}");

            let ty = Type { name, id, inner };
            eprintln!("{ty:?}");
            Ok(ty)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::types::TypeInner;
    use crate::compiler::{targets, Compiler};
    use crate::{Module, SpirvCross};
    use spirv_cross_sys::SpirvCrossError;

    const BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn create_compiler() -> Result<(), SpirvCrossError> {
        let spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        // println!("{:#?}", resources);

        let ty = compiler.get_type(resources.uniform_buffers[0].base_type_id)?;

        match ty.inner {
            TypeInner::Struct(ty) => {
                compiler.get_type(ty.members[0].id)?;
            }
            TypeInner::Vector { .. } => {}
            _ => {}
        }
        Ok(())
    }
}
