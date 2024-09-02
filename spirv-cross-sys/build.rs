use glob::glob;
use std::env;

pub fn add_subdirectory(build: &mut cc::Build) {
    for entry in glob(&*format!("native/SPIRV-Cross/*.cpp")).expect("failed to read glob") {
        if let Ok(path) = entry {
            build.file(path);
        }
    }

    for entry in glob(&*format!("native/SPIRV-Cross/*.c")).expect("failed to read glob") {
        if let Ok(path) = entry {
            build.file(path);
        }
    }
}

pub fn main() {
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping SPIRV-Cross native build for docs.rs.");
        return;
    }

    let mut spvc_build = cc::Build::new();
    spvc_build
        .cpp(true)
        .std("c++14")
        .define("SPIRV_CROSS_CLI", "OFF")
        .define("SPIRV_CROSS_C_API_GLSL", "1")
        .define("SPIRV_CROSS_C_API_HLSL", "1")
        .define("SPIRV_CROSS_C_API_MSL", "1")
        .define("SPIRV_CROSS_C_API_CPP", "1")
        .define("SPIRV_CROSS_C_API_REFLECT", "1")
        .includes(&["native/SPIRV-Cross", "native/SPIRV-CROSS/include"])
        .file("native/spvc_set.cpp");

    add_subdirectory(&mut spvc_build);
    spvc_build.compile("spirv-cross");
    println!("cargo:rustc-link-lib=static=spirv-cross");
}
