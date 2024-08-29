pub use spirv_cross_sys::SpirvCrossError;

pub type Result<T> = std::result::Result<T, SpirvCrossError>;
