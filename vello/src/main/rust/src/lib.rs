#![forbid(unsafe_attr_outside_unsafe, unsafe_op_in_unsafe_fn)]
// Don't allow unsafe code in the main module. Note that it is allowed in the other modules
#![deny(unsafe_code)]

pub mod ffi;
pub mod util;

use std::collections::HashMap;

use guillotiere::{euclid::Size2D, SimpleAtlasAllocator};
use ndk::native_window::NativeWindow;
use parley::{Alignment, FontContext, FontWeight, PositionedLayoutItem, StyleProperty};
use vello::{
    kurbo::{Affine, Rect, Vec2},
    peniko::{Color, Fill, Mix},
    skrifa::raw::tables::glyf::PointCoord,
    util::{RenderContext, RenderSurface},
    AaSupport, RenderParams, Renderer, RendererOptions, Scene,
};
use wgpu::{
    rwh::{DisplayHandle, HasDisplayHandle, HasWindowHandle},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoder, CommandEncoderDescriptor, Device, Instance, InstanceFlags, MultisampleState,
    PipelineCompilationOptions, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipelineDescriptor, SurfaceTexture, TextureDescriptor, TextureFormat,
    TextureViewDescriptor,
};

// TODO: Bytemuck? a struct?
type SurfaceId = i64;

type LayoutContext = parley::LayoutContext<vello::peniko::Brush>;

pub struct VelloJni {
    cx: vello::util::RenderContext,
    renderer: Option<RendererResources>,
    surfaces: HashMap<SurfaceId, TargetSurface>,

    font_ctx: FontContext,
    layout_ctx: LayoutContext,

    scene: Scene,
}

struct RendererResources {
    renderer: vello::Renderer,
    target_texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    blit_pipelines: HashMap<TextureFormat, BlitPipeline>,
}

struct BlitPipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: BindGroupLayout,
    config_buffer: Buffer,
}

impl BlitPipeline {
    fn new(device: &Device, format: TextureFormat) -> Self {
        const SHADERS: &str = r#"
        @vertex
        fn vs_main(@builtin(vertex_index) ix: u32) -> @builtin(position) vec4<f32> {
            // Generate a full screen quad in normalized device coordinates
            var vertex = vec2(-1.0, 1.0);
            switch ix {
                case 1u: {
                    vertex = vec2(-1.0, -1.0);
                }
                case 2u, 4u: {
                    vertex = vec2(1.0, -1.0);
                }
                case 5u: {
                    vertex = vec2(1.0, 1.0);
                }
                default: {}
            }
            return vec4(vertex, 0.0, 1.0);
        }

        @group(0) @binding(0)
        var fine_output: texture_2d<f32>;

        @group(0) @binding(1)
        var<uniform> fine_input_coords: vec2<f32>;

        @fragment
        fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
            let rgba_sep = textureLoad(fine_output, vec2<i32>(pos.xy + fine_input_coords), 0);
            return vec4(rgba_sep.rgb * rgba_sep.a, rgba_sep.a);
        }
    "#;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blit shaders"),
            source: wgpu::ShaderSource::Wgsl(SHADERS.into()),
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            cache: None,
            depth_stencil: None,
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            label: Some("vello_jni.blit"),
            layout: None,
            multisample: MultisampleState::default(),
            multiview: None,
            primitive: PrimitiveState::default(),
        });

        let bind_group_layout = render_pipeline.get_bind_group_layout(0);
        Self {
            render_pipeline,
            bind_group_layout,
            config_buffer: device.create_buffer(&BufferDescriptor {
                label: None,
                size: 8,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    fn blit(
        &self,
        device_handle: &vello::util::DeviceHandle,
        from_texture: &wgpu::TextureView,
        to_texture: &wgpu::TextureView,
        x: i32,
        y: i32,
        encoder: &mut CommandEncoder,
    ) {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("vello_jni.blit.pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                resolve_target: None,
                view: to_texture,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let [x_0, x_1, x_2, x_3] = x.to_f32().to_le_bytes();
        let [y_0, y_1, y_2, y_3] = y.to_f32().to_le_bytes();
        device_handle.queue.write_buffer(
            &self.config_buffer,
            0,
            &[x_0, x_1, x_2, x_3, y_0, y_1, y_2, y_3],
        );
        let bind_group = device_handle
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(from_texture),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            self.config_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
            });

        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_pipeline(&self.render_pipeline);
        pass.draw(0..6, 0..1);
        // This drop operation is semantic, so we make it explicit
        drop(pass);
    }
}

#[derive(Clone)]
pub enum SurfaceKind {
    VariableFont {
        text: String,
        size: f32,
        weight: f32,
        // We don't store the Parley layout here, because if we are using this, we are re-rendering anyway.
    },
    Unset,
}

/// A subset of [Roboto Flex](https://fonts.google.com/specimen/Roboto+Flex), used under the OFL.
/// This is a variable font, and so can have its axes be animated.
/// The version in the repository supports the numbers 0-9 and `:`, to this examples use of
/// it for clocks.
/// Full details can be found in `xilem/resources/fonts/roboto_flex/README` from
/// the workspace root.
const ROBOTO_FLEX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/roboto_flex/",
    // The full font file is *not* included in this repository, due to size constraints.
    // If you download the full font, you can use it by moving it into the roboto_flex folder,
    // then swapping which of the following two lines is commented out:
    // "RobotoFlex-VariableFont_GRAD,XOPQ,XTRA,YOPQ,YTAS,YTDE,YTFI,YTLC,YTUC,opsz,slnt,wdth,wght.ttf",
    "RobotoFlex-Subset.ttf"
));

impl SurfaceKind {
    fn scene(
        &self,
        _width: u32,
        _height: u32,
        font_ctx: &mut FontContext,
        layout_ctx: &mut LayoutContext,
    ) -> Scene {
        let mut scene = Scene::new();
        match self {
            SurfaceKind::Unset => {}
            SurfaceKind::VariableFont { text, size, weight } => {
                let mut builder = layout_ctx.ranged_builder(font_ctx, text, 1.0);
                builder.push_default(StyleProperty::FontStack(parley::FontStack::Single(
                    parley::FontFamily::Named("Roboto Flex".into()),
                )));
                builder.push_default(StyleProperty::FontSize(*size));
                builder.push_default(StyleProperty::FontWeight(FontWeight::new(*weight)));
                builder.push_default(StyleProperty::Brush(vello::peniko::Brush::Solid(
                    Color::BLACK,
                )));
                builder.push_default(StyleProperty::LineHeight(1.3));
                let mut layout = builder.build(text);
                layout.break_all_lines(Some(500.));
                layout.align(Some(500.), Alignment::Start);
                for line in layout.lines() {
                    for item in line.items() {
                        let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                            continue;
                        };
                        let mut x = glyph_run.offset();
                        let y = glyph_run.baseline();
                        let run = glyph_run.run();
                        let font = run.font();
                        let font_size = run.font_size();
                        let synthesis = run.synthesis();
                        let glyph_xform = synthesis
                            .skew()
                            .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));
                        let coords = run
                            .normalized_coords()
                            .iter()
                            .map(|coord| {
                                vello::skrifa::instance::NormalizedCoord::from_bits(*coord)
                            })
                            .collect::<Vec<_>>();
                        scene
                            .draw_glyphs(font)
                            .brush(&glyph_run.style().brush)
                            // We think this might be animated, so don't enable hinting
                            .hint(false)
                            .glyph_transform(glyph_xform)
                            .font_size(font_size)
                            .normalized_coords(&coords)
                            .draw(
                                Fill::NonZero,
                                glyph_run.glyphs().map(|glyph| {
                                    let gx = x + glyph.x;
                                    let gy = y - glyph.y;
                                    x += glyph.advance;
                                    vello::Glyph {
                                        id: glyph.id as _,
                                        x: gx,
                                        y: gy,
                                    }
                                }),
                            );
                    }
                }
            }
        }
        scene
    }
}

struct TargetSurface {
    /// The Vello rendering surface for this texture.
    ///
    /// Note that we do *not* use `render_to_surface` in this implementation.
    render_surface: RenderSurface<'static>,
    /// The Android window underlying this target.
    ///
    /// TODO: Why is this here - it is currently unused?
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
        let mut font_ctx = FontContext::new();
        font_ctx.collection.register_fonts(ROBOTO_FLEX.into());

        VelloJni {
            cx,
            renderer: Default::default(),
            surfaces: Default::default(),
            // TODO: Install the default font here
            font_ctx,
            layout_ctx: LayoutContext::new(),
            scene: Default::default(),
        }
    }

    fn new_window(&mut self, window: NativeWindow, surface_id: SurfaceId, width: u32, height: u32) {
        log::info!("Window Size: {width}x{height}");
        assert!(
            !self.surfaces.contains_key(&surface_id),
            "Tried to use duplicate surface id."
        );
        let render_surface = pollster::block_on(self.cx.create_surface(
            AndroidWindowHandle {
                window: window.clone(),
            },
            width,
            height,
            wgpu::PresentMode::Mailbox,
        ))
        .expect("Could create surface");
        assert_eq!(
            render_surface.dev_id, 0,
            "Cannot handle more than one device at a time for MVP."
        );
        let target_surface = TargetSurface {
            render_surface,
            window,
            kind: SurfaceKind::Unset,
        };
        self.surfaces.insert(surface_id, target_surface);
    }

    fn perform_render(&mut self, surfaces: &[SurfaceId]) {
        if surfaces.is_empty() {
            return;
        }
        self.hydrate_renderer();
        let renderer = self.renderer.as_mut().unwrap();
        let mut allocator = SimpleAtlasAllocator::new(Size2D {
            width: renderer.target_texture.width().try_into().unwrap(),
            height: renderer.target_texture.height().try_into().unwrap(),
            ..Default::default()
        });
        let final_scene: &mut Scene = &mut self.scene;
        final_scene.reset();
        let mut allocations: HashMap<SurfaceId, guillotiere::Rectangle> = HashMap::new();
        // Render into one big scene atlas.
        for surface_id in surfaces {
            let surface = self.surfaces.get(surface_id).unwrap();
            let width = surface.render_surface.config.width;
            let height = surface.render_surface.config.height;
            let zone = allocator
                .allocate(Size2D {
                    width: width.try_into().unwrap(),
                    height: height.try_into().unwrap(),
                    ..Default::default()
                })
                .expect("Should have room for surface");
            allocations.insert(*surface_id, zone);
            final_scene.push_layer(
                Mix::Clip,
                1.0,
                Affine::IDENTITY,
                &Rect {
                    x0: zone.min.x.into(),
                    y0: zone.min.y.into(),
                    x1: zone.max.x.into(),
                    y1: zone.max.y.into(),
                },
            );
            let scene = surface
                .kind
                .scene(width, height, &mut self.font_ctx, &mut self.layout_ctx);
            final_scene.append(
                &scene,
                Some(Affine::translate(Vec2::new(
                    zone.min.x.into(),
                    zone.min.y.into(),
                ))),
            );
            final_scene.pop_layer();
        }
        let device_handle = &self.cx.devices[0];
        renderer
            .renderer
            .render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                final_scene,
                &renderer.texture_view,
                &RenderParams {
                    antialiasing_method: vello::AaConfig::Area,
                    base_color: Color::WHITE,
                    height: renderer.target_texture.height(),
                    width: renderer.target_texture.width(),
                },
            )
            .unwrap();
        let mut encoder = device_handle
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        let mut targets: Vec<SurfaceTexture> = Vec::new();
        for (surface_id, range) in allocations {
            let surface = self.surfaces.get(&surface_id).unwrap();
            let blit = renderer
                .blit_pipelines
                .entry(surface.render_surface.format)
                .or_insert_with(|| {
                    BlitPipeline::new(&device_handle.device, surface.render_surface.format)
                });
            let current_texture = surface
                .render_surface
                .surface
                .get_current_texture()
                .unwrap();
            blit.blit(
                device_handle,
                &renderer.texture_view,
                &current_texture
                    .texture
                    .create_view(&TextureViewDescriptor::default()),
                range.min.x,
                range.min.y,
                &mut encoder,
            );
            targets.push(current_texture);
        }
        device_handle.queue.submit([encoder.finish()]);
        for target in targets {
            target.present();
        }
        device_handle.device.poll(wgpu::MaintainBase::Poll);
    }

    fn hydrate_renderer(&mut self) {
        if self.renderer.is_some() {
            return;
        }
        let device = &self.cx.devices[0].device;
        let renderer = Renderer::new(
            device,
            RendererOptions {
                // We don't use the built-in blit pipeline.
                surface_format: None,
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
            },
        )
        .unwrap();
        let target_texture = device.create_texture(&TextureDescriptor {
            label: Some("VelloJNI Target Texture"),
            size: wgpu::Extent3d {
                width: 2048,
                height: 2560,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.renderer = Some(RendererResources {
            renderer,
            target_texture,
            blit_pipelines: HashMap::new(),
            texture_view,
        })
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
