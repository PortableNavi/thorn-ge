use spirv_builder::{MetadataPrintout, SpirvBuilder};
use std::env::var;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;


const TARGET: &str = "spirv-unknown-vulkan1.2";
const DEFAULT_OUT: &str = "shaders";
const DEFAULT_LIST_NAME: &str = "shader-list.txt";


type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;


fn main()
{
    let args = std::env::args().collect::<Vec<_>>();

    let crates = match args.get(1)
    {
        Some(crates) => crates,
        None =>
        {
            println!("Error: No input directory given");
            exit(1);
        }
    };

    let target = var("THORN_SHADER_TARGET").unwrap_or(TARGET.into());
    let out = var("THORN_SHADER_OUT").unwrap_or(DEFAULT_OUT.into());
    let list_name = var("THORN_SHADER_LIST_OUT").unwrap_or(DEFAULT_LIST_NAME.into());

    std::fs::create_dir_all(&out).expect("Failed to create output dir");

    if let Err(_) = compile_shaders(crates, out, list_name, &target)
    {
        println!("Error: Failed to compile shaders");
        exit(1);
    }
}


fn compile_shaders(
    crates: impl AsRef<Path>,
    out: impl AsRef<Path>,
    shader_list_name: impl AsRef<Path>,
    target: &str,
) -> Result<()>
{
    let shader_crates = std::fs::read_dir(crates)?;
    let mut shader_obj_paths = vec![];

    for shader in shader_crates
    {
        let shader = shader?.path().to_path_buf();

        let shader_name = shader
            .as_path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();


        println!("--- Compiling Shader: {shader_name:?}");

        if let Ok(obj) = compile_shader(&shader, target)
        {
            println!("--- Finished Compiling Shader: {shader_name:?}");

            let shader_file = out.as_ref().join(format!("{}.spv", shader_name));
            std::fs::copy(obj, &shader_file)?;
            let shader_file = shader_file.canonicalize()?;
            shader_obj_paths.push(shader_file.to_string_lossy().to_string());
        }
        else
        {
            println!("--- Failed Compiling Shader: {shader_name:?}");
        }
    }

    if shader_obj_paths.is_empty()
    {
        println!("Error: No Shader Crates Found");
        exit(1);
    }

    output_shader_list(shader_list_name, &shader_obj_paths)
}


fn output_shader_list(list_path: impl AsRef<Path>, paths: &[String]) -> Result<()>
{
    let mut file = std::fs::File::create(list_path)?;

    for path in paths
    {
        writeln!(&mut file, "{path}")?;
    }

    Ok(())
}


fn compile_shader(path: impl AsRef<Path>, target: &str) -> Result<PathBuf>
{
    let module = SpirvBuilder::new(path, target)
        .print_metadata(MetadataPrintout::None)
        .build()?
        .module
        .unwrap_single()
        .to_path_buf();

    Ok(module)
}
