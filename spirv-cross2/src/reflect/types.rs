use crate::error;
use crate::Compiler;
use spirv::StorageClass;
use spirv_cross_sys::{BaseType, SpvId, VariableId};

use crate::error::{SpirvCrossError, ToContextError};
use crate::handle::Handle;
use crate::handle::{ConstantId, TypeId};
use crate::sealed::Sealed;
use crate::string::CompilerStr;
use spirv_cross_sys as sys;

/// The kind of scalar
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum BitWidth {
    /// 1 bit, padded to 1 byte.
    Bit = 1,
    /// 8 bits, 1 byte.
    Byte = 8,
    /// 16 bits, 2 bytes.
    HalfWord = 16,
    /// 32 bits, 4 bytes.
    Word = 32,
    /// 64 bits, 8 bytes.
    DoubleWord = 64,
}

impl BitWidth {
    /// Get the size of the bit width in bytes.
    ///
    /// Bit-sized types are padded to a whole byte.
    pub const fn byte_size(&self) -> usize {
        match self {
            BitWidth::Bit => 1,
            BitWidth::Byte => 1,
            BitWidth::HalfWord => 2,
            BitWidth::Word => 4,
            BitWidth::DoubleWord => 8,
        }
    }
}

/// A scalar type.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Scalar {
    /// How the value’s bits are to be interpreted.
    pub kind: ScalarKind,
    /// The size of the value in bits.
    pub size: BitWidth,
}

impl TryFrom<BaseType> for Scalar {
    type Error = SpirvCrossError;

    fn try_from(value: BaseType) -> Result<Self, Self::Error> {
        Ok(match value {
            BaseType::Boolean => Scalar {
                kind: ScalarKind::Bool,
                size: BitWidth::Bit,
            },
            BaseType::Int8 => Scalar {
                kind: ScalarKind::Int,
                size: BitWidth::Byte,
            },
            BaseType::Int16 => Scalar {
                kind: ScalarKind::Int,
                size: BitWidth::HalfWord,
            },
            BaseType::Int32 => Scalar {
                kind: ScalarKind::Int,
                size: BitWidth::Word,
            },
            BaseType::Int64 => Scalar {
                kind: ScalarKind::Int,
                size: BitWidth::DoubleWord,
            },
            BaseType::Uint8 => Scalar {
                kind: ScalarKind::Uint,
                size: BitWidth::Byte,
            },
            BaseType::Uint16 => Scalar {
                kind: ScalarKind::Uint,
                size: BitWidth::HalfWord,
            },
            BaseType::Uint32 => Scalar {
                kind: ScalarKind::Uint,
                size: BitWidth::Word,
            },
            BaseType::Uint64 => Scalar {
                kind: ScalarKind::Uint,
                size: BitWidth::DoubleWord,
            },
            BaseType::Fp16 => Scalar {
                kind: ScalarKind::Float,
                size: BitWidth::HalfWord,
            },
            BaseType::Fp32 => Scalar {
                kind: ScalarKind::Float,
                size: BitWidth::Word,
            },
            BaseType::Fp64 => Scalar {
                kind: ScalarKind::Float,
                size: BitWidth::DoubleWord,
            },

            _ => {
                return Err(SpirvCrossError::InvalidArgument(String::from(
                    "Invalid base type used to instantiate a scalar",
                )))
            }
        })
    }
}

/// A type definition.
#[derive(Debug, Clone)]
pub struct Type<'a> {
    /// The SPIR-V ID of the type.
    pub id: Handle<TypeId>,
    /// The name of the type, if any.
    pub name: Option<CompilerStr<'a>>,
    /// Inner details about the type.
    pub inner: TypeInner<'a>,
    /// A size hint for the type,
    /// representing the minimum size the type could be.
    pub size_hint: TypeSizeHint,
}

/// Type definition for a struct member.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructMember<'a> {
    /// The type ID of the struct member.
    pub id: Handle<TypeId>,
    /// The type ID of the parent struct.
    pub struct_type: Handle<TypeId>,
    /// The name of the struct member.
    pub name: Option<CompilerStr<'a>>,
    /// The index of the member inside the struct.
    pub index: usize,
    /// The offset in bytes from the beginning of the struct.
    pub offset: u32,
    /// The declared size of the struct member.
    pub size: usize,
    /// The matrix stride of the member, if any.
    ///
    /// Matrix strides are only decorated on struct members.
    pub matrix_stride: Option<u32>,
    /// The array stride of the member, if any.
    ///
    /// Array strides are only decorated on struct members.
    pub array_stride: Option<u32>,
}

/// Type definition for a struct.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructType<'a> {
    /// The type ID of the struct.
    pub id: Handle<TypeId>,
    /// The size of the struct in bytes.
    pub size: usize,
    /// The members of the struct.
    pub members: Vec<StructMember<'a>>,
}

/// Valid values that specify the dimensions of an array.
///
/// Most of the time, these will be [`ArrayDimension::Literal`].
/// If an array dimension is specified as a specialization constant,
/// then the dimension will be [`ArrayDimension::Constant`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ArrayDimension {
    /// A literal array dimension, i.e. `array[4]`.
    Literal(u32),
    /// An array dimension specified as a specialization constant.
    ///
    /// This would show up in something like the following
    ///
    /// ```glsl
    /// layout (constant_id = 0) const int SSAO_KERNEL_SIZE = 2;
    /// vec4[SSAO_KERNEL_SIZE] kernel;
    /// ```
    Constant(Handle<ConstantId>),
}

/// Class of image or texture handle.
#[derive(Debug, Clone, Eq, PartialEq)]
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
        /// Whether this image is arrayed.
        arrayed: bool,
    },
    /// Storage images.
    Storage {
        /// The image format of the storage image.
        format: spirv::ImageFormat,
    },
}

/// Type definition for an image or texture handle.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ImageType {
    /// The id of the type.
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
#[derive(Debug, Clone, Eq, PartialEq)]
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
        /// Whether this pointer is a forward pointer (i.e. `base` is another pointer type).
        forward: bool,
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
        /// The stride, in bytes, of the array’s elements, if this array type
        /// appears as a struct member.
        stride: Option<u32>,
    },
    /// A texture or image handle.
    Image(ImageType),
    /// An opaque acceleration structure.
    AccelerationStructure,
    /// An opaque sampler.
    Sampler,
}

/// A size hole requiring the stride of a matrix,
/// and whether the matrix is column or row major.
///
/// The hole is a `(usize, bool)` tuple, which
/// is the stride of the matrix, and whether the
/// matrix is row major. By default, the matrix is
/// considered column major.
#[derive(Debug, Clone)]
pub struct MatrixStrideHole {
    columns: usize,
    rows: usize,
    declared: usize,
}

impl Sealed for MatrixStrideHole {}
impl ResolveSize for MatrixStrideHole {
    type Hole = (usize, bool);

    fn declared(&self) -> usize {
        self.declared
    }

    fn resolve(&self, hole: Self::Hole) -> usize {
        let (stride, is_row_major) = hole;
        if is_row_major {
            stride * self.rows
        } else {
            stride * self.columns
        }
    }
}

/// A size hole requiring the number of elements in a runtime array.
///
/// This hole must be resolved with the size of the array.
#[derive(Debug, Clone)]
pub struct ArraySizeHole {
    stride: usize,
    declared: usize,
}

/// A size hole representing a missing or unknown array stride.
///
/// This hole must be resolved with a function that calculates the stride,
/// given the size hint of the base type of the array.
///
/// The declared size of this hole is the number of elements
/// times the declared size of the base type.
#[derive(Debug, Clone)]
pub struct UnknownStrideHole {
    hint: Box<TypeSizeHint>,
    count: usize,
}

impl Sealed for UnknownStrideHole {}
impl ResolveSize for UnknownStrideHole {
    type Hole = Box<dyn FnOnce(&TypeSizeHint) -> usize>;

    fn declared(&self) -> usize {
        self.count * self.hint.declared()
    }

    fn resolve(&self, hole: Self::Hole) -> usize {
        self.count * hole(&self.hint)
    }
}

impl ResolveSize for usize {
    type Hole = core::convert::Infallible;

    fn declared(&self) -> usize {
        *self
    }

    fn resolve(&self, _hole: Self::Hole) -> usize {
        self.declared()
    }
}

impl ResolveSize for ArraySizeHole {
    type Hole = usize;

    fn declared(&self) -> usize {
        self.declared
    }

    fn resolve(&self, count: Self::Hole) -> usize {
        count * self.stride
    }
}

impl Sealed for ArraySizeHole {}
impl Sealed for usize {}

/// A size hint for a type. This could be a statically known size,
/// or need to resolve a hole before getting a more accurate.
///
/// Size hints resolve array sizes involving specialization constants.
///
/// If an array stride is found, it will calculate a statically known size with
/// the array stride.
#[derive(Debug, Clone)]
pub enum TypeSizeHint {
    /// A statically known type size hint.
    Static(usize),
    /// The size of a runtime array, which is missing an element count.
    RuntimeArray(ArraySizeHole),
    /// A matrix type.
    Matrix(MatrixStrideHole),
    /// The array stride is missing or unknowable.
    UnknownArrayStride(UnknownStrideHole),
}

impl TypeSizeHint {
    /// Get the statically known, declared size of a type hint,
    /// ignoring any holes in the calculation.
    pub fn declared(&self) -> usize {
        match &self {
            TypeSizeHint::Static(sz) => *sz,
            TypeSizeHint::RuntimeArray(hole) => hole.declared(),
            TypeSizeHint::UnknownArrayStride(hole) => hole.declared(),
            TypeSizeHint::Matrix(hole) => hole.declared(),
        }
    }

    /// Whether the size hint is statically known.
    pub fn is_static(&self) -> bool {
        matches!(self, TypeSizeHint::Static(_))
    }
}

/// Trait for size hints that need to be resolved against a hole.
pub trait ResolveSize: Sealed {
    /// The type of the hole needed to resolve the size.
    type Hole;

    /// Get the declared size in bytes, regardless of any holes.
    fn declared(&self) -> usize;

    /// Resolve the size (in bytes) against the hole.
    fn resolve(&self, hole: Self::Hole) -> usize;
}

/// Reflection of SPIR-V types.
impl<T> Compiler<T> {
    // None of the names here belong to the context, they belong to the compiler.
    // so 'ctx is unsound to return.

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
                let name = CompilerStr::from_ptr(
                    sys::spvc_compiler_get_member_name(self.ptr.as_ptr(), struct_ty_id, i),
                    self.ctx.drop_guard(),
                );

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
        name: Option<CompilerStr<'a>>,
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

            let Some(storage_class) = spirv::StorageClass::from_u32(storage_class.0 as u32) else {
                return Err(SpirvCrossError::InvalidSpirv(format!(
                    "Unknown StorageClass found: {}",
                    storage_class.0
                )));
            };

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

            let id = self.create_handle(id);
            let stride = self
                .decoration(id, spirv::Decoration::ArrayStride)?
                .and_then(|s| s.as_literal());

            let inner = TypeInner::Array {
                base: self.create_handle(base_type_id),
                storage: storage_class,
                dimensions: array_dims,
                stride,
            };

            let size_hint = self.type_size_hint(&inner)?;

            Ok(Type {
                name,
                id,
                inner,
                size_hint,
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

            let Some(format) = spirv::ImageFormat::from_u32(format.0 as u32) else {
                return Err(SpirvCrossError::InvalidSpirv(format!(
                    "Unknown image format found: {}",
                    format.0
                )));
            };

            let Some(dimension) = spirv::Dim::from_u32(dimension.0 as u32) else {
                return Err(SpirvCrossError::InvalidSpirv(format!(
                    "Unknown image dimension found: {}",
                    dimension.0
                )));
            };

            let class = if storage {
                ImageClass::Storage { format }
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
            let name = CompilerStr::from_ptr(
                sys::spvc_compiler_get_name(self.ptr.as_ptr(), id.0),
                self.ctx.drop_guard(),
            );

            let name = if name.as_ref().is_empty() {
                None
            } else {
                Some(name)
            };

            let array_dim_len = sys::spvc_type_get_num_array_dimensions(ty);
            if array_dim_len != 0 {
                return self.process_array(id, name);
            }

            // pointer types
            if sys::spvc_rs_type_is_pointer(ty) {
                let storage_class = sys::spvc_type_get_storage_class(ty);
                let Some(storage_class) = spirv::StorageClass::from_u32(storage_class.0 as u32)
                else {
                    return Err(SpirvCrossError::InvalidSpirv(format!(
                        "Unknown StorageClass found: {}",
                        storage_class.0
                    )));
                };

                let forward = sys::spvc_rs_type_is_forward_pointer(ty);

                let inner = TypeInner::Pointer {
                    base: self.create_handle(base_type_id),
                    storage: storage_class,
                    forward,
                };

                let size_hint = self.type_size_hint(&inner)?;

                return Ok(Type {
                    name,
                    id: self.create_handle(id),
                    inner,
                    size_hint,
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
                    TypeInner::Image(self.process_image(id)?)
                }
                BaseType::Sampler => TypeInner::Sampler,
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
                    let storage_class = sys::spvc_type_get_storage_class(ty);
                    let Some(storage_class) = spirv::StorageClass::from_u32(storage_class.0 as u32)
                    else {
                        return Err(SpirvCrossError::InvalidSpirv(format!(
                            "Unknown StorageClass found: {}",
                            storage_class.0
                        )));
                    };

                    let forward = sys::spvc_rs_type_is_forward_pointer(ty);

                    TypeInner::Pointer {
                        base: self.create_handle(base_type_id),
                        storage: storage_class,
                        forward,
                    }
                }

                BaseType::AccelerationStructure => TypeInner::AccelerationStructure,
            };

            let size_hint = self.type_size_hint(&inner)?;
            let ty = Type {
                name,
                id: self.create_handle(id),
                inner,
                size_hint,
            };
            Ok(ty)
        }
    }

    /// Get the minimum size of this type in bytes,
    /// as declared in the shader.
    ///
    /// This will resolve array sizes involving specialization constants.
    fn type_size_hint(&self, ty: &TypeInner) -> error::Result<TypeSizeHint> {
        Ok(match ty {
            TypeInner::Pointer { .. } => TypeSizeHint::Static(BitWidth::Word.byte_size()),
            TypeInner::Struct(s) => {
                if let Some(stride) = self.struct_has_runtime_array(s)? {
                    TypeSizeHint::RuntimeArray(ArraySizeHole {
                        stride: stride as usize,
                        declared: s.size,
                    })
                } else {
                    TypeSizeHint::Static(s.size)
                }
            }
            TypeInner::Scalar(s) => TypeSizeHint::Static(s.size.byte_size()),
            TypeInner::Vector { width, scalar } => {
                TypeSizeHint::Static((*width as usize) * scalar.size.byte_size())
            }

            TypeInner::Matrix {
                columns,
                rows,
                scalar,
            } => {
                // Matrices have alignment 4, so we get the next power of 4.
                let rows_aligned = (rows + 3 & !0x3) as usize;

                let scalar_width = scalar.size.byte_size();
                let columns = *columns as usize;
                let declared = rows_aligned * scalar_width * columns;
                TypeSizeHint::Matrix(MatrixStrideHole {
                    columns,
                    rows: *rows as usize,
                    declared,
                })
            }
            TypeInner::Array {
                dimensions,
                stride,
                base,
                ..
            } => {
                let mut count = 1usize;
                for dim in dimensions.iter() {
                    match dim {
                        ArrayDimension::Literal(a) => count = count * (*a as usize),
                        ArrayDimension::Constant(c) => {
                            let value = self.specialization_constant_value::<u32>(*c)?;
                            count = count * value as usize;
                        } // prod = prod * 1
                    }
                }

                if let Some(stride) = stride {
                    TypeSizeHint::Static(count * (*stride as usize))
                } else {
                    // resolve the size of the basetype
                    let base_stride = self.type_description(*base)?.size_hint;
                    if base_stride.is_static() {
                        TypeSizeHint::Static(count * base_stride.declared())
                    } else {
                        TypeSizeHint::UnknownArrayStride(UnknownStrideHole {
                            hint: Box::new(base_stride),
                            count,
                        })
                    }
                }
            }
            TypeInner::Image(_)
            | TypeInner::AccelerationStructure
            | TypeInner::Sampler
            | TypeInner::Unknown
            | TypeInner::Void => TypeSizeHint::Static(0),
        })
    }

    /// Check if the struct has a runtime array. If so, return the stride
    /// of the array.
    fn struct_has_runtime_array(&self, struct_type: &StructType) -> error::Result<Option<u32>> {
        if let Some(last) = struct_type.members.last() {
            let Some(array_stride) = last.array_stride else {
                return Ok(None);
            };

            let inner = self.type_description(last.id)?.inner;
            if let TypeInner::Array { dimensions, .. } = inner {
                if let Some(ArrayDimension::Literal(0)) = dimensions.first() {
                    return Ok(Some(array_stride));
                }
            }
        }

        Ok(None)
    }

    /// Get the underlying type of the variable.
    pub fn variable_type(
        &self,
        variable: impl Into<Handle<VariableId>>,
    ) -> error::Result<Handle<TypeId>> {
        let variable = variable.into();
        let variable_id = self.yield_id(variable)?;

        unsafe {
            let mut type_id = TypeId(SpvId(0));
            sys::spvc_rs_compiler_variable_get_type(self.ptr.as_ptr(), variable_id, &mut type_id)
                .ok(self)?;

            Ok(self.create_handle(type_id))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::SpirvCrossError;
    use crate::Compiler;
    use crate::{targets, Module};

    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn get_stage_outputs() -> Result<(), SpirvCrossError> {
        let vec = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&vec));

        let compiler: Compiler<targets::None> = Compiler::new(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        // println!("{:#?}", resources);

        let ty = compiler.type_description(resources.uniform_buffers[0].base_type_id)?;
        eprintln!("{ty:?}");

        drop(compiler);
        eprintln!("{resources:?}");
        eprintln!("{resources:?}");
        // match ty.inner {
        //     TypeInner::Struct(ty) => {
        //         compiler.get_type(ty.members[0].id)?;
        //     }
        //     TypeInner::Vector { .. } => {}
        //     _ => {}
        // }
        Ok(())
    }

    #[test]
    pub fn set_member_name_validity_test() -> Result<(), SpirvCrossError> {
        let vec = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&vec));

        let mut compiler: Compiler<targets::None> = Compiler::new(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        // println!("{:#?}", resources);

        let ty = compiler.type_description(resources.uniform_buffers[0].base_type_id)?;
        let id = ty.id;

        let name = compiler.member_name(id, 0)?;
        assert_eq!(Some("MVP"), name.as_deref());

        compiler.set_member_name(ty.id, 0, "NotMVP")?;
        // assert_eq!(Some("MVP"), name.as_deref());

        let name = compiler.member_name(id, 0)?;
        assert_eq!(Some("NotMVP"), name.as_deref());
        let resources = compiler.shader_resources()?.all_resources()?;

        let ty = compiler.type_description(resources.uniform_buffers[0].base_type_id)?;

        Ok(())
    }

    #[test]
    pub fn get_variable_type_test() -> Result<(), SpirvCrossError> {
        let vec = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&vec));

        let mut compiler: Compiler<targets::None> = Compiler::new(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        let variable = resources.uniform_buffers[0].id;
        assert_eq!(
            resources.uniform_buffers[0].type_id.id(),
            compiler.variable_type(variable)?.id()
        );

        eprintln!("{:?}", resources);
        Ok(())
    }
}
