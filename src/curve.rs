use {
    crate::input::{
        BezierMode,
        CatmullRomMode,
        Shape,
    },
    eyre::{
        ensure,
        Result,
    },
    nalgebra::{
        Vector2,
        Vector3,
    },
    std::{
        array::from_fn as new_array,
        iter::once,
    },
};

pub fn to_line_strip(shape: Shape) -> Result<Vec<[f32; 2]>> {
    match shape {
        Shape::Lines {
            points,
        } => Result::Ok(points),
        Shape::Bezier {
            points,
            samples,
            mode,
        } => match mode {
            BezierMode::Normal => bezier::<NormalBezierFn>(points, samples),
            BezierMode::DeCasteljau => bezier::<DeCasteljauBezierFn>(points, samples),
        },
        Shape::CatmullRom {
            points,
            samples,
            mode,
        } => {
            ensure!(
                points.len() >= 4,
                "need at least four points to draw a catmull rom spline"
            );

            Result::Ok(
                points
                    .windows(4)
                    .flat_map(|ps| {
                        let ps = new_array::<_, 4, _>(|i| Vector2::new(ps[i][0], ps[i][1]));

                        let is = new_array::<_, 3, _>(|i| match mode {
                            CatmullRomMode::Uniform => 1.0,
                            CatmullRomMode::Chordal => (ps[i + 1] - ps[i]).norm(),
                            CatmullRomMode::Centripetal => {
                                (ps[i + 1] - ps[i]).norm_squared().powf(0.25)
                            },
                        });

                        let ts = new_array::<f32, 4, _>(|i| is[0..i].iter().sum());

                        (0..samples).map(move |i| {
                            let t = ts[1] + is[1] * (i as f32 / samples as f32);

                            let r#as = new_array::<_, 3, _>(|i| {
                                let r = (t - ts[i]) / is[i];
                                (1.0 - r) * ps[i] + r * ps[i + 1]
                            });

                            let bs = new_array::<_, 2, _>(|i| {
                                let r = (t - ts[i]) / (is[i] + is[i + 1]);
                                (1.0 - r) * r#as[i] + r * r#as[i + 1]
                            });

                            let cs = new_array::<_, 1, _>(|i| {
                                let r = (t - ts[i + 1]) / is[i + 1];
                                (1.0 - r) * bs[i] + r * bs[i + 1]
                            });

                            cs[0].into()
                        })
                    })
                    .chain(once(points[points.len() - 2]))
                    .collect(),
            )
        },
    }
}

fn bezier<F: BezierFn>(points: Vec<[f32; 3]>, samples: usize) -> Result<Vec<[f32; 2]>> {
    ensure!(
        !points.is_empty(),
        "need at least one point to draw a bezier curve"
    );

    let mut f = F::new(points.len() - 1);

    Result::Ok(
        (0..samples)
            .map(|i| {
                let p = f.call(
                    i as f32 / (samples - 1) as f32,
                    points
                        .iter()
                        .map(|point| point[2] * Vector3::new(point[0], point[1], 1.0)),
                );

                (p.xy() / p.z).into()
            })
            .collect(),
    )
}

trait BezierFn {
    fn new(n: usize) -> Self;
    fn call(&mut self, t: f32, ps: impl Iterator<Item = Vector3<f32>>) -> Vector3<f32>;
}

struct NormalBezierFn {
    n: usize,
    cs: Vec<f32>,
}

impl BezierFn for NormalBezierFn {
    fn new(n: usize) -> Self {
        let mut cs = Vec::with_capacity(n / 2 + 1);
        cs.push(1.0);

        for i in 1..cs.capacity() {
            cs.push(cs[i - 1] * (n + 1 - i) as f32 / i as f32);
        }

        Self {
            n,
            cs,
        }
    }

    fn call(&mut self, t: f32, ps: impl Iterator<Item = Vector3<f32>>) -> Vector3<f32> {
        ps.enumerate()
            .map(|(i, p)| {
                self.cs[usize::min(i, self.n - i)]
                    * t.powf(i as _)
                    * (1.0 - t).powf((self.n - i) as _)
                    * p
            })
            .sum()
    }
}

struct DeCasteljauBezierFn {
    ps: Vec<Vector3<f32>>,
}

impl BezierFn for DeCasteljauBezierFn {
    fn new(n: usize) -> Self {
        Self {
            ps: Vec::with_capacity(n + 1),
        }
    }

    fn call(&mut self, t: f32, ps: impl Iterator<Item = Vector3<f32>>) -> Vector3<f32> {
        self.ps.extend(ps);

        while self.ps.len() > 1 {
            for i in 0..self.ps.len() - 1 {
                self.ps[i] = t * self.ps[i] + (1.0 - t) * self.ps[i + 1];
            }

            self.ps.pop();
        }

        self.ps.pop().unwrap()
    }
}
