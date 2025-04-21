
use super::{
    command_pool::{CommandPool, CommandPools},
    logical_device::LogicalDevice,
    swapchain::Swapchain,
};
use crate::{layer_inspect, prelude::*};
use ash::vk;


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum State
{
    Ready,
    Recording,
    RecordingEnded,
    Ongoing,
    Waiting,
    Submitted,
}


#[derive(Clone)]
pub struct CommandBuffer
{
    pub buffer: vk::CommandBuffer,
    pub state: State,
    pub single_use: bool,
    pub primary: bool,

    pool: CommandPool,
    device: Layer<LogicalDevice>,
}


impl CommandBuffer
{
    pub fn new_from_pool(
        pool: CommandPool,
        device: Layer<LogicalDevice>,
        primary: bool,
        single_use: bool,
    ) -> ThResult<Self>
    {
        let buffer = pool.allocate_buffer(primary)?;
        log::info!("Vulkan command buffer created");

        Ok(Self {
            buffer,
            single_use,
            primary,
            state: State::Ready,
            pool,
            device,
        })
    }

    pub fn begin(&mut self, pass_continue: bool, shared: bool)
    {
        if self.state != State::Ready
        {
            log::warn!(
                "Tried to begin recording a command buffer when it was not ready. Command buffer was in state {:?}",
                self.state
            );
            return;
        }

        let flags = if_do! {
            (f = vk::CommandBufferUsageFlags::empty()) => {
                self.single_use => f |= vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT;
                pass_continue   => f |= vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE;
                shared          => f |= vk::CommandBufferUsageFlags::SIMULTANEOUS_USE;
            }
        };

        let begin_info = vk::CommandBufferBeginInfo::default().flags(flags);

        unsafe {
            self.device
                .read()
                .unwrap()
                .logical_device
                .begin_command_buffer(self.buffer, &begin_info)
        };

        self.state = State::Recording;
    }

    pub fn end(&mut self)
    {
        if self.state != State::Recording
        {
            log::warn!(
                "Tried to end a command buffer while it was not recording. Command buffer was in state {:?}",
                self.state
            );

            return;
        }

        unsafe {
            self.device
                .read()
                .unwrap()
                .logical_device
                .end_command_buffer(self.buffer)
        };

        self.state = State::RecordingEnded;
    }

    pub fn submit_blocking(&mut self, queue: vk::Queue)
    {
        if self.state != State::RecordingEnded
        {
            log::warn!(
                "Tried to submit a command buffer while its receording was not ended. Command buffer was in state: {:?}",
                self.state
            );

            return;
        }

        let buffers = [self.buffer];
        let submit_info = [vk::SubmitInfo::default().command_buffers(&buffers)];

        unsafe {
            let device = self.device.read().unwrap().logical_device.clone();
            device.queue_submit(queue, &submit_info, vk::Fence::null());
            device.queue_wait_idle(queue);
        }
    }

    pub fn submit(
        &mut self,
        queue: vk::Queue,
        wait: vk::Semaphore,
        signal: vk::Semaphore,
        fence: vk::Fence,
    ) -> ThResult<()>
    {
        if self.state != State::RecordingEnded
        {
            log::warn!(
                "Tried to submit a command buffer while its receording was not ended. Command buffer was in state: {:?}",
                self.state
            );

            return Err(ThError::RendererError("Queue was not submitable".into()));
        }

        let buffers = [self.buffer];
        let wait = [wait];
        let signal = [signal];
        let stage_flags = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = [vk::SubmitInfo::default()
            .command_buffers(&buffers)
            .wait_dst_stage_mask(&stage_flags)
            .wait_semaphores(&wait)
            .signal_semaphores(&signal)];

        unsafe {
            let device = self.device.read().unwrap().logical_device.clone();
            device.queue_submit(queue, &submit_info, fence);
        }

        self.state = State::Ready;
        Ok(())
    }

    pub fn end_single_use(&mut self, queue: vk::Queue)
    {
        if !self.single_use
        {
            log::warn!(
                "Tried to end a non single use command buffer as a single use command buffer"
            );

            return;
        }

        self.end();
        self.submit_blocking(queue);
        self.destroy();
    }

    pub fn reset(&mut self)
    {
        self.state = State::Ready;

        layer_inspect!(d=self.device => unsafe {
            d.logical_device.reset_command_buffer(self.buffer, vk::CommandBufferResetFlags::default());
        });
    }

    pub fn set_viewport(&mut self, viewport: vk::Viewport, scissor: vk::Rect2D)
    {
        layer_inspect!(d=self.device => unsafe {
            d.logical_device.cmd_set_viewport(self.buffer, 0, &[viewport]);
            d.logical_device.cmd_set_scissor(self.buffer, 0, &[scissor]);
        })
    }

    pub fn destroy(&mut self)
    {
        self.pool.free_buffer(self.buffer);
        log::info!("Vulkan command buffer destroyed");
    }
}


pub struct CommandBuffers
{
    pub graphics: Vec<CommandBuffer>,
    pool: Layer<CommandPools>,
}


impl CommandBuffers
{
    pub fn new(reg: &LayerReg<()>) -> ThResult<Self>
    {
        let pools = reg.get_unchecked::<CommandPools>();
        let pools = pools.write().unwrap();

        let count = reg
            .get_unchecked::<Swapchain>()
            .read()
            .unwrap()
            .images
            .len();


        let mut graphics = Vec::with_capacity(count);

        for _ in 0..count
        {
            graphics.push(pools.graphics.new_buffer_primary()?);
        }

        let me = Self {
            graphics,
            pool: reg.get_unchecked(),
        };

        log::info!("Vulkan command buffers created");
        Ok(me)
    }

    pub fn destroy(&mut self)
    {
        for buffer in &mut self.graphics
        {
            buffer.destroy();
        }

        log::info!("Vulkan command buffers destroyed");
    }
}


impl LayerDispatch<()> for CommandBuffers {}
