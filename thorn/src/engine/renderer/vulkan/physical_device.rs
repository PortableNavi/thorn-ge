use super::{instance::Instance, surface::Surface};
use crate::prelude::*;
use ash::vk;
use std::cmp::Ordering;


//TODO: Allow for configuration...
#[derive(Clone)]
pub struct PhysicalDeviceProps
{
    pub device_name: String,
    pub graphics_queue: Option<u32>,
    pub present_queue: Option<u32>,
    pub compute_queue: Option<u32>,
    pub transfer_queue: Option<u32>,
    pub has_sampler_anisotropy: bool,
    pub is_discrete: bool,
    pub extension_names: Vec<String>,

    #[allow(unused)]
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,

    pub features: vk::PhysicalDeviceFeatures,
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}


impl PhysicalDeviceProps
{
    fn of(device: vk::PhysicalDevice, instance: &Instance, surface: &Surface) -> ThResult<Self>
    {
        let props = unsafe { instance.instance.get_physical_device_properties(device) };
        let features = unsafe { instance.instance.get_physical_device_features(device) };

        // let memory = unsafe {
        //     instance
        //         .instance
        //         .get_physical_device_memory_properties(device)
        // };

        // Get the device name
        let device_name = props
            .device_name_as_c_str()
            .map(|n| n.to_string_lossy())
            .unwrap_or("unknown graphics device".into())
            .to_string();

        // Check whether or not this device is a discrete gpu
        let is_discrete = props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;

        let mut graphics_queue = None;
        let mut present_queue = None;
        let mut compute_queue = None;
        let mut transfer_queue = None;

        // enumerate the queue families
        let queue_families = unsafe {
            instance
                .instance
                .get_physical_device_queue_family_properties(device)
        };

        // Find indices for all queues and try to find a dedicated transfer queue...
        let mut transfer_score = u8::MAX;
        for (i, q) in queue_families.iter().enumerate()
        {
            let mut score = 0;

            if q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                graphics_queue = Some(i as u32);
                score += 1;
            }

            if q.queue_flags.contains(vk::QueueFlags::COMPUTE)
            {
                compute_queue = Some(i as u32);
                score += 1;
            }

            let surface_support = unsafe {
                surface.loader.get_physical_device_surface_support(
                    device,
                    i as u32,
                    surface.surface,
                )
            }?;

            if surface_support
            {
                present_queue = Some(i as u32);
                score += 1;
            }

            if q.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && (score < transfer_score || transfer_queue.is_none())
            {
                transfer_queue = Some(i as u32);
                transfer_score = score;
            }
        }

        // Get the surface capabilities...
        let surface_capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device, surface.surface)
        }?;

        // Get available surface formats...
        let surface_formats = unsafe {
            surface
                .loader
                .get_physical_device_surface_formats(device, surface.surface)
        }?;

        // Get available present modes...
        let present_modes = unsafe {
            surface
                .loader
                .get_physical_device_surface_present_modes(device, surface.surface)
        }?;

        // Get this device's extensions
        let extension_names = unsafe {
            instance
                .instance
                .enumerate_device_extension_properties(device)
        }?
        .iter()
        .filter_map(|e| {
            e.extension_name_as_c_str()
                .ok()
                .map(|s| s.to_string_lossy().to_string())
        })
        .collect();

        let props = Self {
            device_name,
            is_discrete,
            graphics_queue,
            present_queue,
            compute_queue,
            transfer_queue,
            surface_capabilities,
            present_modes,
            surface_formats,
            extension_names,
            features,
            has_sampler_anisotropy: features.sampler_anisotropy != 0,
        };

        Ok(props)
    }

    fn check_device(
        &self,
        device: vk::PhysicalDevice,
        instance: &Instance,
        surface: &Surface,
    ) -> Option<(vk::PhysicalDevice, Self)>
    {
        let props = match Self::of(device, instance, surface)
        {
            Ok(props) => props,

            Err(e) =>
            {
                log::error!("Failed to query GPU properties: {e}");
                return None;
            }
        };

        let dname = &props.device_name;

        for name in &self.extension_names
        {
            if !props.extension_names.contains(name)
            {
                log::error!(
                    "One GPU {dname:?} was not useable because of missing vulkan extension support for: {name}"
                );

                return None;
            }
        }

        if self.is_discrete && !props.is_discrete
        {
            log::error!("One GPU {dname:?} was not useable because it was not discrete");
            return None;
        }

        if self.has_sampler_anisotropy && !props.has_sampler_anisotropy
        {
            log::error!(
                "One GPU {dname:?} was not useable because it did not support sampler anisotropy"
            );
            return None;
        }

        if props.present_modes.is_empty() || props.surface_formats.is_empty()
        {
            log::error!(
                "One GPU {dname:?} was not useable because it does not have any present modes ot does not have a surface format"
            );
            return None;
        }

        if (self.graphics_queue.is_none() || props.graphics_queue.is_some())
            && (self.present_queue.is_none() || props.present_queue.is_some())
            && (self.compute_queue.is_none() || props.compute_queue.is_some())
            && (self.transfer_queue.is_none() || props.transfer_queue.is_some())
        {
            log::info!("Found usable GPU {dname:?}");
            Some((device, props))
        }
        else
        {
            log::error!(
                "One GPU {dname:?} was not useable because one or more required queue families were unsupported"
            );

            None
        }
    }

    //TODO: implement...
    fn compare_to(&self, other: &Self) -> Ordering
    {
        // Prefer discrete gpus
        if self.is_discrete && !other.is_discrete
        {
            return Ordering::Greater;
        }

        Ordering::Equal
    }
}


impl Default for PhysicalDeviceProps
{
    fn default() -> Self
    {
        Self {
            device_name: String::new(),
            graphics_queue: Some(0),
            present_queue: Some(0),
            compute_queue: None,
            transfer_queue: Some(0),
            has_sampler_anisotropy: true,
            is_discrete: false,
            extension_names: vec![ash::khr::swapchain::NAME.to_string_lossy().to_string()],
            surface_capabilities: vk::SurfaceCapabilitiesKHR::default(),
            surface_formats: vec![],
            present_modes: vec![vk::PresentModeKHR::MAILBOX],
            features: vk::PhysicalDeviceFeatures::default(),
        }
    }
}


#[allow(unused)]
pub struct PhysicalDevice
{
    pub device: vk::PhysicalDevice,
    pub props: PhysicalDeviceProps,
    surface: Layer<Surface>,
}


impl PhysicalDevice
{
    pub fn new(reg: &LayerReg<()>) -> ThResult<Self>
    {
        let instance = reg.get_unchecked::<Instance>();
        let instance = instance.read().unwrap();

        let surface = reg.get_unchecked::<Surface>();
        let surface = surface.read().unwrap();

        let requirements = PhysicalDeviceProps::default();

        let devices = unsafe { instance.instance.enumerate_physical_devices()? };

        // Filter out devices that do not meet the requirements
        let mut useable_devices = devices
            .iter()
            .filter_map(|d| requirements.check_device(*d, &instance, &surface))
            .collect::<Vec<_>>();

        // Rank the available devices
        useable_devices.sort_unstable_by(|a, b| a.1.compare_to(&b.1));

        // Chose the device with the highest ranking to use...
        let (device, props) = useable_devices
            .last()
            .cloned()
            .ok_or(ThError::RendererError("No useable GPU found".into()))?;

        log::info!("Vulkan physical device created");
        Ok(Self {
            device,
            props,
            surface: reg.get_unchecked(),
        })
    }

    pub fn update_capabilities(&mut self)
    {
        let surface = self.surface.read().unwrap();

        // I dont really expect this to fail, at this point
        if let Ok(caps) = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(self.device, surface.surface)
        }
        {
            self.props.surface_capabilities = caps;
        }
    }

    pub fn destroy(&mut self)
    {
        log::info!("Vulkan physical device destroyed");
    }
}


impl LayerDispatch<()> for PhysicalDevice {}
