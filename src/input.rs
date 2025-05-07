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
    pub size: [u32; 2],
    pub color: u32,
}

#[derive(Debug, Deserialize)]
pub struct Curve {
    #[serde(flatten)]
    pub shape: Shape,
    pub color: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Shape {
    Lines {
        points: Vec<[f32; 2]>,
    },
    Bezier {
        points: Vec<[f32; 3]>,
        samples: usize,
        #[serde(flatten)]
        mode: BezierMode,
    },
    CatmullRom {
        points: Vec<[f32; 2]>,
        samples: usize,
        #[serde(flatten)]
        mode: CatmullRomMode,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "mode")]
pub enum BezierMode {
    Normal,
    DeCasteljau,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "mode")]
pub enum CatmullRomMode {
    Uniform,
    Chordal,
    Centripetal,
}
