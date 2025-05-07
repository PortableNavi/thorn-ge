use std::marker::PhantomData;

use crate::prelude::*;


pub struct EventEmitterPlugin<E>(PhantomData<E>);
impl<E: Send + Sync + 'static> Plugin<LayerEvent> for EventEmitterPlugin<E>
{
    fn info(&self) -> PluginInfo
    {
        PluginInfo::build::<EventEmitter<E>>()
    }

    fn load(
        &mut self,
        _reg: &LayerReg<LayerEvent>,
    ) -> Result<AnyLayer<LayerEvent>, Box<dyn std::error::Error>>
    {
        Ok(AnyLayer::new(EventEmitter::<E>::new()))
    }
}


impl<E> Default for EventEmitterPlugin<E>
{
    fn default() -> Self
    {
        EventEmitterPlugin(PhantomData)
    }
}


pub struct EventEmitter<E>
{
    buffer: Vec<E>,
}


impl<E> EventEmitter<E>
{
    #[allow(clippy::new_without_default)]
    fn new() -> Self
    {
        Self { buffer: vec![] }
    }

    pub fn emit(&mut self, event: E)
    {
        self.buffer.push(event);
    }

    pub(crate) fn drain_into(&mut self, b: &mut Vec<E>)
    {
        b.append(&mut self.buffer);
    }
}


impl<E> LayerDispatch<LayerEvent> for EventEmitter<E>
{
    fn dispatch(&mut self, _event: &LayerEvent) {}
}
