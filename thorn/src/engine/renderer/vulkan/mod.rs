use winit::raw_window_handle::RawWindowHandle;

use super::api::RenderAPI;


pub(crate) struct VulkanRenderer {}


impl VulkanRenderer
{
    pub(crate) fn new() -> Self
    {
        Self {}
    }
}


impl RenderAPI for VulkanRenderer
{
    fn initialize(&mut self, _rwh: RawWindowHandle) -> crate::prelude::ThResult<()>
    {
        log::info!("Vulkan Renderer Initialized");
        Ok(())
    }

    fn destroy(&mut self)
    {
        log::info!("Vulkan Renderer Destroyed");
    }

    fn frame_prepare(&mut self) {}
    fn frame_render(&mut self) {}
    fn frame_finish(&mut self) {}
}
