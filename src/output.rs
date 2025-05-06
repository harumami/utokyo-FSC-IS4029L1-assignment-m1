use {
    crate::{
        args::Output as Kind,
        input::Input,
    },
    eyre::{
        bail,
        Context as _,
        OptionExt,
        Result,
    },
    futures::executor::block_on,
    image::{
        write_buffer_with_format as write_image,
        ColorType,
        ImageFormat,
    },
    std::{
        io::{
            stdout,
            Cursor,
            Write as _,
        },
        slice::from_raw_parts as new_slice,
        sync::mpsc::channel,
    },
    tracing::error,
    wgpu::{
        include_wgsl,
        util::{
            BufferInitDescriptor,
            DeviceExt as _,
        },
        vertex_attr_array,
        BackendOptions,
        Backends,
        BlendState,
        BufferDescriptor,
        BufferUsages,
        Color,
        ColorTargetState,
        ColorWrites,
        CommandEncoderDescriptor,
        DeviceDescriptor,
        Extent3d,
        Face,
        Features,
        FragmentState,
        FrontFace,
        Instance,
        InstanceDescriptor,
        InstanceFlags,
        LoadOp,
        MapMode,
        MemoryHints,
        MultisampleState,
        NoopBackendOptions,
        Operations,
        Origin3d,
        PollType,
        PolygonMode,
        PowerPreference,
        PrimitiveState,
        PrimitiveTopology,
        RenderPassColorAttachment,
        RenderPassDescriptor,
        RenderPipelineDescriptor,
        RequestAdapterOptions,
        StoreOp,
        TexelCopyBufferInfo,
        TexelCopyBufferLayout,
        TexelCopyTextureInfo,
        TextureAspect,
        TextureDescriptor,
        TextureDimension,
        TextureFormat,
        TextureUsages,
        TextureViewDescriptor,
        Trace,
        VertexBufferLayout,
        VertexFormat,
        VertexState,
        VertexStepMode,
        COPY_BYTES_PER_ROW_ALIGNMENT,
    },
};

pub fn generate(kind: Kind, input: Input) -> Result<()> {
    let instance = Instance::new(&InstanceDescriptor {
        backends: Backends::METAL | Backends::DX12,
        flags: match cfg!(debug_assertions) {
            true => InstanceFlags::debugging(),
            false => InstanceFlags::empty(),
        },
        backend_options: BackendOptions {
            gl: Default::default(),
            dx12: Default::default(),
            noop: NoopBackendOptions {
                enable: false,
            },
        },
    });

    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Option::None,
    }))?;

    let (device, queue) = block_on(adapter.request_device(&DeviceDescriptor {
        label: Option::None,
        required_features: Features::empty(),
        required_limits: Default::default(),
        memory_hints: MemoryHints::Performance,
        trace: Trace::Off,
    }))?;

    let module = device.create_shader_module(include_wgsl!("shader.wgsl"));
    let color_type = ColorType::Rgba8;

    let format = match color_type {
        ColorType::L8 => TextureFormat::R8Unorm,
        ColorType::La8 => TextureFormat::Rg8Unorm,
        ColorType::Rgba8 => TextureFormat::Rgba8Unorm,
        ColorType::L16 => TextureFormat::R16Unorm,
        ColorType::La16 => TextureFormat::Rg16Unorm,
        ColorType::Rgba16 => TextureFormat::Rgba16Unorm,
        _ => bail!("{:?} is not supported", color_type),
    };

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Option::None,
        layout: Option::None,
        vertex: VertexState {
            module: &module,
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
            module: &module,
            entry_point: Option::None,
            compilation_options: Default::default(),
            targets: &[Option::Some(ColorTargetState {
                format,
                blend: Option::Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::all(),
            })],
        }),
        multiview: Option::None,
        cache: Option::None,
    });

    let vertices: [[f32; 2]; 6] = [
        [0.5, 0.5],
        [-0.5, 0.5],
        [0.5, -0.5],
        [-0.5, -0.5],
        [0.5, -0.5],
        [-0.5, 0.5],
    ];

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Option::None,
        contents: unsafe { new_slice(vertices.as_ptr() as *const u8, size_of_val(&vertices)) },
        usage: BufferUsages::VERTEX,
    });

    let extent = Extent3d {
        width: input.canvas.width,
        height: input.canvas.height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: Option::None,
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let view = texture.create_view(&TextureViewDescriptor {
        label: Option::None,
        format: Option::None,
        dimension: Option::None,
        usage: Option::None,
        aspect: TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Option::None,
        base_array_layer: 0,
        array_layer_count: Option::None,
    });

    let block_size = format
        .block_copy_size(Option::None)
        .ok_or_eyre("cannot calculate a block copy size")?;

    let physical_size = extent.physical_size(format);
    let row_size = block_size * physical_size.width;
    let row_count = physical_size.height * physical_size.depth_or_array_layers;
    let row_alignment = COPY_BYTES_PER_ROW_ALIGNMENT;
    let aligned_row_size = row_size.div_ceil(row_alignment) * row_alignment;

    let texture_buffer = device.create_buffer(&BufferDescriptor {
        label: Option::None,
        size: (aligned_row_size * row_count) as _,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Option::None,
    });

    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Option::None,
        color_attachments: &[Option::Some(RenderPassColorAttachment {
            view: &view,
            resolve_target: Option::None,
            ops: Operations {
                load: LoadOp::Clear({
                    let max = u8::MAX as f64;
                    Color {
                        r: input.canvas.color.r as f64 / max,
                        g: input.canvas.color.g as f64 / max,
                        b: input.canvas.color.b as f64 / max,
                        a: 1.0,
                    }
                }),
                store: StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Option::None,
        timestamp_writes: Option::None,
        occlusion_query_set: Option::None,
    });

    pass.set_pipeline(&pipeline);
    pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    pass.draw(0..vertices.len() as u32, 0..1);
    drop(pass);

    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: &texture_buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Option::Some(aligned_row_size),
                rows_per_image: Option::None,
            },
        },
        extent,
    );

    queue.submit([encoder.finish()]);
    let texture_slice = texture_buffer.slice(..);
    let (sender, receiver) = channel();

    texture_slice.map_async(MapMode::Read, move |result| {
        if let Result::Err(error) = sender
            .send(result)
            .wrap_err("cannot send a result from a callback")
        {
            error!("{:?}", error);
        }
    });

    device.poll(PollType::Wait)?;
    receiver.recv()??;
    let mut image_data = Vec::with_capacity((row_size * row_count) as _);
    let texture_view = texture_slice.get_mapped_range();

    for i in 0..row_count {
        image_data.extend_from_slice(
            &texture_view[(i * aligned_row_size) as usize..][..row_size as usize],
        );
    }

    let mut image = Vec::new();

    write_image(
        &mut Cursor::new(&mut image),
        &image_data,
        extent.width,
        extent.height,
        color_type,
        match kind {
            Kind::Png => ImageFormat::Png,
            Kind::WebP => ImageFormat::WebP,
        },
    )?;

    stdout().lock().write_all(&image)?;
    Result::Ok(())
}
