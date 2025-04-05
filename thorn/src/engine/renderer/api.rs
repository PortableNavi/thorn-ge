use crate::error::ThResult;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};


pub(crate) trait RenderAPI: Send + Sync
{
    fn initialize(
        &mut self,
        rdh: RawDisplayHandle,
        rwh: RawWindowHandle,
        w: u32,
        h: u32,
    ) -> ThResult<()>;
    fn destroy(&mut self);
    fn surface_size_changed(&mut self, w: u32, h: u32) -> ThResult<()>;
    fn frame_prepare(&mut self);
    fn frame_render(&mut self);
    fn frame_finish(&mut self);
}
