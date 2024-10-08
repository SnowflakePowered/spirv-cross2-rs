use crate::error::{SpirvCrossError, ToContextError};
use crate::handle::{Handle, TypeId, VariableId};
use crate::sealed::Sealed;
use crate::string::CompilerStr;
use crate::{error, Compiler, PhantomCompiler, ToStatic};
use spirv_cross_sys as sys;
use spirv_cross_sys::{
    spvc_reflected_builtin_resource, spvc_reflected_resource, spvc_resources_s, spvc_set,
};
use std::ptr::NonNull;
use std::slice;

/// The type of built-in resources to query.
pub use spirv_cross_sys::BuiltinResourceType;

use crate::iter::impl_iterator;
/// The type of resource to query.
pub use spirv_cross_sys::ResourceType;

/// A handle to shader resources.
pub struct ShaderResources(NonNull<spvc_resources_s>, PhantomCompiler);

impl<T> Compiler<T> {
    /// Query shader resources, use ids with reflection interface to modify or query binding points, etc.
    pub fn shader_resources(&self) -> crate::error::Result<ShaderResources> {
        // SAFETY: 'ctx is Ok
        // since this gets allocated forever
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_cross_c.cpp#L1925
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
    ) -> error::Result<ShaderResources> {
        // SAFETY: 'ctx is Ok
        // since this gets allocated forever
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_cross_c.cpp#L1925
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
pub struct InterfaceVariableSet(spvc_set, Handle<()>, PhantomCompiler);

impl InterfaceVariableSet {
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

            let mut handles: Vec<Handle<VariableId>> = vec
                .into_iter()
                .map(|id| self.2.create_handle(VariableId::from(id)))
                .collect();

            handles.sort_by_key(|h| h.id());

            handles
        }
    }
}

// reflection
impl<T> Compiler<T> {
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
    pub fn active_interface_variables(&self) -> error::Result<InterfaceVariableSet> {
        unsafe {
            let mut set = std::ptr::null();

            // SAFETY: 'ctx is sound here
            // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_cross_c.cpp#L1888

            sys::spvc_compiler_get_active_interface_variables(self.ptr.as_ptr(), &mut set)
                .ok(self)?;

            Ok(InterfaceVariableSet(
                set,
                self.create_handle(()),
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
                .ok(&*self)?;
            Ok(())
        }
    }
}

/// Iterator over reflected resources, created by [`ShaderResources::resources_for_type`].
pub struct ResourceIter<'a>(PhantomCompiler, slice::Iter<'a, spvc_reflected_resource>);

impl_iterator!(ResourceIter<'a>: Resource<'a> as map |s, o: &'a spvc_reflected_resource| {
    Resource::from_raw(s.0.clone(), o)
} for <'a> [1]);

/// Iterator over reflected builtin resources, created by [`ShaderResources::builtin_resources_for_type`].
pub struct BuiltinResourceIter<'a>(
    PhantomCompiler,
    slice::Iter<'a, spvc_reflected_builtin_resource>,
);

impl_iterator!(BuiltinResourceIter<'a>: BuiltinResource<'a> as and_then |s, o: &'a spvc_reflected_builtin_resource| {
    BuiltinResource::from_raw(s.0.clone(), o)
} for <'a> [1]);

/// Description of a shader resource.
#[derive(Debug)]
pub struct Resource<'a> {
    /// A handle to the variable this resource points to.
    pub id: Handle<VariableId>,
    /// A handle to the base type of this resource.
    pub base_type_id: Handle<TypeId>,
    /// A handle to the type of this resource, often a pointer
    /// or array.
    pub type_id: Handle<TypeId>,
    /// The name of this resource.
    pub name: CompilerStr<'a>,
}

impl<'a> Resource<'a> {
    fn from_raw(comp: PhantomCompiler, value: &'a spvc_reflected_resource) -> Self {
        Self {
            id: comp.create_handle(value.id),
            base_type_id: comp.create_handle(value.base_type_id),
            type_id: comp.create_handle(value.type_id),
            // There should never be invalid UTF-8 in a shader.
            // as per SPIR-V spec: The character set is Unicode in the UTF-8 encoding scheme.
            // so this will be free 100% of the time.
            name: unsafe { CompilerStr::from_ptr(value.name, comp.ctx.clone()) },
        }
    }
}

impl<'a, 'b> From<&'a Resource<'b>> for Handle<VariableId> {
    fn from(value: &'a Resource<'b>) -> Self {
        value.id
    }
}

impl From<Resource<'_>> for Handle<VariableId> {
    fn from(value: Resource<'_>) -> Self {
        value.id
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
            name: CompilerStr::from_string(self.name.to_string()),
        }
    }
}

impl Clone for Resource<'_> {
    fn clone(&self) -> Resource<'static> {
        self.to_static()
    }
}

/// Description of a built-in shader resource.
#[derive(Debug)]
pub struct BuiltinResource<'a> {
    /// The SPIR-V built-in for this resource.
    pub builtin: spirv::BuiltIn,
    /// A handle to the type ID of the value.
    pub value_type_id: Handle<TypeId>,
    /// The resource data for this built-in resource.
    pub resource: Resource<'a>,
}

impl<'a, 'b> From<&'a BuiltinResource<'b>> for Handle<VariableId> {
    fn from(value: &'a BuiltinResource<'b>) -> Self {
        value.resource.id
    }
}

impl From<BuiltinResource<'_>> for Handle<VariableId> {
    fn from(value: BuiltinResource<'_>) -> Self {
        value.resource.id
    }
}

impl<'a> BuiltinResource<'a> {
    fn from_raw(comp: PhantomCompiler, value: &'a spvc_reflected_builtin_resource) -> Option<Self> {
        // builtin is potentially uninit, we need to check.
        let Some(builtin) = spirv::BuiltIn::from_u32(value.builtin.0 as u32) else {
            if cfg!(debug_assertions) {
                panic!("Unexpected SpvBuiltIn in spvc_reflected_builtin_resource!")
            } else {
                return None;
            }
        };

        Some(Self {
            builtin,
            value_type_id: comp.create_handle(value.value_type_id),
            resource: Resource::from_raw(comp, &value.resource),
        })
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

impl Clone for BuiltinResource<'_> {
    fn clone(&self) -> BuiltinResource<'static> {
        self.to_static()
    }
}

/// All SPIR-V resources declared in the module.
#[derive(Debug)]
pub struct AllResources<'a> {
    /// Uniform buffer (UBOs) resources.
    pub uniform_buffers: Vec<Resource<'a>>,
    /// Storage buffer (SSBO) resources.
    pub storage_buffers: Vec<Resource<'a>>,
    /// Shader stage inputs.
    pub stage_inputs: Vec<Resource<'a>>,
    /// Shader stage outputs.
    pub stage_outputs: Vec<Resource<'a>>,
    /// Shader subpass inputs.
    pub subpass_inputs: Vec<Resource<'a>>,
    /// Storage images (i.e. `imageND`).
    pub storage_images: Vec<Resource<'a>>,
    /// Sampled images (i.e. `samplerND`).
    pub sampled_images: Vec<Resource<'a>>,
    /// Atomic counters.
    pub atomic_counters: Vec<Resource<'a>>,
    /// Acceleration structures.
    pub acceleration_structures: Vec<Resource<'a>>,
    /// Legacy OpenGL plain uniforms.
    pub gl_plain_uniforms: Vec<Resource<'a>>,

    /// Push constant buffers.
    ///
    /// There is only ever at most one push constant buffer,
    /// but this is multiplicit in case this restriction is lifted.
    pub push_constant_buffers: Vec<Resource<'a>>,
    /// Record buffers.
    pub shader_record_buffers: Vec<Resource<'a>>,

    /// For Vulkan GLSL and HLSL sources, split images (i.e. `textureND`).
    pub separate_images: Vec<Resource<'a>>,

    /// For Vulkan GLSL and HLSL sources, split samplers (i.e. `sampler`).
    pub separate_samplers: Vec<Resource<'a>>,

    /// Shader built-in inputs.
    pub builtin_inputs: Vec<BuiltinResource<'a>>,
    /// Shader built-in outputs.
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

impl Clone for AllResources<'_> {
    fn clone(&self) -> AllResources<'static> {
        self.to_static()
    }
}

impl ShaderResources {
    /// Get an iterator for all resources of the given type.
    pub fn resources_for_type(&self, ty: ResourceType) -> error::Result<ResourceIter<'static>> {
        // SAFETY: 'ctx is sound here,
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_cross_c.cpp#L1802
        // Furthermore, once allocated, the lifetime of spvc_resources_s is tied to that of the context.
        // so all child resources inherit the lifetime.
        let mut count = 0;
        let mut out = std::ptr::null();
        unsafe {
            spirv_cross_sys::spvc_resources_get_resource_list_for_type(
                self.0.as_ptr(),
                ty,
                &mut out,
                &mut count,
            )
            .ok(&self.1)?;
        }

        let slice = unsafe { slice::from_raw_parts(out, count) };

        Ok(ResourceIter(self.1.clone(), slice.iter()))
    }

    /// Get an iterator for all builtin resources of the given type.
    pub fn builtin_resources_for_type(
        &self,
        ty: BuiltinResourceType,
    ) -> error::Result<BuiltinResourceIter<'static>> {
        let mut count = 0;
        let mut out = std::ptr::null();

        // SAFETY: 'ctx is sound here,
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_cross_c.cpp#L1826
        // Furthermore, once allocated, the lifetime of spvc_resources_s is tied to that of the context.
        // so all child resources inherit the lifetime.
        unsafe {
            spirv_cross_sys::spvc_resources_get_builtin_resource_list_for_type(
                self.0.as_ptr(),
                ty,
                &mut out,
                &mut count,
            )
            .ok(&self.1)?;
        }

        let slice = unsafe { slice::from_raw_parts(out.cast(), count) };

        Ok(BuiltinResourceIter(self.1.clone(), slice.iter()))
    }

    /// Get all resources declared in the shader.
    ///
    /// This will allocate a `Vec` for every resource type.
    #[rustfmt::skip]
    pub fn all_resources(&self) -> error::Result<AllResources<'static>> {
          // SAFETY: 'ctx is sound by transitive property of resources_for_type
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
                gl_plain_uniforms: self.resources_for_type(ResourceType::GlPlainUniform)?.collect(),
                push_constant_buffers: self.resources_for_type(ResourceType::PushConstant)?.collect(),
                shader_record_buffers: self.resources_for_type(ResourceType::ShaderRecordBuffer)?.collect(),
                separate_images: self.resources_for_type(ResourceType::SeparateImage)?.collect(),
                separate_samplers: self.resources_for_type(ResourceType::SeparateSamplers)?.collect(),
                builtin_inputs: self.builtin_resources_for_type(BuiltinResourceType::StageInput)?.collect(),
                builtin_outputs: self.builtin_resources_for_type(BuiltinResourceType::StageOutput)?.collect(),
        })
    }
}
