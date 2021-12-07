use {
    crate::{
        camera::{self, CameraUniform},
        model::{self, Vertex},
        octant_model::{DrawOctant, OctantMesh},
        octree::Octant,
        points_model::{DrawPoints, PointMesh},
        texture,
        types::Point,
    },
    log::warn,
    wgpu::util::DeviceExt,
    winit::{
        event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
        window::Window,
    },
};

pub struct RenderState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,

    camera_uniform: CameraUniform,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,

    depth_texture: texture::Texture,

    camera: camera::Camera,
    camera_controller: camera::CameraController,

    render_octants: bool,
    octants: Option<OctantMesh>,

    render_points: bool,
    points: Option<PointMesh>,
}

impl RenderState {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
        });

        // Camera code
        let mut camera = camera::Camera::new((0.0, 5.0, 10.0), &config); //cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let camera_controller = camera::CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::new();

        camera_uniform.update_view_proj(&mut camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[model::ModelVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        RenderState {
            surface,
            device,
            queue,
            config,
            size,

            render_pipeline,

            camera_uniform,
            camera_bind_group,
            camera_buffer,

            depth_texture,

            camera,
            camera_controller,

            // Octree octants
            render_octants: false,
            octants: None,

            render_points: true,
            points: None,
        }
    }

    pub fn set_octree_model(&mut self, octants: Vec<Octant>) {
        self.octants = Some(OctantMesh::new(&self.device, &self.queue, &octants));
    }

    pub fn set_points_model(&mut self, points: Vec<Point>) {
        self.points = Some(PointMesh::new(&self.device, &self.queue, &points));
    }

    pub fn update(&mut self, lrt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, lrt, &self.config);
        self.camera_uniform.update_view_proj(&mut self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Process device input for the graphical state machine
    pub fn device_input(&mut self, event: &DeviceEvent) -> bool {
        warn!("Unhandled device input event: {:?}", event);
        false
    }

    /// Process input for the graphical state machine
    ///
    /// Returns optional control flow directive based upon event input.
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput { state, virtual_keycode: Some(key), .. },
                ..
            } => {
                if *key == VirtualKeyCode::O && *state == ElementState::Pressed {
                    self.render_octants = !self.render_octants;
                }
                self.camera_controller.process_keyboard(*key, *state)
            }
            WindowEvent::MouseWheel { delta, .. } => self.camera_controller.process_scroll(delta),
            evt => {
                warn!("Unhandled input event: {:?}", evt);
                false
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.9, g: 0.8, b: 0.6, a: 1.0 }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            if let Some(octants) = &self.octants {
                if self.render_octants {
                    render_pass.draw_octants(octants);
                }
            }

            if let Some(points) = &self.points {
                if self.render_points {
                    render_pass.draw_points(points);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
