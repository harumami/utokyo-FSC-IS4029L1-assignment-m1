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
    tracing::info,
};

#[derive(Debug, Parser)]
pub struct Arguments {
    pub input: Input,
    pub output: Output,
}

impl Arguments {
    pub fn parse() -> Result<Result<Self, Error>> {
        match Parser::try_parse() {
            Result::Ok(args) => {
                info!("{args:?}");
                Result::Ok(Result::Ok(args))
            },
            Result::Err(error) => match error.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    Result::Ok(Result::Err(error))
                },
                _ => Result::Err(error.into()),
            },
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Input {
    Json,
    Toml,
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Output {
    Png,
    WebP,
}
