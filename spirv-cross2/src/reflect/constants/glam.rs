#![cfg(feature = "glam-types")]
#![cfg_attr(docsrs, doc(cfg(feature = "glam-types")))]
use glam::*;
use crate::reflect::constants::impl_vec_constant;

impl_vec_constant!(Vec2 [f32; 2] for [x, y]);
impl_vec_constant!(Vec3 [f32; 3] for [x, y, z]);
impl_vec_constant!(Vec3A [f32; 3] for [x, y, z]);

impl_vec_constant!(Vec4 [f32; 4] for [x, y, z, w]);

impl_vec_constant!(DVec2 [f64; 2] for [x, y]);
impl_vec_constant!(DVec3 [f64; 3] for [x, y, z]);
impl_vec_constant!(DVec4 [f64; 4] for [x, y, z, w]);

impl_vec_constant!(IVec2 [i32; 2] for [x, y]);
impl_vec_constant!(IVec3 [i32; 3] for [x, y, z]);
impl_vec_constant!(IVec4 [i32; 4] for [x, y, z, w]);

impl_vec_constant!(BVec2 [bool; 2] for [x, y]);
impl_vec_constant!(BVec3 [bool; 3] for [x, y, z]);
impl_vec_constant!(BVec4 [bool; 4] for [x, y, z, w]);

impl_vec_constant!(UVec2 [u32; 2] for [x, y]);
impl_vec_constant!(UVec3 [u32; 3] for [x, y, z]);
impl_vec_constant!(UVec4 [u32; 4] for [x, y, z, w]);

impl_vec_constant!(I16Vec2 [i16; 2] for [x, y]);
impl_vec_constant!(I16Vec3 [i16; 3] for [x, y, z]);
impl_vec_constant!(I16Vec4 [i16; 4] for [x, y, z, w]);

impl_vec_constant!(U16Vec2 [u16; 2] for [x, y]);
impl_vec_constant!(U16Vec3 [u16; 3] for [x, y, z]);
impl_vec_constant!(U16Vec4 [u16; 4] for [x, y, z, w]);

impl_vec_constant!(I64Vec2 [i64; 2] for [x, y]);
impl_vec_constant!(I64Vec3 [i64; 3] for [x, y, z]);
impl_vec_constant!(I64Vec4 [i64; 4] for [x, y, z, w]);

impl_vec_constant!(U64Vec2 [u64; 2] for [x, y]);
impl_vec_constant!(U64Vec3 [u64; 3] for [x, y, z]);
impl_vec_constant!(U64Vec4 [u64; 4] for [x, y, z, w]);

macro_rules! impl_mat_constant {
     ($mat_ty:ty [$base_ty:ty; $len:literal] for [$vec_ty:ty; $($component:literal),*])  => {
         impl $crate::sealed::Sealed for $mat_ty {}
         impl $crate::reflect::ConstantValue for $mat_ty {
             const COLUMNS: usize = $len;
             const VECSIZE: usize = $len;
             type BaseArrayType = [$base_ty; $len];
             type ArrayType = [[$base_ty; $len]; $len];
             type BaseType = $base_ty;

             fn to_array(value: Self) -> Self::ArrayType {
                 value.to_cols_array_2d()
             }

             fn from_array(value: Self::ArrayType) -> Self {
                <$mat_ty>::from_cols(
                    $(<$vec_ty>::from_array(value[$component])),*
                )
             }
         }
     };
}

impl_mat_constant!(Mat4 [f32; 4] for [Vec4; 0, 1, 2, 3]);
impl_mat_constant!(Mat3 [f32; 3] for [Vec3; 0, 1, 2]);
impl_mat_constant!(Mat3A [f32; 3] for [Vec3A; 0, 1, 2]);
impl_mat_constant!(Mat2 [f32; 2] for [Vec2; 0, 1]);

impl_mat_constant!(DMat4 [f64; 4] for [DVec4; 0, 1, 2, 3]);
impl_mat_constant!(DMat3 [f64; 3] for [DVec3; 0, 1, 2]);
impl_mat_constant!(DMat2 [f64; 2] for [DVec2; 0, 1]);

#[cfg(test)]
mod test {
    use crate::reflect::ConstantValue;

    #[test]
    pub fn round_trip_mat4() {
        let mat4 = glam::Mat4::orthographic_lh(1.0, 2.0, 3.0, 4.0,5.0, 6.0);
        let arr = ConstantValue::to_array(mat4.clone());
        let returned = ConstantValue::from_array(arr);

        assert_eq!(mat4, returned);
    }

    #[test]
    pub fn round_trip_mat3() {
        let mat4 = glam::Mat4::orthographic_lh(1.0, 2.0, 3.0, 4.0,5.0, 6.0);
        let mat3 = glam::Mat3::from_mat4_minor(mat4, 1, 2);
        let arr = ConstantValue::to_array(mat3.clone());
        let returned = ConstantValue::from_array(arr);

        assert_eq!(mat3, returned);
    }

    #[test]
    pub fn round_trip_mat2() {
        let mat4 = glam::Mat4::orthographic_lh(1.0, 2.0, 3.0, 4.0,5.0, 6.0);
        let mat3 = glam::Mat3::from_mat4_minor(mat4, 1, 2);
        let mat2 = glam::Mat2::from_mat3_minor(mat3, 1, 2);
        let arr = ConstantValue::to_array(mat2.clone());
        let returned = ConstantValue::from_array(arr);

        assert_eq!(mat2, returned);
    }
}