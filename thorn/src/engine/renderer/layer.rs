use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use super::{api::RenderAPI, vulkan::VulkanRenderer};
use crate::prelude::*;


pub enum Backend
{
    Vulkan,
}


pub struct RendererPlugin(pub Backend);
impl Plugin<LayerEvent> for RendererPlugin
{
    fn info(&self) -> PluginInfo
    {
        PluginInfo::build::<Renderer>()
            .dep::<Tasks>()
            .dep::<EventReceiver<PlatformEvent>>()
    }

    fn load(
        &mut self,
        reg: &LayerReg<LayerEvent>,
    ) -> Result<AnyLayer<LayerEvent>, Box<dyn std::error::Error>>
    {
        #[allow(clippy::single_match)]
        let api = match self.0
        {
            Backend::Vulkan => Box::new(VulkanRenderer::new()),
        };

        let tasks = reg
            .get()
            .ok_or(ThError::Error("Failed to fetch core layer".into()))?;

        let event_receiver = reg.get().ok_or(ThError::Error(
            "Failed to fetch platform event receiver".into(),
        ))?;

        Ok(AnyLayer::new(Renderer {
            api,
            tasks,
            event_receiver,
        }))
    }

    fn notify_loaded(&mut self, reg: &LayerReg<LayerEvent>)
    {
        let me = reg.get::<Renderer>().unwrap();
        me.write().unwrap().init(me.clone());
    }

    fn notify_unloaded(&mut self, reg: &LayerReg<LayerEvent>)
    {
        reg.get::<Renderer>().unwrap().write().unwrap().destroy();
    }
}


pub struct Renderer
{
    api: Box<dyn RenderAPI>,
    tasks: Layer<Tasks>,
    event_receiver: Layer<EventReceiver<PlatformEvent>>,
}


impl Renderer
{
    fn init(&mut self, me: Layer<Self>)
    {
        self.tasks.write().unwrap().hook(me.clone());
        self.event_receiver.write().unwrap().subscribe(me);
    }

    fn destroy(&mut self)
    {
        self.api.destroy();
    }

    pub fn initialize(&mut self, rdh: RawDisplayHandle, rwh: RawWindowHandle) -> ThResult<()>
    {
        self.api.initialize(rdh, rwh)
    }
}


impl LayerDispatch<LayerEvent> for Renderer
{
    fn dispatch(&mut self, _event: &LayerEvent) {}
}


//TODO: To be implemented
impl CoreHook for Renderer
{
    fn prepare(&mut self)
    {
        self.api.frame_prepare();
    }

    fn tick(&mut self, _frame_info: &FrameInfo)
    {
        self.api.frame_render();
    }

    fn finish(&mut self)
    {
        self.api.frame_finish();
    }
}


//TODO: To be implemented
impl EventSubscriber<PlatformEvent> for Renderer
{
    fn receive_event(&mut self, _event: &PlatformEvent) {}
}
