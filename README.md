# spirv-cross2

Safe and sound Rust bindings to [SPIRV-Cross](https://github.com/KhronosGroup/SPIRV-Cross).

[![Latest Version](https://img.shields.io/crates/v/spirv-cross2.svg)](https://crates.io/crates/spirv-cross2) [![Docs](https://docs.rs/spirv-cross2/badge.svg)](https://docs.rs/spirv-cross2)
![Crates.io MSRV](https://img.shields.io/crates/msrv/spirv-cross2)
![License](https://img.shields.io/crates/l/spirv-cross2)

 All backends exposed by the SPIRV-Cross C API are fully supported, including

 * GLSL
 * HLSL
 * MSL
 * JSON
 * C++
 * Reflection Only

 The API provided is roughly similar to the SPIRV-Cross [`Compiler`](https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross.hpp) C++ API,
 with some inspiration from [naga](https://docs.rs/naga/latest/naga/index.html). A best effort has been
 made to ensure that these bindings are sound, and that mutations occur strictly within Rust's borrow rules.

 ## Usage
 Here is an example of using the API to do some reflection and compile to GLSL.

 ```rust
 use spirv_cross2::compile::{CompilableTarget, CompiledArtifact};
 use spirv_cross2::{Module, SpirvCrossContext, SpirvCrossError};
 use spirv_cross2::compile::glsl::GlslVersion;
 use spirv_cross2::reflect::{DecorationValue, ResourceType};
 use spirv_cross2::spirv;
 use spirv_cross2::targets::Glsl;

 fn compile_spirv(words: &[u32]) -> Result<CompiledArtifact<'static, Glsl>, SpirvCrossError> {
     let module = Module::from_words(words);
     let context = SpirvCrossContext::new()?;

     let mut compiler = context.into_compiler::<Glsl>(module)?;

     let resources = compiler.shader_resources()?;

     for resource in resources.resources_for_type(ResourceType::SampledImage)? {
         let Some(DecorationValue::Literal(set)) =
                 compiler.decoration(resource.id,  spirv::Decoration::DescriptorSet)? else {
             continue;
         };
         let Some(DecorationValue::Literal(binding)) =
             compiler.decoration(resource.id,  spirv::Decoration::Binding)? else {
             continue;
         };

         println!("Image {} at set = {}, binding = {}", resource.name, set, binding);

         // Modify the decoration to prepare it for GLSL.
         compiler.set_decoration(resource.id, spirv::Decoration::DescriptorSet,
                 DecorationValue::unset())?;

         // Some arbitrary remapping if we want.
         compiler.set_decoration(resource.id, spirv::Decoration::Binding,
             Some(set * 16 + binding))?;
     }

     let mut options = Glsl::options();
     options.version = GlslVersion::Glsl300Es;

     compiler.compile(&options)
 }
 ```

### `f16` and vector specialization constants support
When querying specialization constants, spirv-cross2 includes optional support for `f16` via [half](https://crates.io/crates/half) and `Vec2`, `Vec3`, `Vec4`, and `Mat4` types
via [gfx-maths](https://crates.io/crates/gfx-maths). This is included by default, but can be disabled by disabling default features.

```toml
[dependencies]
spirv-cross2 = { default-features = false } 
```

## License
This project is licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.

## Contributing 

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed under the terms of both the Apache License, Version 2.0 and the MIT license without any additional terms or conditions.