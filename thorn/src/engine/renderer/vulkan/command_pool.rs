use super::{
    command_buffer::CommandBuffer,
    logical_device::LogicalDevice,
    physical_device::PhysicalDevice,
};
use crate::prelude::*;
use ash::vk;


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum State
{
    Ready,
    Recording,
    Ongoing,
    Waiting,
    Submitted,
}


#[derive(Clone)]
pub struct CommandPool
{
    pub pool: vk::CommandPool,
    device: Layer<LogicalDevice>,
}


impl CommandPool
{
    pub fn new(reg: &LayerReg<()>, family: u32) -> ThResult<Self>
    {
        let device = reg.get_unchecked::<LogicalDevice>();
        let device = device.read().unwrap();

        let create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(family)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let pool = unsafe {
            device
                .logical_device
                .create_command_pool(&create_info, None)
        }?;

        log::info!("Vulkan command pool created");

        Ok(Self {
            pool,
            device: reg.get_unchecked(),
        })
    }

    pub fn new_buffer(&self, primary: bool, single_use: bool) -> ThResult<CommandBuffer>
    {
        CommandBuffer::new_from_pool(self.clone(), self.device.clone(), primary, single_use)
    }

    /// Creates a new single use, primary command buffer
    pub fn new_buffer_single_use(&self) -> ThResult<CommandBuffer>
    {
        let mut buffer = self.new_buffer(true, true)?;
        buffer.begin(false, false);
        Ok(buffer)
    }

    /// Creates a new multiple use, primary command buffer
    pub fn new_buffer_primary(&self) -> ThResult<CommandBuffer>
    {
        Self::new_buffer(self, true, false)
    }

    pub fn allocate_buffer(&self, primary: bool) -> ThResult<vk::CommandBuffer>
    {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.pool)
            .level(either! {primary => vk::CommandBufferLevel::PRIMARY; vk::CommandBufferLevel::SECONDARY})
            .command_buffer_count(1);

        let buffer = unsafe {
            self.device
                .read()
                .unwrap()
                .logical_device
                .allocate_command_buffers(&allocate_info)
        }?[0];

        Ok(buffer)
    }

    pub fn free_buffer(&self, buffer: vk::CommandBuffer)
    {
        unsafe {
            self.device
                .read()
                .unwrap()
                .logical_device
                .free_command_buffers(self.pool, &[buffer]);
        }
    }

    pub fn destroy(&mut self)
    {
        unsafe {
            self.device
                .read()
                .unwrap()
                .logical_device
                .destroy_command_pool(self.pool, None);
        }

        log::info!("Vulkan command pool destroyed");
    }
}


pub struct CommandPools
{
    pub graphics: CommandPool,
}


impl CommandPools
{
    pub fn new(reg: &LayerReg<()>) -> ThResult<Self>
    {
        let graphics_family = reg
            .get_unchecked::<PhysicalDevice>()
            .read()
            .unwrap()
            .props
            .graphics_queue
            .ok_or(ThError::RendererError("No Graphics Queue".into()))?;

        let me = Self {
            graphics: CommandPool::new(reg, graphics_family)?,
        };

        log::info!("Vulkan command pools created");

        Ok(me)
    }

    pub fn destroy(&mut self)
    {
        self.graphics.destroy();
        log::info!("Vulkan command pools destroyed");
    }
}


impl LayerDispatch<()> for CommandPools {}
