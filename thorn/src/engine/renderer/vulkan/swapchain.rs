use std::collections::HashSet;

use super::{
    image::VkImage2D,
    instance::Instance,
    logical_device::LogicalDevice,
    physical_device::PhysicalDevice,
    surface::Surface,
};

use crate::prelude::*;
use ash::vk::{self, Handle};


//TODO: Implement Sync layer and use it here...
//TODO: Implement Framebuffer layer and use it here...


pub struct Swapchain
{
    pub width: u32,
    pub height: u32,
    pub image_format: vk::SurfaceFormatKHR,
    pub depth_buffer_format: vk::Format,
    pub max_buffered_frames: u32,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_device: ash::khr::swapchain::Device,
    pub images: Vec<vk::Image>,
    pub views: Vec<vk::ImageView>,
    pub depth_buffer: VkImage2D,

    device: Layer<LogicalDevice>,
    physical_device: Layer<PhysicalDevice>,
    surface: Layer<Surface>,
    dirty: Option<(u32, u32)>,
    //sync: Layer<Sync>,
}


impl Swapchain
{
    pub fn new(reg: &LayerReg<()>, width: u32, height: u32) -> ThResult<Self>
    {
        let swapchain_device = ash::khr::swapchain::Device::new(
            &reg.get_unchecked::<Instance>().read().unwrap().instance,
            &reg.get_unchecked::<LogicalDevice>()
                .read()
                .unwrap()
                .logical_device,
        );

        let mut me = Self {
            width,
            height,
            depth_buffer: VkImage2D::default(),
            swapchain_device,
            depth_buffer_format: vk::Format::default(),
            image_format: vk::SurfaceFormatKHR::default(),
            max_buffered_frames: 2,
            swapchain: vk::SwapchainKHR::null(),
            images: vec![],
            views: vec![],
            device: reg.get_unchecked(),
            physical_device: reg.get_unchecked(),
            surface: reg.get_unchecked(),
            dirty: None,
        };

        me.create()?;

        log::info!("Vulkan swapchain created");
        Ok(me)
    }

    pub fn mark_dirty(&mut self, w: u32, h: u32)
    {
        self.dirty = Some((w, h));
    }

    pub fn recreate_if_dirty(&mut self) -> ThResult<()>
    {
        if let Some((w, h)) = self.dirty
        {
            return self.recreate(w, h);
        }

        Ok(())
    }

    pub fn recreate(&mut self, w: u32, h: u32) -> ThResult<()>
    {
        self.width = w;
        self.height = h;

        self.create()?;
        self.dirty = None;

        log::info!("Vulkan swapchain recreated");
        Ok(())
    }

    pub fn get_next_image_index(&mut self, timeout_ns: u64) -> ThResult<u32>
    {
        let image_available = vk::Semaphore::null(); //TODO: get this from self.sync
        let fence = vk::Fence::null(); //TODO: get this from self.sync

        let result = unsafe {
            self.swapchain_device.acquire_next_image(
                self.swapchain,
                timeout_ns,
                image_available,
                fence,
            )
        };

        let (index, optimal) = match result
        {
            Ok(val) => val,

            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) =>
            {
                self.recreate(0, 0)?; //TODO: Get width and height from framebuffer...
                return Err(ThError::from(vk::Result::ERROR_OUT_OF_DATE_KHR));
            }

            Err(e) => return Err(ThError::from(e)),
        };

        if !optimal
        {
            log::warn!("Image is not optimal for current surface");
        }

        Ok(index)
    }

    pub fn present(&mut self, image_index: u32) -> ThResult<()>
    {
        let render_complete = vk::Semaphore::null(); //TODO: Get this from self.sync

        let swapchains = &[self.swapchain];
        let wait_semaphores = &[render_complete];
        let image_indices = &[image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(wait_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let queue = self.device.read().unwrap().present_queue.clone();

        if let Some(q) = queue
        {
            match unsafe { self.swapchain_device.queue_present(q, &present_info) }
            {
                Ok(false) =>
                {
                    log::warn!("Surface is suboptimal for image");
                    self.recreate(0, 0)?; //TODO: Get width and height from framebuffer...
                }

                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) =>
                {
                    log::warn!("Swapchain is out of date");
                    self.recreate(0, 0)?; //TODO: Get width and height from framebuffer...
                }

                Ok(true) => (),
                Err(e) => return Err(e.into()),
            }
        }
        else
        {
            log::warn!("Tried to present the swapchain without a present queue...");
        }

        Ok(())
    }

    pub fn destroy(&mut self)
    {
        let _ = self.soft_destroy();
        log::info!("Vulkan swapchain destroyed");
    }

    fn create(&mut self) -> ThResult<()>
    {
        // Choose a Format and present mode. Preferences are:
        // * B8G8R8A8_UNORM | SRGB_NONLINEAR
        // * MAILBOX

        let format = {
            let formats = &self.physical_device.read().unwrap().props.surface_formats;

            *formats
                .iter()
                .find(|e| {
                    e.format == vk::Format::B8G8R8A8_UNORM
                        && e.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .unwrap_or(formats.get(0).ok_or(ThError::RendererError(
                    "No Supported Surface format found".into(),
                ))?)
        };

        let present_mode = self
            .physical_device
            .read()
            .unwrap()
            .props
            .present_modes
            .iter()
            .find(|m| **m == vk::PresentModeKHR::MAILBOX)
            .map(|e| *e)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        self.physical_device.write().unwrap().update_capabilities();

        let caps = &self
            .physical_device
            .read()
            .unwrap()
            .props
            .surface_capabilities
            .clone();

        let extend = caps.current_extent;

        self.width = extend
            .width
            .clamp(caps.min_image_extent.width, caps.max_image_extent.width);

        self.height = extend
            .height
            .clamp(caps.min_image_extent.height, caps.max_image_extent.height);

        // Get one more image than the minimum images allowed but not more than the maximum images supported
        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count != 0
        {
            image_count = image_count.min(caps.max_image_count);
        }

        let present_queue_index = self.physical_device.read().unwrap().props.present_queue;
        let graphics_queue_index = self.physical_device.read().unwrap().props.graphics_queue;

        let sharing_mode = {
            if present_queue_index == graphics_queue_index
            {
                vk::SharingMode::CONCURRENT
            }
            else
            {
                vk::SharingMode::EXCLUSIVE
            }
        };

        let mut queues = HashSet::new();
        if let Some(p) = present_queue_index
        {
            queues.insert(p);
        }
        if let Some(g) = present_queue_index
        {
            queues.insert(g);
        }

        let queues = queues.into_iter().collect::<Vec<_>>();

        let depth_format = *self
            .physical_device
            .read()
            .unwrap()
            .props
            .depth_formats
            .get(0)
            .ok_or(ThError::RendererError(
                "Failed to select a depth format".into(),
            ))?;

        let depth_buffer = VkImage2D::new(
            self.device.clone(),
            self.width,
            self.height,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::DEPTH,
        )?;

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .queue_family_indices(&queues)
            .image_sharing_mode(sharing_mode)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .clipped(true)
            .surface(self.surface.read().unwrap().surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .present_mode(present_mode)
            .image_color_space(format.color_space)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .old_swapchain(self.swapchain)
            .image_extent(vk::Extent2D {
                width: self.width,
                height: self.height,
            });

        let swapchain = unsafe { self.swapchain_device.create_swapchain(&create_info, None) }?;
        let images = unsafe { self.swapchain_device.get_swapchain_images(swapchain)? };

        let mut views = vec![];
        for img in &images
        {
            let create_info = vk::ImageViewCreateInfo::default()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .image(*img)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );

            let view = unsafe {
                self.device
                    .read()
                    .unwrap()
                    .logical_device
                    .create_image_view(&create_info, None)
            }?;

            views.push(view);
        }

        // Destroy old resources...
        self.soft_destroy()?;

        // Assign the new resources...
        self.image_format = format;
        self.images = images;
        self.views = views;
        self.depth_buffer_format = depth_format;
        self.swapchain = swapchain;
        self.depth_buffer = depth_buffer;

        Ok(())
    }

    fn soft_destroy(&mut self) -> ThResult<()>
    {
        unsafe {
            if !self.swapchain.is_null()
            {
                self.swapchain_device
                    .destroy_swapchain(self.swapchain, None);
            }

            while let Some(view) = self.views.pop()
            {
                self.device
                    .read()
                    .unwrap()
                    .logical_device
                    .destroy_image_view(view, None);
            }

            if !self.depth_buffer.image.is_null()
            {
                self.depth_buffer.destroy(self.device.clone());
            }
        }

        Ok(())
    }
}


impl LayerDispatch<()> for Swapchain {}
