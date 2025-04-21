use super::{
    api::{FrameStatus, RenderAPI},
    vulkan::VulkanRenderer,
};
use crate::prelude::*;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};


pub enum Backend
{
    Vulkan,
}


#[derive(PartialEq, Eq, Clone, Copy)]
enum FrameState
{
    Begin,
    Render,
    Finish,
    Failed,
    Destroy,
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
            frame_state: FrameState::Finish,
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
    frame_state: FrameState,
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
        self.frame_state = FrameState::Destroy;
        //self.api.destroy();
    }

    pub fn initialize(
        &mut self,
        rdh: RawDisplayHandle,
        rwh: RawWindowHandle,
        w: u32,
        h: u32,
    ) -> ThResult<()>
    {
        self.api.initialize(rdh, rwh, w, h)
    }
}


impl Drop for Renderer
{
    fn drop(&mut self)
    {
        self.api.destroy();
    }
}


impl LayerDispatch<LayerEvent> for Renderer
{
    fn dispatch(&mut self, _event: &LayerEvent) {}
}


impl CoreHook for Renderer
{
    fn prepare(&mut self)
    {
        if self.frame_state != FrameState::Finish
        {
            log::warn!("Tried to begin frame after last frame didnt finish properly");
            self.frame_state = FrameState::Failed;
            return;
        }

        if let FrameStatus::Success = self.api.frame_prepare()
        {
            self.frame_state = FrameState::Begin;
        }
    }

    fn tick(&mut self, _frame_info: &FrameInfo)
    {
        if self.frame_state != FrameState::Begin
        {
            log::warn!("Tried to render frame after begining it failed");
            self.frame_state = FrameState::Failed;
            return;
        }

        if let FrameStatus::Success = self.api.frame_render()
        {
            self.frame_state = FrameState::Render;
        }
    }

    fn finish(&mut self)
    {
        if self.frame_state != FrameState::Render
        {
            log::warn!("Tried to end frame after rendering failed");
            self.frame_state = FrameState::Failed;
            return;
        }

        if let FrameStatus::Success = self.api.frame_finish()
        {
            self.frame_state = FrameState::Finish;
        }
    }
}


impl EventSubscriber<PlatformEvent> for Renderer
{
    fn receive_event(&mut self, event: &PlatformEvent)
    {
        if self.frame_state == FrameState::Destroy
        {
            return;
        }

        #[allow(clippy::single_match)]
        match event
        {
            PlatformEvent::WindowSizeChange(w, h) =>
            {
                if let Err(e) = self.api.surface_size_changed(*w, *h)
                {
                    log::error!("{e}");
                }
            }

            _ => (),
        }
    }
}
