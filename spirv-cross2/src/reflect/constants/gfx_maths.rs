#![cfg(feature = "gfx-math-types")]
#![cfg_attr(docsrs, doc(cfg(feature = "gfx-math-types")))]
use crate::reflect::ConstantValue;
use crate::sealed::Sealed;
use gfx_maths::{Mat4, Vec2, Vec3, Vec4};
use crate::reflect::constants::impl_vec_constant;

impl_vec_constant!(Vec2 [f32; 2] for [x, y]);
impl_vec_constant!(Vec3 [f32; 3] for [x, y, z]);
impl_vec_constant!(Vec4 [f32; 4] for [x, y, z, w]);

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

#[cfg(test)]
mod test {
    use crate::reflect::ConstantValue;

    #[test]
    pub fn round_trip_mat4() {
        let mat4 = gfx_maths::Mat4::inverse_orthographic_opengl(1.0, 2.0, 3.0, 4.0,5.0, 6.0);
        let arr = ConstantValue::to_array(mat4.clone());
        let returned = ConstantValue::from_array(arr);

        assert_eq!(mat4, returned);
    }
}