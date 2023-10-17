mod function;
mod velocity;

use mikes_ge::ge::GE;
use mikes_ge::grammer::Grammer;
use mikes_pso::{bounds::Bound, particle::Particle, pso::pso, vector::Vector};
use std::borrow::Borrow;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use velocity::Velocity;

fn run_all_functions() {
    const SIZE: usize = 30;
    let functions = function::functions::<SIZE>();

    let mut file = File::create(format!("./results/result_{}.csv", SIZE)).unwrap();
    file.write(b"avg_mse, mse, time(s)\n").unwrap();

    // unique solution
    // println!("Starting Single Function Runs");
    for function in &functions {
        let res = run_functions(&[function]);
        file.write(res.to_csv().as_bytes()).unwrap();
    }

    // general solution
    // println!("Starting Multi Function Runs");
    let res = run_functions(&functions);
    file.write(res.to_csv().as_bytes()).unwrap();

    let mut file = File::create(format!("./results/canonical_{}.csv", SIZE)).unwrap();
    file.write(b"avg_mse, mse, time(s)\n").unwrap();

    // println!("Starting Canoncial PSO");
    // println!("Starting Single Function Runs");
    for function in &functions {
        let res = run_canonical_pso(&[function]);
        file.write(res.to_csv().as_bytes()).unwrap();
    }

    // general solution
    // println!("Starting Multi Function Runs");
    let res = run_canonical_pso(&functions);
    file.write(res.to_csv().as_bytes()).unwrap();
}

struct FunctionResult {
    avg_mse: f64,
    mse: f64,
    time: Duration,
}

impl FunctionResult {
    fn to_csv(&self) -> String {
        format!(
            "{}, {}, {:.4}\n",
            self.avg_mse,
            self.mse,
            self.time.as_secs_f64()
        )
    }
}

pub fn canonical_velocity<const DIMS: usize>(
    current: &Particle<DIMS>,
    best: &Particle<DIMS>,
) -> Vector<DIMS> {
    let c1 = 1.3;
    let c2 = 1.7;
    let w = 0.6;
    let (r1, r2): (f64, f64) = rand::random();

    let vel = w * current.velocity()
        + c1 * r1 * (best.coordinates() - current.coordinates())
        + c2 * r2 * (current.best() - current.coordinates());

    vel
}

fn run_canonical_pso<const SIZE: usize>(
    functions: &[impl Borrow<function::Function<SIZE>>],
) -> FunctionResult {
    let mut total_mse = 0.0;
    let mut best_mse = f64::MAX;
    let start = Instant::now();
    for r in 0..30 {
        // println!("Run: {r}");

        // running the pso
        let mut mse = 0.0;
        for function in functions {
            let function = function.borrow();
            let particle = pso(
                100,
                100,
                &function.bounds,
                canonical_velocity,
                &function.func,
            );
            let minima = (function.func)(&particle.coordinates());
            mse += (minima - function.minima) * (minima - function.minima);
        }

        best_mse = best_mse.min(mse);
        total_mse += mse;
    }
    let end = start.elapsed();

    FunctionResult {
        avg_mse: total_mse / 30.0,
        mse: best_mse,
        time: end,
    }
}

fn run_functions<const SIZE: usize>(
    functions: &[impl Borrow<function::Function<SIZE>>],
) -> FunctionResult {
    let train = functions
        .iter()
        .map(|function| {
            (
                (
                    &function.borrow().func as &Box<dyn for<'a> Fn(&'a Vector<SIZE>) -> f64>,
                    function.borrow().bounds.as_slice(),
                ),
                function.borrow().minima,
            )
        })
        .collect::<Vec<_>>();

    let mut total_mse = 0.0;
    let mut best_mse = f64::MAX;
    let start = Instant::now();
    for r in 0..30 {
        // println!("Run: {r}");

        let mut ge = GE::<(&Box<dyn Fn(&Vector<SIZE>) -> f64>, &[Bound]), f64, Velocity<SIZE>>::new(
            100,
            (0.7, 0.3, 0.0),
            3,
            10,
            100,
            5,
            1,
            &train,
        );
        let chromosome = ge.start();

        // creating the velocity equation
        let velocity = Velocity::generate(&chromosome);
        // dbg!(&velocity);
        let func = |current: &_, best: &_| velocity.runner(current, best);

        // running the pso
        let mut mse = 0.0;
        for function in functions {
            let function = function.borrow();
            let particle = pso(100, 100, &function.bounds, func, &function.func);
            let minima = (function.func)(&particle.coordinates());
            mse += (minima - function.minima) * (minima - function.minima);
        }

        best_mse = best_mse.min(mse);
        total_mse += mse;
    }
    let end = start.elapsed();

    FunctionResult {
        avg_mse: total_mse / 30.0,
        mse: best_mse,
        time: end,
    }
}

fn main() {
    run_all_functions()
}
