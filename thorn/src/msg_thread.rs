use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread::JoinHandle,
};


pub struct MsgThread<M: Send + Sync, R: Send + Sync>
{
    handle: JoinHandle<R>,
    sender: Sender<M>,
}


impl<M, R> MsgThread<M, R>
where
    R: Send + Sync + 'static,
    M: Send + Sync + 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Receiver<M>) -> R + Send + Sync + 'static,
    {
        let (sender, recv) = channel();
        let handle = std::thread::spawn(|| f(recv));

        Self { handle, sender }
    }

    pub fn msg(&self, msg: M)
    {
        let _ = self.sender.send(msg);
    }

    pub fn join(self) -> std::thread::Result<R>
    {
        self.handle.join()
    }

    pub fn is_finished(&self) -> bool
    {
        self.handle.is_finished()
    }
}
