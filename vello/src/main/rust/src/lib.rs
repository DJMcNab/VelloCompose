#![forbid(unsafe_attr_outside_unsafe, unsafe_op_in_unsafe_fn)]
// Don't allow unsafe code in the main module. Note that it is allowed in the other modules
#![deny(unsafe_code)]

pub mod ffi;
pub mod util;

use std::collections::HashMap;

use ndk::native_window::NativeWindow;
use parley::{
    fontique::{Collection, CollectionOptions, SourceCache},
    FontContext, LayoutContext,
};
use vello::{
    kurbo::{Affine, Rect},
    peniko::{Brush, Color, Fill},
    util::{RenderContext, RenderSurface},
    AaSupport, RenderParams, Renderer, RendererOptions, Scene,
};
use wgpu::{
    rwh::{DisplayHandle, HasDisplayHandle, HasWindowHandle},
    Instance, InstanceFlags, TextureFormat,
};

pub struct VelloJni {
    cx: vello::util::RenderContext,
    /// Renderers. One per device in `cx`.
    // Practically, we expect this to always be a 1-vector.
    renderers: Vec<vello::Renderer>,
    surfaces: HashMap<i64, TargetSurface>,
    blit_pipelines: HashMap<TextureFormat, BlitPipeline>,

    fonts: FontContext,
    layout_ctx: LayoutContext<vello::peniko::Brush>,
}

struct BlitPipeline {}

pub enum SurfaceKind {
    VariableFont {
        text: String,
        size: f32,
        weight: f32,
        // We don't store the Parley layout here, because if we are using this, we are re-rendering anyway.
    },
    Unset,
}

struct TargetSurface {
    /// The Vello rendering surface for this texture.
    ///
    /// Note that we do *not* use `render_to_surface` in this implementation.
    render_surface: RenderSurface<'static>,
    /// The Android window underlying this target.
    ///
    /// TODO: This is currently unused
    window: NativeWindow,
    /// The style of rendering used for this `Surface`
    kind: SurfaceKind,
}

impl VelloJni {
    fn new() -> Self {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::PRIMARY),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            flags: InstanceFlags::DEBUG,
            ..Default::default()
        });
        let cx = RenderContext {
            instance,
            devices: Vec::new(),
        };
        VelloJni {
            cx,
            renderers: Default::default(),
            surfaces: Default::default(),
            blit_pipelines: Default::default(),
            fonts: FontContext {
                // We will be Arc<Mutex>ing this struct, so it doesn't need to be shared.
                source_cache: SourceCache::default(),
                collection: Collection::new(CollectionOptions {
                    shared: false,
                    system_fonts: false,
                }),
            },
            layout_ctx: LayoutContext::new(),
        }
    }

    fn new_window(&mut self, window: NativeWindow, surface_id: i64, width: u32, height: u32) {
        todo!()
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
                    base_color: Color::ALICE_BLUE,
                    antialiasing_method: vello::AaConfig::Area,
                    width: texture.texture.width(),
                    height: texture.texture.height(),
                },
            )
            .expect("Could render");
        texture.present();
        self.cx.devices[surface.dev_id]
            .device
            .poll(wgpu::MaintainBase::Poll);
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

    fn perform_render(&mut self, surfaces: &[i64]) -> _ {
        todo!()
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
