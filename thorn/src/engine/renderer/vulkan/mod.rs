mod instance;

use super::api::RenderAPI;
use crate::prelude::*;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};


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
    fn initialize(
        &mut self,
        rdh: RawDisplayHandle,
        rwh: RawWindowHandle,
    ) -> crate::prelude::ThResult<()>
    {
        if self.reg.get::<instance::Instance>().is_none()
        {
            self.reg.insert(instance::Instance::new(rdh, rwh, &[])?);
        }

        log::info!("Vulkan Renderer Initialized");
        Ok(())
    }

    fn destroy(&mut self)
    {
        if let Some(instance) = self.reg.get::<instance::Instance>()
        {
            instance.write().unwrap().destroy();
        }

        log::info!("Vulkan Renderer Destroyed");
    }

    fn frame_prepare(&mut self) {}
    fn frame_render(&mut self) {}
    fn frame_finish(&mut self) {}
}
