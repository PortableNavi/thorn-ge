use dialog::DialogBox;
use std::sync::{Arc, LazyLock, Mutex};
use std::thread::{self, JoinHandle};


/// Show a non blocking message box
pub fn show_msg_box(msg: impl Into<String>)
{
    static MSG_BOX_THREAD: LazyLock<Mutex<MsgBoxThread>> =
        LazyLock::new(|| Mutex::new(MsgBoxThread::new()));

    MSG_BOX_THREAD.lock().unwrap().show_msg(msg);
}


struct MsgBoxThread
{
    messages: Arc<Mutex<Vec<String>>>,
    handle: Option<JoinHandle<()>>,
}


impl MsgBoxThread
{
    fn new() -> Self
    {
        Self {
            messages: Arc::new(Mutex::new(vec![])),
            handle: None,
        }
    }

    fn show_msg(&mut self, msg: impl Into<String>)
    {
        self.messages.lock().unwrap().push(msg.into());

        if let Some(handle) = &self.handle
        {
            if handle.is_finished()
            {
                let _ = self.handle.take().unwrap().join();
            }
        }
        else
        {
            let messages = self.messages.clone();
            self.handle = Some(thread::spawn(move || Self::msg_work(messages)));
        }
    }

    fn msg_work(msgs: Arc<Mutex<Vec<String>>>)
    {
        loop
        {
            let msgs = msgs.lock().unwrap().drain(..).collect::<Vec<_>>();

            if msgs.is_empty()
            {
                break;
            }

            for msg in msgs
            {
                Self::show_msg_box(msg);
            }
        }
    }

    fn show_msg_box(msg: String)
    {
        if let Err(e) = dialog::Message::new(msg)
            .title(std::option_env!("THORN_APP_NAME").unwrap_or("Thorn Application"))
            .show()
        {
            log::error!("Failed to show system dialog box: {e}");
        }
    }
}


// Some qol macros for convinience
#[macro_export]
macro_rules! either
{
    {$condition:expr => $on_true:expr; $on_false:expr} => {if $condition {$on_true} else {$on_false}};
}

#[macro_export]
macro_rules! if_do {
    (($name:ident=$val:expr) => { $($condition:expr => $on_true:expr;)* }) => {
        {
            let mut $name = $val;
            $(if $condition {$on_true;})*
            $name
        }
    };

    ($($condition:expr => $on_true:expr;)*) => {{
        $(if $condition {$on_true;})*
        ()
    }};
}
