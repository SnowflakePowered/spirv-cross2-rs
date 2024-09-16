#![cfg(feature = "gfx-math-types")]
#![cfg_attr(docsrs, doc(cfg(feature = "gfx-math-types")))]
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
