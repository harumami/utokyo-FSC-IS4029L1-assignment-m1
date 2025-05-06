use {
    crate::args::Input as Kind,
    eyre::Result,
    serde::Deserialize,
    serde_json::from_reader as json_from_reader,
    std::io::{
        stdin,
        BufReader,
        Read as _,
    },
    toml::from_str as toml_from_str,
    tracing::info,
};

#[derive(Debug, Deserialize)]
pub struct Input {
    pub canvas: Canvas,
    #[serde(default)]
    pub curve: Vec<Curve>,
}

impl Input {
    pub fn deserialize(kind: Kind) -> Result<Self> {
        let mut stdin = stdin().lock();

        let input = match kind {
            Kind::Json => json_from_reader(BufReader::new(stdin))?,
            Kind::Toml => {
                let mut string = String::new();
                stdin.read_to_string(&mut string)?;
                toml_from_str(&string)?
            },
        };

        info!("{input:?}");
        Result::Ok(input)
    }
}

#[derive(Debug, Deserialize)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub color: u32,
}

#[derive(Debug, Deserialize)]
pub struct Curve {
    #[serde(flatten)]
    pub param: Parameter,
    pub color: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum Parameter {
    Bezier {
        points: Vec<[f32; 2]>,
        samples: usize,
    },
}
