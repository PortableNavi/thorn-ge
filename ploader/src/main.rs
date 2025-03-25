mod loader;
use std::time::Duration;

use loader::PluginLoader;
use plugin::SamplePlugin;
use thorn::engine::{
    core::{Core, CoreMsg, CorePlugin},
    event::{EngineEvent, EventEmitterPlugin, EventReceiverPlugin},
    gobject_manager::GobjectManagerPlugin,
    tasks::TasksPlugin,
};


fn main()
{
    let mut loader = PluginLoader::new();

    let (sender, core) = std::sync::mpsc::channel();

    // Engine Plugins
    loader.discover_plugin(CorePlugin(sender));
    loader.discover_plugin(EventEmitterPlugin::<EngineEvent>::default());
    loader.discover_plugin(EventReceiverPlugin::<EngineEvent>::default());
    loader.discover_plugin(TasksPlugin);
    loader.discover_plugin(GobjectManagerPlugin);

    // Static library plugins
    loader.discover_plugin(SamplePlugin);

    // Load all plugins
    loader.load_all().unwrap();

    loop
    {
        match core.recv_timeout(Duration::from_secs(1))
        {
            // Dispatch layer events from core to all other layers.
            Ok(CoreMsg::Dispatch(event)) => loader.registry_mut().dispatch(event),

            // Check if the main loop is still alive on timeout
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) =>
            {
                if !loader
                    .registry_mut()
                    .get::<Core>()
                    .unwrap()
                    .read()
                    .unwrap()
                    .is_alive()
                {
                    break;
                }
            }

            // break if the core connection drops or a termination msg is send.
            Err(_) | Ok(CoreMsg::Terminate) => break,
        }
    }

    // unload all plugins...
    loader.unload_all();
}
