mod emitter;
mod receiver;

pub use emitter::{EventEmitter, EventEmitterPlugin};
pub use receiver::{EventReceiver, EventReceiverPlugin, EventSubscriber};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent
{
    TestEventA,
    TestEventB,
}
