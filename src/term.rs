use std::process::{
    ExitCode,
    Termination,
};

pub enum Term {
    Ok,
    Io,
    Eyre,
    Tracing,
    Clap,
    Input,
    Curve,
    Output,
}

impl Termination for Term {
    fn report(self) -> ExitCode {
        (self as u8).into()
    }
}
