use crate::sealed::Sealed;
use spirv_cross_sys::{spvc_constant, spvc_specialization_constant, SpvId, TypeId};
use std::mem::MaybeUninit;
use std::slice;

use crate::compiler::{Compiler, PhantomCompiler};
use crate::error;
use crate::error::ToContextError;
use crate::handle::{ConstantId, Handle};
use spirv_cross_sys as sys;

pub trait Scalar: Sealed {
    #[doc(hidden)]
    unsafe fn get(constant: spvc_constant, column: u32, row: u32) -> Self;

    #[doc(hidden)]
    unsafe fn set(constant: spvc_constant, column: u32, row: u32, value: Self);
}

macro_rules! impl_spvc_constant {
    ($get:ident  $set:ident $prim:ty) => {
        impl Sealed for $prim {}
        impl Scalar for $prim {
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
impl Scalar for half::f16 {
    unsafe fn get(constant: spvc_constant, column: u32, row: u32) -> Self {
        let f32 = unsafe { sys::spvc_constant_get_scalar_fp16(constant, column, row) };
        half::f16::from_f32(f32)
    }

    unsafe fn set(constant: spvc_constant, column: u32, row: u32, value: Self) {
        unsafe { sys::spvc_constant_set_scalar_fp16(constant, column, row, value.to_bits()) }
    }
}

#[derive(Debug, Clone)]
pub struct SpecializationConstant {
    pub id: Handle<ConstantId>,
    pub constant_id: u32,
}

#[derive(Debug, Clone)]
pub struct WorkgroupSizeSpecializationConstants {
    pub x: Option<SpecializationConstant>,
    pub y: Option<SpecializationConstant>,
    pub z: Option<SpecializationConstant>,
    pub builtin_workgroup_size_handle: Option<Handle<ConstantId>>,
}

/// An iterator over specialization constants.
pub struct SpecializationConstantIter<'a>(
    PhantomCompiler<'a>,
    slice::Iter<'a, spvc_specialization_constant>,
);

impl Iterator for SpecializationConstantIter<'_> {
    type Item = SpecializationConstant;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|o| SpecializationConstant {
            id: self.0.create_handle(o.id),
            constant_id: o.constant_id,
        })
    }
}

/// Reflection of specialization constants.
impl<'a, T> Compiler<'a, T> {
    pub fn set_specialization_constant_value<S: Scalar>(
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
            S::set(handle, column, row, value)
        }
        Ok(())
    }

    pub fn specialization_constant_value<S: Scalar>(
        &self,
        handle: Handle<ConstantId>,
        column: u32,
        row: u32,
    ) -> error::Result<S> {
        let constant = self.yield_id(handle)?;
        unsafe {
            // SAFETY: yield_id ensures safety.
            let handle = sys::spvc_compiler_get_constant_handle(self.ptr.as_ptr(), constant);

            
            Ok(S::get(handle, column, row))
        }
    }

    pub fn specialization_constants(&self) -> error::Result<SpecializationConstantIter<'a>> {
        unsafe {
            let mut constants = std::ptr::null();
            let mut size = 0;
            sys::spvc_compiler_get_specialization_constants(
                self.ptr.as_ptr(),
                &mut constants,
                &mut size,
            )
            .ok(self)?;

            let slice = slice::from_raw_parts(constants, size);
            Ok(SpecializationConstantIter(
                self.phantom(),
                slice.into_iter(),
            ))
        }
    }

    pub fn specialization_sub_constants(
        &self,
        constant: Handle<ConstantId>,
    ) -> error::Result<Vec<Handle<ConstantId>>> {
        let id = self.yield_id(constant)?;
        unsafe {
            let constant = sys::spvc_compiler_get_constant_handle(self.ptr.as_ptr(), id);
            let mut constants = std::ptr::null();
            let mut size = 0;
            sys::spvc_constant_get_subconstants(constant, &mut constants, &mut size);

            Ok(slice::from_raw_parts(constants, size)
                .iter()
                .map(|id| self.create_handle(*id))
                .collect())
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
    /// If LocalSizeId is used, there is no uvec3 value representing the workgroup size, so the return value is 0,
    /// but x, y and z are written as normal if the components are specialization constants.
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

            let x =  self.create_handle_if_not_zero(x.id).map(|id|
                SpecializationConstant {
                    id,
                    constant_id: x.constant_id
                }
            );

            let y =  self.create_handle_if_not_zero(y.id).map(|id|
                SpecializationConstant {
                    id,
                    constant_id: y.constant_id
                }
            );

            let z =  self.create_handle_if_not_zero(z.id).map(|id|
                SpecializationConstant {
                    id,
                    constant_id: z.constant_id
                }
            );

            WorkgroupSizeSpecializationConstants {
                x,
                y,
                z,
                builtin_workgroup_size_handle: constant,
            }
        }
    }

    /// Get the type of the specialization constant.
    pub fn specialization_constant_type(&self, constant: Handle<ConstantId>) -> error::Result<Handle<TypeId>> {
        let constant = self.yield_id(constant)?;
        let type_id = unsafe {
            // SAFETY: yield_id ensures this is valid for the ID
            let constant = sys::spvc_compiler_get_constant_handle(self.ptr.as_ptr(), constant);
            self.create_handle(sys::spvc_constant_get_type(constant))
        };

        Ok(type_id)
    }
}
