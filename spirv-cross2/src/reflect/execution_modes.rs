use crate::error::ToContextError;
use crate::handle::Handle;
use crate::reflect::try_valid_slice;
use crate::Compiler;
use crate::{error, spirv};
use spirv_cross_sys as sys;
use spirv_cross_sys::ConstantId;

/// Arguments to an `OpExecutionMode`.
#[derive(Debug)]
pub enum ExecutionModeArguments {
    /// No arguments.
    ///
    /// This is also used to set execution modes for modes that don't have arguments.
    None,
    /// A single literal argument.
    Literal(u32),
    /// Arguments to `LocalSize` execution mode.
    LocalSize {
        /// Workgroup size x.
        x: u32,
        /// Workgroup size y.
        y: u32,
        /// Workgroup size z.
        z: u32,
    },
    /// Arguments to `LocalSizeId` execution mode.
    LocalSizeId {
        /// Workgroup size x ID.
        x: Handle<ConstantId>,
        /// Workgroup size y ID.
        y: Handle<ConstantId>,
        /// Workgroup size z ID.
        z: Handle<ConstantId>,
    },
}

impl ExecutionModeArguments {
    fn expand(self) -> [u32; 3] {
        match self {
            ExecutionModeArguments::None => [0, 0, 0],
            ExecutionModeArguments::Literal(a) => [a, 0, 0],
            ExecutionModeArguments::LocalSize { x, y, z } => [x, y, z],
            ExecutionModeArguments::LocalSizeId { x, y, z } => [x.id(), y.id(), z.id()],
        }
    }
}

impl<'ctx, T> Compiler<'ctx, T> {
    /// Set or unset execution modes and arguments.
    ///
    /// If arguments is `None`, unsets the execution mode. To set an execution mode that does not
    /// take arguments, pass `Some(ExecutionModeArguments::None)`.
    pub fn set_execution_mode(
        &mut self,
        mode: spirv::ExecutionMode,
        arguments: Option<ExecutionModeArguments>,
    ) {
        unsafe {
            let Some(arguments) = arguments else {
                return sys::spvc_compiler_unset_execution_mode(self.ptr.as_ptr(), mode);
            };

            let [x, y, z] = arguments.expand();

            sys::spvc_compiler_set_execution_mode_with_arguments(self.ptr.as_ptr(), mode, x, y, z);
        }
    }

    /// Query `OpExecutionMode`.
    pub fn execution_modes(&self) -> error::Result<&'ctx [spirv::ExecutionMode]> {
        unsafe {
            let mut size = 0;
            let mut modes = std::ptr::null();

            sys::spvc_compiler_get_execution_modes(self.ptr.as_ptr(), &mut modes, &mut size)
                .ok(self)?;

            // SAFETY: 'ctx is sound here.
            // https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross_c.cpp#L2250

            const _: () = assert!(size_of::<spirv::ExecutionMode>() == size_of::<u32>());
            try_valid_slice(modes, size)
        }
    }

    /// Get arguments used by the execution mode.
    ///
    /// If the execution mode is unused, returns `None`.
    ///
    /// LocalSizeId query returns an ID. If LocalSizeId execution mode is not used, it returns None.
    /// LocalSize always returns a literal. If execution mode is LocalSizeId, the literal (spec constant or not) is still returned.
    pub fn execution_mode_arguments(
        &self,
        mode: spirv::ExecutionMode,
    ) -> error::Result<Option<ExecutionModeArguments>> {
        Ok(match mode {
            spirv::ExecutionMode::LocalSize => unsafe {
                let x = sys::spvc_compiler_get_execution_mode_argument_by_index(
                    self.ptr.as_ptr(),
                    mode,
                    0,
                );
                let y = sys::spvc_compiler_get_execution_mode_argument_by_index(
                    self.ptr.as_ptr(),
                    mode,
                    1,
                );
                let z = sys::spvc_compiler_get_execution_mode_argument_by_index(
                    self.ptr.as_ptr(),
                    mode,
                    2,
                );

                if x * y * z == 0 {
                    None
                } else {
                    Some(ExecutionModeArguments::LocalSize { x, y, z })
                }
            },
            spirv::ExecutionMode::LocalSizeId => unsafe {
                let x = sys::spvc_compiler_get_execution_mode_argument_by_index(
                    self.ptr.as_ptr(),
                    mode,
                    0,
                );
                let y = sys::spvc_compiler_get_execution_mode_argument_by_index(
                    self.ptr.as_ptr(),
                    mode,
                    1,
                );
                let z = sys::spvc_compiler_get_execution_mode_argument_by_index(
                    self.ptr.as_ptr(),
                    mode,
                    2,
                );

                if x * y * z == 0 {
                    // If one is zero, then all are zero.
                    None
                } else {
                    Some(ExecutionModeArguments::LocalSizeId {
                        x: self.create_handle(ConstantId::from(x)),
                        y: self.create_handle(ConstantId::from(y)),
                        z: self.create_handle(ConstantId::from(z)),
                    })
                }
            },
            spirv::ExecutionMode::Invocations
            | spirv::ExecutionMode::OutputVertices
            | spirv::ExecutionMode::OutputPrimitivesEXT => unsafe {
                if !self.execution_modes()?.contains(&mode) {
                    return Ok(None);
                };

                let x = sys::spvc_compiler_get_execution_mode_argument_by_index(
                    self.ptr.as_ptr(),
                    mode,
                    0,
                );
                Some(ExecutionModeArguments::Literal(x))
            },
            _ => {
                if !self.execution_modes()?.contains(&mode) {
                    return Ok(None);
                };

                Some(ExecutionModeArguments::None)
            }
        })
    }
}

#[cfg(test)]
mod test {
    use crate::error::SpirvCrossError;
    use crate::Compiler;
    use crate::{spirv, targets, Module, SpirvCrossContext};

    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn execution_modes() -> Result<(), SpirvCrossError> {
        let spv = SpirvCrossContext::new()?;
        let vec = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&vec));

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        let ty = compiler.execution_modes()?;
        assert_eq!([spirv::ExecutionMode::OriginUpperLeft], ty);

        Ok(())
    }
}
