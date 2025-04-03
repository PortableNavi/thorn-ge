use crate::prelude::*;
use ash::vk;


pub struct PhysicalDevice
{
    pub device: vk::PhysicalDevice,
}


impl PhysicalDevice
{
    pub fn new(reg: &mut LayerReg<()>) -> ThResult<Self>
    {
        Err(ThError::RendererError("WIP".into()))
    }

    pub fn destroy(&mut self) {}
}


impl LayerDispatch<()> for PhysicalDevice {}
