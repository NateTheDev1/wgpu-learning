use wgpu::{
    Backends, Color, CommandEncoderDescriptor, CompositeAlphaMode, Limits, LoadOp, Operations,
    PresentMode, RenderPassColorAttachment, RenderPassDescriptor, SurfaceConfiguration,
    TextureUsages, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    window: Window,
    clear_color: Color,
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

        let clear_color = wgpu::Color::BLACK;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            clear_color,
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
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
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
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                // Draws color to the view (TextureView)
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        // Builds command buffer and sends to GPU render queue.
        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}
