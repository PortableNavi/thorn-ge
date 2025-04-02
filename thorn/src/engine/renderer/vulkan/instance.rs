use crate::{
    error::{ThError, ThResult},
    layer::LayerDispatch,
};
use ash::{Entry, vk};


pub struct Instance
{
    pub entry: Entry,
    pub instance: ash::Instance,
}


impl Instance
{
    pub fn new() -> ThResult<Self>
    {
        let entry = unsafe { Entry::load() }.map_err(|e| ThError::Error(e.to_string()))?;

        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 0, 0),
            p_application_name: c"ThornApplication".as_ptr(),
            p_engine_name: c"Thorn".as_ptr(),
            ..Default::default()
        };

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };

        let instance = unsafe { entry.create_instance(&create_info, None) }?;

        Ok(Self { entry, instance })
    }
}


impl LayerDispatch<()> for Instance {}
