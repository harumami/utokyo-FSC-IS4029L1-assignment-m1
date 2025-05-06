mod args;
mod input;
mod output;
mod term;

use {
    crate::{
        args::Args,
        input::Input,
        term::Term,
    },
    color_eyre::config::HookBuilder,
    eyre::{
        Context as _,
        Result,
    },
    output::generate,
    std::io::stderr,
    tracing::error,
    tracing_subscriber::{
        fmt::Subscriber,
        util::SubscriberInitExt as _,
    },
};

fn main() -> Term {
    if let Result::Err(error) = HookBuilder::new()
        .capture_span_trace_by_default(true)
        .install()
    {
        eprintln!("{:?}", error);
        return Term::Eyre;
    }

    if let Result::Err(error) = Subscriber::builder()
        .with_writer(stderr)
        .finish()
        .try_init()
        .wrap_err("cannot init a subscriber")
    {
        eprintln!("{:?}", error);
        return Term::Tracing;
    }

    let args = match Args::parse() {
        Result::Ok(Result::Ok(args)) => args,
        Result::Ok(Result::Err(error)) => match error.print() {
            Result::Ok(()) => return Term::Ok,
            Result::Err(error) => {
                error!("{:?}", error);
                return Term::Io;
            },
        },
        Result::Err(error) => {
            error!("{:?}", error);
            return Term::Clap;
        },
    };

    let input = match Input::deserialize(args.input) {
        Result::Ok(input) => input,
        Result::Err(error) => {
            error!("{:?}", error);
            return Term::Input;
        },
    };

    if let Result::Err(error) = generate(args.output, input) {
        error!("{:?}", error);
        return Term::Output;
    }

    Term::Ok
}
