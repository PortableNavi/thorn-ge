use std::time::Duration;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerEvent
{
    Tick(Duration),
    Panic,
}
