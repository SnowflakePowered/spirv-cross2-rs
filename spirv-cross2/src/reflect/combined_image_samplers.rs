use crate::error::{SpirvCrossError, ToContextError};
use crate::handle::{Handle, PhantomCompiler, VariableId};
use crate::{error, Compiler};
use spirv_cross_sys as sys;
use std::slice;
use crate::sync::WithContext;

/// A proof that [`Compiler::create_dummy_sampler_for_combined_images`] was called.
#[derive(Debug, Copy, Clone)]
pub struct BuiltDummySamplerProof {
    /// The handle to a sampler object, if one was needed to be created.
    pub sampler_id: Option<Handle<VariableId>>,
    label: Handle<()>,
}

/// Iterator for [`CombinedImageSampler`].
pub struct CombinedImageSamplerIter<'a>(
    PhantomCompiler<'a>,
    slice::Iter<'a, sys::spvc_combined_image_sampler>,
);

impl ExactSizeIterator for CombinedImageSamplerIter<'_> {
    fn len(&self) -> usize {
        self.1.len()
    }
}

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
    /// A handle to the created combined image sampler.
    pub combined_id: Handle<VariableId>,
    /// A handle to the split image of the combined image sampler.
    pub image_id: Handle<VariableId>,
    /// A handle to the split sampler of the combined image sampler.
    pub sampler_id: Handle<VariableId>,
}

impl<'ctx, T, L: WithContext> Compiler<'ctx, T, L> {
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
                self.lock.ptr_mut().as_ptr(),
                &mut var_id,
            )
            .ok(&*self)?;

            let sampler_id = self.create_handle_if_not_zero(var_id);

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
                "The provided proof of building combined image samplers is invalid",
            )));
        }

        unsafe {
            sys::spvc_compiler_build_combined_image_samplers(self.ptr.as_ptr()).ok(self)?;

            Ok(())
        }
    }

    /// Gets a remapping for the combined image samplers.
    pub fn combined_image_samplers(&self) -> error::Result<CombinedImageSamplerIter<'ctx>> {
        unsafe {
            let mut samplers = std::ptr::null();
            let mut size = 0;

            // SAFETY: 'ctx is sound here.
            // https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross_c.cpp#L2497
            sys::spvc_compiler_get_combined_image_samplers(
                self.ptr.as_ptr(),
                &mut samplers,
                &mut size,
            )
            .ok(self)?;
            let slice = slice::from_raw_parts(samplers, size);
            Ok(CombinedImageSamplerIter(self.phantom(), slice.iter()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::SpirvCrossError;
    use crate::Compiler;
    use crate::{targets, Module, SpirvCrossContext};

    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn test_combined_image_sampler_build() -> Result<(), SpirvCrossError> {
        let spv = SpirvCrossContext::new()?;
        let vec = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&vec));

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
