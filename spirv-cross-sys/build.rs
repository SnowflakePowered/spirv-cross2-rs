use std::env;
use std::path::PathBuf;

pub fn main() {
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping SPIRV-Cross native build for docs.rs.");
        return;
    }

    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_cross_c.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_cross_c.h");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_cfg.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_cpp.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_cross.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_cross_parsed_ir.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_cross_util.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_glsl.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_hlsl.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_msl.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_parser.cpp");
    println!("cargo:rerun-if-changed=native/SPIRV-Cross/spirv_reflect.cpp");
    println!("cargo:rerun-if-changed=native/spirv_cross_c_ext_rs.cpp");

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
        // spirv_cross_c_ext_rs.cpp #includes spirv_cross_c.cpp
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
        spvc_build.define("SPIRV_CROSS_C_API_REFLECT", "1");
    }

    if is_wasm_unknown() {
        configure_wasm(&mut spvc_build);
    }

    spvc_build.compile("spirv-cross");
    println!("cargo:rustc-link-lib=static=spirv-cross");

    if is_wasm_unknown() {
        link_wasm_sysroot();
    }
}

fn is_wasm_unknown() -> bool {
    env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32")
        && env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("unknown")
}

fn wasi_sysroot() -> PathBuf {
    println!("cargo:rerun-if-env-changed=WASI_SDK_PATH");
    let sdk = env::var("WASI_SDK_PATH")
        .expect("WASI_SDK_PATH must point to a wasi-sdk install to build for wasm32-unknown-unknown");
    PathBuf::from(sdk).join("share").join("wasi-sysroot")
}

/// wasm32-unknown-unknown has no libc or libc++, so build against wasi-sdk's sysroot.
fn configure_wasm(build: &mut cc::Build) {
    let sysroot = wasi_sysroot();
    let sdk = sysroot.parent().unwrap().parent().unwrap();
    let exe = if cfg!(windows) { ".exe" } else { "" };
    build
        .cpp_link_stdlib(None)
        .compiler(sdk.join("bin").join(format!("clang++{exe}")))
        .archiver(sdk.join("bin").join(format!("llvm-ar{exe}")))
        .flag("--target=wasm32-wasip1")
        .flag(&format!("--sysroot={}", sysroot.display()))
        .flag("-fwasm-exceptions")
        .flag("-fno-rtti")
        .flag("-mno-atomics")
        .define("_LIBCPP_HAS_NO_THREADS", None)
        .define("_WASI_EMULATED_SIGNAL", None)
        .include(sysroot.join("include").join("wasm32-wasip1"))
        .include(sysroot.join("include").join("c++").join("v1"));
}

fn link_wasm_sysroot() {
    let lib = wasi_sysroot().join("lib").join("wasm32-wasip1");
    println!("cargo:rustc-link-search=native={}", lib.join("eh").display());
    println!("cargo:rustc-link-search=native={}", lib.display());
    println!("cargo:rustc-link-lib=static=c++");
    println!("cargo:rustc-link-lib=static=c++abi");
    println!("cargo:rustc-link-lib=static=unwind");
    println!("cargo:rustc-link-lib=static=c");
}
