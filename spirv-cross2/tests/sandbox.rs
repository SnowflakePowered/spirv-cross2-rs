use glslang;
use glslang::SpirvVersion::{SPIRV1_0, SPIRV1_1};
use glslang::{
    CompilerOptions, OpenGlVersion, ShaderInput, ShaderSource, ShaderStage, Target, VulkanVersion,
};
use spirv_cross2::compiler::types::TypeInner;
use spirv_cross2::error::SpirvCrossError;
use spirv_cross2::Module;

#[test]
pub fn sandbox() -> Result<(), SpirvCrossError> {
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
    let compiler =
        cross.into_compiler::<spirv_cross2::compiler::targets::None>(Module::from_words(&spv))?;
    let res = compiler.shader_resources()?.all_resources()?;

    let counter = &res.storage_buffers[0];

    let TypeInner::Struct(struct_ty) = compiler.get_type(counter.base_type_id)?.inner else {
        panic!("unknown type")
    };

    eprintln!("{:?}", compiler.declared_struct_size_with_runtime_array(struct_ty, 4));

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
    let compiler =
        cross.into_compiler::<spirv_cross2::compiler::targets::None>(Module::from_words(&spv))?;
    let res = compiler.shader_resources()?.all_resources()?;

    let counter = &res.sampled_images[1];

    eprintln!("{:?}", compiler.get_type(counter.base_type_id));

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
    let compiler =
        cross.into_compiler::<spirv_cross2::compiler::targets::None>(Module::from_words(&spv))?;
    let res = compiler.shader_resources()?.all_resources()?;

    let counter = &res.stage_outputs[0];

    eprintln!("{:?}", compiler.get_type(counter.type_id));

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
    let compiler =
        cross.into_compiler::<spirv_cross2::compiler::targets::None>(Module::from_words(&spv))?;
    let res = compiler.shader_resources()?.all_resources()?;

    let counter = &res.atomic_counters[0];

    eprintln!("{:?}", compiler.get_type(counter.base_type_id));

    Ok(())
}
