#[allow(clippy::module_inception)]
mod layer;
pub use layer::*;


use std::any::TypeId;
use std::collections::HashMap;


pub struct LayerReg<E>
{
    layers: HashMap<TypeId, AnyLayer<E>>,
}


impl<E> LayerReg<E>
{
    pub fn new() -> Self
    {
        Self {
            layers: HashMap::new(),
        }
    }

    pub fn insert_any(&mut self, layer: AnyLayer<E>) -> Option<AnyLayer<E>>
    {
        if self.layers.contains_key(&layer.id())
        {
            return Some(layer);
        }

        self.layers.insert(layer.id(), layer);

        None
    }

    pub fn insert<T>(&mut self, layer: T) -> Option<T>
    where
        T: LayerDispatch<E> + Send + Sync + 'static,
    {
        let key = TypeId::of::<T>();

        if self.layers.contains_key(&key)
        {
            return Some(layer);
        }

        let layer = Layer::new(layer);
        self.layers.insert(key, layer.into());

        None
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Layer<T>>
    {
        self.layers
            .get(&TypeId::of::<T>())
            .map(|l| Layer::try_from(l).unwrap())
    }

    pub fn get_unchecked<T: Send + Sync + 'static>(&self) -> Layer<T>
    {
        self.get().unwrap()
    }

    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<Layer<T>>
    {
        self.layers
            .remove(&TypeId::of::<T>())
            .map(|l| Layer::try_from(&l).unwrap())
    }

    pub fn dispatch(&mut self, event: E)
    {
        for layer in self.layers.values_mut()
        {
            layer.dispatch(&event);
        }
    }
}


impl<E> Default for LayerReg<E>
{
    fn default() -> Self
    {
        Self::new()
    }
}


#[macro_export]
macro_rules! reg_inspect {
    ($reg:expr, $name:ident = $layer:ident => $f:expr) => {{
        if let Some(layer) = $reg.get::<$layer>()
        {
            #[allow(unused_mut)]
            if let Ok(mut $name) = layer.write()
            {
                $f;
            }
        }
    }};
}


#[macro_export]
macro_rules! layer_inspect {
    ($name:ident = $layer:expr => $f:expr) => {{
        #[allow(unused_mut)]
        if let Ok(mut $name) = $layer.write()
        {
            $f
        }
    }};
}


#[macro_export]
macro_rules! layer_read {
    ($layer:expr) => {
        $layer.read().unwrap()
    };
}


#[macro_export]
macro_rules! layer_write {
    ($layer:expr) => {
        $layer.write().unwrap()
    };
}


#[macro_export]
macro_rules! reg_read {
    ($reg:expr, $layer:ident) => {
        $reg.get_unchecked::<$layer>().read().unwrap()
    };
}


#[macro_export]
macro_rules! reg_write {
    ($reg:expr, $layer:ident) => {
        $reg.get_unchecked::<$layer>().write().unwrap()
    };
}
