use std::{collections::HashSet, ffi::CString};

use ash::vk;

use super::{instance::Instance, physical_device::PhysicalDevice};
use crate::{engine::renderer::vulkan::physical_device::PhysicalDeviceProps, prelude::*};


pub struct LogicalDevice
{
    pub logical_device: ash::Device,
    pub graphics_queue: Option<vk::Queue>,
    pub present_queue: Option<vk::Queue>,
    pub compute_queue: Option<vk::Queue>,
    pub transfer_queue: Option<vk::Queue>,
}


impl LogicalDevice
{
    pub fn new(reg: &LayerReg<()>) -> ThResult<Self>
    {
        let device = reg.get_unchecked::<PhysicalDevice>();
        let device = device.read().unwrap();

        // Get all unique queue indices, so that shared queues arent created twice.
        let mut queues = HashSet::new();

        if let Some(graphics) = device.props.graphics_queue
        {
            queues.insert(graphics);
        }

        if let Some(present) = device.props.present_queue
        {
            queues.insert(present);
        }

        if let Some(compute) = device.props.compute_queue
        {
            queues.insert(compute);
        }

        if let Some(transfer) = device.props.transfer_queue
        {
            queues.insert(transfer);
        }

        let mut queue_create_infos = vec![];
        for q in queues.iter()
        {
            //TODO: Figure something about the graphics q prio out, leaving this as a stub...
            #[allow(clippy::if_same_then_else)]
            let prios: &[f32] = {
                if Some(*q) == device.props.graphics_queue
                {
                    &[1.0]
                }
                else
                {
                    &[1.0]
                }
            };

            let create_info = vk::DeviceQueueCreateInfo::default()
                .queue_priorities(prios)
                .queue_family_index(*q);

            queue_create_infos.push(create_info);
        }

        let extension_names = PhysicalDeviceProps::default()
            .extension_names
            .iter()
            .filter_map(|n| CString::new(n.as_str()).ok())
            .collect::<Vec<_>>();

        let extension_names = extension_names
            .iter()
            .map(|n| n.as_c_str().as_ptr())
            .collect::<Vec<_>>();

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extension_names)
            .enabled_features(&device.props.features);

        let logical_device = unsafe {
            reg.get_unchecked::<Instance>()
                .write()
                .unwrap()
                .instance
                .create_device(device.device, &create_info, None)
        }?;

        let graphics_queue = device
            .props
            .graphics_queue
            .map(|q| unsafe { logical_device.get_device_queue(q, 0) });

        let present_queue = device
            .props
            .present_queue
            .map(|q| unsafe { logical_device.get_device_queue(q, 0) });

        let compute_queue = device
            .props
            .compute_queue
            .map(|q| unsafe { logical_device.get_device_queue(q, 0) });

        let transfer_queue = device
            .props
            .transfer_queue
            .map(|q| unsafe { logical_device.get_device_queue(q, 0) });

        log::info!("Vulkan logical device created");
        Ok(Self {
            logical_device,
            graphics_queue,
            present_queue,
            compute_queue,
            transfer_queue,
        })
    }

    pub fn destroy(&mut self)
    {
        unsafe {
            self.logical_device.destroy_device(None);
        }

        log::info!("Vulkan logical device destroyed");
    }
}


impl LayerDispatch<()> for LogicalDevice {}
