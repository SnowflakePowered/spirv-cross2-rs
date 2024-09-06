use crate::sealed::Sealed;
use spirv_cross_sys::{spvc_constant, spvc_specialization_constant, TypeId};
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};
use std::slice;

use crate::error::{SpirvCrossError, ToContextError};
use crate::handle::{ConstantId, Handle, PhantomCompiler};
use crate::{error, Compiler};
use spirv_cross_sys as sys;

/// A marker trait for types that can be represented as a scalar SPIR-V constant.
pub trait ConstantScalar: Default + Sealed + Copy {
    #[doc(hidden)]
    unsafe fn get(constant: spvc_constant, column: u32, row: u32) -> Self;

    #[doc(hidden)]
    unsafe fn set(constant: spvc_constant, column: u32, row: u32, value: Self);
}

macro_rules! impl_spvc_constant {
    ($get:ident  $set:ident $prim:ty) => {
        impl Sealed for $prim {}
        impl ConstantScalar for $prim {
            unsafe fn get(constant: spvc_constant, column: u32, row: u32) -> Self {
                unsafe { ::spirv_cross_sys::$get(constant, column, row) as Self }
            }

            unsafe fn set(constant: spvc_constant, column: u32, row: u32, value: Self) {
                unsafe { ::spirv_cross_sys::$set(constant, column, row, value) }
            }
        }
    };
}

impl_spvc_constant!(spvc_constant_get_scalar_i8 spvc_constant_set_scalar_i8 i8);
impl_spvc_constant!(spvc_constant_get_scalar_i16 spvc_constant_set_scalar_i16 i16);
impl_spvc_constant!(spvc_constant_get_scalar_i32 spvc_constant_set_scalar_i32 i32);
impl_spvc_constant!(spvc_constant_get_scalar_i64 spvc_constant_set_scalar_i64 i64);

impl_spvc_constant!(spvc_constant_get_scalar_u8 spvc_constant_set_scalar_u8 u8);
impl_spvc_constant!(spvc_constant_get_scalar_u16 spvc_constant_set_scalar_u16 u16);
impl_spvc_constant!(spvc_constant_get_scalar_u32 spvc_constant_set_scalar_u32 u32);
impl_spvc_constant!(spvc_constant_get_scalar_u64 spvc_constant_set_scalar_u64 u64);

impl_spvc_constant!(spvc_constant_get_scalar_fp32 spvc_constant_set_scalar_fp32 f32);
impl_spvc_constant!(spvc_constant_get_scalar_fp64 spvc_constant_set_scalar_fp64 f64);

#[cfg(feature = "f16")]
impl Sealed for half::f16 {}

#[cfg(feature = "f16")]
#[cfg_attr(docsrs, doc(cfg(feature = "f16")))]
impl ConstantScalar for half::f16 {
    unsafe fn get(constant: spvc_constant, column: u32, row: u32) -> Self {
        let f32 = unsafe { sys::spvc_constant_get_scalar_fp16(constant, column, row) };
        half::f16::from_f32(f32)
    }

    unsafe fn set(constant: spvc_constant, column: u32, row: u32, value: Self) {
        unsafe { sys::spvc_constant_set_scalar_fp16(constant, column, row, value.to_bits()) }
    }
}

/// A SPIR-V specialization constant
#[derive(Debug, Clone)]
pub struct SpecializationConstant {
    /// The handle to the constant.
    pub id: Handle<ConstantId>,
    /// The declared `constant_id` of the constant.
    pub constant_id: u32,
}

/// Specialization constants for a workgroup size.
#[derive(Debug, Clone)]
pub struct WorkgroupSizeSpecializationConstants {
    /// Workgroup size in _x_.
    pub x: Option<SpecializationConstant>,
    /// Workgroup size in _y_.
    pub y: Option<SpecializationConstant>,
    /// Workgroup size in _z_.
    pub z: Option<SpecializationConstant>,
    /// The constant ID of the builtin `WorkGroupSize`
    pub builtin_workgroup_size_handle: Option<Handle<ConstantId>>,
}

/// An iterator over specialization constants, created by [`Compiler::specialization_constants`].
pub struct SpecializationConstantIter<'a>(
    PhantomCompiler<'a>,
    slice::Iter<'a, spvc_specialization_constant>,
);

impl ExactSizeIterator for SpecializationConstantIter<'_> {
    fn len(&self) -> usize {
        self.1.len()
    }
}

impl Iterator for SpecializationConstantIter<'_> {
    type Item = SpecializationConstant;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|o| SpecializationConstant {
            id: self.0.create_handle(o.id),
            constant_id: o.constant_id,
        })
    }
}

/// Iterator for specialization subconstants created by
/// [`Compiler::specialization_sub_constants`].
pub struct SpecializationSubConstantIter<'a>(PhantomCompiler<'a>, slice::Iter<'a, ConstantId>);

impl ExactSizeIterator for SpecializationSubConstantIter<'_> {
    fn len(&self) -> usize {
        self.1.len()
    }
}

impl Iterator for SpecializationSubConstantIter<'_> {
    type Item = Handle<ConstantId>;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|o| self.0.create_handle(*o))
    }
}

/// Reflection of specialization constants.
impl<'ctx, T> Compiler<'ctx, T> {
    // check bounds of the constant, otherwise you can write to arbitrary memory.
    unsafe fn bounds_check_constant(
        handle: spvc_constant,
        column: u32,
        row: u32,
    ) -> error::Result<()> {
        // SPIRConstant is at most mat4, so anything above that is OOB.
        if column >= 4 || row >= 4 {
            return Err(SpirvCrossError::IndexOutOfBounds { row, column });
        }

        let vecsize = sys::spvc_rs_constant_get_vecsize(handle);
        let colsize = sys::spvc_rs_constant_get_matrix_colsize(handle);

        if column >= colsize || row >= vecsize {
            return Err(SpirvCrossError::IndexOutOfBounds { row, column });
        }

        Ok(())
    }

    /// Set the value of the specialization value at the given column and row.
    ///
    /// The type is inferred from the input, but it is not type checked against the SPIR-V.
    ///
    /// Using this function wrong is not unsafe, but could cause the output shader to
    /// be invalid.
    ///
    /// [`Compiler::set_specialization_constant_value`] is more efficient and easier to use in
    /// most cases, which will handle row and column for vector and matrix scalars. This function
    /// remains to deal with more esoteric matrix shapes, or for getting only a single
    /// element of a vector or matrix.
    pub fn set_specialization_constant_scalar<S: ConstantScalar>(
        &mut self,
        handle: Handle<ConstantId>,
        column: u32,
        row: u32,
        value: S,
    ) -> error::Result<()> {
        let constant = self.yield_id(handle)?;
        unsafe {
            // SAFETY: yield_id ensures safety.
            let handle = sys::spvc_compiler_get_constant_handle(self.ptr.as_ptr(), constant);
            Self::bounds_check_constant(handle, column, row)?;
            S::set(handle, column, row, value)
        }
        Ok(())
    }

    /// Get the value of the specialization value at the given column and row.
    ///
    /// The type is inferred from the return value, and is not type-checked
    /// against the input SPIR-V.
    ///
    /// If the inferred type differs from what is expected, an indeterminate
    /// but initialized value will be returned.
    ///
    /// [`Compiler::specialization_constant_value`] is more efficient and easier to use in
    /// most cases, which will handle row and column for vector and matrix scalars. This function
    /// remains to deal with more esoteric matrix shapes, or for getting only a single
    /// element of a vector or matrix.
    pub fn specialization_constant_scalar<S: ConstantScalar>(
        &self,
        handle: Handle<ConstantId>,
        column: u32,
        row: u32,
    ) -> error::Result<S> {
        let constant = self.yield_id(handle)?;
        unsafe {
            // SAFETY: yield_id ensures safety.
            let handle = sys::spvc_compiler_get_constant_handle(self.ptr.as_ptr(), constant);
            Self::bounds_check_constant(handle, column, row)?;

            Ok(S::get(handle, column, row))
        }
    }

    /// Query declared specialization constants.
    pub fn specialization_constants(&self) -> error::Result<SpecializationConstantIter<'ctx>> {
        unsafe {
            let mut constants = std::ptr::null();
            let mut size = 0;
            sys::spvc_compiler_get_specialization_constants(
                self.ptr.as_ptr(),
                &mut constants,
                &mut size,
            )
            .ok(self)?;

            // SAFETY: 'ctx is sound here.
            // https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross_c.cpp#L2522
            let slice = slice::from_raw_parts(constants, size);
            Ok(SpecializationConstantIter(self.phantom(), slice.iter()))
        }
    }

    /// Get subconstants for composite type specialization constants.
    pub fn specialization_sub_constants(
        &self,
        constant: Handle<ConstantId>,
    ) -> error::Result<SpecializationSubConstantIter> {
        let id = self.yield_id(constant)?;
        unsafe {
            let constant = sys::spvc_compiler_get_constant_handle(self.ptr.as_ptr(), id);
            let mut constants = std::ptr::null();
            let mut size = 0;
            sys::spvc_constant_get_subconstants(constant, &mut constants, &mut size);

            Ok(SpecializationSubConstantIter(
                self.phantom(),
                slice::from_raw_parts(constants, size).iter(),
            ))
        }
    }

    /// In SPIR-V, the compute work group size can be represented by a constant vector, in which case
    /// the LocalSize execution mode is ignored.
    ///
    /// This constant vector can be a constant vector, specialization constant vector, or partly specialized constant vector.
    /// To modify and query work group dimensions which are specialization constants, SPIRConstant values must be modified
    /// directly via get_constant() rather than using LocalSize directly. This function will return which constants should be modified.
    ///
    /// To modify dimensions which are *not* specialization constants, set_execution_mode should be used directly.
    /// Arguments to set_execution_mode which are specialization constants are effectively ignored during compilation.
    /// NOTE: This is somewhat different from how SPIR-V works. In SPIR-V, the constant vector will completely replace LocalSize,
    /// while in this interface, LocalSize is only ignored for specialization constants.
    ///
    /// The specialization constant will be written to x, y and z arguments.
    /// If the component is not a specialization constant, a zeroed out struct will be written.
    /// The return value is the constant ID of the builtin WorkGroupSize, but this is not expected to be useful
    /// for most use cases.
    ///
    /// If `LocalSizeId` is used, there is no uvec3 value representing the workgroup size, so the return value is 0,
    /// but _x_, _y_ and _z_ are written as normal if the components are specialization constants.
    pub fn work_group_size_specialization_constants(&self) -> WorkgroupSizeSpecializationConstants {
        unsafe {
            let mut x = MaybeUninit::zeroed();
            let mut y = MaybeUninit::zeroed();
            let mut z = MaybeUninit::zeroed();

            let constant = sys::spvc_compiler_get_work_group_size_specialization_constants(
                self.ptr.as_ptr(),
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                z.as_mut_ptr(),
            );

            let constant = self.create_handle_if_not_zero(constant);

            let x = x.assume_init();
            let y = y.assume_init();
            let z = z.assume_init();

            let x = self
                .create_handle_if_not_zero(x.id)
                .map(|id| SpecializationConstant {
                    id,
                    constant_id: x.constant_id,
                });

            let y = self
                .create_handle_if_not_zero(y.id)
                .map(|id| SpecializationConstant {
                    id,
                    constant_id: y.constant_id,
                });

            let z = self
                .create_handle_if_not_zero(z.id)
                .map(|id| SpecializationConstant {
                    id,
                    constant_id: z.constant_id,
                });

            WorkgroupSizeSpecializationConstants {
                x,
                y,
                z,
                builtin_workgroup_size_handle: constant,
            }
        }
    }

    /// Get the type of the specialization constant.
    pub fn specialization_constant_type(
        &self,
        constant: Handle<ConstantId>,
    ) -> error::Result<Handle<TypeId>> {
        let constant = self.yield_id(constant)?;
        let type_id = unsafe {
            // SAFETY: yield_id ensures this is valid for the ID
            let constant = sys::spvc_compiler_get_constant_handle(self.ptr.as_ptr(), constant);
            self.create_handle(sys::spvc_constant_get_type(constant))
        };

        Ok(type_id)
    }
}

/// A marker trait for types that can be represented as a SPIR-V constant.
pub trait ConstantValue: Sealed + Sized {
    // None of anything here is a public API.
    // As soon as generic_const_expr is stable, we can get rid of
    // almost all of this silliness.
    #[doc(hidden)]
    const COLUMNS: usize;
    #[doc(hidden)]
    const VECSIZE: usize;
    #[doc(hidden)]
    type BaseArrayType: Default + Index<usize, Output = Self::BaseType> + IndexMut<usize>;
    #[doc(hidden)]
    type ArrayType: Default + Index<usize, Output = Self::BaseArrayType> + IndexMut<usize>;
    #[doc(hidden)]
    type BaseType: ConstantScalar;

    #[doc(hidden)]
    fn from_array(value: Self::ArrayType) -> Self;

    #[doc(hidden)]
    fn to_array(value: Self) -> Self::ArrayType;
}

impl<T: ConstantScalar> ConstantValue for T {
    const COLUMNS: usize = 1;
    const VECSIZE: usize = 1;
    type BaseArrayType = [T; 1];
    type ArrayType = [[T; 1]; 1];
    type BaseType = T;

    fn from_array(value: Self::ArrayType) -> Self {
        value[0][0]
    }

    fn to_array(value: Self) -> Self::ArrayType {
        [[value]]
    }
}

#[cfg(feature = "gfx-math-types")]
#[cfg_attr(docsrs, doc(cfg(feature = "gfx-math-types")))]
mod gfx_maths_types {
    use crate::reflect::ConstantValue;
    use crate::sealed::Sealed;
    use gfx_maths::{Mat4, Vec2, Vec3, Vec4};

    impl Sealed for Vec2 {}
    impl ConstantValue for Vec2 {
        const COLUMNS: usize = 1;
        const VECSIZE: usize = 2;
        type BaseArrayType = [f32; 2];
        type ArrayType = [[f32; 2]; 1];
        type BaseType = f32;

        fn from_array(value: Self::ArrayType) -> Self {
            value[0].into()
        }

        fn to_array(value: Self) -> Self::ArrayType {
            [[value.x, value.y]]
        }
    }

    impl Sealed for Vec3 {}
    impl ConstantValue for Vec3 {
        const COLUMNS: usize = 1;
        const VECSIZE: usize = 3;
        type BaseArrayType = [f32; 3];
        type ArrayType = [[f32; 3]; 1];
        type BaseType = f32;

        fn from_array(value: Self::ArrayType) -> Self {
            value[0].into()
        }

        fn to_array(value: Self) -> Self::ArrayType {
            [[value.x, value.y, value.z]]
        }
    }

    impl Sealed for Vec4 {}
    impl ConstantValue for Vec4 {
        const COLUMNS: usize = 1;
        const VECSIZE: usize = 4;
        type BaseArrayType = [f32; 4];
        type ArrayType = [[f32; 4]; 1];
        type BaseType = f32;

        fn from_array(value: Self::ArrayType) -> Self {
            value[0].into()
        }

        fn to_array(value: Self) -> Self::ArrayType {
            [[value.x, value.y, value.z, value.w]]
        }
    }

    impl Sealed for Mat4 {}
    impl ConstantValue for Mat4 {
        const COLUMNS: usize = 4;
        const VECSIZE: usize = 4;
        type BaseArrayType = [f32; 4];
        type ArrayType = [[f32; 4]; 4];
        type BaseType = f32;

        fn from_array(value: Self::ArrayType) -> Self {
            value.into()
        }

        fn to_array(value: Self) -> Self::ArrayType {
            let mut array = [[0f32; 4]; 4];
            // gfx-math uses
            // so we assign it back in the same order.

            // const fn cr(c: usize, r: usize) -> usize {
            //     r + c * 4
            // }
            array[0][0] = value[(0, 0)];
            array[0][1] = value[(0, 1)];
            array[0][2] = value[(0, 2)];
            array[0][3] = value[(0, 3)];

            array[1][0] = value[(1, 0)];
            array[1][1] = value[(1, 1)];
            array[1][2] = value[(1, 2)];
            array[1][3] = value[(1, 3)];

            array[2][0] = value[(2, 0)];
            array[2][1] = value[(2, 1)];
            array[2][2] = value[(2, 2)];
            array[2][3] = value[(2, 3)];

            array[3][0] = value[(3, 0)];
            array[3][1] = value[(3, 1)];
            array[3][2] = value[(3, 2)];
            array[3][3] = value[(3, 3)];

            array
        }
    }
}

impl<'a, T> Compiler<'a, T> {
    /// Get the value of the specialization value.
    ///
    /// The type is inferred from the return value, and is not type-checked
    /// against the input SPIR-V.
    ///
    /// If the output type dimensions are too large for the constant,
    /// [`SpirvCrossError::IndexOutOfBounds`] will be returned.
    ///
    /// If the inferred type differs from what is expected, an indeterminate
    /// but initialized value will be returned.
    pub fn specialization_constant_value<S: ConstantValue>(
        &self,
        handle: Handle<ConstantId>,
    ) -> error::Result<S> {
        let constant = self.yield_id(handle)?;
        unsafe {
            // SAFETY: yield_id ensures safety.
            let handle = sys::spvc_compiler_get_constant_handle(self.ptr.as_ptr(), constant);
            // Self::bounds_check_constant(handle, column, row)?;
            let mut output = S::ArrayType::default();

            // bounds check the limits of the type.
            Self::bounds_check_constant(handle, S::COLUMNS as u32 - 1, S::VECSIZE as u32 - 1)?;

            for column in 0..S::COLUMNS {
                for row in 0..S::VECSIZE {
                    let value = S::BaseType::get(handle, column as u32, row as u32);
                    output[column][row] = value;
                }
            }
            Ok(S::from_array(output))
        }
    }

    /// Set the value of the specialization value.
    ///
    /// The type is inferred from the input, but it is not type checked against the SPIR-V.
    ///
    /// Using this function wrong is not unsafe, but could cause the output shader to
    /// be invalid.
    ///
    /// If the input dimensions are too large for the constant type,
    /// [`SpirvCrossError::IndexOutOfBounds`] will be returned.
    pub fn set_specialization_constant_value<S: ConstantValue>(
        &mut self,
        handle: Handle<ConstantId>,
        value: S,
    ) -> error::Result<()> {
        let constant = self.yield_id(handle)?;
        unsafe {
            // SAFETY: yield_id ensures safety.
            let handle = sys::spvc_compiler_get_constant_handle(self.ptr.as_ptr(), constant);

            // bounds check the limits of the type.
            Self::bounds_check_constant(handle, S::COLUMNS as u32 - 1, S::VECSIZE as u32 - 1)?;

            let value = S::to_array(value);
            for column in 0..S::COLUMNS {
                for row in 0..S::VECSIZE {
                    S::BaseType::set(handle, column as u32, row as u32, value[column][row]);
                }
            }
        }
        Ok(())
    }
}
