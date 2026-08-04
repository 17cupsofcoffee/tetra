use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;
use std::slice;

use hashbrown::{HashMap, HashSet};
use wgpu::naga;

use crate::error::{Result, TetraError};
use crate::graphics::{
    mesh::{BufferUsage, Vertex, VertexWinding},
    StencilState, StencilTest,
};
use crate::graphics::{
    BlendFactor, BlendOperation, BlendState, Color, FilterMode, GraphicsDeviceInfo, StencilAction,
    TextureFormat,
};
use crate::math::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use crate::platform::glsl_compat;

const MAX_TEXTURE_UNITS: usize = 16;

const CLEAR_SHADER: &str = "
@group(0) @binding(0) var<uniform> clear_color: vec4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );

    return vec4<f32>(positions[index], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return clear_color;
}
";

/// A snapshot of a texture, as bound to a 'texture unit' (which is emulated
/// for compatibility with the old OpenGL-style API).
#[derive(Debug, Clone)]
struct BoundTexture {
    id: u64,
    view: wgpu::TextureView,
    filter_mode: FilterMode,
}

#[derive(Debug, Clone)]
struct TargetInfo {
    attachment: wgpu::TextureView,
    resolve: Option<wgpu::TextureView>,
    depth_stencil: Option<wgpu::TextureView>,
    format: wgpu::TextureFormat,
    samples: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone)]
struct CanvasTarget {
    width: u32,
    height: u32,
    samples: u32,
    format: wgpu::TextureFormat,
    color_view: wgpu::TextureView,
    msaa_view: Option<wgpu::TextureView>,
    depth_stencil_view: Option<wgpu::TextureView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PipelineKey {
    shader: u64,
    format: wgpu::TextureFormat,
    samples: u32,
    has_depth_stencil: bool,
    blend: BlendState,
    cull: bool,
    front_face: VertexWinding,
    color_mask: [bool; 4],
    stencil: StencilState,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ClearPipelineKey {
    format: wgpu::TextureFormat,
    samples: u32,
    has_depth_stencil: bool,
    color_mask: [bool; 4],
}

pub struct GraphicsDevice {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_dirty: bool,
    adapter: wgpu::Adapter,

    vsync: bool,

    backbuffer_samples: u32,
    backbuffer_has_stencil: bool,
    backbuffer_msaa: Option<wgpu::TextureView>,
    backbuffer_depth_stencil: Option<wgpu::TextureView>,

    // Frame state
    encoder: Option<wgpu::CommandEncoder>,
    frame_surface: Option<(wgpu::SurfaceTexture, wgpu::TextureView)>,
    used_resources: HashSet<u64>,

    // Emulated OpenGL-style render state
    cull: bool,
    front_face: VertexWinding,
    blend: BlendState,
    stencil: StencilState,
    stencil_reference: u8,
    color_mask: [bool; 4],
    scissor_test: bool,
    scissor_rect: (i32, i32, i32, i32),
    viewport_rect: (i32, i32, i32, i32),
    canvas: Option<CanvasTarget>,
    texture_units: Vec<Option<BoundTexture>>,

    // Caches and helpers
    pipelines: HashMap<PipelineKey, wgpu::RenderPipeline>,
    clear_pipelines: HashMap<ClearPipelineKey, wgpu::RenderPipeline>,
    clear_shader: wgpu::ShaderModule,
    clear_pipeline_layout: wgpu::PipelineLayout,
    clear_buffer: wgpu::Buffer,
    clear_buffer_id: u64,
    clear_bind_group: wgpu::BindGroup,
    nearest_sampler: wgpu::Sampler,
    linear_sampler: wgpu::Sampler,
    dummy_texture: BoundTexture,

    next_id: Rc<Cell<u64>>,
}

fn next_id(counter: &Rc<Cell<u64>>) -> u64 {
    let id = counter.get();
    counter.set(id + 1);
    id
}

fn map_err<E: std::fmt::Display>(prefix: &str) -> impl Fn(E) -> TetraError + '_ {
    move |e| TetraError::PlatformError(format!("{}: {}", prefix, e))
}

impl GraphicsDevice {
    pub fn new<W>(
        window: &W,
        width: i32,
        height: i32,
        vsync: bool,
        multisampling: u8,
        stencil_buffer: bool,
    ) -> Result<GraphicsDevice>
    where
        W: wgpu::rwh::HasWindowHandle + wgpu::rwh::HasDisplayHandle,
    {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());

        let surface = unsafe {
            let target = wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(
                    window
                        .display_handle()
                        .map_err(map_err("failed to get display handle"))?
                        .as_raw(),
                ),
                raw_window_handle: window
                    .window_handle()
                    .map_err(map_err("failed to get window handle"))?
                    .as_raw(),
            };

            instance
                .create_surface_unsafe(target)
                .map_err(map_err("failed to create rendering surface"))?
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
            apply_limit_buckets: false,
        }))
        .map_err(map_err("failed to find a suitable graphics adapter"))?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("tetra"),
            ..Default::default()
        }))
        .map_err(map_err("failed to create graphics device"))?;

        let caps = surface.get_capabilities(&adapter);

        // Tetra's rendering pipeline (like its OpenGL-based predecessor) does
        // not use automatic sRGB conversion, so a non-sRGB surface format is
        // preferred.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .or_else(|| caps.formats.first().copied())
            .ok_or_else(|| {
                TetraError::PlatformError("no supported surface formats found".into())
            })?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: width.max(1) as u32,
            height: height.max(1) as u32,
            present_mode: present_mode(vsync),
            desired_maximum_frame_latency: 2,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let backbuffer_samples = if multisampling > 0 {
            supported_sample_count(&adapter, format, multisampling)
        } else {
            1
        };

        let clear_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tetra_clear"),
            source: wgpu::ShaderSource::Wgsl(CLEAR_SHADER.into()),
        });

        let clear_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("tetra_clear"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                    count: None,
                }],
            });

        let clear_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("tetra_clear"),
                bind_group_layouts: &[Some(&clear_bind_group_layout)],
                immediate_size: 0,
            });

        let clear_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tetra_clear"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let clear_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tetra_clear"),
            layout: &clear_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: clear_buffer.as_entire_binding(),
            }],
        });

        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("tetra_nearest"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("tetra_linear"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tetra_dummy"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            dummy.as_image_copy(),
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let dummy_view = dummy.create_view(&wgpu::TextureViewDescriptor::default());

        let next_id_counter = Rc::new(Cell::new(1));
        let clear_buffer_id = next_id(&next_id_counter);
        let dummy_id = next_id(&next_id_counter);

        let mut device = GraphicsDevice {
            device,
            queue,
            surface,
            surface_config,
            surface_dirty: false,
            adapter,

            vsync,

            backbuffer_samples,
            backbuffer_has_stencil: stencil_buffer,
            backbuffer_msaa: None,
            backbuffer_depth_stencil: None,

            encoder: None,
            frame_surface: None,
            used_resources: HashSet::new(),

            cull: true,
            front_face: VertexWinding::CounterClockwise,
            blend: BlendState::default(),
            stencil: StencilState::disabled(),
            stencil_reference: 0,
            color_mask: [true; 4],
            scissor_test: false,
            scissor_rect: (0, 0, 0, 0),
            viewport_rect: (0, 0, width, height),
            canvas: None,
            texture_units: vec![None; MAX_TEXTURE_UNITS],

            pipelines: HashMap::new(),
            clear_pipelines: HashMap::new(),
            clear_shader,
            clear_pipeline_layout,
            clear_buffer,
            clear_buffer_id,
            clear_bind_group,
            nearest_sampler,
            linear_sampler,
            dummy_texture: BoundTexture {
                id: dummy_id,
                view: dummy_view,
                filter_mode: FilterMode::Nearest,
            },

            next_id: next_id_counter,
        };

        device.recreate_backbuffer_attachments();

        Ok(device)
    }

    pub fn get_info(&self) -> GraphicsDeviceInfo {
        let info = self.adapter.get_info();

        GraphicsDeviceInfo {
            vendor: format!("{:#06x}", info.vendor),
            renderer: info.name,
            backend: format!("wgpu ({:?})", info.backend),
            driver: format!("{} {}", info.driver, info.driver_info)
                .trim()
                .to_owned(),
        }
    }

    // == Frame management ==

    fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.encoder.get_or_insert_with(|| {
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("tetra"),
                })
        })
    }

    fn submit_pending(&mut self) {
        if let Some(encoder) = self.encoder.take() {
            self.queue.submit(Some(encoder.finish()));
        }

        self.used_resources.clear();
    }

    /// Ensures that overwriting the given resource will not affect any
    /// rendering that has already been recorded (as `Queue::write_buffer` and
    /// friends are processed *before* any command buffers in the same
    /// submission).
    fn prepare_write(&mut self, id: u64) {
        if self.used_resources.contains(&id) {
            self.submit_pending();
        }
    }

    pub fn present(&mut self) {
        self.submit_pending();

        if let Some((surface_texture, _)) = self.frame_surface.take() {
            self.queue.present(surface_texture);
        }
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        if self.vsync != vsync {
            self.vsync = vsync;
            self.surface_config.present_mode = present_mode(vsync);
            self.surface_dirty = true;
        }
    }

    pub fn is_vsync_enabled(&self) -> bool {
        self.vsync
    }

    fn recreate_backbuffer_attachments(&mut self) {
        let width = self.surface_config.width;
        let height = self.surface_config.height;
        let samples = self.backbuffer_samples;

        self.backbuffer_msaa = if samples > 1 {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("tetra_backbuffer_msaa"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            Some(texture.create_view(&wgpu::TextureViewDescriptor::default()))
        } else {
            None
        };

        self.backbuffer_depth_stencil = if self.backbuffer_has_stencil {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("tetra_backbuffer_depth_stencil"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            Some(texture.create_view(&wgpu::TextureViewDescriptor::default()))
        } else {
            None
        };
    }

    fn ensure_surface(&mut self) -> Option<()> {
        if self.frame_surface.is_some() {
            return Some(());
        }

        if self.surface_config.width == 0 || self.surface_config.height == 0 {
            return None;
        }

        if self.surface_dirty {
            self.surface.configure(&self.device, &self.surface_config);
            self.recreate_backbuffer_attachments();
            self.surface_dirty = false;
        }

        use wgpu::CurrentSurfaceTexture;

        let surface_texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.surface_config);
                self.recreate_backbuffer_attachments();

                match self.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
                    _ => return None,
                }
            }
            _ => return None,
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.frame_surface = Some((surface_texture, view));

        Some(())
    }

    fn current_target(&mut self) -> Option<TargetInfo> {
        match &self.canvas {
            Some(canvas) => Some(TargetInfo {
                attachment: canvas
                    .msaa_view
                    .clone()
                    .unwrap_or_else(|| canvas.color_view.clone()),
                resolve: None,
                depth_stencil: canvas.depth_stencil_view.clone(),
                format: canvas.format,
                samples: canvas.samples,
                width: canvas.width,
                height: canvas.height,
            }),

            None => {
                self.ensure_surface()?;

                let (_, surface_view) = self.frame_surface.as_ref()?;

                let (attachment, resolve) = match &self.backbuffer_msaa {
                    Some(msaa_view) => (msaa_view.clone(), Some(surface_view.clone())),
                    None => (surface_view.clone(), None),
                };

                Some(TargetInfo {
                    attachment,
                    resolve,
                    depth_stencil: self.backbuffer_depth_stencil.clone(),
                    format: self.surface_config.format,
                    samples: self.backbuffer_samples,
                    width: self.surface_config.width,
                    height: self.surface_config.height,
                })
            }
        }
    }

    // == Render state ==

    pub fn clear(&mut self, color: Color) {
        if self.scissor_test || self.color_mask != [true; 4] {
            self.clear_via_quad(color);
            return;
        }

        let Some(target) = self.current_target() else {
            return;
        };

        let encoder = self.encoder();

        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tetra_clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.attachment,
                depth_slice: None,
                resolve_target: target.resolve.as_ref(),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: color.r as f64,
                        g: color.g as f64,
                        b: color.b as f64,
                        a: color.a as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: target.depth_stencil.as_ref().map(|view| {
                wgpu::RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                }
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        drop(pass);
    }

    fn clear_via_quad(&mut self, color: Color) {
        let Some(target) = self.current_target() else {
            return;
        };

        self.prepare_write(self.clear_buffer_id);

        self.queue.write_buffer(
            &self.clear_buffer,
            0,
            bytemuck::cast_slice(&[color.r, color.g, color.b, color.a]),
        );

        let key = ClearPipelineKey {
            format: target.format,
            samples: target.samples,
            has_depth_stencil: target.depth_stencil.is_some(),
            color_mask: self.color_mask,
        };

        if !self.clear_pipelines.contains_key(&key) {
            let pipeline = self
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("tetra_clear"),
                    layout: Some(&self.clear_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &self.clear_shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: key.has_depth_stencil.then(|| wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth24PlusStencil8,
                        depth_write_enabled: Some(false),
                        depth_compare: Some(wgpu::CompareFunction::Always),
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: key.samples,
                        ..Default::default()
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &self.clear_shader,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: key.format,
                            blend: None,
                            write_mask: color_writes(key.color_mask),
                        })],
                    }),
                    multiview_mask: None,
                    cache: None,
                });

            self.clear_pipelines.insert(key.clone(), pipeline);
        }

        let scissor = self.scissor_state(&target);

        self.used_resources.insert(self.clear_buffer_id);

        let pipeline = &self.clear_pipelines[&key];
        let clear_bind_group = &self.clear_bind_group;

        let encoder = self.encoder.get_or_insert_with(|| {
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("tetra"),
                })
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tetra_clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.attachment,
                depth_slice: None,
                resolve_target: target.resolve.as_ref(),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: target.depth_stencil.as_ref().map(|view| {
                wgpu::RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                }
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if let Some((x, y, w, h)) = scissor {
            pass.set_scissor_rect(x, y, w, h);
        }

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, clear_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    fn scissor_state(&self, target: &TargetInfo) -> Option<(u32, u32, u32, u32)> {
        if !self.scissor_test {
            return None;
        }

        let (x, y, w, h) = self.scissor_rect;

        let x0 = x.clamp(0, target.width as i32) as u32;
        let y0 = y.clamp(0, target.height as i32) as u32;
        let x1 = (x + w).clamp(0, target.width as i32) as u32;
        let y1 = (y + h).clamp(0, target.height as i32) as u32;

        Some((x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0)))
    }

    fn viewport_state(&self, target: &TargetInfo) -> (f32, f32, f32, f32) {
        let (x, y, w, h) = self.viewport_rect;

        let x0 = x.clamp(0, target.width as i32) as f32;
        let y0 = y.clamp(0, target.height as i32) as f32;
        let x1 = (x + w).clamp(0, target.width as i32) as f32;
        let y1 = (y + h).clamp(0, target.height as i32) as f32;

        (x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
    }

    pub fn front_face(&mut self, front_face: VertexWinding) {
        self.front_face = front_face;
    }

    pub fn cull_face(&mut self, cull_face: bool) {
        self.cull = cull_face;
    }

    pub fn scissor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.scissor_rect = (x, y, width, height);
    }

    pub fn scissor_test(&mut self, scissor_test: bool) {
        self.scissor_test = scissor_test;
    }

    pub fn set_stencil_state(&mut self, state: StencilState) {
        self.stencil = state;
        self.stencil_reference = state.reference_value;
    }

    pub fn clear_stencil(&mut self, value: u8) {
        let Some(target) = self.current_target() else {
            return;
        };

        let Some(depth_stencil) = &target.depth_stencil else {
            return;
        };

        let encoder = self.encoder();

        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tetra_clear_stencil"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_stencil,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(value as u32),
                    store: wgpu::StoreOp::Store,
                }),
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        drop(pass);
    }

    pub fn set_color_mask(&mut self, red: bool, green: bool, blue: bool, alpha: bool) {
        self.color_mask = [red, green, blue, alpha];
    }

    pub fn set_blend_state(&mut self, blend_state: BlendState) {
        self.blend = blend_state;
    }

    pub fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.viewport_rect = (x, y, width, height);

        // When rendering to the screen, the viewport is also used to keep
        // track of the size of the backbuffer (Tetra always sets it to the
        // full size of the window).
        if self.canvas.is_none() {
            let width = width.max(0) as u32;
            let height = height.max(0) as u32;

            if width != self.surface_config.width || height != self.surface_config.height {
                self.surface_config.width = width;
                self.surface_config.height = height;
                self.surface_dirty = true;
            }
        }
    }

    // == Buffers ==

    pub fn new_vertex_buffer(
        &mut self,
        count: usize,
        _usage: BufferUsage,
    ) -> Result<RawVertexBuffer> {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tetra_vertex_buffer"),
            size: (count * mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(RawVertexBuffer {
            id: next_id(&self.next_id),
            buffer,
            count,
        })
    }

    pub fn set_vertex_buffer_data(
        &mut self,
        buffer: &RawVertexBuffer,
        data: &[Vertex],
        offset: usize,
    ) {
        assert!(
            data.len() + offset <= buffer.count(),
            "tried to write out of bounds buffer data"
        );

        if data.is_empty() {
            return;
        }

        self.prepare_write(buffer.id);

        self.queue.write_buffer(
            &buffer.buffer,
            (buffer.stride() * offset) as u64,
            bytemuck::cast_slice(data),
        );
    }

    pub fn new_index_buffer(&mut self, count: usize, _usage: BufferUsage) -> Result<RawIndexBuffer> {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tetra_index_buffer"),
            size: (count * mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(RawIndexBuffer {
            id: next_id(&self.next_id),
            buffer,
            count,
        })
    }

    pub fn set_index_buffer_data(&mut self, buffer: &RawIndexBuffer, data: &[u32], offset: usize) {
        assert!(
            data.len() + offset <= buffer.count(),
            "tried to write out of bounds buffer data"
        );

        if data.is_empty() {
            return;
        }

        self.prepare_write(buffer.id);

        self.queue.write_buffer(
            &buffer.buffer,
            (buffer.stride() * offset) as u64,
            bytemuck::cast_slice(data),
        );
    }

    // == Shaders ==

    pub fn new_shader(&mut self, vertex_shader: &str, fragment_shader: &str) -> Result<RawShader> {
        let processed = glsl_compat::process(vertex_shader, fragment_shader);

        let vs_module = parse_glsl(&processed.vertex, naga::ShaderStage::Vertex)?;
        let fs_module = parse_glsl(&processed.fragment, naga::ShaderStage::Fragment)?;

        // Reflect the shader's uniforms, so that data can be uploaded to
        // them by name later.
        let mut uniforms = Vec::new();
        let mut samplers = Vec::new();
        let mut by_name = HashMap::new();

        for (name, binding) in &processed.interface.uniforms {
            let kind = find_uniform_kind(&vs_module, name).or_else(|| find_uniform_kind(&fs_module, name));

            // If the uniform was optimized out of both stages, it does not
            // get a slot - matching OpenGL, where unused uniforms have no
            // location.
            let Some(kind) = kind else {
                continue;
            };

            let size = kind.buffer_size();

            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(name),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            by_name.insert(name.clone(), Location::Data(uniforms.len()));

            uniforms.push(UniformSlot {
                binding: *binding,
                buffer,
                size,
                kind,
            });
        }

        for (name, tex_binding, smp_binding) in &processed.interface.samplers {
            let tex_name = format!("{}{}", name, glsl_compat::TEXTURE_SUFFIX);

            if find_global_type(&vs_module, &tex_name)
                .or_else(|| find_global_type(&fs_module, &tex_name))
                .is_none()
            {
                continue;
            }

            by_name.insert(name.clone(), Location::Sampler(samplers.len()));

            samplers.push(SamplerSlot {
                tex_binding: *tex_binding,
                smp_binding: *smp_binding,
            });
        }

        // Work out which of Tetra's fixed vertex attributes this shader
        // actually uses.
        let mut attributes = Vec::new();

        if let Some(entry_point) = vs_module.entry_points.first() {
            for arg in &entry_point.function.arguments {
                let (Some(name), Some(naga::Binding::Location { location, .. })) =
                    (&arg.name, &arg.binding)
                else {
                    continue;
                };

                let attribute = match name.as_str() {
                    "a_position" => wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: *location,
                    },
                    "a_uv" => wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 8,
                        shader_location: *location,
                    },
                    "a_color" => wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 16,
                        shader_location: *location,
                    },
                    _ => continue,
                };

                attributes.push(attribute);
            }
        }

        // Build the bind group layout from the reflected resources.
        let mut layout_entries = Vec::new();

        for slot in &uniforms {
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: slot.binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(slot.size as u64),
                },
                count: None,
            });
        }

        for slot in &samplers {
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: slot.tex_binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });

            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: slot.smp_binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }

        let error_scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);

        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("tetra_shader"),
                    entries: &layout_entries,
                });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("tetra_shader"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });

        let vs = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("tetra_vertex_shader"),
                source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(vs_module)),
            });

        let fs = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("tetra_fragment_shader"),
                source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(fs_module)),
            });

        if let Some(error) = pollster::block_on(error_scope.pop()) {
            return Err(TetraError::InvalidShader(error.to_string()));
        }

        let blobs = uniforms.iter().map(|u| vec![0; u.size as usize]).collect();
        let dirty = vec![false; uniforms.len()];
        let sampler_units = vec![0; samplers.len()];

        Ok(RawShader {
            id: next_id(&self.next_id),
            vs,
            fs,
            bind_group_layout,
            pipeline_layout,
            attributes,
            uniforms,
            samplers,
            by_name,
            dynamic: RefCell::new(ShaderDynamic {
                blobs,
                dirty,
                sampler_units,
                bind_group: None,
            }),
        })
    }

    pub fn get_uniform_location(&self, shader: &RawShader, name: &str) -> Option<UniformLocation> {
        shader
            .by_name
            .get(name)
            .map(|&location| UniformLocation { location })
    }

    fn set_uniform_data(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        data: &[u8],
        elem_size: usize,
    ) {
        let Some(location) = location else {
            return;
        };

        match location.location {
            Location::Sampler(index) => {
                // Sampler uniforms are set via glUniform1i in OpenGL, so
                // integer data is interpreted as a texture unit.
                if elem_size == 4 && data.len() >= 4 {
                    let unit = i32::from_ne_bytes([data[0], data[1], data[2], data[3]]);

                    let mut dynamic = shader.dynamic.borrow_mut();

                    if dynamic.sampler_units[index] != unit.max(0) as u32 {
                        dynamic.sampler_units[index] = unit.max(0) as u32;
                        dynamic.bind_group = None;
                    }
                }
            }

            Location::Data(index) => {
                let slot = &shader.uniforms[index];
                let mut dynamic = shader.dynamic.borrow_mut();

                let mut blob = mem::take(&mut dynamic.blobs[index]);
                let changed = slot.kind.write(&mut blob, data, elem_size);
                dynamic.blobs[index] = blob;

                if changed {
                    dynamic.dirty[index] = true;
                }
            }
        }
    }

    pub fn set_uniform_i32(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[i32],
    ) {
        self.set_uniform_data(shader, location, bytemuck::cast_slice(values), 4);
    }

    pub fn set_uniform_u32(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[u32],
    ) {
        self.set_uniform_data(shader, location, bytemuck::cast_slice(values), 4);
    }

    pub fn set_uniform_f32(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[f32],
    ) {
        self.set_uniform_data(shader, location, bytemuck::cast_slice(values), 4);
    }

    pub fn set_uniform_vec2(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[Vec2<f32>],
    ) {
        // SAFETY: Type is aligned and has no padding.
        let data: &[u8] = unsafe { cast_slice_assume_aligned(values) };
        self.set_uniform_data(shader, location, data, 8);
    }

    pub fn set_uniform_vec3(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[Vec3<f32>],
    ) {
        // SAFETY: Type is aligned and has no padding.
        let data: &[u8] = unsafe { cast_slice_assume_aligned(values) };
        self.set_uniform_data(shader, location, data, 12);
    }

    pub fn set_uniform_vec4(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[Vec4<f32>],
    ) {
        // SAFETY: Type is aligned and has no padding.
        let data: &[u8] = unsafe { cast_slice_assume_aligned(values) };
        self.set_uniform_data(shader, location, data, 16);
    }

    pub fn set_uniform_color(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[Color],
    ) {
        // SAFETY: Type is aligned and has no padding.
        let data: &[u8] = unsafe { cast_slice_assume_aligned(values) };
        self.set_uniform_data(shader, location, data, 16);
    }

    pub fn set_uniform_mat2(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[Mat2<f32>],
    ) {
        let values: Vec<Mat2<f32>> = if Mat2::<f32>::GL_SHOULD_TRANSPOSE {
            values.iter().map(|m| m.transposed()).collect()
        } else {
            values.to_vec()
        };

        assert!(values.iter().all(Mat2::is_packed));

        // SAFETY: Type is aligned and has no padding.
        let data: &[u8] = unsafe { cast_slice_assume_aligned(&values) };
        self.set_uniform_data(shader, location, data, 16);
    }

    pub fn set_uniform_mat3(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[Mat3<f32>],
    ) {
        let values: Vec<Mat3<f32>> = if Mat3::<f32>::GL_SHOULD_TRANSPOSE {
            values.iter().map(|m| m.transposed()).collect()
        } else {
            values.to_vec()
        };

        assert!(values.iter().all(Mat3::is_packed));

        // SAFETY: Type is aligned and has no padding.
        let data: &[u8] = unsafe { cast_slice_assume_aligned(&values) };
        self.set_uniform_data(shader, location, data, 36);
    }

    pub fn set_uniform_mat4(
        &mut self,
        shader: &RawShader,
        location: Option<&UniformLocation>,
        values: &[Mat4<f32>],
    ) {
        let values: Vec<Mat4<f32>> = if Mat4::<f32>::GL_SHOULD_TRANSPOSE {
            values.iter().map(|m| m.transposed()).collect()
        } else {
            values.to_vec()
        };

        assert!(values.iter().all(Mat4::is_packed));

        // SAFETY: Type is aligned and has no padding.
        let data: &[u8] = unsafe { cast_slice_assume_aligned(&values) };
        self.set_uniform_data(shader, location, data, 64);
    }

    // == Textures ==

    pub fn new_texture(
        &mut self,
        width: i32,
        height: i32,
        format: TextureFormat,
        filter_mode: FilterMode,
    ) -> Result<RawTexture> {
        if width <= 0 || height <= 0 {
            return Err(TetraError::PlatformError(
                "texture dimensions must be positive".into(),
            ));
        }

        let error_scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tetra_texture"),
            size: wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.to_wgpu(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        if let Some(error) = pollster::block_on(error_scope.pop()) {
            return Err(TetraError::PlatformError(format!(
                "failed to create texture: {}",
                error
            )));
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(RawTexture {
            id: next_id(&self.next_id),
            texture,
            view,
            width,
            height,
            format,
            filter_mode: Cell::new(filter_mode),
        })
    }

    pub fn set_texture_data(
        &mut self,
        texture: &RawTexture,
        data: &[u8],
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result {
        assert!(
            x >= 0 && y >= 0 && x + width <= texture.width && y + height <= texture.height,
            "tried to write outside of texture bounds"
        );

        let expected = width as usize * height as usize * texture.format.stride();
        let actual = data.len();

        if expected > actual {
            return Err(TetraError::NotEnoughData { expected, actual });
        }

        if width == 0 || height == 0 {
            return Ok(());
        }

        self.prepare_write(texture.id);

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as u32,
                    y: y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &data[..expected],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((width as usize * texture.format.stride()) as u32),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
        );

        Ok(())
    }

    pub fn get_texture_data(&mut self, texture: &RawTexture) -> Vec<u8> {
        // Make sure any rendering to this texture has actually been
        // submitted before reading it back.
        self.submit_pending();

        let stride = texture.format.stride();
        let unpadded_bytes_per_row = texture.width as usize * stride;
        let padded_bytes_per_row = unpadded_bytes_per_row.next_multiple_of(256);

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tetra_readback"),
            size: (padded_bytes_per_row * texture.height as usize) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tetra_readback"),
            });

        encoder.copy_texture_to_buffer(
            texture.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: texture.width as u32,
                height: texture.height as u32,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        buffer.map_async(wgpu::MapMode::Read, .., |_| {});
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());

        let mapped = buffer
            .get_mapped_range(..)
            .expect("readback buffer should be mapped");

        let mut output = Vec::with_capacity(unpadded_bytes_per_row * texture.height as usize);

        for row in mapped.chunks(padded_bytes_per_row) {
            output.extend_from_slice(&row[..unpadded_bytes_per_row]);
        }

        drop(mapped);
        buffer.unmap();

        output
    }

    pub fn set_texture_filter_mode(&mut self, texture: &RawTexture, filter_mode: FilterMode) {
        texture.filter_mode.set(filter_mode);
    }

    pub fn attach_texture_to_sampler(&mut self, texture: &RawTexture, unit: u32) -> Result {
        let slot = self
            .texture_units
            .get_mut(unit as usize)
            .ok_or_else(|| TetraError::PlatformError("invalid texture unit".into()))?;

        *slot = Some(BoundTexture {
            id: texture.id,
            view: texture.view.clone(),
            filter_mode: texture.filter_mode.get(),
        });

        Ok(())
    }

    // == Canvases ==

    pub fn new_canvas(
        &mut self,
        width: i32,
        height: i32,
        format: TextureFormat,
        filter_mode: FilterMode,
        samples: u8,
        with_stencil_buffer: bool,
    ) -> Result<RawCanvasWithAttachments> {
        let color = self.new_texture(width, height, format, filter_mode)?;

        let samples = if samples > 0 {
            supported_sample_count(&self.adapter, format.to_wgpu(), samples)
        } else {
            1
        };

        let error_scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);

        let multisample_color = if samples > 1 {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("tetra_canvas_msaa"),
                size: wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: format.to_wgpu(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            Some(texture)
        } else {
            None
        };

        let depth_stencil = if with_stencil_buffer {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("tetra_canvas_depth_stencil"),
                size: wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            Some(texture)
        } else {
            None
        };

        if let Some(error) = pollster::block_on(error_scope.pop()) {
            return Err(TetraError::PlatformError(format!(
                "failed to create canvas: {}",
                error
            )));
        }

        let msaa_view = multisample_color
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        let depth_stencil_view = depth_stencil
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        let canvas = RawCanvas {
            id: next_id(&self.next_id),
            target: CanvasTarget {
                width: width as u32,
                height: height as u32,
                samples,
                format: format.to_wgpu(),
                color_view: color.view.clone(),
                msaa_view,
                depth_stencil_view,
            },
        };

        Ok(RawCanvasWithAttachments {
            canvas,
            color,
            multisample_color: multisample_color.map(|texture| RawRenderbuffer {
                id: next_id(&self.next_id),
                _texture: texture,
            }),
            depth_stencil: depth_stencil.map(|texture| RawRenderbuffer {
                id: next_id(&self.next_id),
                _texture: texture,
            }),
        })
    }

    pub fn set_canvas(&mut self, canvas: Option<&RawCanvas>) {
        self.canvas = canvas.map(|c| c.target.clone());
    }

    pub fn resolve(&mut self, canvas: &RawCanvas, texture: &RawTexture) {
        let Some(msaa_view) = &canvas.target.msaa_view else {
            return;
        };

        let encoder = self.encoder();

        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tetra_resolve"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: msaa_view,
                depth_slice: None,
                resolve_target: Some(&texture.view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        drop(pass);

        self.used_resources.insert(texture.id);
    }

    // == Drawing ==

    pub fn draw(
        &mut self,
        vertex_buffer: &RawVertexBuffer,
        index_buffer: Option<&RawIndexBuffer>,
        texture: &RawTexture,
        shader: &RawShader,
        offset: usize,
        count: usize,
    ) {
        self.draw_instanced(
            vertex_buffer,
            index_buffer,
            texture,
            shader,
            offset,
            count,
            1,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_instanced(
        &mut self,
        vertex_buffer: &RawVertexBuffer,
        index_buffer: Option<&RawIndexBuffer>,
        texture: &RawTexture,
        shader: &RawShader,
        offset: usize,
        count: usize,
        instances: usize,
    ) {
        if count == 0 || instances == 0 {
            return;
        }

        let Some(target) = self.current_target() else {
            return;
        };

        // Texture unit 0 is always the 'active' texture.
        self.texture_units[0] = Some(BoundTexture {
            id: texture.id,
            view: texture.view.clone(),
            filter_mode: texture.filter_mode.get(),
        });

        self.flush_uniforms(shader);
        self.update_bind_group(shader);

        let pipeline_key = PipelineKey {
            shader: shader.id,
            format: target.format,
            samples: target.samples,
            has_depth_stencil: target.depth_stencil.is_some(),
            blend: self.blend,
            cull: self.cull,
            front_face: self.front_face,
            color_mask: self.color_mask,
            stencil: self.stencil,
        };

        if !self.pipelines.contains_key(&pipeline_key) {
            let pipeline = self.create_pipeline(shader, &pipeline_key);
            self.pipelines.insert(pipeline_key.clone(), pipeline);
        }

        let scissor = self.scissor_state(&target);
        let viewport = self.viewport_state(&target);

        if viewport.2 <= 0.0 || viewport.3 <= 0.0 {
            return;
        }

        // Mark everything referenced by this draw call as in use, so that
        // subsequent data uploads know to flush first.
        self.used_resources.insert(vertex_buffer.id);
        self.used_resources.insert(shader.id);
        self.used_resources.insert(texture.id);

        if let Some(index_buffer) = index_buffer {
            self.used_resources.insert(index_buffer.id);
        }

        for unit in self.texture_units.iter().flatten() {
            self.used_resources.insert(unit.id);
        }

        let pipeline = &self.pipelines[&pipeline_key];
        let stencil_reference = self.stencil_reference;

        let dynamic = shader.dynamic.borrow();
        let Some((bind_group, _)) = &dynamic.bind_group else {
            return;
        };

        let encoder = self.encoder.get_or_insert_with(|| {
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("tetra"),
                })
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tetra_draw"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.attachment,
                depth_slice: None,
                resolve_target: target.resolve.as_ref(),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: target.depth_stencil.as_ref().map(|view| {
                wgpu::RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                }
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_viewport(viewport.0, viewport.1, viewport.2, viewport.3, 0.0, 1.0);

        if let Some((x, y, w, h)) = scissor {
            pass.set_scissor_rect(x, y, w, h);
        }

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.set_stencil_reference(stencil_reference as u32);
        pass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));

        match index_buffer {
            Some(index_buffer) => {
                pass.set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);

                let max_count = index_buffer.count();

                let offset = usize::min(offset, max_count.saturating_sub(1));
                let count = usize::min(count, max_count.saturating_sub(offset));

                pass.draw_indexed(
                    offset as u32..(offset + count) as u32,
                    0,
                    0..instances as u32,
                );
            }

            None => {
                let max_count = vertex_buffer.count();

                let offset = usize::min(offset, max_count.saturating_sub(1));
                let count = usize::min(count, max_count.saturating_sub(offset));

                pass.draw(offset as u32..(offset + count) as u32, 0..instances as u32);
            }
        }
    }

    fn flush_uniforms(&mut self, shader: &RawShader) {
        let any_dirty = {
            let dynamic = shader.dynamic.borrow();
            dynamic.dirty.iter().any(|d| *d)
        };

        if !any_dirty {
            return;
        }

        self.prepare_write(shader.id);

        let mut dynamic = shader.dynamic.borrow_mut();
        let dynamic = &mut *dynamic;

        for (index, dirty) in dynamic.dirty.iter_mut().enumerate() {
            if *dirty {
                self.queue
                    .write_buffer(&shader.uniforms[index].buffer, 0, &dynamic.blobs[index]);
                *dirty = false;
            }
        }
    }

    fn update_bind_group(&mut self, shader: &RawShader) {
        let mut dynamic = shader.dynamic.borrow_mut();

        let key: Vec<(u64, FilterMode)> = dynamic
            .sampler_units
            .iter()
            .map(|&unit| {
                let bound = self
                    .texture_units
                    .get(unit as usize)
                    .and_then(|t| t.as_ref())
                    .unwrap_or(&self.dummy_texture);

                (bound.id, bound.filter_mode)
            })
            .collect();

        if let Some((_, cached_key)) = &dynamic.bind_group {
            if *cached_key == key {
                return;
            }
        }

        let mut entries = Vec::new();

        for slot in &shader.uniforms {
            entries.push(wgpu::BindGroupEntry {
                binding: slot.binding,
                resource: slot.buffer.as_entire_binding(),
            });
        }

        let bound_textures: Vec<&BoundTexture> = dynamic
            .sampler_units
            .iter()
            .map(|&unit| {
                self.texture_units
                    .get(unit as usize)
                    .and_then(|t| t.as_ref())
                    .unwrap_or(&self.dummy_texture)
            })
            .collect();

        for (slot, bound) in shader.samplers.iter().zip(&bound_textures) {
            entries.push(wgpu::BindGroupEntry {
                binding: slot.tex_binding,
                resource: wgpu::BindingResource::TextureView(&bound.view),
            });

            entries.push(wgpu::BindGroupEntry {
                binding: slot.smp_binding,
                resource: wgpu::BindingResource::Sampler(match bound.filter_mode {
                    FilterMode::Nearest => &self.nearest_sampler,
                    FilterMode::Linear => &self.linear_sampler,
                }),
            });
        }

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tetra_shader"),
            layout: &shader.bind_group_layout,
            entries: &entries,
        });

        dynamic.bind_group = Some((bind_group, key));
    }

    fn create_pipeline(&self, shader: &RawShader, key: &PipelineKey) -> wgpu::RenderPipeline {
        let stencil_face = if key.stencil.enabled {
            wgpu::StencilFaceState {
                compare: key.stencil.test.to_wgpu(),
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: key.stencil.action.to_wgpu(),
            }
        } else {
            wgpu::StencilFaceState::IGNORE
        };

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("tetra_pipeline"),
                layout: Some(&shader.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.vs,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &shader.attributes,
                    })],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: match key.front_face {
                        VertexWinding::Clockwise => wgpu::FrontFace::Cw,
                        VertexWinding::CounterClockwise => wgpu::FrontFace::Ccw,
                    },
                    cull_mode: key.cull.then_some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: key.has_depth_stencil.then(|| wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Always),
                    stencil: wgpu::StencilState {
                        front: stencil_face,
                        back: stencil_face,
                        read_mask: key.stencil.read_mask as u32,
                        write_mask: if key.stencil.enabled {
                            key.stencil.write_mask as u32
                        } else {
                            0
                        },
                    },
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: key.samples,
                    ..Default::default()
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader.fs,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: key.format,
                        blend: Some(key.blend.to_wgpu()),
                        write_mask: color_writes(key.color_mask),
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
    }
}

fn present_mode(vsync: bool) -> wgpu::PresentMode {
    if vsync {
        wgpu::PresentMode::AutoVsync
    } else {
        wgpu::PresentMode::AutoNoVsync
    }
}

fn color_writes(mask: [bool; 4]) -> wgpu::ColorWrites {
    let mut writes = wgpu::ColorWrites::empty();

    if mask[0] {
        writes |= wgpu::ColorWrites::RED;
    }

    if mask[1] {
        writes |= wgpu::ColorWrites::GREEN;
    }

    if mask[2] {
        writes |= wgpu::ColorWrites::BLUE;
    }

    if mask[3] {
        writes |= wgpu::ColorWrites::ALPHA;
    }

    writes
}

fn supported_sample_count(
    adapter: &wgpu::Adapter,
    format: wgpu::TextureFormat,
    requested: u8,
) -> u32 {
    let flags = adapter.get_texture_format_features(format).flags;

    let supported: Vec<u32> = [2, 4, 8, 16]
        .into_iter()
        .filter(|&count| flags.sample_count_supported(count))
        .collect();

    let requested = requested as u32;

    supported
        .iter()
        .copied()
        .find(|&count| count >= requested)
        .or_else(|| supported.last().copied())
        .unwrap_or(1)
}

fn parse_glsl(source: &str, stage: naga::ShaderStage) -> Result<naga::Module> {
    let mut frontend = naga::front::glsl::Frontend::default();

    let mut module = frontend
        .parse(&naga::front::glsl::Options::from(stage), source)
        .map_err(|e| TetraError::InvalidShader(e.emit_to_string(source)))?;

    patch_uniform_array_strides(&mut module);

    Ok(module)
}

/// Naga's GLSL frontend generates tightly-packed array types, but the
/// uniform address space requires array strides to be a multiple of 16
/// (as per std140). The stride is layout metadata only, so it can safely be
/// patched after parsing - the CPU-side uniform writes use the reflected
/// stride, so both sides stay in agreement.
fn patch_uniform_array_strides(module: &mut naga::Module) {
    let mut to_patch = Vec::new();

    for (_, var) in module.global_variables.iter() {
        if var.space == naga::AddressSpace::Uniform {
            if let naga::TypeInner::Array { base, size, stride } = module.types[var.ty].inner {
                if stride % 16 != 0 {
                    to_patch.push((var.ty, base, size, stride.next_multiple_of(16)));
                }
            }
        }
    }

    for (old_ty, base, size, new_stride) in to_patch {
        let span = module.types.get_span(old_ty);
        let name = module.types[old_ty].name.clone();

        let new_ty = module.types.insert(
            naga::Type {
                name,
                inner: naga::TypeInner::Array {
                    base,
                    size,
                    stride: new_stride,
                },
            },
            span,
        );

        for (_, var) in module.global_variables.iter_mut() {
            if var.ty == old_ty && var.space == naga::AddressSpace::Uniform {
                var.ty = new_ty;
            }
        }
    }
}

fn find_global_type<'a>(module: &'a naga::Module, name: &str) -> Option<&'a naga::TypeInner> {
    module
        .global_variables
        .iter()
        .find(|(_, var)| var.name.as_deref() == Some(name) && var.binding.is_some())
        .map(|(_, var)| &module.types[var.ty].inner)
}

fn find_uniform_kind(module: &naga::Module, name: &str) -> Option<UniformKind> {
    let ty = find_global_type(module, name)?;

    match *ty {
        naga::TypeInner::Array { base, size, stride } => {
            let count = match size {
                naga::ArraySize::Constant(count) => count.get(),
                _ => return None,
            };

            Some(UniformKind {
                element: ElementLayout::from_naga(&module.types[base].inner)?,
                array: Some((stride, count)),
            })
        }

        _ => Some(UniformKind {
            element: ElementLayout::from_naga(ty)?,
            array: None,
        }),
    }
}

/// Describes the GPU-side layout of a uniform, so that tightly-packed
/// OpenGL-style uniform data can be converted when it's uploaded.
#[derive(Debug, Clone, Copy)]
struct UniformKind {
    element: ElementLayout,
    array: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Copy)]
enum ElementLayout {
    /// A scalar or vector - a tight copy of `n` bytes.
    Bytes(u32),

    /// A matrix - `cols` columns of `rows * 4` bytes, spaced `col_stride`
    /// bytes apart.
    Matrix {
        cols: u32,
        rows: u32,
        col_stride: u32,
    },
}

impl ElementLayout {
    fn from_naga(ty: &naga::TypeInner) -> Option<ElementLayout> {
        match *ty {
            naga::TypeInner::Scalar(s) => Some(ElementLayout::Bytes(s.width as u32)),

            naga::TypeInner::Vector { size, scalar } => {
                Some(ElementLayout::Bytes(size as u32 * scalar.width as u32))
            }

            naga::TypeInner::Matrix {
                columns,
                rows,
                scalar,
            } => {
                let col_size = rows as u32 * scalar.width as u32;

                // Column vectors are aligned to their size, rounded up to a
                // power of two (vec3 aligns like vec4).
                let col_stride = match rows as u32 {
                    2 => col_size,
                    _ => col_size.next_multiple_of(16),
                };

                Some(ElementLayout::Matrix {
                    cols: columns as u32,
                    rows: rows as u32,
                    col_stride,
                })
            }

            _ => None,
        }
    }

    fn src_size(&self) -> u32 {
        match *self {
            ElementLayout::Bytes(n) => n,
            ElementLayout::Matrix { cols, rows, .. } => cols * rows * 4,
        }
    }

    fn dst_size(&self) -> u32 {
        match *self {
            ElementLayout::Bytes(n) => n,
            ElementLayout::Matrix {
                cols, col_stride, ..
            } => cols * col_stride,
        }
    }

    fn write(&self, dst: &mut [u8], src: &[u8]) {
        match *self {
            ElementLayout::Bytes(n) => {
                let n = usize::min(n as usize, src.len());
                dst[..n].copy_from_slice(&src[..n]);
            }

            ElementLayout::Matrix {
                cols,
                rows,
                col_stride,
            } => {
                let col_size = (rows * 4) as usize;

                for col in 0..cols as usize {
                    let src_start = col * col_size;
                    let dst_start = col * col_stride as usize;

                    dst[dst_start..dst_start + col_size]
                        .copy_from_slice(&src[src_start..src_start + col_size]);
                }
            }
        }
    }
}

impl UniformKind {
    fn buffer_size(&self) -> u32 {
        match self.array {
            Some((stride, count)) => stride * count,
            None => self.element.dst_size().next_multiple_of(4),
        }
    }

    /// Writes tightly-packed source data into the uniform's buffer layout.
    /// Returns whether the contents actually changed.
    fn write(&self, blob: &mut [u8], src: &[u8], elem_src_size: usize) -> bool {
        let mut scratch = blob.to_vec();

        let expected_src_size = self.element.src_size() as usize;

        // If the caller's element size doesn't match the uniform (e.g.
        // setting a `vec4` uniform from `f32` data), OpenGL would just
        // ignore the call - splitting the source using the caller's element
        // size keeps roughly the same behaviour.
        let elem_size = usize::min(elem_src_size, expected_src_size);

        match self.array {
            Some((stride, count)) => {
                for (i, chunk) in src.chunks(elem_src_size).take(count as usize).enumerate() {
                    let offset = i * stride as usize;
                    let end = usize::min(offset + self.element.dst_size() as usize, scratch.len());

                    if offset >= scratch.len() {
                        break;
                    }

                    self.element
                        .write(&mut scratch[offset..end], &chunk[..elem_size.min(chunk.len())]);
                }
            }

            None => {
                if let Some(chunk) = src.chunks(elem_src_size).next() {
                    self.element.write(
                        &mut scratch[..],
                        &chunk[..elem_size.min(chunk.len())],
                    );
                }
            }
        }

        if scratch != blob {
            blob.copy_from_slice(&scratch);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Location {
    Data(usize),
    Sampler(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformLocation {
    location: Location,
}

#[derive(Debug)]
struct UniformSlot {
    binding: u32,
    buffer: wgpu::Buffer,
    size: u32,
    kind: UniformKind,
}

#[derive(Debug)]
struct SamplerSlot {
    tex_binding: u32,
    smp_binding: u32,
}

#[derive(Debug)]
struct ShaderDynamic {
    blobs: Vec<Vec<u8>>,
    dirty: Vec<bool>,
    sampler_units: Vec<u32>,
    bind_group: Option<(wgpu::BindGroup, Vec<(u64, FilterMode)>)>,
}

#[derive(Debug)]
pub struct RawShader {
    id: u64,
    vs: wgpu::ShaderModule,
    fs: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    attributes: Vec<wgpu::VertexAttribute>,
    uniforms: Vec<UniformSlot>,
    samplers: Vec<SamplerSlot>,
    by_name: HashMap<String, Location>,
    dynamic: RefCell<ShaderDynamic>,
}

impl PartialEq for RawShader {
    fn eq(&self, other: &RawShader) -> bool {
        self.id == other.id
    }
}

#[derive(Debug)]
pub struct RawVertexBuffer {
    id: u64,
    buffer: wgpu::Buffer,
    count: usize,
}

impl RawVertexBuffer {
    /// The number of vertices in the buffer.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The size of each vertex, in bytes.
    pub fn stride(&self) -> usize {
        mem::size_of::<Vertex>()
    }

    /// The size of the buffer, in bytes.
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.count * self.stride()
    }
}

impl PartialEq for RawVertexBuffer {
    fn eq(&self, other: &RawVertexBuffer) -> bool {
        self.id == other.id
    }
}

#[derive(Debug)]
pub struct RawIndexBuffer {
    id: u64,
    buffer: wgpu::Buffer,
    count: usize,
}

impl RawIndexBuffer {
    /// The number of indices in the buffer.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The size of each index, in bytes.
    pub fn stride(&self) -> usize {
        mem::size_of::<u32>()
    }

    /// The size of the buffer, in bytes.
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.count * self.stride()
    }
}

impl PartialEq for RawIndexBuffer {
    fn eq(&self, other: &RawIndexBuffer) -> bool {
        self.id == other.id
    }
}

#[derive(Debug)]
pub struct RawTexture {
    id: u64,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: i32,
    height: i32,
    format: TextureFormat,
    filter_mode: Cell<FilterMode>,
}

impl RawTexture {
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }
}

impl PartialEq for RawTexture {
    fn eq(&self, other: &RawTexture) -> bool {
        self.id == other.id
    }
}

#[derive(Debug)]
pub struct RawCanvas {
    id: u64,
    target: CanvasTarget,
}

impl PartialEq for RawCanvas {
    fn eq(&self, other: &RawCanvas) -> bool {
        self.id == other.id
    }
}

pub struct RawCanvasWithAttachments {
    pub canvas: RawCanvas,
    pub color: RawTexture,
    pub multisample_color: Option<RawRenderbuffer>,
    pub depth_stencil: Option<RawRenderbuffer>,
}

#[derive(Debug)]
pub struct RawRenderbuffer {
    id: u64,
    _texture: wgpu::Texture,
}

impl PartialEq for RawRenderbuffer {
    fn eq(&self, other: &RawRenderbuffer) -> bool {
        self.id == other.id
    }
}

#[doc(hidden)]
impl TextureFormat {
    fn to_wgpu(self) -> wgpu::TextureFormat {
        match self {
            TextureFormat::Rgba8 => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::R8 => wgpu::TextureFormat::R8Unorm,
            TextureFormat::Rg8 => wgpu::TextureFormat::Rg8Unorm,
            TextureFormat::Rgba16F => wgpu::TextureFormat::Rgba16Float,
        }
    }
}

#[doc(hidden)]
impl StencilTest {
    fn to_wgpu(self) -> wgpu::CompareFunction {
        match self {
            StencilTest::Never => wgpu::CompareFunction::Never,
            StencilTest::LessThan => wgpu::CompareFunction::Less,
            StencilTest::LessThanOrEqualTo => wgpu::CompareFunction::LessEqual,
            StencilTest::EqualTo => wgpu::CompareFunction::Equal,
            StencilTest::NotEqualTo => wgpu::CompareFunction::NotEqual,
            StencilTest::GreaterThan => wgpu::CompareFunction::Greater,
            StencilTest::GreaterThanOrEqualTo => wgpu::CompareFunction::GreaterEqual,
            StencilTest::Always => wgpu::CompareFunction::Always,
        }
    }
}

#[doc(hidden)]
impl StencilAction {
    fn to_wgpu(self) -> wgpu::StencilOperation {
        match self {
            StencilAction::Keep => wgpu::StencilOperation::Keep,
            StencilAction::Zero => wgpu::StencilOperation::Zero,
            StencilAction::Replace => wgpu::StencilOperation::Replace,
            StencilAction::Increment => wgpu::StencilOperation::IncrementClamp,
            StencilAction::IncrementWrap => wgpu::StencilOperation::IncrementWrap,
            StencilAction::Decrement => wgpu::StencilOperation::DecrementClamp,
            StencilAction::DecrementWrap => wgpu::StencilOperation::DecrementWrap,
            StencilAction::Invert => wgpu::StencilOperation::Invert,
        }
    }
}

#[doc(hidden)]
impl BlendOperation {
    fn to_wgpu(self) -> wgpu::BlendOperation {
        match self {
            BlendOperation::Add => wgpu::BlendOperation::Add,
            BlendOperation::Subtract => wgpu::BlendOperation::Subtract,
            BlendOperation::ReverseSubtract => wgpu::BlendOperation::ReverseSubtract,
            BlendOperation::Min => wgpu::BlendOperation::Min,
            BlendOperation::Max => wgpu::BlendOperation::Max,
        }
    }
}

#[doc(hidden)]
impl BlendFactor {
    fn to_wgpu(self) -> wgpu::BlendFactor {
        match self {
            BlendFactor::Zero => wgpu::BlendFactor::Zero,
            BlendFactor::One => wgpu::BlendFactor::One,
            BlendFactor::Src => wgpu::BlendFactor::Src,
            BlendFactor::OneMinusSrc => wgpu::BlendFactor::OneMinusSrc,
            BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            BlendFactor::Dst => wgpu::BlendFactor::Dst,
            BlendFactor::OneMinusDst => wgpu::BlendFactor::OneMinusDst,
            BlendFactor::DstAlpha => wgpu::BlendFactor::DstAlpha,
            BlendFactor::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
            BlendFactor::SrcAlphaSaturated => wgpu::BlendFactor::SrcAlphaSaturated,
            BlendFactor::Constant => wgpu::BlendFactor::Constant,
            BlendFactor::OneMinusConstant => wgpu::BlendFactor::OneMinusConstant,
        }
    }
}

#[doc(hidden)]
impl BlendState {
    fn to_wgpu(self) -> wgpu::BlendState {
        wgpu::BlendState {
            color: blend_component(self.color_operation, self.color_src, self.color_dst),
            alpha: blend_component(self.alpha_operation, self.alpha_src, self.alpha_dst),
        }
    }
}

fn blend_component(
    operation: BlendOperation,
    src: BlendFactor,
    dst: BlendFactor,
) -> wgpu::BlendComponent {
    // For min/max, the factors must be one (they're ignored anyway).
    let (src, dst) = match operation {
        BlendOperation::Min | BlendOperation::Max => (BlendFactor::One, BlendFactor::One),
        _ => (src, dst),
    };

    wgpu::BlendComponent {
        operation: operation.to_wgpu(),
        src_factor: src.to_wgpu(),
        dst_factor: dst.to_wgpu(),
    }
}

/// This function exists because Vek isn't currently compatible with Bytemuck, so
/// there's not an easy way of converting slices of that library's types.
///
/// This should be replaced with `bytemuck::cast_slice` as soon as that's possible,
/// and it should not be used for anything other than Vek interop!
unsafe fn cast_slice_assume_aligned<A, B>(a: &[A]) -> &[B] {
    slice::from_raw_parts(
        a.as_ptr() as *const B,
        mem::size_of_val(a) / mem::size_of::<B>(),
    )
}
