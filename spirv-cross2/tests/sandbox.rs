use glslang;
use glslang::SpirvVersion::{SPIRV1_0, SPIRV1_1, SPIRV1_3, SPIRV1_6};
use glslang::{
    CompilerOptions, OpenGlVersion, ShaderInput, ShaderSource, ShaderStage, Target, VulkanVersion,
};
use spirv_cross2::error::SpirvCrossError;
use spirv_cross2::reflect::ExecutionModeArguments;
use spirv_cross2::reflect::TypeInner;
use spirv_cross2::{spirv, Module};
use spirv_cross_sys::{ConstantId, SpvId};

#[test]
pub fn workgroup_size() -> Result<(), SpirvCrossError> {
    const SHADER: &str = r##"#version 450

layout (local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;


layout(set = 0, binding = 0) uniform Config{
    mat4 transform;
    int matrixCount;
} opData;

layout(set = 0, binding = 1) readonly buffer  InputBuffer{
    mat4 matrices[];
} sourceData;

layout(set = 0, binding = 2) buffer  OutputBuffer{
    mat4 matrices[];
} outputData;

void main()
{
    //grab global ID
	uint gID = gl_GlobalInvocationID.x;
    //make sure we don't access past the buffer size
    if(gID < opData.matrixCount)
    {
        // do math
        outputData.matrices[gID] = sourceData.matrices[gID] * opData.transform;
    }
}
"##;

    let glslang = glslang::Compiler::acquire().unwrap();

    let src = ShaderSource::from(SHADER);
    let mut opts = CompilerOptions::default();

    opts.target = Target::Vulkan {
        version: VulkanVersion::Vulkan1_3,
        spirv_version: SPIRV1_6,
    };

    let shader = ShaderInput::new(&src, ShaderStage::Compute, &opts, None).unwrap();
    let spv = glslang.create_shader(shader).unwrap().compile().unwrap();

    let cross = spirv_cross2::SpirvCross::new()?;
    let mut compiler =
        cross.into_compiler::<spirv_cross2::targets::None>(Module::from_words(&spv))?;

    let spec_workgroup = compiler.work_group_size_specialization_constants();
    eprintln!("{:?}", spec_workgroup);

    let Some(args) = compiler.execution_mode_arguments(spirv::ExecutionMode::LocalSize)? else {
        panic!("unexpected")
    };

    let id = spec_workgroup.x.unwrap().id;

    // compiler.set_specialization_constant_value::<u32>(id, 100, 100, 10)?;
    let id = compiler.specialization_constant_value::<u32>(id, 0, 0)?;

    eprintln!("{:?}", args);
    eprintln!("{:?}", id);

    Ok(())
}

#[test]
pub fn sandbox() -> Result<(), SpirvCrossError> {
    const SHADER: &str = r##"#version 450

layout(binding = 0) uniform UBO
{
    float value;
};

layout(location = 0) out vec4 color;
layout(binding = 1) uniform sampler2D tex;

void main() {
    color = texture(tex, vec2(value));
}"##;

    let glslang = glslang::Compiler::acquire().unwrap();

    let src = ShaderSource::from(SHADER);
    let mut opts = CompilerOptions::default();

    opts.target = Target::Vulkan {
        version: VulkanVersion::Vulkan1_0,
        spirv_version: SPIRV1_0,
    };

    let shader = ShaderInput::new(&src, ShaderStage::Vertex, &opts, None).unwrap();
    let spv = glslang.create_shader(shader).unwrap().compile().unwrap();

    let cross = spirv_cross2::SpirvCross::new()?;
    let compiler = cross.into_compiler::<spirv_cross2::targets::None>(Module::from_words(&spv))?;
    let res = compiler.shader_resources()?.all_resources()?;

    let counter = &res.uniform_buffers[0];

    let ranges = compiler.active_buffer_ranges(counter.id)?;
    eprintln!("{:?}", ranges);

    Ok(())
}

#[test]
pub fn runtime_size_array() -> Result<(), SpirvCrossError> {
    const SHADER: &str = r##"#version 450

layout(std430, binding = 0) buffer SSBO
{
    float data[];
};

layout(location = 0) out vec4 color;
layout(binding = 1) uniform sampler2D tex;

void main() {
    color = texture(tex, vec2(0.0));
}"##;

    let glslang = glslang::Compiler::acquire().unwrap();

    let src = ShaderSource::from(SHADER);
    let mut opts = CompilerOptions::default();

    opts.target = Target::Vulkan {
        version: VulkanVersion::Vulkan1_0,
        spirv_version: SPIRV1_0,
    };

    let shader = ShaderInput::new(&src, ShaderStage::Vertex, &opts, None).unwrap();
    let spv = glslang.create_shader(shader).unwrap().compile().unwrap();

    let cross = spirv_cross2::SpirvCross::new()?;
    let compiler = cross.into_compiler::<spirv_cross2::targets::None>(Module::from_words(&spv))?;
    let res = compiler.shader_resources()?.all_resources()?;

    let counter = &res.storage_buffers[0];

    let TypeInner::Struct(struct_ty) = compiler.type_description(counter.base_type_id)?.inner
    else {
        panic!("unknown type")
    };

    eprintln!(
        "{:?}",
        compiler.declared_struct_size_with_runtime_array(struct_ty, 4)
    );

    Ok(())
}

#[test]
pub fn image_type_sandbox() -> Result<(), SpirvCrossError> {
    const SHADER: &str = r##"#version 450

layout (constant_id = 0) const int SSAO_KERNEL_SIZE = 2;


layout(location = 0) out vec4[SSAO_KERNEL_SIZE][4] color;
layout(binding = 1) uniform sampler2D tex;
layout(binding = 1, rgba32f) writeonly uniform image3D threeD;
layout(binding = 2) uniform texture2DMSArray texArrayMs[4];
layout(binding = 3, rgba32f) uniform readonly imageBuffer arrayBuf;
layout(binding = 3) uniform samplerBuffer arrayBufA;
layout(binding = 3, rgba32f) uniform readonly imageBuffer arrayBufAS;

layout(binding = 3) uniform samplerCubeArrayShadow depth;

void main() {
    color[0][0] = texture(tex, vec2(0.0));
}"##;

    let glslang = glslang::Compiler::acquire().unwrap();

    let src = ShaderSource::from(SHADER);
    let mut opts = CompilerOptions::default();

    opts.target = Target::Vulkan {
        version: VulkanVersion::Vulkan1_0,
        spirv_version: SPIRV1_0,
    };

    let shader = ShaderInput::new(&src, ShaderStage::Vertex, &opts, None).unwrap();
    let spv = glslang.create_shader(shader).unwrap().compile().unwrap();

    let cross = spirv_cross2::SpirvCross::new()?;
    let compiler = cross.into_compiler::<spirv_cross2::targets::None>(Module::from_words(&spv))?;
    let res = compiler.shader_resources()?.all_resources()?;

    let counter = &res.sampled_images[1];

    eprintln!("{:?}", compiler.type_description(counter.base_type_id));

    Ok(())
}

#[test]
pub fn const_id_array_dim() -> Result<(), SpirvCrossError> {
    const SHADER: &str = r##"#version 450

layout (constant_id = 0) const int SSAO_KERNEL_SIZE = 2;


layout(location = 0) out vec4[SSAO_KERNEL_SIZE][4] color;
layout(binding = 1) uniform sampler2D tex;

void main() {
    color[0][0] = texture(tex, vec2(0.0));
}"##;

    let glslang = glslang::Compiler::acquire().unwrap();

    let src = ShaderSource::from(SHADER);
    let mut opts = CompilerOptions::default();

    opts.target = Target::Vulkan {
        version: VulkanVersion::Vulkan1_0,
        spirv_version: SPIRV1_0,
    };

    let shader = ShaderInput::new(&src, ShaderStage::Vertex, &opts, None).unwrap();
    let spv = glslang.create_shader(shader).unwrap().compile().unwrap();

    let cross = spirv_cross2::SpirvCross::new()?;
    let compiler = cross.into_compiler::<spirv_cross2::targets::None>(Module::from_words(&spv))?;
    let res = compiler.shader_resources()?.all_resources()?;

    let counter = &res.stage_outputs[0];

    eprintln!("{:?}", compiler.type_description(counter.type_id));

    Ok(())
}

#[test]
pub fn atomic_counters() -> Result<(), SpirvCrossError> {
    const SHADER: &str = r##"#version 450

layout(binding = 0) uniform atomic_uint one;

layout(location = 0) out vec4 color;
layout(binding = 1) uniform sampler2D tex;

void main() {
    color = texture(tex, vec2(0.0));
}"##;

    let glslang = glslang::Compiler::acquire().unwrap();

    let src = ShaderSource::from(SHADER);
    let mut opts = CompilerOptions::default();

    opts.target = Target::OpenGL {
        version: OpenGlVersion::OpenGL4_5,
        spirv_version: Some(SPIRV1_1),
    };

    let shader = ShaderInput::new(&src, ShaderStage::Vertex, &opts, None).unwrap();
    let spv = glslang.create_shader(shader).unwrap().compile().unwrap();

    let cross = spirv_cross2::SpirvCross::new()?;
    let compiler = cross.into_compiler::<spirv_cross2::targets::None>(Module::from_words(&spv))?;
    let res = compiler.shader_resources()?.all_resources()?;

    let counter = &res.atomic_counters[0];

    eprintln!("{:?}", compiler.type_description(counter.base_type_id));

    Ok(())
}
