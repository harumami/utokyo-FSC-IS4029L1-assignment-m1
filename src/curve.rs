use {
    crate::input::Parameter,
    eyre::{
        ensure,
        Result,
    },
    nalgebra::Vector2,
};

pub fn to_line_strip(parameter: Parameter) -> Result<Vec<[f32; 2]>> {
    Result::Ok(match parameter {
        Parameter::Bezier {
            points,
            samples,
        } => {
            ensure!(
                !points.is_empty(),
                "need at least one control point to draw a bezier curve"
            );

            let mut cs = Vec::with_capacity((points.len() - 1) / 2 + 1);
            cs.push(1.0);

            for i in 1..cs.capacity() {
                cs.push(cs[i - 1] * (points.len() - i) as f32 / i as f32);
            }

            (0..samples)
                .map(|i| {
                    let t = i as f32 / (samples - 1) as f32;

                    let p = points
                        .iter()
                        .enumerate()
                        .map(|(j, point)| {
                            cs[usize::min(j, points.len() - j - 1)]
                                * t.powf(j as _)
                                * (1.0 - t).powf((points.len() - j - 1) as _)
                                * Vector2::new(point[0], point[1])
                        })
                        .sum::<Vector2<_>>();

                    [p[0], p[1]]
                })
                .collect()
        },
    })
}
