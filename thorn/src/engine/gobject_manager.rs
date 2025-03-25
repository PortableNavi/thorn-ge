use std::{
    sync::{
        Arc,
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use crate::prelude::*;

use super::tasks::EVERY_FRAME;


pub struct GobjectManagerPlugin;
impl Plugin<LayerEvent> for GobjectManagerPlugin
{
    fn info(&self) -> PluginInfo
    {
        PluginInfo::build::<GobjectManager>()
            .dep::<Tasks>()
            .dep::<EventReceiver<EngineEvent>>()
    }

    fn load(
        &mut self,
        reg: &LayerReg<LayerEvent>,
    ) -> Result<AnyLayer<LayerEvent>, Box<dyn std::error::Error>>
    {
        let tasks = reg
            .get()
            .ok_or(ThError::Error("Failed to fetch engine tasks layer".into()))?;

        let event_receiver = reg.get().ok_or(ThError::Error(
            "Failed to fetch engine event receiver layer".into(),
        ))?;

        Ok(AnyLayer::new(GobjectManager::new(event_receiver, tasks)))
    }

    fn notify_loaded(&mut self, reg: &LayerReg<LayerEvent>)
    {
        let me = reg.get::<GobjectManager>().unwrap();
        me.clone().write().unwrap().init(me);
    }

    fn notify_unloaded(&mut self, reg: &LayerReg<LayerEvent>)
    {
        reg.get::<GobjectManager>()
            .unwrap()
            .write()
            .unwrap()
            .destroy();
    }
}


pub trait Gobject: Send + Sync
{
    fn tick(&mut self, _delta: Duration) {}
    fn test_event_a(&mut self) {}
    fn reset(&mut self) {}
    fn destroy(&mut self) {}
}


#[derive(Clone)]
struct GobjContainer
{
    id: u64,
    obj: Arc<Mutex<dyn Gobject>>,
}


impl GobjContainer
{
    fn new(obj: impl Gobject + 'static) -> Self
    {
        static ID: AtomicU64 = AtomicU64::new(0);

        Self {
            id: ID.fetch_add(1, Ordering::Relaxed),
            obj: Arc::new(Mutex::new(obj)),
        }
    }
}


pub struct GobjectManager
{
    event_receiver: Layer<EventReceiver<EngineEvent>>,
    tasks: Layer<Tasks>,
    task_id: Option<u64>,
    gobjs: Vec<GobjContainer>,
}


impl GobjectManager
{
    pub fn add_gobj(&mut self, gobj: impl Gobject + 'static) -> u64
    {
        let obj = GobjContainer::new(gobj);
        let id = obj.id;
        obj.obj.lock().unwrap().reset();
        self.gobjs.push(obj);
        id
    }

    pub fn remove_obj(&mut self, gobj: u64)
    {
        if let Some(index) = self.gobjs.iter().position(|e| e.id == gobj)
        {
            self.gobjs[index].obj.lock().unwrap().destroy();
            self.gobjs.remove(index);
        }
    }

    fn new(event_receiver: Layer<EventReceiver<EngineEvent>>, tasks: Layer<Tasks>) -> Self
    {
        Self {
            event_receiver,
            tasks,
            task_id: None,
            gobjs: vec![],
        }
    }

    fn init(&mut self, me: Layer<Self>)
    {
        self.event_receiver.write().unwrap().subscribe(me.clone());

        self.task_id = Some(self.tasks.write().unwrap().repeating(
            EVERY_FRAME,
            move |frame_info| {
                Self::gobject_dispatch_task(&me, frame_info)?;
                Ok(())
            },
        ));
    }

    fn destroy(&mut self)
    {
        if let Some(id) = self.task_id
        {
            self.tasks.write().unwrap().cancel(id);

            for id in self.gobjs().iter().map(|e| e.id)
            {
                self.remove_obj(id);
            }
        }
    }

    fn gobjs(&self) -> Vec<GobjContainer>
    {
        self.gobjs.clone()
    }

    fn gobject_dispatch_task(me: &Layer<Self>, frame_info: &FrameInfo) -> ThResult<()>
    {
        let gobjs = me.read().unwrap().gobjs();

        for gobj in gobjs
        {
            gobj.obj.lock().unwrap().tick(frame_info.delta);
        }

        Ok(())
    }
}


impl LayerDispatch<LayerEvent> for GobjectManager
{
    fn dispatch(&mut self, _event: &LayerEvent) {}
}


impl EventSubscriber<EngineEvent> for GobjectManager
{
    fn receive_event(&mut self, event: &EngineEvent)
    {
        if let EngineEvent::TestEventA = event
        {
            for gobj in &self.gobjs
            {
                gobj.obj.lock().unwrap().test_event_a();
            }
        }
    }
}
