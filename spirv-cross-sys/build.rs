use std::env;

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
        .includes(&["native/SPIRV-Cross", "native/SPIRV-CROSS/include"])
        .file("native/SPIRV-Cross/spirv_cfg.cpp")
        .file("native/SPIRV-Cross/spirv_cpp.cpp")
        .file("native/SPIRV-Cross/spirv_cross.cpp")
        .file("native/SPIRV-Cross/spirv_cross_parsed_ir.cpp")
        .file("native/SPIRV-Cross/spirv_cross_util.cpp")
        .file("native/SPIRV-Cross/spirv_glsl.cpp")
        .file("native/SPIRV-Cross/spirv_hlsl.cpp")
        .file("native/SPIRV-Cross/spirv_msl.cpp")
        .file("native/SPIRV-Cross/spirv_parser.cpp")
        .file("native/SPIRV-Cross/spirv_reflect.cpp")
        .file("native/spirv_cross_c_ext_rs.cpp");

    if cfg!(feature = "glsl") {
        spvc_build.define("SPIRV_CROSS_C_API_GLSL", "1");
    }

    if cfg!(feature = "hlsl") {
        spvc_build.define("SPIRV_CROSS_C_API_HLSL", "1");
    }

    if cfg!(feature = "msl") {
        spvc_build.define("SPIRV_CROSS_C_API_MSL", "1");
    }

    if cfg!(feature = "cpp") {
        spvc_build.define("SPIRV_CROSS_C_API_CPP", "1");
    }

    if cfg!(feature = "json") {
        spvc_build.define("SPIRV_CROSS_C_API_JSON", "1");
    }

    spvc_build.compile("spirv-cross");
    println!("cargo:rustc-link-lib=static=spirv-cross");
}
