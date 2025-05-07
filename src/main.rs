mod args;
mod curve;
mod input;
mod output;
mod status;

use {
    crate::{
        args::Arguments,
        curve::to_line_strip,
        input::Input,
        output::{
            generate_image,
            LineStrip,
        },
        status::StatusCode,
    },
    color_eyre::config::HookBuilder,
    eyre::{
        Context as _,
        Result,
    },
    std::{
        io::stderr,
        time::Instant,
    },
    tracing::{
        error,
        info,
    },
    tracing_subscriber::{
        fmt::Subscriber,
        util::SubscriberInitExt as _,
    },
};

fn main() -> StatusCode {
    let start = Instant::now();

    if let Result::Err(error) = HookBuilder::new()
        .capture_span_trace_by_default(true)
        .install()
    {
        eprintln!("{error:?}");
        return StatusCode::Eyre;
    }

    if let Result::Err(error) = Subscriber::builder()
        .with_writer(stderr)
        .finish()
        .try_init()
        .wrap_err("cannot init a subscriber")
    {
        eprintln!("{error:?}");
        return StatusCode::Tracing;
    }

    let args = match Arguments::parse() {
        Result::Ok(Result::Ok(args)) => args,
        Result::Ok(Result::Err(error)) => match error.print() {
            Result::Ok(()) => return StatusCode::Ok,
            Result::Err(error) => {
                error!("{error:?}");
                return StatusCode::Io;
            },
        },
        Result::Err(error) => {
            error!("{error:?}");
            return StatusCode::Clap;
        },
    };

    let input = match Input::deserialize(args.input) {
        Result::Ok(input) => input,
        Result::Err(error) => {
            error!("{error:?}");
            return StatusCode::Input;
        },
    };

    let line_strips = match input
        .curve
        .into_iter()
        .map(|curve| {
            Result::Ok(LineStrip {
                positions: to_line_strip(curve.shape)?,
                color: curve.color,
            })
        })
        .collect::<Result<_>>()
    {
        Result::Ok(line_strips) => line_strips,
        Result::Err(error) => {
            error!("{error:?}");
            return StatusCode::Curve;
        },
    };

    if let Result::Err(error) = generate_image(args.output, input.canvas, line_strips) {
        error!("{error:?}");
        return StatusCode::Output;
    }

    info!("{:?}", Instant::now().duration_since(start));
    StatusCode::Ok
}
