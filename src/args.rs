use {
    clap::{
        error::{
            Error,
            ErrorKind,
        },
        Parser,
        ValueEnum,
    },
    eyre::Result,
};

#[derive(Parser)]
pub struct Args {
    pub input: Input,
    pub output: Output,
}

impl Args {
    pub fn parse() -> Result<Result<Self, Error>> {
        match Parser::try_parse() {
            Result::Ok(args) => Result::Ok(Result::Ok(args)),
            Result::Err(error) => match error.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    Result::Ok(Result::Err(error))
                },
                _ => Result::Err(error.into()),
            },
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum Input {
    Json,
    Toml,
}

#[derive(Clone, ValueEnum)]
pub enum Output {
    Png,
    #[clap(name = "webp")]
    WebP,
}
