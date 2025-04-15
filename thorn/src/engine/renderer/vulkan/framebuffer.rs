use crate::prelude::*;
use ash::vk;

use super::{logical_device::LogicalDevice, renderpass::Renderpass, swapchain::Swapchain};


#[derive(Clone)]
pub struct FrameBuffer
{
    pub frame_buffer: vk::Framebuffer,
    pub index: usize,
    swapchain: Layer<Swapchain>,
    renderpass: Layer<Renderpass>,
    device: Layer<LogicalDevice>,
}


impl FrameBuffer
{
    pub fn new(reg: &LayerReg<()>, index: usize) -> ThResult<Self>
    {
        let me = Self {
            index,
            frame_buffer: Self::generate(
                &reg.get_unchecked(),
                &reg.get_unchecked(),
                &reg.get_unchecked(),
                index,
            )?,
            swapchain: reg.get_unchecked(),
            renderpass: reg.get_unchecked(),
            device: reg.get_unchecked(),
        };

        log::info!("Vulkan framebuffer {index} created");
        Ok(me)
    }

    pub fn regenerate(&mut self) -> ThResult<()>
    {
        let frame_buffer =
            Self::generate(&self.swapchain, &self.renderpass, &self.device, self.index)?;
        self.destroy_fb();
        self.frame_buffer = frame_buffer;
        Ok(())
    }

    fn generate(
        sc: &Layer<Swapchain>,
        rp: &Layer<Renderpass>,
        device: &Layer<LogicalDevice>,
        index: usize,
    ) -> ThResult<vk::Framebuffer>
    {
        let swapchain = sc.read().unwrap();
        let renderpass = rp.read().unwrap();

        let attachments = [
            swapchain.views[0],
            swapchain.depth_buffer.view.unwrap_or_default(),
        ];

        let create_info = vk::FramebufferCreateInfo::default()
            .attachments(&attachments)
            .layers(1)
            .render_pass(renderpass.renderpass)
            .width(renderpass.width)
            .height(renderpass.height);

        let frame_buffer = unsafe {
            device
                .read()
                .unwrap()
                .logical_device
                .create_framebuffer(&create_info, None)
        }?;

        Ok(frame_buffer)
    }

    fn destroy_fb(&self)
    {
        unsafe {
            self.device
                .read()
                .unwrap()
                .logical_device
                .destroy_framebuffer(self.frame_buffer, None);
        }
    }

    pub fn destroy(&mut self)
    {
        self.destroy_fb();
        log::info!("Framebuffer {} destroyed", self.index);
    }
}


pub struct FrameBuffers
{
    pub buffers: Vec<FrameBuffer>,
}


impl FrameBuffers
{
    pub fn new(reg: &LayerReg<()>) -> ThResult<Self>
    {
        let count = reg_read!(reg, Swapchain).images.len();

        let mut buffers = Vec::with_capacity(count);
        for i in 0..count
        {
            buffers.push(FrameBuffer::new(reg, i)?);
        }

        let me = Self { buffers };
        log::info!("Vulkan framebuffers created");
        Ok(me)
    }

    pub fn regenerate(&mut self) -> ThResult<()>
    {
        for b in &mut self.buffers
        {
            b.regenerate()?;
        }

        log::info!("Vulkan framebuffers regenerated");
        Ok(())
    }

    pub fn destroy(&mut self)
    {
        while let Some(mut b) = self.buffers.pop()
        {
            b.destroy();
        }

        log::info!("Vulkan framebuffers destroyed");
    }
}


impl LayerDispatch<()> for FrameBuffers {}
