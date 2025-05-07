use {
    crate::input::{
        BezierMode,
        Shape,
    },
    eyre::{
        ensure,
        Result,
    },
    nalgebra::Vector3,
};

pub fn to_line_strip(shape: Shape) -> Result<Vec<[f32; 2]>> {
    match shape {
        Shape::Bezier {
            points,
            samples,
            mode,
        } => match mode {
            BezierMode::Normal => bezier::<NormalBezierFn>(points, samples),
            BezierMode::DeCasteljau => bezier::<DeCasteljauBezierFn>(points, samples),
        },
    }
}

fn bezier<F: BezierFn>(points: Vec<[f32; 3]>, samples: usize) -> Result<Vec<[f32; 2]>> {
    ensure!(
        !points.is_empty(),
        "need at least one control point to draw a bezier curve"
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
