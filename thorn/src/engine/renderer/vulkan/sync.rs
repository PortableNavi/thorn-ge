use crate::{layer_inspect, layer_read, prelude::*};
use ash::vk;

use super::logical_device::LogicalDevice;


#[derive(Clone)]
pub struct VkFence
{
    pub fence: vk::Fence,

    device: Layer<LogicalDevice>,
    signaled: bool, // Wait on signaled = false
}


impl VkFence
{
    pub fn new(device: Layer<LogicalDevice>, signaled: bool) -> ThResult<Self>
    {
        let mut me = Self {
            fence: vk::Fence::null(),
            device: device.clone(),
            signaled,
        };

        let create_info = vk::FenceCreateInfo::default().flags(
            either!(signaled => vk::FenceCreateFlags::SIGNALED; vk::FenceCreateFlags::empty()),
        );

        layer_inspect!(d=device => {
            me.fence = unsafe {d.logical_device.create_fence(&create_info, None)}?;
        });

        Ok(me)
    }

    pub fn signaled(&self) -> bool
    {
        self.signaled
    }

    pub fn wait(&mut self, timeout_ns: u64) -> ThResult<()>
    {
        if self.signaled
        {
            return Ok(());
        }

        layer_inspect!(d=self.device => unsafe {
            d.logical_device.wait_for_fences(&[self.fence], true, timeout_ns)?;
        });

        self.signaled = true;

        Ok(())
    }

    pub fn reset(&mut self) -> ThResult<()>
    {
        if !self.signaled
        {
            return Ok(());
        }

        layer_inspect!(d=self.device => unsafe {
           d.logical_device.reset_fences(&[self.fence])?;
        });

        self.signaled = false;

        Ok(())
    }

    pub fn destroy(&self)
    {
        layer_inspect!(d=self.device => {
            unsafe {d.logical_device.destroy_fence(self.fence, None);}
        });
    }
}


pub struct VkSync
{
    pub image_available: Vec<vk::Semaphore>,
    pub queue_complete: Vec<vk::Semaphore>,
    pub frame_fences: Vec<VkFence>,
    pub image_frames: Vec<usize>,

    device: Layer<LogicalDevice>,
}


impl VkSync
{
    pub fn new(reg: &LayerReg<()>) -> ThResult<Self>
    {
        log::info!("Vulkan Sync objects created");

        Ok(Self {
            image_available: vec![],
            queue_complete: vec![],
            frame_fences: vec![],
            image_frames: vec![],
            device: reg.get_unchecked(),
        })
    }

    pub fn update_for_image_count(&mut self, image_count: u32) -> ThResult<()>
    {
        let create_info = vk::SemaphoreCreateInfo::default();

        self.frame_fences = Vec::with_capacity(image_count as usize);
        self.image_frames = vec![0; image_count as usize];
        self.image_available = Vec::with_capacity(image_count as usize);
        self.queue_complete = Vec::with_capacity(image_count as usize);

        for _ in 0..image_count
        {
            self.frame_fences
                .push(VkFence::new(self.device.clone(), true)?);

            self.image_available.push(unsafe {
                layer_read!(self.device)
                    .logical_device
                    .create_semaphore(&create_info, None)
            }?);

            self.queue_complete.push(unsafe {
                layer_read!(self.device)
                    .logical_device
                    .create_semaphore(&create_info, None)
            }?);
        }

        Ok(())
    }

    pub fn destroy(&mut self)
    {
        for f in &self.frame_fences
        {
            f.destroy();
        }

        for s in &self.image_available
        {
            layer_inspect!(d=self.device => unsafe {
               d.logical_device.destroy_semaphore(*s, None);
            });
        }

        for s in &self.queue_complete
        {
            layer_inspect!(d=self.device => unsafe {
               d.logical_device.destroy_semaphore(*s, None);
            });
        }

        log::info!("Vulkan Sync objects destroyed");
    }
}


impl LayerDispatch<()> for VkSync {}
