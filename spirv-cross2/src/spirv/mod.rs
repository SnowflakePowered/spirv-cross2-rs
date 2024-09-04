/// [BuiltIn](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#_builtin)
pub use spirv_cross_sys::SpvBuiltIn as BuiltIn;
/// [Capability](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#_capability)
pub use spirv_cross_sys::SpvCapability as Capability;
/// [Decoration](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#_decoration)
pub use spirv_cross_sys::SpvDecoration as Decoration;
/// [Dim](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#_dim)
pub use spirv_cross_sys::SpvDim as Dim;
/// [Execution Mode](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#_execution_mode)
pub use spirv_cross_sys::SpvExecutionMode as ExecutionMode;
/// [Execution Model](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#_execution_model) (i.e. shader stage)
pub use spirv_cross_sys::SpvExecutionModel as ExecutionModel;
/// [FP Rounding Mode](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#_fp_rounding_mode)
pub use spirv_cross_sys::SpvFPRoundingMode as FPRoundingMode;
/// [Image Format](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#_image_format)
pub use spirv_cross_sys::SpvImageFormat as ImageFormat;
/// [Storage Class](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#_storage_class)
pub use spirv_cross_sys::SpvStorageClass as StorageClass;

// The rest of the exports are unused.

// SAFETY:
// FPRoundingModeMax shows up possibly in `Decoration` but we `FromPrimitive::try_from` so we're OK.
//
// The real hazards are `BuiltIn` and `ExecutionModel` which appear in `HLSLResourceBinding`, ` MSLResourceBinding`,
// `MSLShaderInterfaceVar`, and possibly in `spvc_entry_point`.
//
// We need to guard against uninit values there.
