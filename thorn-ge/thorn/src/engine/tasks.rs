use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use crate::engine::core::MainLoopTask;
use crate::prelude::*;


pub const EVERY_FRAME: Duration = Duration::from_secs(0);


pub struct TasksPlugin;
impl Plugin<LayerEvent> for TasksPlugin
{
    fn info(&self) -> PluginInfo
    {
        PluginInfo::build::<Tasks>().dep::<Core>()
    }

    fn load(
        &mut self,
        reg: &LayerReg<LayerEvent>,
    ) -> Result<AnyLayer<LayerEvent>, Box<dyn std::error::Error>>
    {
        let core = reg
            .get::<Core>()
            .ok_or(ThError::Error("Failed to fetch core layer".into()))?;
        Ok(AnyLayer::new(Tasks::new(core)))
    }
}


struct Task
{
    id: u64,
    cb: MainLoopTask,
    repeats: Duration,
    last_executed: Instant,
}


impl Task
{
    fn new<T>(repeats: Duration, task: T) -> Self
    where
        T: Fn(&FrameInfo) -> Result<(), Box<dyn Error>> + Send + Sync + 'static,
    {
        static ID: AtomicU64 = AtomicU64::new(0);
        let id = ID.fetch_add(1, Ordering::Relaxed);

        Task {
            id,
            repeats,
            cb: Arc::new(task),
            last_executed: Instant::now(),
        }
    }
}


pub struct Tasks
{
    tasks: Vec<Task>,
    oneshots: Vec<Task>,
    core: Layer<Core>,
}


impl Tasks
{
    fn new(core: Layer<Core>) -> Self
    {
        Tasks {
            core,
            tasks: vec![],
            oneshots: vec![],
        }
    }

    pub fn oneshot<T>(&mut self, delay: Duration, task: T) -> u64
    where
        T: Fn(&FrameInfo) -> Result<(), Box<dyn Error>> + Send + Sync + 'static,
    {
        let task = Task::new(delay, task);
        let id = task.id;
        self.oneshots.push(task);
        id
    }


    pub fn repeating<T>(&mut self, interval: Duration, task: T) -> u64
    where
        T: Fn(&FrameInfo) -> Result<(), Box<dyn Error>> + Send + Sync + 'static,
    {
        let task = Task::new(interval, task);
        let id = task.id;
        self.tasks.push(task);
        id
    }

    pub fn hook(&mut self, hook: impl CoreHook + 'static)
    {
        self.core.write().unwrap().add_hook(hook);
    }

    pub fn cancel(&mut self, task: u64)
    {
        self.tasks.retain(|e| e.id != task);
        self.oneshots.retain(|e| e.id != task);
    }
}


impl LayerDispatch<LayerEvent> for Tasks
{
    fn dispatch(&mut self, event: &LayerEvent)
    {
        if let LayerEvent::Tick(_) = event
        {
            let mut core = self.core.write().unwrap();

            for task in &mut self.tasks
            {
                if task.last_executed.elapsed() > task.repeats
                {
                    core.schedule_task(task.cb.clone());
                    task.last_executed = Instant::now();
                }
            }

            let mut pending = vec![];
            for task in self.oneshots.drain(..)
            {
                if task.last_executed.elapsed() > task.repeats
                {
                    core.schedule_task(task.cb);
                }
                else
                {
                    pending.push(task);
                }
            }

            self.oneshots = pending;
        }
    }
}
