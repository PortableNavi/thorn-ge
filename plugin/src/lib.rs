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
            .dep::<EventEmitter<EngineEvent>>()
            .dep::<EventReceiver<EngineEvent>>()
            .dep::<GobjectManager>()
            .dep::<Tasks>()
    }

    fn load(
        &mut self,
        reg: &LayerReg<LayerEvent>,
    ) -> Result<AnyLayer<LayerEvent>, Box<dyn std::error::Error>>
    {
        let event_receiver = reg.get().ok_or(ThError::Error(
            "Failed to fetch engine event receiver".into(),
        ))?;

        let event_emitter = reg.get().ok_or(ThError::Error(
            "Failed to fetch engine event emitter".into(),
        ))?;

        let tasks = reg
            .get()
            .ok_or(ThError::Error("Failed to fetch engine task layer".into()))?;

        let gobj_manager = reg
            .get()
            .ok_or(ThError::Error("Failed to fetch engine gobj manager".into()))?;

        Ok(AnyLayer::new(Sample {
            tasks,
            event_receiver,
            event_emitter,
            gobj_manager,
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
    event_receiver: Layer<EventReceiver<EngineEvent>>,
    event_emitter: Layer<EventEmitter<EngineEvent>>,
    tasks: Layer<Tasks>,
    gobj_manager: Layer<GobjectManager>,
}


impl Sample
{
    fn init(&self, me: Layer<Sample>)
    {
        // Register this Layer as a event receiver...
        self.event_emitter
            .write()
            .unwrap()
            .emit(EngineEvent::TestEventA);

        self.event_receiver.write().unwrap().subscribe(me);

        // Schedule a task that prints the fps every second
        self.tasks.write().unwrap().repeating(
            Duration::from_secs(1),
            move |FrameInfo { delta }| {
                println!("FPS: {:.2}", 1.0 / delta.as_secs_f32());
                Ok(())
            },
        );

        let mut gobj_manager = self.gobj_manager.write().unwrap();
        gobj_manager.add_gobj(GobjA);
        gobj_manager.add_gobj(GobjB);
    }
}


impl LayerDispatch<LayerEvent> for Sample
{
    fn dispatch(&mut self, _event: &LayerEvent) {}
}


impl EventSubscriber<EngineEvent> for Sample
{
    fn receive_event(&mut self, event: &EngineEvent)
    {
        println!("{:?}", event);
    }
}
