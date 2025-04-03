mod instance;
mod physical_device;
mod surface;


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

        if self.reg.get::<physical_device::PhysicalDevice>().is_none()
        {
            let device = physical_device::PhysicalDevice::new(&mut self.reg)?;
            self.reg.insert(device);
        }

        if self.reg.get::<surface::Surface>().is_none()
        {
            let surf = surface::Surface::new(&mut self.reg)?;
            self.reg.insert(surf);
        }

        log::info!("Vulkan Renderer Initialized");
        Ok(())
    }

    fn destroy(&mut self)
    {
        if let Some(physical_device) = self.reg.get::<physical_device::PhysicalDevice>()
        {
            physical_device.write().unwrap().destroy();
        }

        if let Some(surface) = self.reg.get::<surface::Surface>()
        {
            surface.write().unwrap().destroy();
        }

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
