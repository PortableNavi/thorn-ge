mod gobj;

use gobj::{GobjA, GobjB};
use std::time::Duration;
use thorn::prelude::*;


pub struct SamplePlugin;
impl Plugin<LayerEvent> for SamplePlugin
{
    fn info(&self) -> PluginInfo
    {
        PluginInfo::build::<Sample>()
            .dep::<EventReceiver<PlatformEvent>>()
            .dep::<GobjectManager>()
            .dep::<Core>()
            .dep::<Tasks>()
    }

    fn load(
        &mut self,
        reg: &LayerReg<LayerEvent>,
    ) -> Result<AnyLayer<LayerEvent>, Box<dyn std::error::Error>>
    {
        let tasks = reg
            .get()
            .ok_or(ThError::Error("Failed to fetch engine task layer".into()))?;

        let gobj_manager = reg
            .get()
            .ok_or(ThError::Error("Failed to fetch engine gobj manager".into()))?;

        let platform_events = reg.get().ok_or(ThError::Error(
            "Failed to fetch platform event receiver".into(),
        ))?;

        let core = reg
            .get()
            .ok_or(ThError::Error("Failed to fetch core layer".into()))?;

        Ok(AnyLayer::new(Sample {
            tasks,
            gobj_manager,
            core,
            platform_events,
        }))
    }

    fn notify_loaded(&mut self, reg: &LayerReg<LayerEvent>)
    {
        let me = reg.get::<Sample>().unwrap();
        let me_cloned = me.clone();
        me.read().unwrap().init(me_cloned);
    }
}


pub struct Sample
{
    platform_events: Layer<EventReceiver<PlatformEvent>>,
    core: Layer<Core>,
    tasks: Layer<Tasks>,
    gobj_manager: Layer<GobjectManager>,
}


impl Sample
{
    fn init(&self, me: Layer<Sample>)
    {
        // subscribe to platform events, in order to handle window close events...
        self.platform_events.write().unwrap().subscribe(me);

        // Schedule a task that prints the fps every second
        self.tasks.write().unwrap().repeating(
            Duration::from_secs(1),
            move |FrameInfo { delta }| {
                println!("FPS: {:.2}", 1.0 / delta.as_secs_f32());
                Ok(())
            },
        );

        // Test out two dummy game objects
        let mut gobj_manager = self.gobj_manager.write().unwrap();
        gobj_manager.add_gobj(GobjA);
        gobj_manager.add_gobj(GobjB);
    }
}


impl LayerDispatch<LayerEvent> for Sample
{
    fn dispatch(&mut self, _event: &LayerEvent) {}
}


impl EventSubscriber<PlatformEvent> for Sample
{
    fn receive_event(&mut self, event: &PlatformEvent)
    {
        if let PlatformEvent::WindowClose = event
        {
            self.core.read().unwrap().terminate();
        }
    }
}
