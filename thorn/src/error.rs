use thiserror::Error;


pub type ThResult<T> = Result<T, ThError>;


#[derive(Debug, Error)]
pub enum ThError
{
    #[error("Failed to cast into {0:?} layer")]
    LayerCastFailed(&'static str),

    #[error("Failed to load Plugin {0:?} because of: {1}")]
    PluginLoadFailed(String, String),

    #[error("Failed to find a suitable plugin load order")]
    PluginLoadOrder,

    #[error("{0}")]
    Error(String),

    #[error("{0}")]
    RendererError(String),
}
