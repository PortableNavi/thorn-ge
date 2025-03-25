use std::any::TypeId;
use std::collections::HashMap;
use thorn::prelude::*;


pub struct Plug
{
    pub info: PluginInfo,
    pub plugin: Box<dyn Plugin<LayerEvent>>,
}


impl Plug
{
    fn new(plugin: impl Plugin<LayerEvent> + 'static) -> Self
    {
        Self {
            info: plugin.info(),
            plugin: Box::new(plugin),
        }
    }

    pub fn deps(&self) -> &[TypeId]
    {
        &self.info.deps
    }

    pub fn id(&self) -> TypeId
    {
        self.info.identity
    }
}


pub struct PluginLoader
{
    plugins: HashMap<TypeId, Plug>,
    registry: LayerReg<LayerEvent>,
    loaded: Vec<Plug>,
}


impl PluginLoader
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self
    {
        Self {
            plugins: HashMap::new(),
            registry: LayerReg::new(),
            loaded: vec![],
        }
    }

    pub fn registry_mut(&mut self) -> &mut LayerReg<LayerEvent>
    {
        &mut self.registry
    }

    pub fn discover_plugin(&mut self, plugin: impl Plugin<LayerEvent> + 'static) -> Option<Plug>
    {
        let plug = Plug::new(plugin);

        if self.plugins.contains_key(&plug.id())
        {
            return Some(plug);
        }

        self.plugins.insert(plug.id(), plug);

        None
    }

    pub fn dep_sort(&self) -> ThResult<Vec<TypeId>>
    {
        let plugs = self.plugins.keys().cloned().collect::<Vec<_>>();
        let mut sorted = vec![];

        loop
        {
            let sorted_len = sorted.len();

            'outer: for plug in &plugs
            {
                if sorted.contains(plug)
                {
                    continue;
                }

                for dep in self.deps_of(*plug)
                {
                    if !sorted.contains(dep)
                    {
                        continue 'outer;
                    }
                }

                sorted.push(*plug);
            }

            if sorted_len == sorted.len()
            {
                break;
            }
        }

        if plugs.len() != sorted.len()
        {
            return Err(ThError::PluginLoadOrder);
        }

        Ok(sorted)
    }

    pub fn load_all(&mut self) -> ThResult<()>
    {
        for plug in self.dep_sort()?
        {
            let plug = self.plugins.remove(&plug);
            if let Some(plug) = plug
            {
                self.load_plugin(plug)?;
            }
        }

        Ok(())
    }

    pub fn unload_all(&mut self)
    {
        while let Some(plug) = self.loaded.pop()
        {
            self.unload_plugin(plug);
        }
    }

    fn deps_of(&self, id: TypeId) -> &[TypeId]
    {
        self.plugins.get(&id).map(|e| e.deps()).unwrap_or(&[])
    }

    fn load_plugin(&mut self, mut plugin: Plug) -> ThResult<()>
    {
        println!(
            "Loading Plugin {} version {}",
            plugin.info.name, plugin.info.version
        );

        match plugin.plugin.load(&self.registry)
        {
            Ok(layer) =>
            {
                if self.registry.insert_any(layer).is_some()
                {
                    return Err(ThError::PluginLoadFailed(
                        plugin.info.name,
                        "The plugin's Layer is already Loaded".into(),
                    ));
                }

                plugin.plugin.notify_loaded(&self.registry);
                self.loaded.push(plugin);
            }

            Err(e) => return Err(ThError::PluginLoadFailed(plugin.info.name, e.to_string())),
        }

        Ok(())
    }

    fn unload_plugin(&mut self, mut plugin: Plug)
    {
        println!(
            "Unloading Plugin {} version {}",
            plugin.info.name, plugin.info.version
        );

        plugin.plugin.notify_unloaded(&self.registry);
        self.plugins.insert(plugin.id(), plugin);
    }
}
