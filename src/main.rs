mod args;
mod curve;
mod input;
mod output;
mod term;

use {
    crate::{
        args::Args,
        curve::to_line_strip,
        input::Input,
        output::{
            generate,
            LineStrip,
        },
        term::Term,
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

fn main() -> Term {
    let start = Instant::now();

    if let Result::Err(error) = HookBuilder::new()
        .capture_span_trace_by_default(true)
        .install()
    {
        eprintln!("{error:?}");
        return Term::Eyre;
    }

    if let Result::Err(error) = Subscriber::builder()
        .with_writer(stderr)
        .finish()
        .try_init()
        .wrap_err("cannot init a subscriber")
    {
        eprintln!("{error:?}");
        return Term::Tracing;
    }

    let args = match Args::parse() {
        Result::Ok(Result::Ok(args)) => args,
        Result::Ok(Result::Err(error)) => match error.print() {
            Result::Ok(()) => return Term::Ok,
            Result::Err(error) => {
                error!("{error:?}");
                return Term::Io;
            },
        },
        Result::Err(error) => {
            error!("{error:?}");
            return Term::Clap;
        },
    };

    let input = match Input::deserialize(args.input) {
        Result::Ok(input) => input,
        Result::Err(error) => {
            error!("{error:?}");
            return Term::Input;
        },
    };

    let line_strips = match input
        .curve
        .into_iter()
        .map(|curve| {
            Result::Ok(LineStrip {
                positions: to_line_strip(curve.param)?,
                color: curve.color,
            })
        })
        .collect::<Result<_>>()
    {
        Result::Ok(line_strips) => line_strips,
        Result::Err(error) => {
            error!("{error:?}");
            return Term::Curve;
        },
    };

    if let Result::Err(error) = generate(args.output, input.canvas, line_strips) {
        error!("{error:?}");
        return Term::Output;
    }

    info!("{:?}", Instant::now().duration_since(start));
    Term::Ok
}
