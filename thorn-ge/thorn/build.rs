#![feature(exit_status_error)]


use std::{
    env::var,
    path::{Path, PathBuf},
    process::Command,
};

use shader_utils::{ShaderBundle, ShaderObj};


type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;


fn main() -> Result<(), Box<dyn std::error::Error>>
{
    println!("cargo:rerun-if-changed=../shaders");
    let target_dir = var("OUT_DIR").unwrap();

    let out = PathBuf::from(format!("{target_dir}/../../.."))
        .canonicalize()?
        .join("shaders");

    let list = out.join("shader_list.txt");

    let crates = PathBuf::from("../shaders/shaders".to_string()).canonicalize()?;
    let compiler = PathBuf::from("../../tools/thorn-shader-compiler".to_string()).canonicalize()?;
    let bundle_path = "../shaders/thorn.shader_bundle";

    println!("Compiler:      {compiler:?}");
    println!("List Dir:      {list:?}");
    println!("Shader Crates: {crates:?}");

    run_shader_compiler(compiler, crates, target_dir)?.save(bundle_path)?;

    Ok(())
}


pub fn run_shader_compiler(
    compiler_path: impl AsRef<Path>,
    shader_crates: impl AsRef<Path>,
    temp_dir: impl AsRef<Path>,
) -> AnyResult<ShaderBundle>
{
    let shader_list = temp_dir.as_ref().join("shader-list.txt");

    Command::new("cargo")
        .env_remove("CARGO")
        .env_remove("CARGO_HOME")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTFLAGS")
        .env_remove("RUSTC")
        .current_dir(compiler_path)
        .env("RUSTUP_TOOLCHAIN", "nightly-2023-05-27")
        .env("THORN_SHADER_OUT", temp_dir.as_ref())
        .env("THORN_SHADER_LIST_OUT", &shader_list)
        .arg("run")
        .arg("-vv")
        .arg(shader_crates.as_ref())
        .stdout(std::io::stdout())
        .spawn()?
        .wait()
        .unwrap()
        .exit_ok()?;

    let shader_paths = std::fs::read_to_string(shader_list)?;

    let mut bundle = ShaderBundle::new();
    for path in shader_paths.lines()
    {
        bundle.add(ShaderObj::from_file(path)?);
    }

    Ok(bundle)
}
