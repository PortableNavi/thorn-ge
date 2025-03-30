use std::any::{Any, TypeId, type_name};
use std::ops::Deref;
use std::sync::{Arc, RwLock};


pub trait LayerDispatch<E>
{
    fn dispatch(&mut self, event: &E);
}


pub struct Layer<T: Send + Sync>(Arc<RwLock<T>>);


impl<T: Send + Sync> Layer<T>
{
    pub fn new(layer: T) -> Self
    {
        Self(Arc::new(RwLock::new(layer)))
    }
}


impl<T: Send + Sync> Deref for Layer<T>
{
    type Target = Arc<RwLock<T>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}


impl<T: Send + Sync> Clone for Layer<T>
{
    fn clone(&self) -> Self
    {
        Self(self.0.clone())
    }
}


impl<E, T> TryFrom<&AnyLayer<E>> for Layer<T>
where
    T: Send + Sync + 'static,
{
    type Error = crate::error::ThError;

    fn try_from(value: &AnyLayer<E>) -> Result<Self, Self::Error>
    {
        if let Some(layer) = value.layer.downcast_ref::<Arc<RwLock<T>>>()
        {
            return Ok(Layer(layer.clone()));
        }

        Err(crate::error::ThError::LayerCastFailed(type_name::<T>()))
    }
}


pub struct AnyLayer<E>
{
    id: TypeId,
    dispatch: Arc<RwLock<dyn LayerDispatch<E> + Send + Sync>>,
    layer: Box<dyn Any + Send + Sync>,
}


impl<E> AnyLayer<E>
{
    pub fn new<T>(from: T) -> Self
    where
        T: LayerDispatch<E> + Send + Sync + 'static,
    {
        Layer::new(from).into()
    }

    pub fn id(&self) -> TypeId
    {
        self.id
    }
}


impl<E> LayerDispatch<E> for AnyLayer<E>
{
    fn dispatch(&mut self, event: &E)
    {
        self.dispatch.write().unwrap().dispatch(event)
    }
}


impl<E, T> From<Layer<T>> for AnyLayer<E>
where
    T: LayerDispatch<E> + Send + Sync + 'static,
{
    fn from(value: Layer<T>) -> Self
    {
        Self {
            dispatch: value.0.clone(),
            layer: Box::new(value.0),
            id: TypeId::of::<T>(),
        }
    }
}
