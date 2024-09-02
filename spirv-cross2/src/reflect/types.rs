use crate::Compiler;
use crate::{error, spirv};
use spirv_cross_sys::BaseType;

use crate::error::{SpirvCrossError, ToContextError};
use crate::handle::Handle;
use crate::handle::{ConstantId, TypeId};
use crate::spirv::StorageClass;
use crate::string::ContextStr;
use spirv_cross_sys as sys;

/// The kind of scalar
#[derive(Debug)]
#[repr(u8)]
pub enum ScalarKind {
    /// Signed integer.
    Int = 0,
    /// Unsigned integer.
    Uint = 1,
    /// Floating point number.
    Float = 2,
    /// Boolean.
    Bool = 3,
}

/// The bit width of a scalar.
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

/// A scalar type.
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
    pub id: Handle<TypeId>,
    pub name: Option<ContextStr<'a>>,
    pub inner: TypeInner<'a>,
}

#[derive(Debug)]
pub struct StructMember<'a> {
    pub id: Handle<TypeId>,
    pub struct_type: Handle<TypeId>,
    pub name: Option<ContextStr<'a>>,
    pub index: usize,
    pub offset: u32,
    pub size: usize,
    pub matrix_stride: Option<u32>,
    pub array_stride: Option<u32>,
}

#[derive(Debug)]
pub struct StructType<'a> {
    pub id: Handle<TypeId>,
    pub size: usize,
    pub members: Vec<StructMember<'a>>,
}

#[derive(Debug)]
pub enum ArrayDimension {
    Literal(u32),
    Constant(Handle<ConstantId>),
}

#[derive(Debug)]
pub enum ImageClass {
    /// Combined image samplers.
    Sampled {
        /// Whether this is a depth sampler (i.e. `samplerNDShadow`.)
        depth: bool,
        /// Whether this is a multisampled image.
        multisampled: bool,
        /// Whether or not this image is arrayed
        arrayed: bool,
    },
    /// Separate image.
    Texture {
        /// Whether this is a multisampled image.
        multisampled: bool,
        /// Whether this image is arrayed
        arrayed: bool,
    },
    /// Storage images.
    LoadStore { format: spirv::ImageFormat },
}

#[derive(Debug)]
pub struct ImageType {
    /// The id of the type,
    pub id: Handle<TypeId>,
    /// The id of the type returned when the image is sampled or read from.
    pub sampled_type: Handle<TypeId>,
    /// The dimension of the image.
    pub dimension: spirv::Dim,
    /// The class of the image.
    pub class: ImageClass,
}

/// Enum with additional type information, depending on the kind of type.
///
/// The design of this API is inspired heavily by [`naga::TypeInner`](https://docs.rs/naga/latest/naga/enum.TypeInner.html),
/// with some changes to fit SPIR-V.
#[derive(Debug)]
pub enum TypeInner<'a> {
    /// Unknown type.
    Unknown,
    /// The void type.
    Void,
    /// A pointer to another type.
    ///
    /// Atomics are represented as [`TypeInner::Pointer`] with
    /// the storage class [`StorageClass::AtomicCounter`].
    Pointer {
        /// A handle to the base type this points to.
        base: Handle<TypeId>,
        /// The storage class of the pointer.
        ///
        /// Atomics are represented as [`TypeInner::Pointer`] with
        /// the storage class [`StorageClass::AtomicCounter`].
        storage: StorageClass,
    },
    /// A struct type.
    Struct(StructType<'a>),
    /// A scalar type.
    Scalar(Scalar),
    /// A vector type.
    ///
    /// For example, `vec4` would have a width of 4,
    /// and a scalar type with [`ScalarKind::Float`] and bit-width 32.
    Vector {
        /// The width of the vector.
        width: u32,
        /// The scalar type of the vector.
        scalar: Scalar,
    },
    /// A matrix type.
    ///
    /// For example, `mat4` would have 4 columns, 4 rows,
    /// and a scalar type with [`ScalarKind::Float`] and bit-width 32.
    Matrix {
        /// The number of columns of the matrix type.
        columns: u32,
        /// The number of rows of the matrix type.
        rows: u32,
        /// The scalar type of the matrix.
        scalar: Scalar,
    },
    /// An array type.
    Array {
        /// The base type that the type is an array of.
        base: Handle<TypeId>,
        /// The storage class of the array.
        storage: StorageClass,
        /// The dimensions of the array.
        ///
        /// Most of the time, these will be [`ArrayDimension::Literal`].
        /// If an array dimension is specified as a specialization constant,
        /// then the dimension will be [`ArrayDimension::Constant`].
        ///
        /// The order of dimensions follow SPIR-V semantics, i.e. backwards compared to C-style
        /// declarations.
        ///
        /// i.e. `int a[4][6]` will return as `[Linear(6), Linear(4)]`.
        dimensions: Vec<ArrayDimension>,
    },
    Image(ImageType),
    /// An opaque acceleration structure.
    AccelerationStructure,
    /// An opaque sampler.
    Sampler,
}

/// Reflection of SPIR-V types.
impl<T> Compiler<'_, T> {
    fn process_struct(&self, struct_ty_id: TypeId) -> error::Result<StructType> {
        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.ptr.as_ptr(), struct_ty_id);
            let base_ty = sys::spvc_type_get_basetype(ty);
            assert_eq!(base_ty, BaseType::Struct);

            let mut struct_size = 0;
            sys::spvc_compiler_get_declared_struct_size(self.ptr.as_ptr(), ty, &mut struct_size)
                .ok(self)?;

            let member_type_len = sys::spvc_type_get_num_member_types(ty);
            let mut members = Vec::with_capacity(member_type_len as usize);
            for i in 0..member_type_len {
                let id = sys::spvc_type_get_member_type(ty, i);
                let name = ContextStr::from_ptr(sys::spvc_compiler_get_member_name(
                    self.ptr.as_ptr(),
                    struct_ty_id,
                    i,
                ));

                let name = if name.as_ref().is_empty() {
                    None
                } else {
                    Some(name)
                };

                let mut size = 0;
                sys::spvc_compiler_get_declared_struct_member_size(
                    self.ptr.as_ptr(),
                    ty,
                    i,
                    &mut size,
                )
                .ok(self)?;

                let mut offset = 0;
                sys::spvc_compiler_type_struct_member_offset(self.ptr.as_ptr(), ty, i, &mut offset)
                    .ok(self)?;

                let mut matrix_stride = 0;
                let matrix_stride = sys::spvc_compiler_type_struct_member_matrix_stride(
                    self.ptr.as_ptr(),
                    ty,
                    i,
                    &mut matrix_stride,
                )
                .ok(self)
                .ok()
                .map(|_| matrix_stride);

                let mut array_stride = 0;
                let array_stride = sys::spvc_compiler_type_struct_member_array_stride(
                    self.ptr.as_ptr(),
                    ty,
                    i,
                    &mut array_stride,
                )
                .ok(self)
                .ok()
                .map(|_| array_stride);

                members.push(StructMember {
                    name,
                    id: self.create_handle(id),
                    struct_type: self.create_handle(struct_ty_id),
                    offset,
                    size,
                    index: i as usize,
                    matrix_stride,
                    array_stride,
                })
            }

            Ok(StructType {
                id: self.create_handle(struct_ty_id),
                size: struct_size,
                members,
            })
        }
    }

    fn process_vector(&self, id: TypeId, vec_width: u32) -> error::Result<TypeInner> {
        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.ptr.as_ptr(), id);
            let base_ty = sys::spvc_type_get_basetype(ty);
            Ok(TypeInner::Vector {
                width: vec_width,
                scalar: base_ty.try_into()?,
            })
        }
    }

    fn process_matrix(&self, id: TypeId, rows: u32, columns: u32) -> error::Result<TypeInner> {
        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.ptr.as_ptr(), id);
            let base_ty = sys::spvc_type_get_basetype(ty);
            Ok(TypeInner::Matrix {
                rows,
                columns,
                scalar: base_ty.try_into()?,
            })
        }
    }

    fn process_array<'a>(
        &self,
        id: TypeId,
        name: Option<ContextStr<'a>>,
    ) -> error::Result<Type<'a>> {
        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.ptr.as_ptr(), id);
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
                        ArrayDimension::Constant(self.create_handle(ConstantId(dim)))
                    }
                })
                .collect();

            Ok(Type {
                name,
                id: self.create_handle(id),
                inner: TypeInner::Array {
                    base: self.create_handle(base_type_id),
                    storage: storage_class,
                    dimensions: array_dims,
                },
            })
        }
    }

    fn process_image(&self, id: TypeId) -> error::Result<ImageType> {
        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.ptr.as_ptr(), id);
            let base_ty = sys::spvc_type_get_basetype(ty);
            let sampled_id = sys::spvc_type_get_image_sampled_type(ty);
            let dimension = sys::spvc_type_get_image_dimension(ty);
            let depth = sys::spvc_type_get_image_is_depth(ty);
            let arrayed = sys::spvc_type_get_image_arrayed(ty);
            let storage = sys::spvc_type_get_image_is_storage(ty);
            let multisampled = sys::spvc_type_get_image_multisampled(ty);
            let format = sys::spvc_type_get_image_storage_format(ty);

            let class = if storage {
                ImageClass::LoadStore { format }
            } else if base_ty == BaseType::SampledImage {
                ImageClass::Sampled {
                    depth,
                    multisampled,
                    arrayed,
                }
            } else {
                ImageClass::Texture {
                    multisampled,
                    arrayed,
                }
            };

            Ok(ImageType {
                id: self.create_handle(id),
                sampled_type: self.create_handle(sampled_id),
                dimension,
                class,
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
    pub fn type_description(&self, id: Handle<TypeId>) -> error::Result<Type> {
        let id = self.yield_id(id)?;

        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.ptr.as_ptr(), id);
            let base_type_id = sys::spvc_type_get_base_type_id(ty);

            let base_ty = sys::spvc_type_get_basetype(ty);
            let name = ContextStr::from_ptr(sys::spvc_compiler_get_name(self.ptr.as_ptr(), id.0));

            let name = if name.as_ref().is_empty() {
                None
            } else {
                Some(name)
            };

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
                    id: self.create_handle(id),
                    inner: TypeInner::Pointer {
                        base: self.create_handle(base_type_id),
                        storage: storage_class,
                    },
                });
            }

            let vec_size = sys::spvc_type_get_vector_size(ty);
            let columns = sys::spvc_type_get_columns(ty);

            // Handle non-scalar case
            let mut maybe_non_scalar = None;
            if vec_size > 1 && columns == 1 {
                maybe_non_scalar = Some(self.process_vector(id, vec_size)?);
            }

            if vec_size > 1 && columns > 1 {
                maybe_non_scalar = Some(self.process_matrix(id, vec_size, columns)?);
            }

            let inner = match base_ty {
                BaseType::Struct => {
                    let ty = self.process_struct(id)?;
                    TypeInner::Struct(ty)
                }
                BaseType::Image | BaseType::SampledImage => {
                    return Ok(Type {
                        id: self.create_handle(id),
                        name,
                        inner: TypeInner::Image(self.process_image(id)?),
                    });
                }
                BaseType::Sampler => {
                    return Ok(Type {
                        id: self.create_handle(id),
                        name,
                        inner: TypeInner::Sampler,
                    });
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
                    if let Some(prep) = maybe_non_scalar {
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
                        id: self.create_handle(id),
                        name,
                        inner: TypeInner::Pointer {
                            base: self.create_handle(base_type_id),
                            storage: storage_class,
                        },
                    });
                }

                BaseType::AccelerationStructure => {
                    return Ok(Type {
                        id: self.create_handle(id),
                        name,
                        inner: TypeInner::AccelerationStructure,
                    })
                }
            };

            let ty = Type {
                name,
                id: self.create_handle(id),
                inner,
            };
            Ok(ty)
        }
    }

    /// Get the size of the struct with the specified runtime array size,
    /// if the struct contains a runtime array.
    pub fn declared_struct_size_with_runtime_array(
        &self,
        struct_type: StructType,
        array_size: usize,
    ) -> error::Result<usize> {
        // port from https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross.cpp#L2006C1-L2007C1
        let mut size = struct_type.size;
        if let Some(last) = struct_type.members.last() {
            let Some(stride) = last.array_stride else {
                return Ok(size);
            };

            let inner = self.type_description(last.id)?.inner;
            if let TypeInner::Array { dimensions, .. } = inner {
                if let Some(ArrayDimension::Literal(0)) = dimensions.first() {
                    size += array_size * stride as usize
                }
            }
        }

        Ok(size)
    }
}

#[cfg(test)]
mod test {
    use crate::error::SpirvCrossError;
    use crate::Compiler;
    use crate::{targets, Module, SpirvCross};

    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn get_stage_outputs() -> Result<(), SpirvCrossError> {
        let spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        // println!("{:#?}", resources);

        let ty = compiler.type_description(resources.uniform_buffers[0].base_type_id)?;
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
