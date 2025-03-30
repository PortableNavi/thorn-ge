use super::winit::WinitMsg;
use crate::prelude::*;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    raw_window_handle::HasWindowHandle,
    window::Window,
};


pub struct ThornWindow
{
    params: WindowParams,
    window: Option<Window>,
    event_emitter: Layer<EventEmitter<PlatformEvent>>,
    renderer: Layer<Renderer>,
    is_renderer_initialized: bool,
}


impl ApplicationHandler<WinitMsg> for ThornWindow
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop)
    {
        if self.window.is_none()
        {
            self.create_window(event_loop);
        }

        if !self.is_renderer_initialized
        {
            match self.renderer.write().unwrap().initialize(
                self.window
                    .as_mut()
                    .unwrap()
                    .window_handle()
                    .expect("No Window handle...")
                    .as_raw(),
            )
            {
                Err(e) => log::error!("Failed to initialize renderer: {e}"),
                Ok(_) => self.is_renderer_initialized = true,
            }
        }

        //TODO: Create new surface
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    )
    {
        match event
        {
            WindowEvent::Resized(size) =>
            {
                self.event_emitter
                    .write()
                    .unwrap()
                    .emit(PlatformEvent::WindowSizeChange(size.width, size.height));
            }

            WindowEvent::Moved(pos) =>
            {
                self.event_emitter
                    .write()
                    .unwrap()
                    .emit(PlatformEvent::WindowPositionChange(pos.x, pos.y));
            }

            WindowEvent::CloseRequested =>
            {
                self.event_emitter
                    .write()
                    .unwrap()
                    .emit(PlatformEvent::WindowClose);
            }

            WindowEvent::Focused(true) =>
            {
                self.event_emitter
                    .write()
                    .unwrap()
                    .emit(PlatformEvent::WindowGotFocus)
            }

            WindowEvent::Focused(false) =>
            {
                self.event_emitter
                    .write()
                    .unwrap()
                    .emit(PlatformEvent::WindowLostFocus)
            }

            _ => (),
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WinitMsg)
    {
        match event
        {
            WinitMsg::Terminate =>
            {
                log::info!("closing window");
                event_loop.exit()
            }
        }
    }
}


impl ThornWindow
{
    fn create_window(&mut self, event_loop: &ActiveEventLoop)
    {
        let mut attributes = Window::default_attributes()
            .with_title(&self.params.title)
            .with_visible(true)
            .with_inner_size(LogicalSize::new(self.params.size.0, self.params.size.1));

        if let Some((x, y)) = self.params.position
        {
            attributes = attributes.with_position(LogicalPosition::new(x, y));
        };

        match event_loop.create_window(attributes)
        {
            Ok(window) =>
            {
                log::info!("Window created");
                self.window = Some(window)
            }

            Err(e) =>
            {
                log::error!("Failed to create a window: {e}");
                self.event_emitter
                    .write()
                    .unwrap()
                    .emit(PlatformEvent::PlatformError(e.to_string()))
            }
        }
    }

    pub fn prepare() -> ThResult<(EventLoop<WinitMsg>, EventLoopProxy<WinitMsg>)>
    {
        let event_loop = EventLoop::with_user_event()
            .build()
            .map_err(|e| ThError::Error(e.to_string()))?;

        let proxy = event_loop.create_proxy();

        Ok((event_loop, proxy))
    }

    pub fn new(
        params: WindowParams,
        event_emitter: Layer<EventEmitter<PlatformEvent>>,
        renderer: Layer<Renderer>,
    ) -> Self
    {
        Self {
            params,
            event_emitter,
            renderer,
            is_renderer_initialized: false,
            window: None,
        }
    }

    pub fn run(mut self, event_loop: EventLoop<WinitMsg>)
    {
        if let Err(e) = event_loop.run_app(&mut self)
        {
            log::error!("{e}");
        }
    }
}
