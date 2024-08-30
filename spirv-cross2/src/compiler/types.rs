use crate::compiler::Compiler;
use crate::error;
use spirv_cross_sys::{spvc_type, BaseType, SpvId, TypeId, VariableId};
use std::borrow::Cow;
use std::ffi::CStr;

use crate::error::{SpirvCrossError, ToContextError};
use crate::spirv::{Decoration, StorageClass};
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
    pub size: usize,
    pub matrix_stride: Option<u32>,
    pub array_stride: Option<u32>,
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub id: TypeId,
    pub size: usize,
    pub members: Vec<StructMember<'a>>,
}

#[derive(Debug)]
pub enum ArrayDimension {
    Literal(u32),
    SpecializationConstant(VariableId),
}

/// Enum with additional type information, depending on the kind of type.
///
/// The design of this API is inspired heavily by [`naga::TypeInner`](https://docs.rs/naga/latest/naga/enum.TypeInner.html),
/// with some changes to fit SPIR-V.
#[derive(Debug)]
pub enum TypeInner<'a> {
    Unknown,
    Void,
    Pointer {
        base: TypeId,
        storage: StorageClass,
    },
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
    Array {
        base: TypeId,
        storage: StorageClass,
        dimensions: Vec<ArrayDimension>,
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

                let mut size = 0;
                sys::spvc_compiler_get_declared_struct_member_size(
                    self.0.as_ptr(),
                    ty,
                    i,
                    &mut size,
                )
                .ok(self)?;

                let mut offset = 0;
                sys::spvc_compiler_type_struct_member_offset(self.0.as_ptr(), ty, i, &mut offset)
                    .ok(self)?;

                let mut matrix_stride = 0;
                let matrix_stride = sys::spvc_compiler_type_struct_member_matrix_stride(
                    self.0.as_ptr(),
                    ty,
                    i,
                    &mut matrix_stride,
                )
                .ok(self)
                .ok()
                .map(|_| matrix_stride);

                let mut array_stride = 0;
                let array_stride = sys::spvc_compiler_type_struct_member_array_stride(
                    self.0.as_ptr(),
                    ty,
                    i,
                    &mut array_stride,
                )
                .ok(self)
                .ok()
                .map(|_| array_stride);

                members.push(StructMember {
                    name,
                    id,
                    offset,
                    size,
                    index: i as usize,
                    matrix_stride,
                    array_stride,
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

    fn process_array<'a>(&self, id: TypeId, name: Option<Cow<'a, str>>) -> error::Result<Type<'a>> {
        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.0.as_ptr(), id);
            let base_type_id = sys::spvc_type_get_base_type_id(ty);

            let array_dim_len = sys::spvc_type_get_num_array_dimensions(ty);

            let mut array_dims = Vec::with_capacity(array_dim_len as usize);
            for i in 0..array_dim_len {
                array_dims.push(sys::spvc_type_get_array_dimension(ty, i))
            }

            let mut array_is_literal = Vec::with_capacity(array_dim_len as usize);
            for i in 0..array_dim_len {
                array_is_literal.push(sys::spvc_type_array_dimension_is_literal(ty, i))
            }

            let storage_class = sys::spvc_type_get_storage_class(ty);

            let array_dims = array_dims
                .into_iter()
                .enumerate()
                .map(|(index, dim)| {
                    if array_is_literal[index] {
                        ArrayDimension::Literal(dim.0)
                    } else {
                        ArrayDimension::SpecializationConstant(VariableId(dim))
                    }
                })
                .collect();

            Ok(Type {
                name,
                id,
                inner: TypeInner::Array {
                    base: base_type_id,
                    storage: storage_class,
                    dimensions: array_dims,
                },
            })
        }
    }

    /// Get the type description for the given type ID.
    ///
    /// In most cases, a `base_type_id` should be passed in unless
    /// pointer specifics are desired.
    ///
    /// Atomics are represented as `TypeInner::Pointer { storage: StorageClass::AtomicCounter, ... }`,
    /// usually with a scalar base type.
    pub fn get_type(&self, id: TypeId) -> error::Result<Type> {
        // let raw = read_from_ptr::<br::ScType>(type_ptr);
        // let member_types = read_into_vec_from_ptr(raw.member_types, raw.member_types_size);
        // let array = read_into_vec_from_ptr(raw.array, raw.array_size);
        // let array_size_literal = read_into_vec_from_ptr(raw.array_size_literal, raw.array_size);
        // let image = raw.image;
        // let result = Type::from_raw(raw.type_, raw.vecsize, raw.columns, member_types, array, array_size_literal, image)?;

        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.0.as_ptr(), id);
            let base_type_id = sys::spvc_type_get_base_type_id(ty);

            let base_ty = sys::spvc_type_get_basetype(ty);
            let name = CStr::from_ptr(sys::spvc_compiler_get_name(self.0.as_ptr(), id.0))
                .to_string_lossy();
            let name = if name.is_empty() { None } else { Some(name) };

            let array_dim_len = sys::spvc_type_get_num_array_dimensions(ty);
            if array_dim_len != 0 {
                return self.process_array(id, name);
            }

            // If it is not an array, has a proper storage class, and the base type id,
            // is not the type id, then it is an `OpTypePointer`.
            //
            // I wish there was a better way to expose this in the C API.
            let storage_class = sys::spvc_type_get_storage_class(ty);
            if storage_class != StorageClass::Generic && base_type_id != id {
                return Ok(Type {
                    name,
                    id,
                    inner: TypeInner::Pointer {
                        base: base_type_id,
                        storage: storage_class,
                    },
                });
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
                    // This should be covered by the pointer type above.
                    return Ok(Type {
                        name,
                        id,
                        inner: TypeInner::Pointer {
                            base: base_type_id,
                            storage: storage_class,
                        },
                    });
                }

                BaseType::AccelerationStructure => {
                    panic!("unhandled")
                }
            };

            let bit_width = sys::spvc_type_get_bit_width(ty);

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
            Ok(ty)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::types::TypeInner;
    use crate::compiler::{targets, Compiler};
    use crate::error::SpirvCrossError;
    use crate::{Module, SpirvCross};

    macro_rules! include_transmute {
        ($file:expr) => {{
            #[repr(C)]
            pub struct AlignedAs<Align, Bytes: ?Sized> {
                pub _align: [Align; 0],
                pub bytes: Bytes,
            }

            static ALIGNED: &AlignedAs<&[u32], [u8]> = &AlignedAs {
                _align: [],
                bytes: *include_bytes!($file),
            };

            &ALIGNED.bytes
        }};
    }

    static BASIC_SPV: &[u8] = include_transmute!("../../basic.spv");

    #[test]
    pub fn get_stage_outputs() -> Result<(), SpirvCrossError> {
        let mut spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        // println!("{:#?}", resources);

        let ty = compiler.get_type(resources.uniform_buffers[0].base_type_id)?;
        eprintln!("{ty:?}");

        // match ty.inner {
        //     TypeInner::Struct(ty) => {
        //         compiler.get_type(ty.members[0].id)?;
        //     }
        //     TypeInner::Vector { .. } => {}
        //     _ => {}
        // }
        Ok(())
    }
}
