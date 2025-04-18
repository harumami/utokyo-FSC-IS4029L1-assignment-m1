use {
    color_eyre::install as install_eyre,
    eyre::{
        OptionExt as _,
        Result,
    },
    futures::executor::block_on,
    std::{
        slice::from_raw_parts as new_slice,
        sync::Arc,
    },
    tracing::error,
    tracing_subscriber::{
        fmt::Subscriber,
        util::SubscriberInitExt as _,
    },
    wgpu::{
        include_wgsl,
        util::{
            BufferInitDescriptor,
            DeviceExt as _,
        },
        vertex_attr_array,
        Adapter,
        Backends,
        BlendState,
        Buffer,
        BufferUsages,
        Color,
        ColorTargetState,
        ColorWrites,
        CommandEncoderDescriptor,
        CompositeAlphaMode,
        Device,
        DeviceDescriptor,
        Face,
        Features,
        FragmentState,
        FrontFace,
        Instance,
        InstanceDescriptor,
        InstanceFlags,
        LoadOp,
        MemoryHints,
        MultisampleState,
        Operations,
        PolygonMode,
        PowerPreference,
        PresentMode,
        PrimitiveState,
        PrimitiveTopology,
        Queue,
        RenderBundle,
        RenderBundleDescriptor,
        RenderBundleEncoderDescriptor,
        RenderPassColorAttachment,
        RenderPassDescriptor,
        RenderPipeline,
        RenderPipelineDescriptor,
        RequestAdapterOptions,
        ShaderModule,
        StoreOp,
        Surface,
        SurfaceConfiguration,
        TextureAspect,
        TextureFormat,
        TextureUsages,
        TextureViewDescriptor,
        Trace,
        VertexBufferLayout,
        VertexFormat,
        VertexState,
        VertexStepMode,
    },
    winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{
            ActiveEventLoop,
            ControlFlow,
            EventLoop,
        },
        window::{
            Window,
            WindowId,
        },
    },
};

fn main() {
    if let Result::Err(error) = App::run() {
        error!("{error}");
    }
}

struct App {
    window: Option<Arc<Window>>,
    instance: Option<Instance>,
    surface: Option<Surface<'static>>,
    adapter: Option<Adapter>,
    format: Option<TextureFormat>,
    device: Option<Device>,
    queue: Option<Queue>,
    module: Option<ShaderModule>,
    pipeline: Option<RenderPipeline>,
    buffer: Option<Buffer>,
    bundle: Option<RenderBundle>,
    viewport: Option<[f32; 4]>,
}

impl App {
    const VERTICES: [[f32; 2]; 3] = [[0.0, 0.5], [-0.5, -0.5], [0.5, -0.5]];

    fn run() -> Result<()> {
        install_eyre()?;
        Subscriber::new().try_init()?;
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = Self {
            window: Option::None,
            instance: Option::None,
            surface: Option::None,
            adapter: Option::None,
            format: Option::None,
            device: Option::None,
            queue: Option::None,
            module: Option::None,
            pipeline: Option::None,
            buffer: Option::None,
            bundle: Option::None,
            viewport: Option::None,
        };

        event_loop.run_app(&mut app)?;
        Result::Ok(())
    }

    fn window(&self) -> Result<&Window> {
        self.window
            .as_deref()
            .ok_or_eyre("a window is not created yet")
    }

    fn instance(&self) -> Result<&Instance> {
        self.instance
            .as_ref()
            .ok_or_eyre("an instance is not created yet")
    }

    fn surface(&self) -> Result<&Surface<'static>> {
        self.surface
            .as_ref()
            .ok_or_eyre("a surface is not created yet")
    }

    fn adapter(&self) -> Result<&Adapter> {
        self.adapter
            .as_ref()
            .ok_or_eyre("an adapter is not requested yet")
    }

    fn format(&self) -> Result<TextureFormat> {
        self.format.ok_or_eyre("a format is not initialized yet")
    }

    fn device(&self) -> Result<&Device> {
        self.device
            .as_ref()
            .ok_or_eyre("a device is not requested yet")
    }

    fn queue(&self) -> Result<&Queue> {
        self.queue
            .as_ref()
            .ok_or_eyre("a queue is not requested yet")
    }

    fn module(&self) -> Result<&ShaderModule> {
        self.module
            .as_ref()
            .ok_or_eyre("a module is not created yet")
    }

    fn pipeline(&self) -> Result<&RenderPipeline> {
        self.pipeline
            .as_ref()
            .ok_or_eyre("a pipeline is not created yet")
    }

    fn buffer(&self) -> Result<&Buffer> {
        self.buffer
            .as_ref()
            .ok_or_eyre("a buffer is not created yet")
    }

    fn bundle(&self) -> Result<&RenderBundle> {
        self.bundle
            .as_ref()
            .ok_or_eyre("a bundle is not created yet")
    }

    fn viewport(&self) -> Result<[f32; 4]> {
        self.viewport
            .ok_or_eyre("a viewport is not initialized yet")
    }

    fn clone_window(&self) -> Result<Arc<Window>> {
        self.window
            .clone()
            .ok_or_eyre("a window is not created yet")
    }

    fn resume(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        if self.window.is_none() {
            self.create_window(event_loop)?;
        }

        if self.instance.is_none() {
            self.create_instance()?;
        }

        if self.surface.is_none() {
            self.create_surface()?;
        }

        if self.adapter.is_none() || self.format.is_none() {
            self.request_adapter()?;
        }

        if self.device.is_none() || self.queue.is_none() {
            self.request_device()?;
        }

        if self.module.is_none() {
            self.create_module()?;
        }

        if self.pipeline.is_none() {
            self.create_pipeline()?;
        }

        if self.buffer.is_none() {
            self.create_buffer()?;
        }

        if self.bundle.is_none() {
            self.create_bundle()?;
        }

        Result::Ok(())
    }

    fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) -> Result<()> {
        let window = self.window()?;

        if window.id() != window_id {
            return Result::Ok(());
        }

        match event {
            WindowEvent::Resized(_) => self.configure_surface()?,
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.draw()?,
            _ => (),
        }

        Result::Ok(())
    }

    fn try_about_to_wait(&self) -> Result<()> {
        self.window()?.request_redraw();
        Result::Ok(())
    }

    fn suspend(&mut self) -> Result<()> {
        self.surface = Option::None;
        Result::Ok(())
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let window = event_loop.create_window(Window::default_attributes())?;
        self.window = Option::Some(Arc::new(window));
        self.surface = Option::None;
        Result::Ok(())
    }

    fn create_instance(&mut self) -> Result<()> {
        let descripter = InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::debugging(),
            backend_options: Default::default(),
        };

        let instance = Instance::new(&descripter);
        self.instance = Option::Some(instance);
        self.surface = Option::None;
        Result::Ok(())
    }

    fn create_surface(&mut self) -> Result<()> {
        let instance = self.instance()?;
        let window = self.clone_window()?;
        let surface = instance.create_surface(window)?;
        self.surface = Option::Some(surface);
        self.adapter = Option::None;
        self.format = Option::None;
        Result::Ok(())
    }

    fn request_adapter(&mut self) -> Result<()> {
        let instance = self.instance()?;
        let surface = self.surface()?;

        let options = RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: self.surface.as_ref(),
        };

        let adapter = block_on(instance.request_adapter(&options))?;
        let format = surface.get_capabilities(&adapter).formats.first().copied();
        self.adapter = Option::Some(adapter);
        self.format = format;
        self.device = Option::None;
        self.queue = Option::None;
        Result::Ok(())
    }

    fn request_device(&mut self) -> Result<()> {
        let adapter = self.adapter()?;

        let descripter = DeviceDescriptor {
            label: Option::None,
            required_features: Features::empty(),
            required_limits: Default::default(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        };

        let (device, queue) = block_on(adapter.request_device(&descripter))?;
        self.device = Option::Some(device);
        self.queue = Option::Some(queue);
        self.module = Option::None;
        self.buffer = Option::None;
        Result::Ok(())
    }

    fn create_module(&mut self) -> Result<()> {
        let device = self.device()?;
        let module_descriptor = include_wgsl!("shader.wgsl");
        let module = device.create_shader_module(module_descriptor);
        self.module = Option::Some(module);
        self.pipeline = Option::None;
        Result::Ok(())
    }

    fn create_pipeline(&mut self) -> Result<()> {
        let format = self.format()?;
        let device = self.device()?;
        let module = self.module()?;

        let fragment_targets = [Option::Some(ColorTargetState {
            format,
            blend: Option::Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::all(),
        })];

        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Option::None,
            layout: Option::None,
            vertex: VertexState {
                module,
                entry_point: Option::None,
                compilation_options: Default::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: VertexFormat::Float32x2.size(),
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x2,
                    ],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: Option::None,
                front_face: FrontFace::Ccw,
                cull_mode: Option::Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Option::None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Option::Some(FragmentState {
                module,
                entry_point: Option::None,
                compilation_options: Default::default(),
                targets: &fragment_targets,
            }),
            multiview: Option::None,
            cache: Option::None,
        };

        let pipeline = device.create_render_pipeline(&pipeline_descriptor);
        self.pipeline = Option::Some(pipeline);
        self.bundle = Option::None;
        Result::Ok(())
    }

    fn create_buffer(&mut self) -> Result<()> {
        let device = self.device()?;
        let data = Self::VERTICES;
        let vertices = unsafe { new_slice(data.as_ptr() as *const u8, size_of_val(&data)) };

        let descriptor = BufferInitDescriptor {
            label: Option::None,
            contents: vertices,
            usage: BufferUsages::VERTEX,
        };

        let buffer = device.create_buffer_init(&descriptor);
        self.buffer = Option::Some(buffer);
        self.bundle = Option::None;
        Result::Ok(())
    }

    fn create_bundle(&mut self) -> Result<()> {
        let format = self.format()?;
        let device = self.device()?;
        let pipeline = self.pipeline()?;
        let buffer = self.buffer()?;

        let descriptor = RenderBundleEncoderDescriptor {
            label: Option::None,
            color_formats: &[Option::Some(format)],
            depth_stencil: Option::None,
            sample_count: 1,
            multiview: Option::None,
        };

        let mut encoder = device.create_render_bundle_encoder(&descriptor);
        encoder.set_pipeline(pipeline);
        encoder.set_vertex_buffer(0, buffer.slice(..));
        encoder.draw(0..Self::VERTICES.len() as u32, 0..1);

        let descriptor = RenderBundleDescriptor {
            label: Option::None,
        };

        let bundle = encoder.finish(&descriptor);
        self.bundle = Option::Some(bundle);
        Result::Ok(())
    }

    fn configure_surface(&mut self) -> Result<()> {
        let window = self.window()?;
        let surface = self.surface()?;
        let format = self.format()?;
        let device = self.device()?;
        let size = window.inner_size();

        let configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
        };

        surface.configure(device, &configuration);
        let w = size.width as f32;
        let h = size.height as f32;
        let m = f32::min(w, h);
        let x = (w - m) / 2.;
        let y = (h - m) / 2.;
        self.viewport = Option::Some([x, y, m, m]);
        Result::Ok(())
    }

    fn draw(&self) -> Result<()> {
        let surface = self.surface()?;
        let device = self.device()?;
        let queue = self.queue()?;
        let bundle = self.bundle()?;
        let viewport = self.viewport()?;
        let texture = surface.get_current_texture()?;

        let descriptor = TextureViewDescriptor {
            label: Option::None,
            format: Option::None,
            dimension: Option::None,
            usage: Option::None,
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Option::None,
            base_array_layer: 0,
            array_layer_count: Option::None,
        };

        let view = texture.texture.create_view(&descriptor);

        let descriptor = CommandEncoderDescriptor {
            label: Option::None,
        };

        let mut encoder = device.create_command_encoder(&descriptor);

        let descriptor = RenderPassDescriptor {
            label: Option::None,
            color_attachments: &[Option::Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: Option::None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Option::None,
            timestamp_writes: Option::None,
            occlusion_query_set: Option::None,
        };

        let mut pass = encoder.begin_render_pass(&descriptor);
        pass.set_viewport(viewport[0], viewport[1], viewport[2], viewport[3], 0., 1.);
        pass.execute_bundles([bundle]);
        drop(pass);
        let commands = encoder.finish();
        queue.submit([commands]);
        texture.present();
        Result::Ok(())
    }

    fn handle_result(event_loop: &ActiveEventLoop, result: Result<()>) {
        if let Result::Err(error) = result {
            error!("{error}");
            event_loop.exit();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let result = self.resume(event_loop);
        Self::handle_result(event_loop, result);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        Self::handle_result(
            event_loop,
            self.handle_window_event(event_loop, window_id, event),
        );
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        Self::handle_result(event_loop, self.try_about_to_wait());
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        Self::handle_result(event_loop, self.suspend());
    }
}
