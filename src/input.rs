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
};

#[derive(Deserialize)]
pub struct Input {
    pub canvas: Canvas,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub curves: Vec<Curve>,
}

impl Input {
    pub fn deserialize(kind: Kind) -> Result<Self> {
        let mut stdin = stdin().lock();

        Result::Ok(match kind {
            Kind::Json => json_from_reader(BufReader::new(stdin))?,
            Kind::Toml => {
                let mut string = String::new();
                stdin.read_to_string(&mut string)?;
                toml_from_str(&string)?
            },
        })
    }
}

#[derive(Deserialize)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub color: Color,
}

#[derive(Deserialize)]
pub struct Group {
    pub points: Vec<(f32, f32)>,
    pub color: Color,
}

#[derive(Deserialize)]
pub struct Curve {
    pub group: usize,
    pub color: Color,
}

#[derive(Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
