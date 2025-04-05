use ash::vk;

use crate::{error::ThResult, layer::Layer};

use super::logical_device::LogicalDevice;


#[derive(Default)]
pub struct VkImage2D
{
    pub image: vk::Image,
    pub view: Option<vk::ImageView>,
    pub width: u32,
    pub height: u32,
    pub format: vk::Format,
    pub memory: vk::DeviceMemory,
}


impl VkImage2D
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: Layer<LogicalDevice>,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        memflags: vk::MemoryPropertyFlags,
        aspect: vk::ImageAspectFlags,
    ) -> ThResult<Self>
    {
        let mut me = Self::new_image_only(
            device.clone(),
            width,
            height,
            format,
            tiling,
            usage,
            memflags,
        )?;

        me.create_view(device, aspect)?;

        Ok(me)
    }

    pub fn new_image_only(
        device: Layer<LogicalDevice>,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        memflags: vk::MemoryPropertyFlags,
    ) -> ThResult<Self>
    {
        let device = &device.read().unwrap();

        let img_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .format(format)
            .tiling(tiling)
            .usage(usage);

        let image = unsafe { device.logical_device.create_image(&img_create_info, None) }?;
        let memreqs = unsafe { device.logical_device.get_image_memory_requirements(image) };
        let memtype: u32 = device.find_memtype_index(memreqs.memory_type_bits, memflags)?;

        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(memreqs.size)
            .memory_type_index(memtype);

        let memory = unsafe { device.logical_device.allocate_memory(&allocate_info, None) }?;
        unsafe { device.logical_device.bind_image_memory(image, memory, 0) }?;


        Ok(Self {
            image,
            width,
            height,
            memory,
            format,
            view: None,
        })
    }

    pub fn create_view(
        &mut self,
        device: Layer<LogicalDevice>,
        aspect: vk::ImageAspectFlags,
    ) -> ThResult<()>
    {
        let view_create_info = vk::ImageViewCreateInfo::default()
            .image(self.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(self.format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        self.view = Some(unsafe {
            device
                .read()
                .unwrap()
                .logical_device
                .create_image_view(&view_create_info, None)
        }?);

        Ok(())
    }

    pub fn destroy(&mut self, device: Layer<LogicalDevice>)
    {
        let device = &device.read().unwrap().logical_device;

        if let Some(view) = self.view.take()
        {
            unsafe { device.destroy_image_view(view, None) };
        }

        unsafe { device.free_memory(self.memory, None) };
        unsafe { device.destroy_image(self.image, None) };
    }
}
