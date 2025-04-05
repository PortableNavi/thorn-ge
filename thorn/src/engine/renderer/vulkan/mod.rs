// Until the basic renderer is finished...
#![allow(unused)]


mod image;
mod instance;
mod logical_device;
mod physical_device;
mod surface;
mod swapchain;


use super::api::RenderAPI;
use crate::prelude::*;
use instance::Instance;
use logical_device::LogicalDevice;
use physical_device::PhysicalDevice;
use surface::Surface;
use swapchain::Swapchain;
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
        w: u32,
        h: u32,
    ) -> crate::prelude::ThResult<()>
    {
        if self.reg.get::<Instance>().is_none()
        {
            self.reg.insert(Instance::new(rdh, rwh, &[])?);
        }

        if self.reg.get::<Surface>().is_none()
        {
            let surf = Surface::new(&self.reg)?;
            self.reg.insert(surf);
        }

        if self.reg.get::<PhysicalDevice>().is_none()
        {
            let device = PhysicalDevice::new(&self.reg)?;
            self.reg.insert(device);
        }

        if self.reg.get::<LogicalDevice>().is_none()
        {
            let device = LogicalDevice::new(&self.reg)?;
            self.reg.insert(device);
        }

        if self.reg.get::<Swapchain>().is_none()
        {
            let swapchain = Swapchain::new(&self.reg, w, h)?;
            self.reg.insert(swapchain);
        }

        log::info!("Vulkan Renderer Initialized");
        Ok(())
    }

    fn destroy(&mut self)
    {
        if let Some(swapchain) = self.reg.get::<Swapchain>()
        {
            swapchain.write().unwrap().destroy();
        }

        if let Some(logical_device) = self.reg.get::<LogicalDevice>()
        {
            logical_device.write().unwrap().destroy();
        }

        if let Some(physical_device) = self.reg.get::<PhysicalDevice>()
        {
            physical_device.write().unwrap().destroy();
        }

        if let Some(surface) = self.reg.get::<Surface>()
        {
            surface.write().unwrap().destroy();
        }

        if let Some(instance) = self.reg.get::<Instance>()
        {
            instance.write().unwrap().destroy();
        }

        log::info!("Vulkan Renderer Destroyed");
    }

    fn frame_prepare(&mut self)
    {
        if let Some(swapchain) = self.reg.get::<Swapchain>()
        {
            let _ = swapchain.write().unwrap().recreate_if_dirty();
        }
    }

    fn frame_render(&mut self) {}
    fn frame_finish(&mut self) {}

    fn surface_size_changed(&mut self, w: u32, h: u32) -> ThResult<()>
    {
        if let Some(swapchain) = self.reg.get::<Swapchain>()
        {
            swapchain.write().unwrap().mark_dirty(w, h);
        }

        Ok(())
    }
}
