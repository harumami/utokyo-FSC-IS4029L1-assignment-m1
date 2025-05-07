use std::process::{
    ExitCode,
    Termination,
};

pub enum StatusCode {
    Ok,
    Io,
    Eyre,
    Tracing,
    Clap,
    Input,
    Curve,
    Output,
}

impl Termination for StatusCode {
    fn report(self) -> ExitCode {
        (self as u8).into()
    }
}
