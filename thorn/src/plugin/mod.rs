pub mod plugin_info;
pub use plugin_info::PluginInfo;


use crate::layer::{AnyLayer, LayerReg};


pub trait Plugin<E>
{
    fn load(&mut self, reg: &LayerReg<E>) -> Result<AnyLayer<E>, Box<dyn std::error::Error>>;
    fn info(&self) -> PluginInfo;
    fn notify_loaded(&mut self, _reg: &LayerReg<E>) {}
    fn notify_unloaded(&mut self, _reg: &LayerReg<E>) {}
}
