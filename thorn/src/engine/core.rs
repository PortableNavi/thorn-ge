use crate::prelude::*;
use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::{Duration, Instant};


pub(crate) type MainLoopTask = Arc<dyn Fn(&FrameInfo) -> Result<(), Box<dyn Error>> + Send + Sync>;


pub enum CoreMsg
{
    Terminate,
    Dispatch(LayerEvent),
}


pub trait CoreHook: Send + Sync
{
    fn prepare(&mut self) {}
    fn tick(&mut self, _frame_info: &FrameInfo) {}
    fn finish(&mut self) {}
}


impl<T: CoreHook + Send + Sync> CoreHook for Layer<T>
{
    fn prepare(&mut self)
    {
        self.write().unwrap().prepare()
    }

    fn tick(&mut self, info: &FrameInfo)
    {
        self.write().unwrap().tick(info)
    }

    fn finish(&mut self)
    {
        self.write().unwrap().finish();
    }
}


pub struct CorePlugin(pub Sender<CoreMsg>);
impl Plugin<LayerEvent> for CorePlugin
{
    fn info(&self) -> PluginInfo
    {
        PluginInfo::build::<Core>()
    }

    fn load(
        &mut self,
        _reg: &LayerReg<LayerEvent>,
    ) -> Result<AnyLayer<LayerEvent>, Box<dyn std::error::Error>>
    {
        Ok(AnyLayer::new(Core::new(self.0.clone())))
    }

    fn notify_loaded(&mut self, reg: &LayerReg<LayerEvent>)
    {
        let core = reg.get::<Core>().unwrap();
        let core_clone = core.clone();
        core.write().unwrap().start_main_loop(core_clone);
    }

    fn notify_unloaded(&mut self, reg: &LayerReg<LayerEvent>)
    {
        reg.get::<Core>()
            .unwrap()
            .write()
            .unwrap()
            .terminate_application();
    }
}


enum MainLoopMsg
{
    Terminate,
    SetFpsCap(u32),
    SetTasks(Vec<MainLoopTask>),
    AddHooks(Vec<Box<dyn CoreHook>>),
}


struct MainLoop
{
    conn: Sender<MainLoopMsg>,
    handle: JoinHandle<()>,
}


pub struct FrameInfo
{
    pub delta: Duration,
}


pub struct Core
{
    loader: Sender<CoreMsg>,
    main_loop: Option<MainLoop>,
    tasks: Option<Vec<MainLoopTask>>,
    hooks: Option<Vec<Box<dyn CoreHook>>>,
}

impl Core
{
    #[allow(clippy::new_without_default)]
    pub fn new(loader: Sender<CoreMsg>) -> Self
    {
        Self {
            loader,
            main_loop: None,
            tasks: None,
            hooks: None,
        }
    }

    pub fn dispatch(&self, event: LayerEvent)
    {
        let _ = self.loader.send(CoreMsg::Dispatch(event));
    }

    pub fn set_fps_cap(&self, max_fps: u32)
    {
        if let Some(m) = &self.main_loop
        {
            let _ = m.conn.send(MainLoopMsg::SetFpsCap(max_fps));
        }
    }

    pub fn terminate(&self)
    {
        if let Some(m) = &self.main_loop
        {
            let _ = m.conn.send(MainLoopMsg::Terminate);
        }
        else
        {
            self.terminate_loader();
        }
    }

    pub fn is_alive(&self) -> bool
    {
        if let Some(m) = &self.main_loop
        {
            return !m.handle.is_finished();
        }

        false
    }

    pub fn schedule_task(&mut self, task: MainLoopTask)
    {
        if self.tasks.is_none()
        {
            self.tasks = Some(vec![task]);
        }
        else
        {
            self.tasks.as_mut().unwrap().push(task);
        }
    }

    pub(crate) fn add_hook(&mut self, hook: impl CoreHook + 'static)
    {
        if self.hooks.is_none()
        {
            self.hooks = Some(vec![Box::new(hook)])
        }
        else
        {
            self.hooks.as_mut().unwrap().push(Box::new(hook));
        }
    }

    fn start_main_loop(&mut self, core: Layer<Core>)
    {
        // I'd rather crash than have 2 main loops...
        assert!(self.main_loop.is_none());

        // Add a Ctrl-C handler
        let core_clone = core.clone();
        let _ = ctrlc::set_handler(move || core_clone.read().unwrap().terminate());

        let (conn, msg) = channel();
        let handle = spawn(|| main_loop(core, msg));
        self.main_loop = Some(MainLoop { conn, handle });
    }

    fn hand_over_tasks(&mut self)
    {
        if let Some(tasks) = self.tasks.take()
        {
            if let Some(m) = &self.main_loop
            {
                let _ = m.conn.send(MainLoopMsg::SetTasks(tasks));
            }
        }
    }

    fn hand_over_hooks(&mut self)
    {
        if let Some(hooks) = self.hooks.take()
        {
            if let Some(m) = &self.main_loop
            {
                let _ = m.conn.send(MainLoopMsg::AddHooks(hooks));
            }
        }
    }

    fn terminate_application(&mut self)
    {
        self.terminate();
        if let Some(m) = self.main_loop.take()
        {
            let _ = m.handle.join();
        }

        self.terminate_loader();
    }

    fn terminate_loader(&self)
    {
        let _ = self.loader.send(CoreMsg::Terminate);
    }
}


impl LayerDispatch<LayerEvent> for Core
{
    fn dispatch(&mut self, event: &LayerEvent)
    {
        if let LayerEvent::Tick(_) = event
        {
            self.hand_over_tasks();
            self.hand_over_hooks();
        }
    }
}


fn main_loop(core: Layer<Core>, msg: Receiver<MainLoopMsg>)
{
    // Hard cap fps at a thousand frames per second,
    // because at completly uncapped fps things start to break...
    const MIN_FRAME_TIME: Duration = Duration::from_millis(1);

    let mut delta = Duration::from_secs(0);
    let mut fps_cap = Duration::from_secs_f64(1.0 / 120.0);
    let mut tasks = vec![];
    let mut hooks = vec![];

    'mainloop: loop
    {
        let frame_start = Instant::now();

        // Dispatch all new messages...
        'msgloop: loop
        {
            match msg.try_recv()
            {
                Ok(MainLoopMsg::SetFpsCap(cap)) =>
                {
                    fps_cap = {
                        let target_frame_time = Duration::from_secs_f64(1.0 / cap as f64);

                        if target_frame_time < MIN_FRAME_TIME
                        {
                            log::warn!(
                                "Warning: Framerate cap is above 1000 FPS. Capping at 1000 FPS..."
                            );
                            MIN_FRAME_TIME
                        }
                        else
                        {
                            target_frame_time
                        }
                    }
                }

                // Receive all new hooks...
                Ok(MainLoopMsg::AddHooks(h)) => hooks.extend(h),

                // Add newly received tasks to the task queue
                Ok(MainLoopMsg::SetTasks(t)) => tasks.extend(t),

                // Do nothing if there is no msg.
                Err(TryRecvError::Empty) => break 'msgloop,

                // Break the mainloop if the loader drops its channel or if a termination msg is sent.
                Err(TryRecvError::Disconnected) | Ok(MainLoopMsg::Terminate) => break 'mainloop,
            }
        }

        // Run all Frame preparation hooks
        for hook in &mut hooks
        {
            hook.prepare();
        }

        // Tick events to all layers before any work in the main loop...
        core.read().unwrap().dispatch(LayerEvent::Tick(delta));

        // FRAME START
        //////////////

        let frame_info = FrameInfo { delta };

        // Run all frame tick hooks
        for hook in &mut hooks
        {
            hook.tick(&frame_info);
        }

        // TODO: Maybe figure out something more smart with the tasks...
        while let Some(task) = tasks.pop()
        {
            if let Err(e) = task(&frame_info)
            {
                log::error!("Error while executing task in main loop: {e:?}");
            }
        }

        // Run all frame end hooks
        for hook in &mut hooks
        {
            hook.finish();
        }

        // FRAME END
        ////////////
        let frame_end = Instant::now();
        delta = frame_end - frame_start;

        let sleep_time = fps_cap.saturating_sub(delta);
        if sleep_time > Duration::from_secs(0)
        {
            sleep(sleep_time);
            delta += sleep_time;
        }
    }

    core.read().unwrap().terminate_loader();
    log::info!("Core loop stopped");
}
