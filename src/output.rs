use {
    crate::{
        args::Output as Kind,
        input::Canvas,
    },
    eyre::{
        bail,
        ensure,
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
        array::from_fn as new_array,
        io::{
            stdout,
            Cursor,
            Write as _,
        },
        slice::from_raw_parts as new_slice,
        sync::mpsc::channel,
    },
    tracing::{
        error,
        info,
    },
    wgpu::{
        include_wgsl,
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
        VertexAttribute,
        VertexBufferLayout,
        VertexState,
        VertexStepMode,
        COPY_BYTES_PER_ROW_ALIGNMENT,
        VERTEX_STRIDE_ALIGNMENT,
    },
};

pub fn generate_image(kind: Kind, canvas: Canvas, line_strips: Vec<LineStrip>) -> Result<()> {
    ensure!(
        canvas.size.iter().all(|s| *s != 0),
        "{:?} is invalid as a size of an image",
        canvas.size
    );

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

    info!("{instance:?}");

    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Option::None,
    }))?;

    info!("{adapter:?}");

    let (device, queue) = block_on(adapter.request_device(&DeviceDescriptor {
        label: Option::None,
        required_features: Features::empty(),
        required_limits: Default::default(),
        memory_hints: MemoryHints::Performance,
        trace: Trace::Off,
    }))?;

    info!("{device:?}");
    info!("{queue:?}");
    let module = device.create_shader_module(include_wgsl!("shader.wgsl"));
    info!("{module:?}");
    let color_type = ColorType::Rgba8;

    let texture_format = match color_type {
        ColorType::L8 => TextureFormat::R8Unorm,
        ColorType::La8 => TextureFormat::Rg8Unorm,
        ColorType::Rgba8 => TextureFormat::Rgba8Unorm,
        ColorType::L16 => TextureFormat::R16Unorm,
        ColorType::La16 => TextureFormat::Rg16Unorm,
        ColorType::Rgba16 => TextureFormat::Rgba16Unorm,
        _ => bail!("{:?} is not supported", color_type),
    };

    let vertex_attributes = vertex_attr_array![
        0 => Float32x2,
        1 => Float32x3,
    ];

    let vertex_size = vertex_attributes
        .iter()
        .map(|attribute| attribute.format.size() + attribute.offset)
        .max()
        .ok_or_eyre("cannot get a size of a vertex")?
        .div_ceil(VERTEX_STRIDE_ALIGNMENT)
        * VERTEX_STRIDE_ALIGNMENT;

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Option::None,
        layout: Option::None,
        vertex: VertexState {
            module: &module,
            entry_point: Option::None,
            compilation_options: Default::default(),
            buffers: &[VertexBufferLayout {
                array_stride: vertex_size,
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attributes,
            }],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::LineList,
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
                format: texture_format,
                blend: Option::Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::all(),
            })],
        }),
        multiview: Option::None,
        cache: Option::None,
    });

    info!("{pipeline:?}");

    let vertex_count = line_strips
        .iter()
        .map(|line_strip| 2 * (line_strip.positions.len() - 1))
        .sum::<usize>();

    let vertex_buffer = device.create_buffer(&BufferDescriptor {
        label: Option::None,
        size: vertex_count as u64 * vertex_size,
        usage: BufferUsages::VERTEX,
        mapped_at_creation: true,
    });

    info!("{vertex_buffer:?}");
    let mut vertex_buffer_view = vertex_buffer.get_mapped_range_mut(..);
    info!("{vertex_buffer_view:?}");

    for (i, (position, color)) in line_strips
        .iter()
        .flat_map(|line_strip| {
            line_strip
                .positions
                .windows(2)
                .flatten()
                .map(|position| (position, line_strip.color))
        })
        .enumerate()
    {
        let vertex = &mut vertex_buffer_view[i * vertex_size as usize..][0..vertex_size as usize];

        write_attribute(
            vertex,
            &vertex_attributes[0],
            &new_array::<_, 2, _>(|i| 2.0 * position[i] / canvas.size[i] as f32 - 1.0),
        );

        write_attribute(vertex, &vertex_attributes[1], &to_rgb(color)?);
    }

    drop(vertex_buffer_view);
    vertex_buffer.unmap();

    let extent = Extent3d {
        width: canvas.size[0],
        height: canvas.size[1],
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: Option::None,
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: texture_format,
        usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    info!("{texture:?}");

    let texture_view = texture.create_view(&TextureViewDescriptor {
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

    info!("{texture_view:?}");

    let block_size = texture_format
        .block_copy_size(Option::None)
        .ok_or_eyre("cannot calculate a block copy size")?;

    let physical_size = extent.physical_size(texture_format);
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

    info!("{texture_buffer:?}");

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Option::None,
    });

    info!("{encoder:?}");

    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Option::None,
        color_attachments: &[Option::Some(RenderPassColorAttachment {
            view: &texture_view,
            resolve_target: Option::None,
            ops: Operations {
                load: LoadOp::Clear({
                    let rgb = to_rgb(canvas.color)?.map(|x| x as f64);

                    Color {
                        r: rgb[0],
                        g: rgb[1],
                        b: rgb[2],
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

    info!("{pass:?}");
    pass.set_pipeline(&pipeline);
    pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    pass.draw(0..vertex_count as u32, 0..1);
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
    let texture_buffer_slice = texture_buffer.slice(..);
    info!("{texture_buffer_slice:?}");
    let (sender, receiver) = channel();

    texture_buffer_slice.map_async(MapMode::Read, move |result| {
        info!("receive a result of map_async");

        if let Result::Err(error) = sender
            .send(result)
            .wrap_err("cannot send a result from a callback")
        {
            error!("{error:?}");
        }
    });

    device.poll(PollType::Wait)?;
    receiver.recv()??;
    let mut image_data = Vec::with_capacity((row_size * row_count) as _);
    let texture_buffer_view = texture_buffer_slice.get_mapped_range();
    info!("{texture_buffer_view:?}");

    for i in 0..row_count {
        image_data.extend_from_slice(
            &texture_buffer_view[(i * aligned_row_size) as usize..][..row_size as usize],
        );
    }

    drop(texture_buffer_view);
    texture_buffer.unmap();
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

pub struct LineStrip {
    pub positions: Vec<[f32; 2]>,
    pub color: u32,
}

fn write_attribute<T>(vertex: &mut [u8], attribute: &VertexAttribute, value: &T) {
    vertex[attribute.offset as usize..][..attribute.format.size() as usize]
        .copy_from_slice(unsafe { new_slice(value as *const _ as _, size_of::<T>()) });
}

fn to_rgb(raw: u32) -> Result<[f32; 3]> {
    let rgb = raw.to_be_bytes();
    ensure!(rgb[0] == 0, "{:X} is invalid as RGB", raw);
    let u8_max = u8::MAX as f32;

    Result::Ok([
        rgb[1] as f32 / u8_max,
        rgb[2] as f32 / u8_max,
        rgb[3] as f32 / u8_max,
    ])
}
