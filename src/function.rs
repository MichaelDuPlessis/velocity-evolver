use mikes_pso::{bounds::Bound, vector::Vector};
use rand::Rng;
use std::f64::consts::{E, PI};

pub struct Function {
    pub func: Box<dyn Fn(&Vector<2>) -> f64>,
    pub minima: f64,
    pub bounds: Vec<Bound>,
}

pub fn functions() -> [Function; 18] {
    [
        Function {
            func: Box::new(|coords: &Vector<2>| {
                0.26 * (coords[0] * coords[0] + coords[1] * coords[1])
                    - 0.48 * coords[0] * coords[1]
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-10.0, 10.0)), Bound::from((-10.0, 10.0))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                4.0 * coords[0] * coords[0] - 2.1 * coords[0] * coords[0] * coords[0] * coords[0]
                    + (coords[0] * coords[0] * coords[0] * coords[0] * coords[0] * coords[0]) / 3.0
                    + coords[0] * coords[1]
                    - 4.0 * coords[1] * coords[1]
                    + 4.0 * coords[1] * coords[1] * coords[1] * coords[1]
            }),
            minima: -1.0316,
            bounds: vec![Bound::from((-5.0, 5.0)), Bound::from((-5.0, 5.0))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords
                    .iter()
                    .enumerate()
                    .map(|(i, &x)| i as f64 * x * x)
                    .sum()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-10.0, 10.0)), Bound::from((-10.0, 10.0))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords.iter().map(|x| x.abs()).sum::<f64>()
                    + coords
                        .iter()
                        .map(|x| x.abs())
                        .reduce(|acc, e| acc * e)
                        .unwrap()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-10.0, 10.0)), Bound::from((-10.0, 10.0))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                let mut x = 0.0;
                for i in 0..2 {
                    for j in 0..i {
                        x += coords[j]
                    }
                }

                x
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-100.0, 100.0)), Bound::from((-100.0, 100.0))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords
                    .iter()
                    .map(|x| x.abs())
                    .max_by(|x, y| x.total_cmp(y))
                    .unwrap()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-100.0, 100.0)), Bound::from((-100.0, 100.0))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords
                    .iter()
                    .enumerate()
                    .map(|(i, x)| i as f64 * x * x)
                    .sum()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-10.0, 10.0)), Bound::from((-10.0, 10.0))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords
                    .iter()
                    .enumerate()
                    .map(|(i, x)| i as f64 * x * x * x * x)
                    .sum()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-1.28, 1.28)), Bound::from((-1.28, 1.28))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords
                    .iter()
                    .enumerate()
                    .map(|(i, x)| x.abs().powi(i as i32 + 1))
                    .sum()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-1.0, 1.0)), Bound::from((-1.0, 1.0))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords
                    .iter()
                    .enumerate()
                    .map(|(i, x)| (10.0_f64.powf(6.0)).powf((i as f64 - 1.0) / (2.0 - 1.0)) * x * x)
                    .sum()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-100.0, 100.0)), Bound::from((-100.0, 100.0))],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords.iter().map(|x| (x + 0.5).floor().powf(2.0)).sum()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-1.28, 1.28)); 2],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords
                    .iter()
                    .enumerate()
                    .map(|(i, x)| i as f64 * x * x * x * x)
                    .sum::<f64>()
                    + rand::thread_rng().gen_range(0.0..1.0)
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-1.28, 1.28)); 2],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords
                    .iter()
                    .map(|x| x * x - 10.0 * (2.0 * PI * x).cos() + 10.0)
                    .sum()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-5.12, 5.12)); 2],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                -20.0
                    * E.powf(-0.2 * coords.iter().map(|x| (x * x) / 2.0).sum::<f64>())
                        .sqrt()
                    - E.powf(coords.iter().map(|x| ((2.0 * PI * x) / 2.0).cos()).sum())
                    + 20.0
                    + E
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-32.0, 32.0)); 2],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                (coords.iter().map(|x| x * x).sum::<f64>()
                    - coords
                        .iter()
                        .enumerate()
                        .map(|(i, x)| (*x / (i as f64).sqrt()).cos())
                        .reduce(|acc, x| acc * x)
                        .unwrap()
                    + 1.0)
                    / 4000.0
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-600.0, 600.0)); 2],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                0.5 + (coords
                    .iter()
                    .map(|x| x * x)
                    .sum::<f64>()
                    .sqrt()
                    .sin()
                    .powf(2.0)
                    - 0.5)
                    / (1.0 + 0.001 * coords.iter().map(|x| x * x).sum::<f64>()).powf(2.0)
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-100.0, 100.0)); 2],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords
                    .iter()
                    .map(|x| x * x * x * x - 16.0 * x * x + 5.0 * x)
                    .sum::<f64>()
                    / 2.0
            }),
            minima: -78.3323,
            bounds: vec![Bound::from((-5.0, 5.0)); 2],
        },
        Function {
            func: Box::new(|coords: &Vector<2>| {
                coords.iter().map(|x| (x * x.sin() + 0.1 * x).abs()).sum()
            }),
            minima: 0.0,
            bounds: vec![Bound::from((-10.0, 10.0)); 2],
        },
    ]
}
