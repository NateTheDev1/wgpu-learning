use bytemuck::cast_slice;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BlendState, Buffer, BufferUsages, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CompositeAlphaMode, FragmentState, FrontFace, IndexFormat, Limits,
    MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PresentMode,
    PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource,
    SurfaceConfiguration, TextureUsages, TextureViewDescriptor, VertexState,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::vertex::{Vertex, INDICES, VERTICES};

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
}

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

        // shortcut
        // let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                // references the entry point for the vertex shader
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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
            depth_stencil: None,
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
            self.surface.configure(&self.device, &self.config);
        }
    }

    // Returns a bool based on wether an event has been fully processed or not.
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

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
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            // Takes the buffer slot to use the vertex buffer. Also takes the slice of the buffer to use. In this case: all of it.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // Can only have one index buffer per render pass.
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            // Drawing something with 3 vertices and 1 instance. This is where @builtin(vertex_index) comes from.
            // Draw ignores the index buffer
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // Builds command buffer and sends to GPU render queue.
        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}
