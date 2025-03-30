pub mod core;
pub mod event;
pub mod gobject_manager;
pub mod platform;
pub mod tasks;

pub mod prelude
{
    pub use super::core::{Core, CoreHook, FrameInfo};
    pub use super::event::{EventEmitter, EventReceiver};
    pub use super::gobject_manager::{Gobject, GobjectManager};
    pub use super::platform::{Platform, PlatformEvent, WindowParams};
    pub use super::tasks::Tasks;
}
