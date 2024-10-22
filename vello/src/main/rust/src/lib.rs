pub mod ffi;
pub mod util;

use ndk::native_window::NativeWindow;
use vello::{
    kurbo::{Affine, Rect},
    peniko::{Brush, Color, Fill},
    util::{RenderContext, RenderSurface},
    AaSupport, RenderParams, Renderer, RendererOptions, Scene,
};
use wgpu::{
    rwh::{DisplayHandle, HasDisplayHandle, HasWindowHandle},
    TextureFormat,
};

pub struct VelloState {
    cx: vello::util::RenderContext,
    renderer: Option<(vello::Renderer, TextureFormat)>,
    surface: Option<(RenderSurface<'static>, NativeWindow)>,
    color: i32,
}

impl VelloState {
    fn new() -> Self {
        VelloState {
            cx: RenderContext::new(),
            renderer: None,
            surface: None,
            color: 0xff00ff,
        }
    }

    fn set_window(&mut self, native_window: NativeWindow) {
        let width = native_window.width().try_into().expect("positive width");
        let height = native_window.height().try_into().expect("positive height");
        log::error!("Window Size: {width}x{height}");
        DisplayHandle::android();
        let surface = pollster::block_on(self.cx.create_surface(
            AndroidWindowHandle {
                window: native_window.clone(),
            },
            width,
            height,
            wgpu::PresentMode::AutoNoVsync,
        ))
        .expect("Could create surface");
        if let Some((_, format)) = self.renderer.as_mut() {
            if surface.format != *format {
                log::warn!(
                    "Deleting renderer due to mismatched surface format. Was {:?}, needed {:?}",
                    *format,
                    surface.format
                );
                let _ = self.renderer.take();
            }
        }
        if self.renderer.is_none() {
            let renderer = Renderer::new(
                &self.cx.devices[surface.dev_id].device,
                RendererOptions {
                    surface_format: Some(surface.format),
                    use_cpu: false,
                    antialiasing_support: AaSupport::area_only(),
                    num_init_threads: None,
                },
            )
            .expect("Could create renderer");
            self.renderer = Some((renderer, surface.format))
        }
        self.surface = Some((surface, native_window));
    }

    fn resize_surface(&mut self) {
        if let Some((surface, window)) = self.surface.as_mut() {
            self.cx.resize_surface(
                surface,
                window.width().try_into().unwrap(),
                window.height().try_into().unwrap(),
            );
        }
    }

    fn remove_window(&mut self) {
        self.surface = None;
    }

    fn render(&mut self, scene: &Scene) {
        let Some((surface, _)) = self.surface.as_ref() else {
            log::warn!("Tried to render with no surface");
            return;
        };
        let Some((renderer, _)) = self.renderer.as_mut() else {
            log::warn!("Tried to render with no renderer");
            return;
        };
        let device = &self.cx.devices[surface.dev_id];
        let Ok(texture) = surface.surface.get_current_texture() else {
            log::warn!("Failed to get surface texture");
            return;
        };
        renderer
            .render_to_surface(
                &device.device,
                &device.queue,
                scene,
                &texture,
                &RenderParams {
                    base_color: Color::WHITE,
                    antialiasing_method: vello::AaConfig::Area,
                    width: texture.texture.width(),
                    height: texture.texture.height(),
                },
            )
            .expect("Could render");
        texture.present();
        self.cx.devices[surface.dev_id]
            .device
            .poll(wgpu::MaintainBase::Wait);
    }

    fn render_default(&mut self) {
        let mut scene = Scene::new();
        let [_, r, g, b]: [u8; 4] = bytemuck::cast(self.color);
        scene.fill(
            Fill::EvenOdd,
            Affine::IDENTITY,
            &Brush::Solid(Color { r, g, b, a: 255 }),
            None,
            &Rect::new(0., 0., 200., 400.),
        );
        self.render(&scene);
    }
}

pub struct AndroidWindowHandle {
    window: NativeWindow,
}

impl HasDisplayHandle for AndroidWindowHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, wgpu::rwh::HandleError> {
        Ok(DisplayHandle::android())
    }
}

impl HasWindowHandle for AndroidWindowHandle {
    fn window_handle(&self) -> Result<wgpu::rwh::WindowHandle<'_>, wgpu::rwh::HandleError> {
        self.window.window_handle()
    }
}
