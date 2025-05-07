use std::any::{TypeId, type_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInfo
{
    pub name: String,
    pub version: String,
    pub identity: TypeId,
    pub deps: Vec<TypeId>,
}


impl PluginInfo
{
    pub fn build<T: 'static>() -> PluginInfo
    {
        let name = type_name::<T>()
            .split("<")
            .filter_map(|p| p.split("::").last())
            .collect::<Vec<_>>()
            .join("<");

        PluginInfo {
            name,
            version: std::env!("CARGO_PKG_VERSION").into(),
            identity: TypeId::of::<T>(),
            deps: Vec::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self
    {
        self.name = name.into();
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self
    {
        self.version = version.into();
        self
    }

    pub fn dep<T: 'static>(mut self) -> Self
    {
        self.deps.push(TypeId::of::<T>());
        self
    }
}
