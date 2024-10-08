use std::{env, fs::DirEntry, io::Result};

use std::{
    env::var,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn main() {
    // This tells cargo to rerun this script if something in /assets/ changes.
    println!("cargo:rerun-if-changed=assets/*");

    let out_dir = env::var("OUT_DIR").expect("Var: OUT_DIR Not found!");
    copy_dir_all("assets/", out_dir).expect("Failed to copy to resource Folder");

    if !should_skip_shader_compilation() {
        println!("Compiling shaders");
        compile_shaders(&get_shader_source_dir_path());
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn should_skip_shader_compilation() -> bool {
    var("SKIP_SHADER_COMPILATION")
        .map(|var| var.parse::<bool>().unwrap_or(false))
        .unwrap_or(false)
}

fn compile_shaders(dir: &Path) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                compile_shaders(&path);
            } else {
                compile_shader(&entry);
            }
        }
    }
}

fn compile_shader(file: &DirEntry) {
    if file.file_type().unwrap().is_file() {
        let path = file.path();
        if let Some(ext) = path.extension() {
            if ext == "spv" || ext == "wgsl" {
                return;
            }
        }

        let name = path.file_name().unwrap().to_str().unwrap();
        let output_name = format!("{}.spv", &name);
        println!("Found file {:?}.\nCompiling...", path.as_os_str());

        let result = Command::new("glslangValidator")
            .current_dir(path.parent().expect("Shader File has not Parent!"))
            .env("--target-env", "vulkan1.3")
            .arg("-V")
            .arg(&path)
            .arg("-o")
            .arg(output_name)
            .output();

        handle_program_result(result);
    }
}

fn get_shader_source_dir_path() -> PathBuf {
    let path = get_root_path().join("assets/shaders");
    println!("Shader source directory: {:?}", path.as_os_str());
    path
}

fn get_root_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn handle_program_result(result: Result<Output>) {
    match result {
        Ok(output) => {
            if output.status.success() {
                println!("Shader compilation succedeed.");
                print!(
                    "stdout: {}",
                    String::from_utf8(output.stdout)
                        .unwrap_or("Failed to print program stdout".to_string())
                );
            } else {
                eprintln!("Shader compilation failed. Status: {}", output.status);
                eprint!(
                    "stdout: {}",
                    String::from_utf8(output.stdout)
                        .unwrap_or("Failed to print program stdout".to_string())
                );
                eprint!(
                    "stderr: {}",
                    String::from_utf8(output.stderr)
                        .unwrap_or("Failed to print program stderr".to_string())
                );
                eprintln!("Shader compilation failed. Status: {}", output.status);
            }
        }
        Err(error) => {
            eprintln!("Failed to compile shader. Cause: {}", error);
        }
    }
}
