#![feature(exitcode_exit_method)]


mod loader;
mod logger;

use loader::PluginLoader;
use plugin::SamplePlugin;
use std::sync::mpsc::RecvTimeoutError;
use std::{process::ExitCode, time::Duration};
use thorn::engine::{
    core::{Core, CoreMsg, CorePlugin},
    event::{EngineEvent, EventEmitterPlugin, EventReceiverPlugin},
    gobject_manager::GobjectManagerPlugin,
    tasks::TasksPlugin,
};


fn main()
{
    if let Err(e) = logger::init()
    {
        eprintln!("ERROR: Failed to initialize logger: {e}");
    }

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
    if loader.load_all().is_err()
    {
        log::error!("Failed to resolve plugin dependencies. exting");
        loader.unload_all();
        ExitCode::FAILURE.exit_process();
    }

    loop
    {
        match core.recv_timeout(Duration::from_secs(1))
        {
            // Dispatch layer events from core to all other layers.
            Ok(CoreMsg::Dispatch(event)) => loader.registry_mut().dispatch(event),

            // Check if the main loop is still alive on timeout
            Err(RecvTimeoutError::Timeout) =>
            {
                log::warn!("Core loop connection timed out...");

                if !loader
                    .registry_mut()
                    .get::<Core>()
                    .unwrap()
                    .read()
                    .unwrap()
                    .is_alive()
                {
                    log::error!("Core loop seems to have crashed. Exiting");
                    break;
                }
            }

            // Exit if the core loop connection drops
            Err(RecvTimeoutError::Disconnected) =>
            {
                log::error!("Core loop seems to have crashed. Exiting");
                break;
            }

            // Exit if termination msg is received
            Ok(CoreMsg::Terminate) => break,
        }
    }

    // unload all plugins...
    loader.unload_all();

    log::info!("Exiting. Good Bye.");
}
