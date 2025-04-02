#![allow(unused)] // Until the renderer is more mature...


use std::{
    collections::HashSet,
    ffi::{CStr, c_char, c_void},
};

use crate::{
    error::{ThError, ThResult},
    layer::LayerDispatch,
};

use ash::{Entry, ext::debug_utils, vk};
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};


// Wrap the window handles in a new type container that i marked as send + sync
// so that it can be stored in a layer. Use carefully. This send and sync is faked...
#[derive(Clone, Copy)]
pub struct RawHandles
{
    pub window: RawWindowHandle,
    pub display: RawDisplayHandle,
}

unsafe impl Send for RawHandles {}
unsafe impl Sync for RawHandles {}


pub struct Instance
{
    pub entry: Entry,
    pub instance: ash::Instance,
    pub debug: Option<(vk::DebugUtilsMessengerEXT, debug_utils::Instance)>,
    pub handles: RawHandles,
}


impl Instance
{
    pub fn new(
        display: RawDisplayHandle,
        window: RawWindowHandle,
        layers: &[&CStr],
    ) -> ThResult<Self>
    {
        // Load Vulkan
        let entry = unsafe { Entry::load() }.map_err(|e| ThError::Error(e.to_string()))?;

        // Fetch extensions for current window...
        let mut enabled_extensions = ash_window::enumerate_required_extensions(display)?.to_vec();

        // Enable extra extensions on mac and ios
        if cfg!(any(target_os = "macos", target_os = "ios"))
        {
            enabled_extensions.push(ash::khr::portability_enumeration::NAME.as_ptr());
            enabled_extensions.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }

        let create_flags = {
            // Extra flags on mac and ios
            if cfg!(any(target_os = "macos", target_os = "ios"))
            {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            }
            // Default flags on other systems...
            else
            {
                vk::InstanceCreateFlags::default()
            }
        };

        // Collect layers...
        let mut enabled_layers = HashSet::<*const c_char>::new();
        layers.iter().for_each(|e| {
            enabled_layers.insert(e.as_ptr());
        });

        // Add validations layer in debug builds...
        #[cfg(debug_assertions)]
        enabled_layers.insert(c"VK_LAYER_KHRONOS_validation".as_ptr());

        // Add debug extension in debug builds ...
        #[cfg(debug_assertions)]
        enabled_extensions.push(debug_utils::NAME.as_ptr());

        let enabled_layers = enabled_layers.into_iter().collect::<Vec<_>>();

        let app_info = vk::ApplicationInfo::default()
            .api_version(vk::make_api_version(0, 1, 0, 0))
            .application_name(c"ThornApplication")
            .engine_name(c"Thorn");

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extensions)
            .enabled_layer_names(&enabled_layers)
            .flags(create_flags);

        let instance = unsafe { entry.create_instance(&create_info, None) }?;

        // Setup debugging
        let debug = {
            if cfg!(debug_assertions)
            {
                Some(setup_debug(&entry, &instance)?)
            }
            else
            {
                None
            }
        };

        log::info!("Vulkan instance created");

        Ok(Self {
            entry,
            instance,
            debug,
            handles: RawHandles { display, window },
        })
    }

    pub fn destroy(&mut self)
    {
        unsafe {
            // Destroy the debugging setup
            if let Some((msg, loader)) = &mut self.debug
            {
                loader.destroy_debug_utils_messenger(*msg, None);
            }

            // Destroy the instance
            self.instance.destroy_instance(None);

            log::info!("Vulkan instance detroyed");
        }
    }
}


impl LayerDispatch<()> for Instance {}


fn setup_debug(
    entry: &Entry,
    instance: &ash::Instance,
) -> ThResult<(vk::DebugUtilsMessengerEXT, debug_utils::Instance)>
{
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(debug_callback));

    let loader = debug_utils::Instance::new(entry, instance);
    let messenger = unsafe { loader.create_debug_utils_messenger(&debug_info, None) }?;

    Ok((messenger, loader))
}


unsafe extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    kind: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut c_void,
) -> vk::Bool32
{
    let data = unsafe { *data };

    let msg_id = {
        if data.p_message_id_name.is_null()
        {
            "<unknown>".into()
        }
        else
        {
            unsafe { CStr::from_ptr(data.p_message_id_name) }.to_string_lossy()
        }
    };

    let msg = {
        if data.p_message.is_null()
        {
            "<unknown>".into()
        }
        else
        {
            unsafe { CStr::from_ptr(data.p_message) }.to_string_lossy()
        }
    };

    match severity
    {
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO =>
        {
            log::info!("[vulkan]({kind:?})({msg_id}): {msg}")
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING =>
        {
            log::warn!("[vulkan]({kind:?})({msg_id}): {msg}")
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR =>
        {
            log::error!("[vulkan]({kind:?})({msg_id}): {msg}")
        }
        _ => (),
    }

    vk::FALSE
}
