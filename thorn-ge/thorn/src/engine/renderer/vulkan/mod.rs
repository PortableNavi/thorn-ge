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


use super::api::{FrameStatus, RenderAPI};
use crate::{math::*, prelude::*, reg_inspect};
use ash::vk;
use command_buffer::CommandBuffers;
use command_pool::CommandPools;
use framebuffer::FrameBuffers;
use instance::Instance;
use logical_device::LogicalDevice;
use physical_device::PhysicalDevice;
use renderpass::Renderpass;
use surface::Surface;
use swapchain::Swapchain;
use sync::VkSync;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};


// 10 FPS
const FENCE_WAIT: u64 = 100_000_000;


pub(crate) struct VulkanRenderer
{
    reg: LayerReg<()>,
    surface_width: u32,
    surface_height: u32,
    buffered_frames: u32,
    frame: usize,
    prev_frame: usize,
    prev_image_index: usize,
    image_index: usize,
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
            prev_frame: 0,
            prev_image_index: 0,
            image_index: 0,
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

    fn destroy(&mut self)
    {
        // Wait until we are ready to shut down.
        reg_inspect!(self.reg, d=LogicalDevice => unsafe {
            let _ = d.logical_device.device_wait_idle();
        });

        reg_inspect!(self.reg, l=FrameBuffers => l.destroy());
        reg_inspect!(self.reg, l=Renderpass => l.destroy());
        reg_inspect!(self.reg, l=CommandBuffers => l.destroy());
        reg_inspect!(self.reg, l=CommandPools => l.destroy());
        reg_inspect!(self.reg, l=Swapchain => l.destroy());
        reg_inspect!(self.reg, l=VkSync => l.destroy());
        reg_inspect!(self.reg, l=LogicalDevice => l.destroy());
        reg_inspect!(self.reg, l=PhysicalDevice => l.destroy());
        reg_inspect!(self.reg, l=Surface => l.destroy());
        reg_inspect!(self.reg, l=Instance => l.destroy());

        log::info!("Vulkan Renderer Destroyed");
    }

    fn frame_prepare(&mut self) -> FrameStatus
    {
        let mut swapchain_dirty = false;
        reg_inspect!(self.reg, swapchain = Swapchain => {

            // Fetch current frame index.
            self.prev_frame = self.frame;
            self.frame = swapchain.current_frame;

            if swapchain.is_dirty()
            {
                reg_inspect!(self.reg, d=LogicalDevice => unsafe {
                   if let Err(e) = d.logical_device.device_wait_idle()
                   {
                       log::error!("Failed to wait until device is idle");
                       return FrameStatus::Failed;
                   }
                });
            }

            // Recreate swapchain if necessary
            if let Ok(result) = swapchain.recreate_if_dirty()
            {
                swapchain_dirty = result;
            }
            else
            {
                log::error!("Swapchain recreation failed");
                return FrameStatus::Failed;
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
                if let Err(e) = fb.regenerate()
                {
                    log::error!("Framebuffer regeneration failed: {e}");
                    return FrameStatus::Failed;
                }
            });
        }

        // Wait for the current image to be available...
        reg_inspect!(self.reg, sync=VkSync => {
            if let Err(e) = sync.frame_fences[self.frame].wait(FENCE_WAIT)
            {
                log::error!("Fence for this frame timed out, skipping");
                return FrameStatus::Failed;
            }
        });


        // Setup new image for this frame
        reg_inspect!(self.reg, swapchain=Swapchain => {
            match swapchain.get_next_image_index(FENCE_WAIT, self.frame)
            {
                Ok(index) => {
                    self.prev_image_index = self.image_index;
                    self.image_index = index as usize;
                }

                Err(e) => {
                    log::error!("Failed to aquire new image index to render to...");
                     return FrameStatus::Failed;
                }
            }
        });

        reg_inspect!(self.reg, buffers=CommandBuffers => {
            let cbuffer = &mut buffers.graphics[self.image_index];
            cbuffer.reset();
            cbuffer.begin(false, false);

            let viewport = vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(self.surface_width as _)
                .height(self.surface_height as _)
                .min_depth(0.0)
                .max_depth(1.0);

            let scissor = vk::Rect2D::default()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(vk::Extent2D { width: self.surface_width, height: self.surface_height });

            cbuffer.set_viewport(viewport, scissor);
        });

        reg_inspect!(self.reg, pass=Renderpass => {
            pass.begin(self.image_index);
        });

        FrameStatus::Success
    }

    fn frame_render(&mut self) -> FrameStatus
    {
        reg_inspect!(self.reg, pass=Renderpass => {
            pass.clear_color += Vec3::new(0.0, 0.001, 0.0);

            if pass.clear_color[G] > 1.0
            {
                pass.clear_color[G] = 0.0;
            }
        });

        FrameStatus::Success
    }

    fn frame_finish(&mut self) -> FrameStatus
    {
        reg_inspect!(self.reg, pass=Renderpass => {
            pass.end(self.image_index);
        });

        reg_inspect!(self.reg, buffers=CommandBuffers => {
            let mut cbuffer = &mut buffers.graphics[self.image_index];
            cbuffer.end();
        });

        reg_inspect!(self.reg, sync=VkSync => {
            sync.image_frames[self.image_index] = self.frame;
            //sync.frame_fences[self.frame].wait(FENCE_WAIT);
            sync.frame_fences[self.frame].reset();

            let wait = sync.image_available[self.frame];
            let signal = sync.queue_complete[self.frame];
            let fence = sync.frame_fences[self.frame].fence;
            let queue = reg_read!(self.reg, LogicalDevice).graphics_queue.unwrap();

            reg_inspect!(self.reg, buffers=CommandBuffers => {
               if let Err(e) = buffers.graphics[self.image_index].submit(queue, wait, signal, fence)
               {
                   log::error!("Failed to submit buffer to queue {e}");
                   return FrameStatus::Failed;
               }
            });
        });

        reg_inspect!(self.reg, swapchain=Swapchain => {
           if let Err(e) = swapchain.present(self.image_index, self.frame)
           {
               log::error!("Failed to present swapchain {e}");
               return FrameStatus::Failed;
           }
        });


        FrameStatus::Success
    }

    fn surface_size_changed(&mut self, w: u32, h: u32) -> ThResult<()>
    {
        reg_inspect!(self.reg, sc = Swapchain => {
            sc.mark_dirty(w, h);
        });

        Ok(())
    }
}
