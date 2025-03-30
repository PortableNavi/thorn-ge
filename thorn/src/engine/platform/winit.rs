use crate::prelude::*;
use winit::event_loop::EventLoopProxy;


pub enum WinitMsg
{
    Terminate,
}


pub struct PlatformPlugin(pub EventLoopProxy<WinitMsg>);
impl Plugin<LayerEvent> for PlatformPlugin
{
    fn info(&self) -> PluginInfo
    {
        PluginInfo::build::<Platform>()
            .dep::<EventEmitter<PlatformEvent>>()
            .dep::<Renderer>()
    }

    fn load(
        &mut self,
        reg: &LayerReg<LayerEvent>,
    ) -> Result<AnyLayer<LayerEvent>, Box<dyn std::error::Error>>
    {
        let event_emitter = reg.get().ok_or(ThError::Error(
            "Failed to fetch platform event emitter".into(),
        ))?;


        Ok(AnyLayer::new(Platform {
            _event_emitter: event_emitter,
            proxy: self.0.clone(),
        }))
    }

    fn notify_unloaded(&mut self, reg: &LayerReg<LayerEvent>)
    {
        reg.get::<Platform>().unwrap().write().unwrap().terminate();
    }
}


pub struct Platform
{
    _event_emitter: Layer<EventEmitter<PlatformEvent>>,
    proxy: EventLoopProxy<WinitMsg>,
}


impl Platform
{
    fn terminate(&mut self)
    {
        let _ = self.proxy.send_event(WinitMsg::Terminate);
    }
}


impl LayerDispatch<LayerEvent> for Platform
{
    fn dispatch(&mut self, _event: &LayerEvent) {}
}
