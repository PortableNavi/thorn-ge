use std::marker::PhantomData;

use crate::prelude::*;


pub struct EventReceiverPlugin<E>(PhantomData<E>);
impl<E: Send + Sync + 'static> Plugin<LayerEvent> for EventReceiverPlugin<E>
{
    fn info(&self) -> PluginInfo
    {
        PluginInfo::build::<EventReceiver<E>>().dep::<EventEmitter<E>>()
    }

    fn load(
        &mut self,
        reg: &LayerReg<LayerEvent>,
    ) -> Result<AnyLayer<LayerEvent>, Box<dyn std::error::Error>>
    {
        let emitter = reg
            .get::<EventEmitter<E>>()
            .ok_or(ThError::Error("Failed to fetch matching emitter".into()))?;

        Ok(AnyLayer::new(EventReceiver::<E>::new(emitter)))
    }
}


impl<E> Default for EventReceiverPlugin<E>
{
    fn default() -> Self
    {
        EventReceiverPlugin(PhantomData)
    }
}


pub struct EventReceiver<E: Send + Sync>
{
    events: Vec<E>,
    emitter: Layer<EventEmitter<E>>,
    subscribers: Vec<Box<dyn EventSubscriber<E>>>,
}


impl<E: Send + Sync> EventReceiver<E>
{
    #[allow(clippy::new_without_default)]
    fn new(emitter: Layer<EventEmitter<E>>) -> Self
    {
        Self {
            emitter,
            subscribers: vec![],
            events: Vec::with_capacity(10),
        }
    }

    pub fn subscribe(&mut self, subscriber: impl EventSubscriber<E> + 'static)
    {
        self.subscribers.push(Box::new(subscriber));
    }
}


impl<E: Send + Sync> LayerDispatch<LayerEvent> for EventReceiver<E>
{
    fn dispatch(&mut self, event: &LayerEvent)
    {
        if let LayerEvent::Tick(_) = event
        {
            self.emitter.write().unwrap().drain_into(&mut self.events);

            for e in &mut self.events
            {
                self.subscribers
                    .iter_mut()
                    .for_each(|sub| sub.receive_event(e))
            }

            self.events.clear();
        }
    }
}


pub trait EventSubscriber<E>: Send + Sync
{
    fn receive_event(&mut self, event: &E);
}


impl<T: Send + Sync, E> EventSubscriber<E> for Layer<T>
where
    T: EventSubscriber<E>,
{
    fn receive_event(&mut self, event: &E)
    {
        self.write().unwrap().receive_event(event);
    }
}
