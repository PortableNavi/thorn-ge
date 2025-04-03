// Reexport platform for each implementation
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
mod winit;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
mod winit_impl;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub use winit::{Platform, PlatformPlugin};
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub use winit_impl::ThornWindow;


//TODO: android and web in the future ???


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformEvent
{
    WindowOpen,
    WindowClose,
    WindowLostFocus,
    WindowGotFocus,
    WindowMinimized,
    WindowSizeChange(u32, u32),
    WindowPositionChange(i32, i32),
    PlatformError(String),
}


#[derive(Debug, Clone, PartialEq)]
pub struct WindowParams
{
    pub title: String,
    pub position: Option<(i32, i32)>,
    pub size: (u32, u32),
}


impl Default for WindowParams
{
    fn default() -> Self
    {
        Self {
            title: std::option_env!("THORN_APP_NAME")
                .unwrap_or("Thorn Application")
                .into(),
            position: None,
            size: (800, 600),
        }
    }
}
