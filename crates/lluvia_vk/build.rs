use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let resources_dir = Path::new(&manifest_dir).join("resources");

    if !resources_dir.exists() {
        return;
    }

    // Tell cargo to rerun this script if anything in resources/ changes
    println!("cargo:rerun-if-changed=resources");

    compile_shaders(&resources_dir);
}

fn compile_shaders(dir: &Path) {
    if !dir.is_dir() {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let include_dir = Path::new(&manifest_dir).join("resources/glsl");

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            compile_shaders(&path);
        } else if path.extension().is_some_and(|ext| ext == "comp") {
            let mut spv_path = path.clone();
            spv_path.set_extension("spv");

            let status = Command::new("glslc")
                .arg("-I")
                .arg(&include_dir)
                .arg(&path)
                .arg("-o")
                .arg(&spv_path)
                .status()
                .expect("Failed to execute glslc. Is it installed?");

            if !status.success() {
                panic!("Failed to compile shader: {:?}", path);
            }
        }
    }
}
