mod instance;

use super::api::RenderAPI;
use crate::prelude::*;
use winit::raw_window_handle::RawWindowHandle;


pub(crate) struct VulkanRenderer
{
    reg: LayerReg<()>,
}


impl VulkanRenderer
{
    pub(crate) fn new() -> Self
    {
        Self {
            reg: LayerReg::new(),
        }
    }
}


impl RenderAPI for VulkanRenderer
{
    fn initialize(&mut self, _rwh: RawWindowHandle) -> crate::prelude::ThResult<()>
    {
        if self.reg.get::<instance::Instance>().is_none()
        {
            self.reg.insert(instance::Instance::new()?);
            log::info!("Vulkan instance created");
        }

        log::info!("Vulkan Renderer Initialized");
        Ok(())
    }

    fn destroy(&mut self)
    {
        log::info!("Vulkan Renderer Destroyed");
    }

    fn frame_prepare(&mut self) {}
    fn frame_render(&mut self) {}
    fn frame_finish(&mut self) {}
}
