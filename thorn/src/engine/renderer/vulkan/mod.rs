// Until the basic renderer is finished...
#![allow(unused)]


mod command_buffer;
mod command_pool;
mod image;
mod instance;
mod logical_device;
mod physical_device;
mod renderpass;
mod surface;
mod swapchain;


use super::api::RenderAPI;
use crate::prelude::*;
use command_buffer::CommandBuffers;
use command_pool::CommandPools;
use instance::Instance;
use logical_device::LogicalDevice;
use physical_device::PhysicalDevice;
use renderpass::Renderpass;
use surface::Surface;
use swapchain::Swapchain;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};


pub(crate) struct VulkanRenderer
{
    reg: LayerReg<()>,
    surface_width: u32,
    surface_height: u32,
}


impl VulkanRenderer
{
    pub(crate) fn new() -> Self
    {
        Self {
            reg: LayerReg::new(),
            surface_width: 0,
            surface_height: 0,
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
        self.surface_width = w;
        self.surface_height = h;

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

        if self.reg.get::<CommandPools>().is_none()
        {
            let pool = CommandPools::new(&self.reg)?;
            self.reg.insert(pool);
        }

        if self.reg.get::<CommandBuffers>().is_none()
        {
            let cbuffer = CommandBuffers::new(&self.reg)?;
            self.reg.insert(cbuffer);
        }

        if self.reg.get::<Renderpass>().is_none()
        {
            let pass = Renderpass::new_default(&self.reg, w, h)?;
            self.reg.insert(pass);
        }

        log::info!("Vulkan Renderer Initialized");
        Ok(())
    }

    fn destroy(&mut self)
    {
        if let Some(renderpass) = self.reg.get::<Renderpass>()
        {
            renderpass.write().unwrap().destroy();
        }

        if let Some(cbuffers) = self.reg.get::<CommandBuffers>()
        {
            cbuffers.write().unwrap().destroy();
        }

        if let Some(pools) = self.reg.get::<CommandPools>()
        {
            pools.write().unwrap().destroy();
        }

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
            let mut swapchain = swapchain.write().unwrap();
            let _ = swapchain.recreate_if_dirty();

            self.surface_width = swapchain.width;
            self.surface_height = swapchain.height;

            self.reg.get::<Renderpass>().inspect(|p| {
                let mut pass = p.write().unwrap();
                pass.width = swapchain.width as f32;
                pass.height = swapchain.height as f32;

                //pass.begin(); //TODO: Uncomment once command buffers and a framebuffer are implemented
            });
        }
    }

    fn frame_render(&mut self) {}

    fn frame_finish(&mut self)
    {
        if let Some(pass) = self.reg.get::<Renderpass>()
        {
            //pass.write().unwrap().end(); //TODO: Uncomment once command buffers and a framebuffer are implemented
        }
    }

    fn surface_size_changed(&mut self, w: u32, h: u32) -> ThResult<()>
    {
        if let Some(swapchain) = self.reg.get::<Swapchain>()
        {
            swapchain.write().unwrap().mark_dirty(w, h);
        }

        Ok(())
    }
}
