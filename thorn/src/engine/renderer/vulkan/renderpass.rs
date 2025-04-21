use crate::prelude::*;
use ash::vk::{self};

use super::{
    command_buffer::CommandBuffers,
    framebuffer::FrameBuffers,
    logical_device::LogicalDevice,
    swapchain::Swapchain,
};


pub enum State
{
    Ready,
    Recording,
    Ongoing,
    Waiting,
    Submitted,
}


pub struct Renderpass
{
    pub renderpass: vk::RenderPass,
    pub pos_x: u32,
    pub pos_y: u32,
    pub width: u32,
    pub height: u32,
    pub clear_color: (f32, f32, f32),
    pub state: State,
    pub depth: f32,
    pub stencil: u32,
    pub attachments: [vk::AttachmentDescription; 2],

    device: Layer<LogicalDevice>,
    swapchain: Layer<Swapchain>,
    command_buffers: Layer<CommandBuffers>,
    pub(super) frame_buffer: Option<Layer<FrameBuffers>>,
}


impl Renderpass
{
    pub fn new_default(reg: &LayerReg<()>, width: u32, height: u32) -> ThResult<Self>
    {
        let device = reg.get_unchecked::<LogicalDevice>();
        let device = &device.read().unwrap();

        let swapchain = reg.get_unchecked::<Swapchain>();
        let swapchain = &swapchain.read().unwrap();

        let clear_color = (0.47, 0.0, 0.16);

        let mut attachments = [vk::AttachmentDescription::default(); 2];

        // Color attachment
        attachments[0] = vk::AttachmentDescription::default()
            .format(swapchain.image_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .flags(vk::AttachmentDescriptionFlags::empty());

        let color_attachment_ref = &[vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        // Depth attachment
        attachments[1] = vk::AttachmentDescription::default()
            .format(swapchain.depth_buffer_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::CLEAR)
            .stencil_store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .flags(vk::AttachmentDescriptionFlags::empty());

        let depth_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        // attachments[2] = vk::AttachmentDescription::default()
        //     .format(swapchain.image_format.format)
        //     .samples(vk::SampleCountFlags::TYPE_1)
        //     .load_op(vk::AttachmentLoadOp::DONT_CARE)
        //     .store_op(vk::AttachmentStoreOp::STORE)
        //     .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        //     .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        //     .initial_layout(vk::ImageLayout::UNDEFINED)
        //     .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        // let resolve_attachment_ref = &[vk::AttachmentReference::default()
        //     .attachment(2)
        //     .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let subpass = &[vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachment_ref)
            //.resolve_attachments(resolve_attachment_ref)
            .depth_stencil_attachment(&depth_attachment_ref)];

        let dependency = &[vk::SubpassDependency::default()
            .dependency_flags(vk::DependencyFlags::empty())
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_READ,
            )];

        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(subpass)
            .dependencies(dependency);

        let renderpass = unsafe { device.logical_device.create_render_pass(&create_info, None) }?;

        log::info!("Vulkan renderpass created");

        Ok(Self {
            renderpass,
            attachments,
            pos_x: 0,
            pos_y: 0,
            width,
            height,
            clear_color,
            state: State::Ready,
            depth: 1.0,
            stencil: 0,
            device: reg.get_unchecked(),
            swapchain: reg.get_unchecked(),
            command_buffers: reg.get_unchecked(),
            frame_buffer: None,
        })
    }

    pub fn destroy(&mut self)
    {
        self.frame_buffer = None; // Avoid cyclic dependency

        unsafe {
            self.device
                .read()
                .inspect(|e| e.logical_device.destroy_render_pass(self.renderpass, None));
        }

        log::info!("Vulkan renderpass destroyed");
    }

    pub fn begin(&mut self, image_index: usize)
    {
        let framebuffer =
            self.frame_buffer.as_ref().unwrap().read().unwrap().buffers[image_index].frame_buffer;

        let clear_values = [
            // Color clear value
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [
                        self.clear_color.0,
                        self.clear_color.1,
                        self.clear_color.2,
                        1.0,
                    ],
                },
            },
            // Depth clear value
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: self.depth,
                    stencil: self.stencil,
                },
            },
        ];

        let begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.renderpass)
            .clear_values(&clear_values)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D {
                    x: self.pos_x as i32,
                    y: self.pos_y as i32,
                },
                extent: vk::Extent2D {
                    width: self.width,
                    height: self.height,
                },
            });


        self.device.read().inspect(|d| unsafe {
            d.logical_device.cmd_begin_render_pass(
                self.command_buffers.read().unwrap().graphics[image_index].buffer,
                &begin_info,
                vk::SubpassContents::INLINE,
            );
        });

        self.state = State::Ongoing;
    }

    pub fn end(&mut self, image_index: usize)
    {
        self.device.read().inspect(|d| unsafe {
            d.logical_device.cmd_end_render_pass(
                self.command_buffers.read().unwrap().graphics[image_index].buffer,
            );
        });
    }
}


impl LayerDispatch<()> for Renderpass {}
