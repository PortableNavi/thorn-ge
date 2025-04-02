use super::instance::Instance;
use crate::prelude::*;
use ash::{khr::surface::Instance as SurfaceLoader, vk};


pub struct Surface
{
    pub surface: vk::SurfaceKHR,
    pub loader: SurfaceLoader,
}


impl Surface
{
    pub fn new(reg: &mut LayerReg<()>) -> ThResult<Self>
    {
        let instance = reg.get_unchecked::<Instance>();
        let instance = instance.read().unwrap();

        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance.instance,
                instance.handles.display,
                instance.handles.window,
                None,
            )?
        };

        let loader = SurfaceLoader::new(&instance.entry, &instance.instance);

        log::info!("Vulkan surface created");
        Ok(Self { surface, loader })
    }

    pub fn destroy(&mut self)
    {
        unsafe {
            self.loader.destroy_surface(self.surface, None);
        }

        log::info!("Vulkan surface destroyed");
    }
}


impl LayerDispatch<()> for Surface {}
