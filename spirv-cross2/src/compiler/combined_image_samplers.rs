// todo:
// SPVC_PUBLIC_API spvc_result spvc_compiler_build_dummy_sampler_for_combined_images(spvc_compiler compiler, spvc_variable_id *id);
// SPVC_PUBLIC_API spvc_result spvc_compiler_build_combined_image_samplers(spvc_compiler compiler);
// SPVC_PUBLIC_API spvc_result spvc_compiler_get_combined_image_samplers(spvc_compiler compiler,
// const spvc_combined_image_sampler **samplers,
// size_t *num_samplers);

use crate::compiler::{Compiler, PhantomCompiler};
use crate::error;
use crate::error::{SpirvCrossError, ToContextError};
pub use crate::handle::{Handle, VariableId};
use spirv_cross_sys as sys;
use std::slice;

#[derive(Debug, Copy, Clone)]
pub struct BuiltDummySamplerProof {
    pub sampler_id: Option<Handle<VariableId>>,
    label: Handle<()>,
}

/// Iterator for [`CombinedImageSampler`].
pub struct CombinedImageSamplerIter<'a>(
    PhantomCompiler<'a>,
    slice::Iter<'a, sys::spvc_combined_image_sampler>,
);

impl Iterator for CombinedImageSamplerIter<'_> {
    type Item = CombinedImageSampler;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|c| {
            let combined_id = self.0.create_handle(c.combined_id);
            let image_id = self.0.create_handle(c.image_id);
            let sampler_id = self.0.create_handle(c.sampler_id);

            CombinedImageSampler {
                combined_id,
                image_id,
                sampler_id,
            }
        })
    }
}

/// A combined image sampler.
pub struct CombinedImageSampler {
    pub combined_id: Handle<VariableId>,
    pub image_id: Handle<VariableId>,
    pub sampler_id: Handle<VariableId>,
}

impl<'a, T> Compiler<'a, T> {
    /// Analyzes all OpImageFetch (texelFetch) opcodes and checks if there are instances where
    /// said instruction is used without a combined image sampler.
    /// GLSL targets do not support the use of texelFetch without a sampler.
    /// To work around this, we must inject a dummy sampler which can be used to form a sampler2D at the call-site of
    /// texelFetch as necessary.
    ///
    /// This must be called to obtain a proof to call [`Compiler::build_combined_image_samplers`].
    ///
    /// The proof contains the ID of a sampler object, if one dummy sampler is necessary. This ID can
    /// be decorated with set/bindings as desired before compiling.
    pub fn create_dummy_sampler_for_combined_images(
        &mut self,
    ) -> error::Result<BuiltDummySamplerProof> {
        unsafe {
            let mut var_id = VariableId::from(0);
            sys::spvc_compiler_build_dummy_sampler_for_combined_images(
                self.ptr.as_ptr(),
                &mut var_id,
            )
            .ok(&*self)?;

            let sampler_id = if var_id.0 .0 == 0 {
                None
            } else {
                Some(self.create_handle(var_id))
            };

            Ok(BuiltDummySamplerProof {
                sampler_id,
                label: self.create_handle(()),
            })
        }
    }

    /// Analyzes all separate image and samplers used from the currently selected entry point,
    /// and re-routes them all to a combined image sampler instead.
    /// This is required to "support" separate image samplers in targets which do not natively support
    /// this feature, like GLSL/ESSL.
    ///
    /// This call will add new sampled images to the SPIR-V,
    /// so it will appear in reflection if [`Compiler::shader_resources`] is called after.
    ///
    /// If any image/sampler remapping was found, no separate image/samplers will appear in the decompiled output,
    /// but will still appear in reflection.
    ///
    /// The resulting samplers will be void of any decorations like name, descriptor sets and binding points,
    /// so this can be added before compilation if desired.
    ///
    /// Combined image samplers originating from this set are always considered active variables.
    /// Arrays of separate samplers are not supported, but arrays of separate images are supported.
    /// Array of images + sampler -> Array of combined image samplers.
    ///
    /// [`Compiler::create_dummy_sampler_for_combined_images`] must be called before this to obtain
    /// a proof that a dummy sampler, if necessary, was created. Passing in a smuggled proof from
    /// a different compiler instance will result in an error.
    pub fn build_combined_image_samplers(
        &mut self,
        proof: BuiltDummySamplerProof,
    ) -> error::Result<()> {
        // check for smuggling
        if !self.handle_is_valid(&proof.label) {
            return Err(SpirvCrossError::InvalidOperation(String::from(
                "The provided proof of ",
            )));
        }

        unsafe {
            sys::spvc_compiler_build_combined_image_samplers(self.ptr.as_ptr()).ok(self)?;

            Ok(())
        }
    }

    /// Gets a remapping for the combined image samplers.
    pub fn combined_image_samplers(&self) -> error::Result<CombinedImageSamplerIter> {
        unsafe {
            let mut samplers = std::ptr::null();
            let mut size = 0;
            sys::spvc_compiler_get_combined_image_samplers(
                self.ptr.as_ptr(),
                &mut samplers,
                &mut size,
            )
            .ok(self)?;
            let slice = slice::from_raw_parts(samplers, size);
            Ok(CombinedImageSamplerIter(self.phantom(), slice.into_iter()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::Compiler;
    use crate::error::SpirvCrossError;
    use crate::{targets, Module, SpirvCross};

    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn test_combined_image_sampler_build() -> Result<(), SpirvCrossError> {
        let mut spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let mut compiler: Compiler<targets::None> = spv.create_compiler(words)?;

        let proof = compiler.create_dummy_sampler_for_combined_images()?;
        compiler.build_combined_image_samplers(proof)?;

        // match ty.inner {
        //     TypeInner::Struct(ty) => {
        //         compiler.get_type(ty.members[0].id)?;
        //     }
        //     TypeInner::Vector { .. } => {}
        //     _ => {}
        // }
        Ok(())
    }
}
