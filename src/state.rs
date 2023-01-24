use bytemuck::cast_slice;
use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, Quaternion, Rotation3, Vector3, Zero};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferBindingType, BufferUsages, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CompareFunction, CompositeAlphaMode, DepthBiasState,
    DepthStencilState, FragmentState, FrontFace, IndexFormat, Limits, LoadOp, MultisampleState,
    Operations, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState,
    PrimitiveTopology, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StencilState, SurfaceConfiguration, TextureSampleType,
    TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexState,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{
    model::{Model, Vertex},
    resources::{load_model, DrawModel},
};

use crate::{
    camera::{Camera, CameraController, CameraUniform},
    instance::{self, Instance, InstanceRaw},
    model::ModelVertex,
    texture::Texture,
    vertex::{INDICES, VERTICES},
};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    window: Window,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    num_vertices: u32,
    index_buffer: Buffer,
    num_indices: u32,
    diffuse_bind_group: BindGroup,
    diffuse_texture: Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    camera_controller: CameraController,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
    depth_texture: Texture,
    obj_model: Model,
}

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: Vector3<f32> = Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // A handle to the GPU
        // Backends:all => Vulkan + Metal + DX12 + Browser WebGPU
        // Handles creation surface(s) and adapter(s)
        let instance = wgpu::Instance::new(Backends::all());

        // The surface lives as long as the window that created it exists.
        // The state owns the window so as long as the function can be called this is safe.
        let surface = unsafe { instance.create_surface(&window) };

        // The real handle to the GPU
        let adapter = instance
            // Can be traded for `enumerate_adapters`
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    // https://docs.rs/wgpu/latest/wgpu/struct.Features.html
                    features: wgpu::Features::empty(),
                    limits: Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = SurfaceConfiguration {
            // Means we want textures to write to the screen
            usage: TextureUsages::RENDER_ATTACHMENT,
            // Preferred format the GPU wants to store the SurfaceTexture(s)
            format: surface.get_supported_formats(&adapter)[0],
            // width and height of the SurfaceTexture
            width: size.width,
            height: size.height,
            // Forces Vsync to monitor refresh rate.
            // https://docs.rs/wgpu/latest/wgpu/enum.PresentMode.html
            // To allow user selection: `let modes = surface.get_supported_modes(&adapter);`
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &config);

        // Textures
        let diffuse_bytes = include_bytes!("assets/happy-tree.png");
        let diffuse_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        // let diffuse_rgba = diffuse_image.to_rgba8();

        // For creating the texture
        // let dimensions = diffuse_image.dimensions();

        // let texture_size = Extent3d {
        //     width: dimensions.0,
        //     height: dimensions.1,
        //     depth_or_array_layers: 1,
        // };

        // let diffuse_texture = device.create_texture(&TextureDescriptor {
        //     size: texture_size,
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: TextureDimension::D2,
        //     // Common storage is srgb
        //     format: TextureFormat::Rgba8UnormSrgb,
        //     // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
        //     // COPY_DST means that we want to copy data to this texture
        //     usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        //     label: Some("diffuse_texture"),
        // });

        // queue.write_texture(
        //     ImageCopyTexture {
        //         texture: &diffuse_texture,
        //         mip_level: 0,
        //         origin: Origin3d::ZERO,
        //         aspect: TextureAspect::All,
        //     },
        //     // the pixel data
        //     &diffuse_rgba,
        //     // the layout of the texture
        //     ImageDataLayout {
        //         offset: 0,
        //         bytes_per_row: num::NonZeroU32::new(4 * dimensions.0),
        //         rows_per_image: num::NonZeroU32::new(dimensions.1),
        //     },
        //     texture_size,
        // );

        // let diffuse_texture_view = diffuse_texture.create_view(&TextureViewDescriptor::default());

        // let diffuse_sampler = device.create_sampler(&SamplerDescriptor {
        //     // address_mode_* determines what the sampler should do when a texture coordinate is outside the texture itself.
        //     // https://sotrh.github.io/learn-wgpu/assets/img/address_mode.66a7cd1a.png
        //     // ClampToEdge: Any texture coordinates outside the texture will return the color of the nearest pixel on the edges of the texture.
        //     // Repeat: The texture will repeat as texture coordinates exceed the texture's dimensions.
        //     // MirrorRepeat: Similar to Repeat, but the image will flip when going over boundaries.
        //     address_mode_u: AddressMode::ClampToEdge,
        //     address_mode_v: AddressMode::ClampToEdge,
        //     address_mode_w: AddressMode::ClampToEdge,
        //     // mag and min describe what to do when the sample is smaller than one texel. (This can happen when mapping is far from or close to the camera)
        //     // Linear: Selects two texels in each dimension and returns a linear interpolation between them.
        //     // Nearest: Selects the texel closest to the texture coordinates. Crisper far away but pixelated up close. This is okay for games like minecraft or voxel games.
        //     mag_filter: FilterMode::Linear,
        //     min_filter: FilterMode::Nearest,
        //     // TODO: What is this? Basic: They are like mag/min in a way
        //     mipmap_filter: FilterMode::Nearest,
        //     ..Default::default()
        // });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    // For sampled texture
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // For sampler
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // BindGroup is a more specific decleration of the BindGroupLayout
        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera = Camera {
            // Camera is 1 unit up and 2 units back
            eye: (0.0, 1.0, 2.0).into(),
            // This looks at the origin point
            target: (0.0, 0.0, 0.0).into(),
            // This says which way is "up"
            up: Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();

        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                // Only accessing vertex index of shader because the camera can use that to manipulate the vertices
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        // Will the buffer change size or not?
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let camera_controller = CameraController::new(0.1);

        // shortcut
        // let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                // references the entry point for the vertex shader
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                // references the entry point for the fragment shader
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            // how to interpret the vertices when converting to triangles
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                // Tells WGPU if a triangle is facing the camera or not.
                front_face: FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less, // Tells us when to discard a pixel. LESS means pixels will be drawn front to back.
                stencil: StencilState::default(), // Controls values for stencil testing. We are not using this.
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                // how many samples the pipeline will use
                count: 1,
                // Specifies which samples are used. Here we are using all.
                mask: !0,
                // Anti-Aliasing related
                alpha_to_coverage_enabled: false,
            },
            // Indicates how many array layers the render attachments can have. We are not rendering any to array textures (None)
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        let num_vertices = VERTICES.len() as u32;

        const SPACE_BETWEEN: f32 = 3.0;

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0))
                    } else {
                        Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: cast_slice(&instance_data),
            usage: BufferUsages::VERTEX,
        });

        let obj_model = load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
            .await
            .unwrap();

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            num_vertices,
            diffuse_bind_group,
            camera_controller,
            diffuse_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            instance_buffer,
            instances,
            depth_texture,
            obj_model,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            self.surface.configure(&self.device, &self.config);
        }
    }

    // Returns a bool based on wether an event has been fully processed or not.
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue
            .write_buffer(&self.camera_buffer, 0, cast_slice(&[self.camera_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Creates a command encoder that sends commands to the GPU.
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Rust Tip: Releases any variables once block is done. Releases mut encoder.
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                // Draws color to the view (TextureView)
                // This is what @location(0) in the fragment shader targets
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);

            // Takes the buffer slot to use the vertex buffer. Also takes the slice of the buffer to use. In this case: all of it.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            // Can only have one index buffer per render pass.
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            // Drawing something with 3 vertices and 1 instance. This is where @builtin(vertex_index) comes from.
            // Draw ignores the index buffer
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);

            let mesh = &self.obj_model.meshes[0];
            let material = &self.obj_model.materials[mesh.material];

            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
            );
        }

        // Builds command buffer and sends to GPU render queue.
        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}
