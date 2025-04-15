// Until the basic renderer is finished...
#![allow(unused)]


mod command_buffer;
mod command_pool;
mod framebuffer;
mod image;
mod instance;
mod logical_device;
mod physical_device;
mod renderpass;
mod surface;
mod swapchain;
mod sync;


use super::api::RenderAPI;
use crate::{prelude::*, reg_inspect};
use ash::vk::{CommandBuffer, RenderPass};
use command_buffer::CommandBuffers;
use command_pool::CommandPools;
use framebuffer::{FrameBuffer, FrameBuffers};
use instance::Instance;
use logical_device::LogicalDevice;
use physical_device::PhysicalDevice;
use renderpass::Renderpass;
use surface::Surface;
use swapchain::Swapchain;
use sync::VkSync;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};


pub(crate) struct VulkanRenderer
{
    reg: LayerReg<()>,
    surface_width: u32,
    surface_height: u32,
    buffered_frames: u32,
    frame: usize,
}


impl VulkanRenderer
{
    pub(crate) fn new() -> Self
    {
        Self {
            reg: LayerReg::new(),
            surface_width: 0,
            surface_height: 0,
            buffered_frames: 3,
            frame: 0,
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

        if self.reg.get::<VkSync>().is_none()
        {
            let sync = VkSync::new(&self.reg)?;
            self.reg.insert(sync);
        }

        if self.reg.get::<Swapchain>().is_none()
        {
            let swapchain = Swapchain::new(&self.reg, w, h)?;

            self.buffered_frames = swapchain.max_buffered_frames;

            log::info!("Total Buffered Frames: {}", self.buffered_frames);

            reg_inspect!(self.reg, sync=VkSync => {
                sync.update_for_image_count(self.buffered_frames)?;
            });

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

        if self.reg.get::<FrameBuffers>().is_none()
        {
            let fbuffers = FrameBuffers::new(&self.reg)?;
            self.reg.insert(fbuffers);

            reg_inspect!(self.reg, pass = Renderpass => {
                pass.frame_buffer = self.reg.get();
            });
        }

        log::info!("Vulkan Renderer Initialized");
        Ok(())
    }

    //TODO: Use the new reg_inspect macro...
    fn destroy(&mut self)
    {
        // Wait until we are ready to shut down.
        reg_inspect!(self.reg, d=LogicalDevice => unsafe {
            let _ = d.logical_device.device_wait_idle();
        });

        if let Some(fbuffers) = self.reg.get::<FrameBuffers>()
        {
            fbuffers.write().unwrap().destroy();
        }

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

        reg_inspect!(self.reg, sync=VkSync => sync.destroy());

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
        let mut swapchain_dirty = false;
        reg_inspect!(self.reg, swapchain = Swapchain => {

            // Fetch current frame index.
            self.frame = swapchain.current_frame;

            // Recreate swapchain if necessary
            if let Ok(result) = swapchain.recreate_if_dirty()
            {
                swapchain_dirty = result;
            }

            self.surface_width = swapchain.width;
            self.surface_height = swapchain.height;
            self.buffered_frames = swapchain.max_buffered_frames;
        });

        // Set new framebuffer dimensions
        reg_inspect!(self.reg, pass = Renderpass => {
            pass.width = self.surface_width;
            pass.height = self.surface_height;
        });

        // Update the Framebuffer if the swapchain was dirty
        if swapchain_dirty
        {
            reg_inspect!(self.reg, fb = FrameBuffers => {
                let _ = fb.regenerate();
            });
        }

        // Begin the frame...
        begin_frame(&self.reg, 0);
    }

    fn frame_render(&mut self) {}

    fn frame_finish(&mut self) {}

    fn surface_size_changed(&mut self, w: u32, h: u32) -> ThResult<()>
    {
        reg_inspect!(self.reg, sc = Swapchain => {
            sc.mark_dirty(w, h);
        });

        Ok(())
    }
}

fn begin_frame(reg: &LayerReg<()>, index: usize)
{
    // reg_inspect!(reg, cbuffers=CommandBuffers => {
    //     cbuffers.graphics[index].begin(false, false);
    // });

    // reg_inspect!(reg, pass=Renderpass => {
    //     pass.begin(index);
    // });
}


fn end_frame(reg: &LayerReg<()>, index: usize)
{
    reg_inspect!(reg, pass=Renderpass => {
        pass.end(index);
    });

    reg_inspect!(reg, cbuffers=CommandBuffers => {
        cbuffers.graphics[index].end();
    });
}
