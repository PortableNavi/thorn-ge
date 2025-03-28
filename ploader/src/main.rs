#![feature(exitcode_exit_method)]


mod loader;
mod logger;


use loader::PluginLoader;
use plugin::SamplePlugin;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread;
use std::{process::ExitCode, time::Duration};
use thorn::engine::platform::ThornWindow;
use thorn::engine::{
    core::{Core, CoreMsg, CorePlugin},
    event::{EngineEvent, EventEmitterPlugin, EventReceiverPlugin},
    gobject_manager::GobjectManagerPlugin,
    platform::{PlatformEvent, PlatformPlugin},
    tasks::TasksPlugin,
};
use thorn::prelude::WindowParams;


fn main()
{
    if let Err(e) = logger::init()
    {
        eprintln!("ERROR: Failed to initialize logger: {e}");
    }

    let mut loader = PluginLoader::new();
    let (sender, core) = std::sync::mpsc::channel();

    let (winit, proxy) = match ThornWindow::prepare()
    {
        Ok(d) => d,
        Err(e) =>
        {
            log::error!("{e}");
            ExitCode::FAILURE.exit_process();
        }
    };

    // Engine Plugins
    loader.discover_plugin(CorePlugin(sender));
    loader.discover_plugin(EventEmitterPlugin::<EngineEvent>::default());
    loader.discover_plugin(EventReceiverPlugin::<EngineEvent>::default());
    loader.discover_plugin(EventEmitterPlugin::<PlatformEvent>::default());
    loader.discover_plugin(EventReceiverPlugin::<PlatformEvent>::default());
    loader.discover_plugin(PlatformPlugin(proxy));
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

    let window = ThornWindow::new(
        WindowParams::default(),
        loader.registry_mut().get().unwrap(),
    );

    let plugin_manager_handle = thread::spawn(move || manage_plugins(loader, core));
    window.run(winit);

    if let Err(e) = plugin_manager_handle.join()
    {
        log::error!("{e:?}");
    }
}


fn manage_plugins(mut loader: PluginLoader, core: Receiver<CoreMsg>)
{
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
