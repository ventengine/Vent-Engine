use crate::render::Dimension;

use crate::render::model_renderer::ModelRenderer3D;
use std::mem;
use std::path::Path;
use vent_assets::{Vertex, Vertex3D};
use vent_common::render::{DefaultRenderer, UBO3D};
use vent_ecs::world::World;
use wgpu::util::DeviceExt;

use super::{model::Model3D, camera::Camera};

pub struct VentApplicationManager {
    multi_renderer: Box<dyn MultiDimensionRenderer>,
}

impl VentApplicationManager {
    pub fn new(
        dimension: Dimension,
        default_renderer: &DefaultRenderer,
        camera: &mut dyn Camera,
    ) -> Self {
        Self {
            multi_renderer: match dimension {
                Dimension::D2 => Box::new(Renderer2D::init(
                    &default_renderer.config,
                    &default_renderer.adapter,
                    &default_renderer.device,
                    &default_renderer.queue,
                    camera,
                )),
                Dimension::D3 => Box::new(Renderer3D::init(
                    &default_renderer.config,
                    &default_renderer.adapter,
                    &default_renderer.device,
                    &default_renderer.queue,
                    camera,
                )),
            },
        }
    }

    pub fn update(&self) {}

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
        aspect_ratio: f32,
    ) {
        self.multi_renderer
            .render(encoder, view, queue, camera, aspect_ratio)
    }

    pub fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    ) {
        self.multi_renderer.resize(config, _device, queue, camera);
    }
}

pub trait MultiDimensionRenderer {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    ) -> Self
    where
        Self: Sized;

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    );

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
        aspect_ratio: f32,
    );
}

pub struct Renderer2D {}

impl MultiDimensionRenderer for Renderer2D {
    fn init(
        _config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _camera: &mut dyn Camera,
    ) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn resize(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _camera: &mut dyn Camera,
    ) {
        todo!()
    }

    fn render(
        &mut self,
        _encoder: &mut wgpu::CommandEncoder,
        _view: &wgpu::TextureView,
        _queue: &wgpu::Queue,
        _camera: &mut dyn Camera,
        _aspect_ratio: f32,
    ) {
        todo!()
    }
}

pub struct Renderer3D {
    mesh_renderer: ModelRenderer3D,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
    depth_texture: wgpu::TextureView,
}

impl MultiDimensionRenderer for Renderer3D {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    ) -> Self
    where
        Self: Sized,
    {
        // Create pipeline layout
        let vertex_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("3D Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<UBO3D>() as wgpu::BufferAddress
                        ),
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(mem::size_of::<
                                vent_assets::model::Material,
                            >()
                                as wgpu::BufferAddress),
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("3D Pipeline Layout"),
            bind_group_layouts: &[&vertex_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create other resources
        let ubo = camera.build_view_matrix_3d(config.width as f32 / config.height as f32);
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::bytes_of(&ubo),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &vertex_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: Some("3D Bind Group"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/shaders/app/3D/shader.wgsl"
        )));
        let vertex_buffers = [Vertex3D::LAYOUT];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Renderer Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: vent_assets::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let pipeline_wire = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            let pipeline_wire = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("3D Pipeline Wireframe"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_wire",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                operation: wgpu::BlendOperation::Add,
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            },
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    front_face: wgpu::FrontFace::Cw,
                    polygon_mode: wgpu::PolygonMode::Line,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
            Some(pipeline_wire)
        } else {
            None
        };

        let mut mesh_renderer = ModelRenderer3D::default();

        // -------------- DEMO -------------------
        let mut world = World::new();

        let model = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/models/test/Sponza-GLTF/Sponza.gltf"
        );

        pollster::block_on(async {
            let mut mesh = Model3D::new(
                vent_assets::Model3D::load(
                    device,
                    queue,
                    Path::new(model),
                    &texture_bind_group_layout,
                )
                .await,
            );
            mesh.rotation.x += 100.0;
            mesh_renderer.insert(world.create_entity(), mesh);
        });

        // -------------------------------

        let depth_texture = vent_assets::Texture::create_depth_texture(device, config, None);

        Self {
            mesh_renderer,
            bind_group,
            uniform_buf,
            pipeline,
            pipeline_wire,
            depth_texture,
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    ) {
        self.depth_texture = vent_assets::Texture::create_depth_texture(device, config, None);

        let ubo = camera.build_view_matrix_3d(config.width as f32 / config.height as f32);
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[ubo]));
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
        aspect_ratio: f32,
    ) {
        let mut ubo = camera.build_view_matrix_3d(aspect_ratio);
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3D Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            if let Some(ref pipe) = self.pipeline_wire {
                rpass.set_pipeline(pipe);
            }
            self.mesh_renderer.render(&mut rpass, &mut ubo);
        }
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[ubo]));
    }
}
