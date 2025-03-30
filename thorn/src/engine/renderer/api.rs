use crate::error::ThResult;
use winit::raw_window_handle::RawWindowHandle;


pub(crate) trait RenderAPI: Send + Sync
{
    fn initialize(&mut self, rwh: RawWindowHandle) -> ThResult<()>;
    fn destroy(&mut self);
    fn frame_prepare(&mut self);
    fn frame_render(&mut self);
    fn frame_finish(&mut self);
}
