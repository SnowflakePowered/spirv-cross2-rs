use crate::compiler::{InterfaceVariableSet, ShaderResources};
use crate::{error, spirv, ToStatic};
use spirv_cross_sys::{
    spvc_context_s, spvc_reflected_builtin_resource, spvc_reflected_resource, spvc_resources_s,
    BuiltinResourceType, ContextRooted, ResourceType, TypeId, VariableId,
};
use std::borrow::{Borrow, Cow};
use std::ffi::CStr;
use std::ptr::NonNull;
use std::slice;

impl<'a> InterfaceVariableSet<'a> {
    /// Get the SPIR-V IDs for the active interface variables.
    ///
    /// This is only meant to be used for reflection.
    pub fn reflect(&self) -> Vec<u32> {
        unsafe {
            // Get the length of allocation
            let mut length = 0;
            spirv_cross_sys::spvc_rs_expose_set(self.0, std::ptr::null_mut(), &mut length);

            // write into the vec
            let mut vec = vec![0; length];
            spirv_cross_sys::spvc_rs_expose_set(self.0, vec.as_mut_ptr(), &mut length);

            vec
        }
    }
}

impl ContextRooted for &ShaderResources<'_> {
    fn context(&self) -> NonNull<spvc_context_s> {
        self.1 .0
    }
}

/// Iterator over reflected resources.
pub struct ResourceIter<'a>(slice::Iter<'a, spvc_reflected_resource>);

impl<'a> Iterator for ResourceIter<'a> {
    type Item = Resource<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Resource::from)
    }
}

/// Iterator over reflected builtin resources
pub struct BuiltinResourceIter<'a>(slice::Iter<'a, spvc_reflected_builtin_resource>);

impl<'a> Iterator for BuiltinResourceIter<'a> {
    type Item = BuiltinResource<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(BuiltinResource::from)
    }
}

#[derive(Debug)]
pub struct Resource<'a> {
    pub id: VariableId,
    pub base_type_id: TypeId,
    pub type_id: TypeId,
    pub name: Cow<'a, str>,
}

impl ToStatic for Resource<'_> {
    type Static<'a>

    = Resource<'static>    where
        'a: 'static;

    fn to_static(&self) -> Self::Static<'static> {
        Resource {
            id: self.id,
            base_type_id: self.base_type_id,
            type_id: self.type_id,
            name: Cow::Owned(self.name.to_string()),
        }
    }
}

/// Cloning a [`Resource`] will detach its lifetime from the [`crate::SpirvCross`] context
/// from which it originated.
impl Clone for Resource<'_> {
    fn clone(&self) -> Resource<'static> {
        self.to_static()
    }
}

#[derive(Debug)]
pub struct BuiltinResource<'a> {
    pub builtin: spirv::BuiltIn,
    pub value_type_id: TypeId,
    pub resource: Resource<'a>,
}

impl ToStatic for BuiltinResource<'_> {
    type Static<'a>

    = BuiltinResource<'static>    where
        'a: 'static;

    fn to_static(&self) -> Self::Static<'static> {
        let resource: BuiltinResource<'static> = BuiltinResource {
            builtin: self.builtin,
            value_type_id: self.value_type_id,
            resource: self.resource.to_static(),
        };
        resource
    }
}

/// Cloning a [`BuiltinResource`] will detach its lifetime from the [`crate::SpirvCross`] context
/// from which it originated.
impl Clone for BuiltinResource<'_> {
    fn clone(&self) -> BuiltinResource<'static> {
        self.to_static()
    }
}

#[derive(Debug)]
pub struct AllResources<'a> {
    pub uniform_buffers: Vec<Resource<'a>>,
    pub storage_buffers: Vec<Resource<'a>>,
    pub stage_inputs: Vec<Resource<'a>>,
    pub stage_outputs: Vec<Resource<'a>>,
    pub subpass_inputs: Vec<Resource<'a>>,
    pub storage_images: Vec<Resource<'a>>,
    pub sampled_images: Vec<Resource<'a>>,
    pub atomic_counters: Vec<Resource<'a>>,
    pub acceleration_structures: Vec<Resource<'a>>,

    /// Currently unsupported.
    pub gl_plain_uniforms: Vec<Resource<'a>>,

    pub push_constant_buffers: Vec<Resource<'a>>,
    pub shader_record_buffers: Vec<Resource<'a>>,

    // For Vulkan GLSL and HLSL source,
    // these correspond to separate texture2D and samplers respectively.
    pub separate_images: Vec<Resource<'a>>,
    pub separate_samplers: Vec<Resource<'a>>,

    pub builtin_inputs: Vec<BuiltinResource<'a>>,
    pub builtin_outputs: Vec<BuiltinResource<'a>>,
}

impl ToStatic for AllResources<'_> {
    type Static<'a>

    = AllResources<'static>     where
        'a: 'static;

    fn to_static(&self) -> Self::Static<'static> {
        AllResources {
            uniform_buffers: self
                .uniform_buffers
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            storage_buffers: self
                .storage_buffers
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            stage_inputs: self.stage_inputs.iter().map(ToStatic::to_static).collect(),
            stage_outputs: self.stage_outputs.iter().map(ToStatic::to_static).collect(),
            subpass_inputs: self
                .subpass_inputs
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            storage_images: self
                .storage_images
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            sampled_images: self
                .sampled_images
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            atomic_counters: self
                .atomic_counters
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            acceleration_structures: self
                .acceleration_structures
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            gl_plain_uniforms: self
                .gl_plain_uniforms
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            push_constant_buffers: self
                .push_constant_buffers
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            shader_record_buffers: self
                .shader_record_buffers
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            separate_images: self
                .separate_images
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            separate_samplers: self
                .separate_samplers
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            builtin_inputs: self
                .builtin_inputs
                .iter()
                .map(ToStatic::to_static)
                .collect(),
            builtin_outputs: self
                .builtin_outputs
                .iter()
                .map(ToStatic::to_static)
                .collect(),
        }
    }
}

/// Cloning a [`AllResources`] will detach its lifetime from the [`crate::SpirvCross`] context
/// from which it originated.
impl Clone for AllResources<'_> {
    fn clone(&self) -> AllResources<'static> {
        self.to_static()
    }
}

impl<'a> From<&'a spvc_reflected_resource> for Resource<'a> {
    fn from(value: &'a spvc_reflected_resource) -> Self {
        Self {
            id: value.id,
            base_type_id: value.base_type_id,
            type_id: value.type_id,
            // There should never be invalid UTF-8 in a shader.
            // as per SPIR-V spec: The character set is Unicode in the UTF-8 encoding scheme.
            // so this will be free 100% of the time.
            name: unsafe { CStr::from_ptr(value.name) }.to_string_lossy(),
        }
    }
}

impl<'a> From<&'a spvc_reflected_builtin_resource> for BuiltinResource<'a> {
    fn from(value: &'a spvc_reflected_builtin_resource) -> Self {
        Self {
            builtin: value.builtin,
            value_type_id: value.value_type_id,
            resource: Resource::from(&value.resource),
        }
    }
}

impl<'a> ShaderResources<'a> {
    pub fn resources_for_type(&self, ty: ResourceType) -> error::Result<ResourceIter<'a>> {
        let mut count = 0;
        let mut out = std::ptr::null();
        unsafe {
            spirv_cross_sys::spvc_resources_get_resource_list_for_type(
                self.0.as_ptr(),
                ty,
                &mut out,
                &mut count,
            )
            .ok(self)?;
        }

        let slice = unsafe { std::slice::from_raw_parts(out, count) };

        Ok(ResourceIter(slice.into_iter()))
    }

    pub fn builtin_resources_for_type(
        &self,
        ty: BuiltinResourceType,
    ) -> error::Result<BuiltinResourceIter<'a>> {
        let mut count = 0;
        let mut out = std::ptr::null();
        unsafe {
            spirv_cross_sys::spvc_resources_get_builtin_resource_list_for_type(
                self.0.as_ptr(),
                ty,
                &mut out,
                &mut count,
            )
            .ok(self)?;
        }

        let slice = unsafe { std::slice::from_raw_parts(out, count) };

        Ok(BuiltinResourceIter(slice.into_iter()))
    }

    /// Get all resources declared in the shader.
    #[rustfmt::skip]
    pub fn all_resources(&self) -> error::Result<AllResources<'a>> {
        Ok(AllResources {
                uniform_buffers: self.resources_for_type(ResourceType::UniformBuffer)?.collect(),
                storage_buffers: self.resources_for_type(ResourceType::StorageBuffer)?.collect(),
                stage_inputs: self.resources_for_type(ResourceType::StageInput)?.collect(),
                stage_outputs: self.resources_for_type(ResourceType::StageOutput)?.collect(),
                subpass_inputs: self.resources_for_type(ResourceType::SubpassInput)?.collect(),
                storage_images: self.resources_for_type(ResourceType::StorageImage)?.collect(),
                sampled_images: self.resources_for_type(ResourceType::SampledImage)?.collect(),
                atomic_counters: self.resources_for_type(ResourceType::AtomicCounter)?.collect(),
                acceleration_structures: self.resources_for_type(ResourceType::AccelerationStructure)?.collect(),
                gl_plain_uniforms: vec![],
                push_constant_buffers: self.resources_for_type(ResourceType::PushConstant)?.collect(),
                shader_record_buffers: self.resources_for_type(ResourceType::ShaderRecordBuffer)?.collect(),
                separate_images: self.resources_for_type(ResourceType::SeparateImage)?.collect(),
                separate_samplers: self.resources_for_type(ResourceType::SeparateSamplers)?.collect(),
                builtin_inputs: self.builtin_resources_for_type(BuiltinResourceType::StageInput)?.collect(),
                builtin_outputs: self.builtin_resources_for_type(BuiltinResourceType::StageOutput)?.collect(),
        })
    }
}
