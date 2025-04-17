use {
    color_eyre::install as install_eyre,
    eyre::{
        OptionExt as _,
        Result,
    },
    futures::executor::block_on,
    std::sync::Arc,
    tracing::error,
    tracing_subscriber::{
        fmt::Subscriber,
        util::SubscriberInitExt as _,
    },
    wgpu::{
        include_wgsl,
        Adapter,
        Backends,
        BlendState,
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
        RenderPassColorAttachment,
        RenderPassDescriptor,
        RenderPipelineDescriptor,
        RequestAdapterOptions,
        StoreOp,
        Surface,
        SurfaceConfiguration,
        TextureAspect,
        TextureUsages,
        TextureViewDescriptor,
        Trace,
        VertexState,
    },
    winit::{
        application::ApplicationHandler,
        event::{
            StartCause,
            WindowEvent,
        },
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
    instance: Option<Instance>,
    window: Option<Arc<Window>>,
    surface: Option<Surface<'static>>,
    adapter: Option<Adapter>,
    device: Option<Device>,
    queue: Option<Queue>,
}

impl App {
    fn run() -> Result<()> {
        install_eyre()?;
        Subscriber::new().try_init()?;
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = Self {
            instance: Option::None,
            window: Option::None,
            surface: Option::None,
            adapter: Option::None,
            device: Option::None,
            queue: Option::None,
        };

        event_loop.run_app(&mut app)?;
        Result::Ok(())
    }

    fn instance(&self) -> Result<&Instance> {
        self.instance
            .as_ref()
            .ok_or_eyre("an instance is not created yet")
    }

    fn window(&self) -> Result<&Window> {
        self.window
            .as_deref()
            .ok_or_eyre("a window is not created yet")
    }

    fn surface(&self) -> Result<&Surface<'static>> {
        self.surface
            .as_ref()
            .ok_or_eyre("a surface is not created yet")
    }

    fn adapter(&self) -> Result<&Adapter> {
        self.adapter
            .as_ref()
            .ok_or_eyre("an adapter is not created yet")
    }

    fn device(&self) -> Result<&Device> {
        self.device
            .as_ref()
            .ok_or_eyre("a device is not created yet")
    }

    fn queue(&self) -> Result<&Queue> {
        self.queue.as_ref().ok_or_eyre("a queue is not created yet")
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

        if self.surface.is_none() {
            self.create_surface()?;
        }

        if self.adapter.is_none() {
            self.request_adapter()?;
        }

        if self.device.is_none() || self.queue.is_none() {
            self.request_device()?;
        }

        Result::Ok(())
    }

    fn handle_window_event(
        &self,
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

    fn handle_new_events(&mut self, cause: StartCause) -> Result<()> {
        if cause == StartCause::Init {
            self.create_instance()?;
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

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let window = event_loop.create_window(Window::default_attributes())?;
        self.window = Option::Some(Arc::new(window));
        self.surface = Option::None;
        Result::Ok(())
    }

    fn create_surface(&mut self) -> Result<()> {
        let instance = self.instance()?;
        let window = self.clone_window()?;
        let surface = instance.create_surface(window)?;
        self.surface = Option::Some(surface);
        self.adapter = Option::None;
        Result::Ok(())
    }

    fn request_adapter(&mut self) -> Result<()> {
        let instance = self.instance()?;

        let options = RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: self.surface.as_ref(),
        };

        let adapter = block_on(instance.request_adapter(&options))?;
        self.adapter = Option::Some(adapter);
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
        Result::Ok(())
    }

    fn configure_surface(&self) -> Result<()> {
        let window = self.window()?;
        let adapter = self.adapter()?;
        let surface = self.surface()?;
        let device = self.device()?;

        let format = surface
            .get_capabilities(adapter)
            .formats
            .first()
            .copied()
            .ok_or_eyre("the surface does not support any format")?;

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
        Result::Ok(())
    }

    fn draw(&self) -> Result<()> {
        let surface = self.surface()?;
        let device = self.device()?;
        let queue = self.queue()?;
        let texture = surface.get_current_texture()?;

        let view_descriptor = TextureViewDescriptor {
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

        let view = texture.texture.create_view(&view_descriptor);

        let encoder_descriptor = CommandEncoderDescriptor {
            label: Option::None,
        };

        let mut encoder = device.create_command_encoder(&encoder_descriptor);

        let pass_descriptor = RenderPassDescriptor {
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

        let mut pass = encoder.begin_render_pass(&pass_descriptor);
        let module_descriptor = include_wgsl!("shader.wgsl");
        let module = device.create_shader_module(module_descriptor);

        let fragment_targets = [Option::Some(ColorTargetState {
            format: texture.texture.format(),
            blend: Option::Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::all(),
        })];

        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Option::None,
            layout: Option::None,
            vertex: VertexState {
                module: &module,
                entry_point: Option::None,
                compilation_options: Default::default(),
                buffers: &[],
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
                module: &module,
                entry_point: Option::None,
                compilation_options: Default::default(),
                targets: &fragment_targets,
            }),
            multiview: Option::None,
            cache: Option::None,
        };

        let pipeline = device.create_render_pipeline(&pipeline_descriptor);
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
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

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        Self::handle_result(event_loop, self.handle_new_events(cause));
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        Self::handle_result(event_loop, self.try_about_to_wait());
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        Self::handle_result(event_loop, self.suspend());
    }
}
