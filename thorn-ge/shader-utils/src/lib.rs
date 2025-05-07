use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Error, Result},
    path::Path,
};


pub type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;


#[derive(Clone, Serialize, Deserialize)]
pub struct ShaderObj
{
    name: String,
    data: Vec<u8>,
}


impl ShaderObj
{
    pub fn new(name: String, data: Vec<u8>) -> Self
    {
        Self { name, data }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self>
    {
        let name = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let data = std::fs::read(path)?;
        Ok(Self { name, data })
    }

    pub fn data(&self) -> &[u8]
    {
        &self.data
    }

    pub fn name(&self) -> &str
    {
        &self.name
    }
}


#[derive(Default, Serialize, Deserialize)]
pub struct ShaderBundle
{
    shaders: HashMap<String, ShaderObj>,
}


impl ShaderBundle
{
    pub fn new() -> Self
    {
        Self::default()
    }

    pub fn add(&mut self, shader: ShaderObj)
    {
        self.shaders.insert(shader.name.clone(), shader);
    }

    pub fn get(&self, name: &str) -> Option<&ShaderObj>
    {
        self.shaders.get(name)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()>
    {
        let mut file = File::create(path)?;
        let _ = bincode::serde::encode_into_std_write(self, &mut file, bincode::config::standard())
            .map_err(Error::other)?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self>
    {
        let mut file = File::open(path)?;
        bincode::serde::decode_from_std_read(&mut file, bincode::config::standard())
            .map_err(Error::other)
    }
}
