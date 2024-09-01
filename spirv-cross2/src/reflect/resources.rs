use crate::error::{ContextRooted, SpirvCrossError, ToContextError};
use crate::handle::Handle;
use crate::sealed::Sealed;
use crate::string::MaybeCStr;
use crate::{error, spirv, Compiler, PhantomCompiler, ToStatic};
use spirv_cross_sys as sys;
use spirv_cross_sys::{
    spvc_context_s, spvc_reflected_builtin_resource, spvc_reflected_resource, spvc_resources_s,
    spvc_set, BuiltinResourceType, ResourceType, SpvId, TypeId, VariableId,
};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::slice;

pub struct ShaderResources<'a>(NonNull<spvc_resources_s>, PhantomCompiler<'a>);

impl ContextRooted for &ShaderResources<'_> {
    #[inline(always)]
    fn context(&self) -> NonNull<spvc_context_s> {
        self.1.context()
    }
}

impl<'a, T> Compiler<'a, T> {
    /// Query shader resources, use ids with reflection interface to modify or query binding points, etc.
    pub fn shader_resources(&self) -> crate::error::Result<ShaderResources> {
        unsafe {
            let mut resources = std::ptr::null_mut();
            sys::spvc_compiler_create_shader_resources(self.ptr.as_ptr(), &mut resources)
                .ok(self)?;

            let Some(resources) = NonNull::new(resources) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(ShaderResources(resources, self.phantom()))
        }
    }

    /// Query shader resources, but only return the variables which are part of active_variables.
    /// E.g.: get_shader_resources(get_active_variables()) to only return the variables which are statically
    /// accessed.
    pub fn shader_resources_for_active_variables(
        &self,
        set: InterfaceVariableSet,
    ) -> crate::error::Result<ShaderResources> {
        unsafe {
            let mut resources = std::ptr::null_mut();
            sys::spvc_compiler_create_shader_resources_for_active_variables(
                self.ptr.as_ptr(),
                &mut resources,
                set.0,
            )
            .ok(self)?;

            let Some(resources) = NonNull::new(resources) else {
                return Err(SpirvCrossError::OutOfMemory(String::from("Out of memory")));
            };

            Ok(ShaderResources(resources, self.phantom()))
        }
    }
}

/// A handle to a set of interface variables.
pub struct InterfaceVariableSet<'a>(spvc_set, Handle<PhantomData<&'a ()>>, PhantomCompiler<'a>);

impl<'a> InterfaceVariableSet<'a> {
    /// Get the SPIR-V IDs for the active interface variables.
    ///
    /// This is only meant to be used for reflection. It is not possible
    /// to modify the contents of an [`InterfaceVariableSet`].
    pub fn to_handles(&self) -> Vec<Handle<VariableId>> {
        unsafe {
            // Get the length of allocation
            let mut length = 0;
            spirv_cross_sys::spvc_rs_expose_set(self.0, std::ptr::null_mut(), &mut length);

            // write into the vec
            let mut vec = vec![0; length];
            spirv_cross_sys::spvc_rs_expose_set(self.0, vec.as_mut_ptr(), &mut length);

            vec.into_iter()
                .map(|id| self.2.create_handle(VariableId(SpvId(id))))
                .collect()
        }
    }
}

// reflection
impl<'a, T> Compiler<'a, T> {
    /// Returns a set of all global variables which are statically accessed
    /// by the control flow graph from the current entry point.
    /// Only variables which change the interface for a shader are returned, that is,
    /// variables with storage class of Input, Output, Uniform, UniformConstant, PushConstant and AtomicCounter
    /// storage classes are returned.
    ///
    /// To use the returned set as the filter for which variables are used during compilation,
    /// this set can be moved to set_enabled_interface_variables().
    ///
    /// The return object is opaque to Rust, but its contents inspected by using [`InterfaceVariableSet::to_handles`].
    /// There is no way to modify the contents or use your own `InterfaceVariableSet`.
    pub fn active_interface_variables(&self) -> error::Result<InterfaceVariableSet<'a>> {
        unsafe {
            let mut set = std::ptr::null();
            sys::spvc_compiler_get_active_interface_variables(self.ptr.as_ptr(), &mut set)
                .ok(self)?;

            Ok(InterfaceVariableSet(
                set,
                self.create_handle(PhantomData),
                self.phantom(),
            ))
        }
    }

    /// Sets the interface variables which are used during compilation.
    /// By default, all variables are used.
    /// Once set, [`Compiler::compile`] will only consider the set in active_variables.
    pub fn set_enabled_interface_variables(
        &mut self,
        set: InterfaceVariableSet,
    ) -> error::Result<()> {
        if !self.handle_is_valid(&set.1) {
            return Err(SpirvCrossError::InvalidOperation(String::from(
                "The interface variable set is invalid for this compiler instance.",
            )));
        }
        unsafe {
            sys::spvc_compiler_set_enabled_interface_variables(self.ptr.as_ptr(), set.0)
                .ok(self)?;
            Ok(())
        }
    }
}

/// Iterator over reflected resources.
pub struct ResourceIter<'a>(
    PhantomCompiler<'a>,
    slice::Iter<'a, spvc_reflected_resource>,
);

impl<'a> Iterator for ResourceIter<'a> {
    type Item = Resource<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|o| Resource::from_raw(self.0, o))
    }
}

/// Iterator over reflected builtin resources
pub struct BuiltinResourceIter<'a>(
    PhantomCompiler<'a>,
    slice::Iter<'a, spvc_reflected_builtin_resource>,
);

impl<'a> Iterator for BuiltinResourceIter<'a> {
    type Item = BuiltinResource<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|o| BuiltinResource::from_raw(self.0, o))
    }
}

#[derive(Debug)]
pub struct Resource<'a> {
    pub id: Handle<VariableId>,
    pub base_type_id: Handle<TypeId>,
    pub type_id: Handle<TypeId>,
    pub name: MaybeCStr<'a>,
}

impl<'a> Resource<'a> {
    fn from_raw(comp: PhantomCompiler<'a>, value: &'a spvc_reflected_resource) -> Self {
        Self {
            id: comp.create_handle(value.id),
            base_type_id: comp.create_handle(value.base_type_id),
            type_id: comp.create_handle(value.type_id),
            // There should never be invalid UTF-8 in a shader.
            // as per SPIR-V spec: The character set is Unicode in the UTF-8 encoding scheme.
            // so this will be free 100% of the time.
            name: unsafe { MaybeCStr::from_ptr(value.name) },
        }
    }
}

impl Sealed for Resource<'_> {}
impl ToStatic for Resource<'_> {
    type Static<'a>

    = Resource<'static>
    where
        'a: 'static;

    fn to_static(&self) -> Self::Static<'static> {
        Resource {
            id: self.id,
            base_type_id: self.base_type_id,
            type_id: self.type_id,
            name: MaybeCStr::from_string(self.name.to_string()),
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
    pub value_type_id: Handle<TypeId>,
    pub resource: Resource<'a>,
}

impl<'a> BuiltinResource<'a> {
    fn from_raw(comp: PhantomCompiler<'a>, value: &'a spvc_reflected_builtin_resource) -> Self {
        Self {
            builtin: value.builtin,
            value_type_id: comp.create_handle(value.value_type_id),
            resource: Resource::from_raw(comp, &value.resource),
        }
    }
}

impl Sealed for BuiltinResource<'_> {}
impl ToStatic for BuiltinResource<'_> {
    type Static<'a>

    = BuiltinResource<'static>
    where
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
    #[deprecated = "Currently unsupported by the C API."]
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

impl Sealed for AllResources<'_> {}
impl ToStatic for AllResources<'_> {
    type Static<'a>

    = AllResources<'static>
    where
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

impl<'a> ShaderResources<'a> {
    /// Get an iterator for all resources of the given type.
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

        Ok(ResourceIter(self.1, slice.iter()))
    }

    /// Get an iterator for all builtin resources of the given type.
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

        Ok(BuiltinResourceIter(self.1, slice.iter()))
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
