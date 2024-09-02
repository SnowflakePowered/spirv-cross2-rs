use crate::error;
use crate::error::ToContextError;
use crate::handle::Handle;
use crate::Compiler;
use spirv_cross_sys as sys;
use spirv_cross_sys::{SpvId, VariableId};

/// A range over a buffer.
pub use spirv_cross_sys::BufferRange;

/// Reflection of buffers (UBO, SSBOs, and PushConstant blocks).
impl<'a, T> Compiler<'a, T> {
    /// Returns a list of which members of a struct are potentially in use by a
    /// SPIR-V shader. The granularity of this analysis is per-member of a struct.
    /// This can be used for Buffer (UBO), BufferBlock/StorageBuffer (SSBO) and PushConstant blocks.
    pub fn active_buffer_ranges(
        &self,
        handle: impl Into<Handle<VariableId>>,
    ) -> error::Result<&'a [BufferRange]> {
        let handle = handle.into();
        let handle = self.yield_id(handle)?;

        unsafe {
            let mut ranges = std::ptr::null();
            let mut size = 0;
            sys::spvc_compiler_get_active_buffer_ranges(
                self.ptr.as_ptr(),
                handle,
                &mut ranges,
                &mut size,
            )
            .ok(self)?;

            Ok(std::slice::from_raw_parts(ranges, size))
        }
    }

    /// Queries if a buffer object has a neighbor "counter" buffer.
    /// If so, the ID of that counter buffer will be returned.
    ///
    /// If SPV_GOOGLE_hlsl_functionality1 is used, this can be used even with a stripped SPIR-V module.
    /// Otherwise, this query is purely based on OpName identifiers as found in the SPIR-V module, and will
    /// only return true if OpSource was reported HLSL.
    /// To rely on this functionality, ensure that the SPIR-V module is not stripped.
    pub fn hlsl_counter_buffer(
        &self,
        variable: impl Into<Handle<VariableId>>,
    ) -> error::Result<Option<Handle<VariableId>>> {
        let variable = variable.into();
        let id = self.yield_id(variable)?;
        unsafe {
            let mut counter = VariableId(SpvId(0));
            if sys::spvc_compiler_buffer_get_hlsl_counter_buffer(
                self.ptr.as_ptr(),
                id,
                &mut counter,
            ) {
                Ok(Some(self.create_handle(counter)))
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::SpirvCrossError;
    use crate::targets;
    use crate::Compiler;
    use crate::{Module, SpirvCrossContext};
    use spirv_cross_sys::ResourceType;

    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn get_active_buffer_ranges() -> Result<(), SpirvCrossError> {
        let spv = SpirvCrossContext::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let ubo: Vec<_> = compiler
            .shader_resources()?
            .resources_for_type(ResourceType::UniformBuffer)?
            .collect();

        let ubo = ubo[0].id;
        let ranges = compiler.active_buffer_ranges(ubo)?;

        eprintln!("{:?}", ranges);
        Ok(())
    }
}
