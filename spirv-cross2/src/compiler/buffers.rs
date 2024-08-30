use crate::compiler::Compiler;

use crate::error;
use crate::handle::Handle;
use spirv_cross_sys::VariableId;

use spirv_cross_sys as sys;

use crate::error::ToContextError;
/// A range over a buffer.
pub use spirv_cross_sys::BufferRange;

/// Reflection of buffers (UBO, SSBOs, and PushConstant blocks).
impl<'a, T> Compiler<'a, T> {
    /// Returns a list of which members of a struct are potentially in use by a
    /// SPIR-V shader. The granularity of this analysis is per-member of a struct.
    /// This can be used for Buffer (UBO), BufferBlock/StorageBuffer (SSBO) and PushConstant blocks.
    pub fn active_buffer_ranges(
        &self,
        handle: Handle<VariableId>,
    ) -> error::Result<&'a [BufferRange]> {
        let handle = self.yield_id(handle)?;

        unsafe {
            let mut ranges = std::ptr::null();
            let mut size = 0;
            sys::spvc_compiler_get_active_buffer_ranges(
                self.0.as_ptr(),
                handle,
                &mut ranges,
                &mut size,
            )
            .ok(self)?;

            Ok(std::slice::from_raw_parts(ranges, size))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::{targets, Compiler};
    use crate::error::SpirvCrossError;
    use crate::{Module, SpirvCross};
    use spirv_cross_sys::ResourceType;
    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn get_active_buffer_ranges() -> Result<(), SpirvCrossError> {
        let mut spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let mut compiler: Compiler<targets::None> = spv.create_compiler(words)?;
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
