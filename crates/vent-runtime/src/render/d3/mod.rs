use std::mem;

use glam::Mat4;
use vent_assets::{Mesh3D, Vertex, Vertex3D};
use vent_ecs::world::World;
use wgpu::util::DeviceExt;

use self::light_renderer::LightRenderer;

use super::{
    camera::{Camera, Camera3D},
    model::Entity3D,
    model_renderer::ModelRenderer3D,
    Renderer,
};

pub mod light_renderer;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UBO3D {
    pub view_position: [f32; 3],
    pub _padding: u32,
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub transformation: [[f32; 4]; 4],
}

impl Default for UBO3D {
    fn default() -> Self {
        Self {
            view_position: Default::default(),
            _padding: Default::default(),
            projection: Default::default(),
            view: Default::default(),
            transformation: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

pub struct Renderer3D {
    mesh_renderer: ModelRenderer3D,
    light_renderer: LightRenderer,
    tmp_light_mesh: Mesh3D,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
}

impl Renderer for Renderer3D {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    ) -> Self
    where
        Self: Sized,
    {
        let camera: &Camera3D = camera.downcast_ref().unwrap();
        // Create pipeline layout
        let vertex_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("3D Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

        let light_renderer = LightRenderer::new(device, &vertex_group_layout, config.format);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("3D Pipeline Layout"),
            bind_group_layouts: &[
                &vertex_group_layout,
                &texture_bind_group_layout,
                &light_renderer.light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // Create other resources
        let ubo = camera.ubo();
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
        let vertex_buffer_layout = [Vertex3D::LAYOUT];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Renderer Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layout,
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

        // TODO
        let pipeline_wire = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            let shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/res/shaders/app/3D/wireframe.wgsl"
            )));

            let pipeline_wire = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("3D Pipeline Wireframe"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
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
            let mesh = Entity3D::new(
                vent_assets::Model3D::load(device, queue, model, &texture_bind_group_layout).await,
            );
            mesh_renderer.insert(world.create_entity(), mesh);
        });

        // -------------------------------

        let tmp_light_mesh = create_tmp_cube(device);

        Self {
            mesh_renderer,
            light_renderer,
            tmp_light_mesh,
            bind_group,
            uniform_buf,
            pipeline,
            pipeline_wire,
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    ) {
        let camera: &mut Camera3D = camera.downcast_mut().unwrap();

        camera.recreate_projection(config.width as f32 / config.height as f32);
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[camera.ubo()]));
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    ) {
        let camera: &mut Camera3D = camera.downcast_mut().unwrap();

        let mut ubo = camera.ubo();
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
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            self.light_renderer
                .render(&mut rpass, &self.bind_group, &self.tmp_light_mesh);

            rpass.set_pipeline(&self.pipeline);
            if let Some(ref pipe) = self.pipeline_wire {
                rpass.set_pipeline(pipe);
            }
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_bind_group(2, &self.light_renderer.light_bind_group, &[]);
            self.mesh_renderer.render(&mut rpass, &mut ubo);
        }
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[ubo]));
    }
}

fn create_tmp_cube(device: &wgpu::Device) -> vent_assets::Mesh3D {
    let indices = [
        //Top
        2, 6, 7, 2, 3, 7, //Bottom
        0, 4, 5, 0, 1, 5, //Left
        0, 2, 6, 0, 4, 6, //Right
        1, 3, 7, 1, 5, 7, //Front
        0, 2, 3, 0, 1, 3, //Back
        4, 6, 7, 4, 5, 7,
    ];

    let vertices = [
        Vertex3D {
            position: [-1.0, -1.0, 0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //0
        Vertex3D {
            position: [1.0, -1.0, 0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //1
        Vertex3D {
            position: [-1.0, 1.0, 0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //2
        Vertex3D {
            position: [1.0, 1.0, 0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //3
        Vertex3D {
            position: [-1.0, -1.0, -0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //4
        Vertex3D {
            position: [0.0, -1.0, -0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //5
        Vertex3D {
            position: [-1.0, 1.0, -0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //6
        Vertex3D {
            position: [1.0, 1.0, -0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //7
    ];

    vent_assets::Mesh3D::new(device, &vertices, &indices, None, None)
}
