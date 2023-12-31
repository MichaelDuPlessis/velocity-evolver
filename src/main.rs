mod function;
mod velocity;

use function::Function;
use mikes_ge::ge::GE;
use mikes_ge::grammar::Grammar;
use mikes_pso::particle::Particle;
use mikes_pso::{bounds::Bound, pso::pso, vector::Vector};
use std::borrow::Borrow;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use threadpool::ThreadPool;
use velocity::Velocity;

fn run_all_functions() {
    const SIZE: usize = 100;
    let functions = function::functions::<SIZE>();
    //
    let results = Arc::new(Mutex::new(Vec::with_capacity(17)));
    let pool = ThreadPool::default();

    // unique solution
    println!("Starting Single Function Runs");
    for (i, function) in functions.into_iter().enumerate() {
        let results = Arc::clone(&results);
        pool.execute(move || {
            println!("Starting function: {i}");
            let res = run_functions(function);
            results.lock().unwrap().push((i, res));
            println!("Finished function: {i}");
        });
    }

    // general solution
    println!("Starting Multi Function Runs");

    pool.execute(|| {
        let mut file = File::create("./results_copy/reusable100.csv").unwrap();
        file.write(b"min, mean, std, time(s)\n").unwrap();

        let functions = function::functions::<SIZE>();
        let train = functions
            .iter()
            .map(|function| {
                (
                    (
                        &function.borrow().func
                            as &Box<dyn for<'a> Fn(&'a Vector<SIZE>) -> f64 + Send>,
                        function.borrow().bounds.as_slice(),
                    ),
                    function.borrow().minima,
                )
            })
            .collect::<Vec<_>>();

        let start = Instant::now();
        // println!("Run: {r}");

        let mut ge =
            GE::<(&Box<dyn Fn(&Vector<SIZE>) -> f64 + Send>, &[Bound]), f64, Velocity<SIZE>>::new(
                100,
                (0.7, 0.3, 0.0),
                3,
                10,
                100,
                5,
                30,
                &train,
            );
        let chromosome = ge.start();
        let end = start.elapsed();

        // creating the velocity equation
        let velocity = Velocity::generate(&chromosome);
        // dbg!(&velocity);
        let func = |current: &_, best: &_| velocity.runner(current, best);

        // running the pso
        for function in &functions {
            let mut results = Vec::with_capacity(30);
            for _ in 0..30 {
                let function = function.borrow();
                let particle = pso(100, 100, &function.bounds, func, &function.func);
                let minima = (function.func)(&particle.coordinates());
                results.push(minima)
            }
            let mut min = f64::MAX;
            for result in &results {
                if result < &min {
                    min = *result
                }
            }
            let mean = results.iter().sum::<f64>() / 30.0;
            let std = (results.iter().map(|r| (r - mean) * (r - mean)).sum::<f64>()
                / results.len() as f64)
                .sqrt();
            file.write(format!("{min}, {mean}, {std}, {:.4}\n", end.as_secs_f64()).as_bytes())
                .unwrap();
        }
    });

    pool.join();

    let mut file = File::create("./results_copy/disposable100.csv").unwrap();
    file.write(b"min, mean, std, time(s)\n").unwrap();
    results
        .lock()
        .unwrap()
        .sort_unstable_by(|(i, _), (j, _)| i.cmp(j));

    for (_, result) in results.lock().unwrap().iter() {
        file.write(result.to_csv().as_bytes()).unwrap();
    }

    // println!("Starting Canoncial PSO");
    // println!("Starting Single Function Runs");
    // for function in &functions {
    //     let res = run_canonical_pso(&[function]);
    //     file.write(res.to_csv().as_bytes()).unwrap();
    // }

    // general solution
    // println!("Starting Multi Function Runs");
    // let res = run_canonical_pso(&functions);
    // file.write(res.to_csv().as_bytes()).unwrap();
}

struct FunctionResult {
    min: f64,
    mean: f64,
    std: f64,
    time: Duration,
}

impl FunctionResult {
    fn to_csv(&self) -> String {
        format!(
            "{}, {}, {}, {:.4}\n",
            self.min,
            self.mean,
            self.std,
            self.time.as_secs_f64()
        )
    }
}

pub fn canonical_velocity<const DIMS: usize>(
    current: &Particle<DIMS>,
    best: &Particle<DIMS>,
) -> Vector<DIMS> {
    let c1 = 2.0;
    let c2 = 2.0;
    let w = 0.4;
    let (r1, r2): (f64, f64) = rand::random();

    let vel = w * current.velocity()
        + c1 * r1 * (best.coordinates() - current.coordinates())
        + c2 * r2 * (current.best() - current.coordinates());

    vel
}

fn run_canonical_pso<const SIZE: usize>(
    functions: &[impl Borrow<function::Function<SIZE>>],
) -> FunctionResult {
    let mut results = Vec::with_capacity(30);
    let start = Instant::now();
    for r in 0..30 {
        // println!("Run: {r}");

        // running the pso
        for function in functions {
            let function = function.borrow();
            let particle = pso(
                40,
                2500,
                &function.bounds,
                canonical_velocity,
                &function.func,
            );
            let minima = (function.func)(&particle.coordinates());
            results.push(minima)
        }
    }
    let end = start.elapsed();

    let mut min = f64::MAX;
    for result in &results {
        if result < &min {
            min = *result
        }
    }

    let mean = results.iter().sum::<f64>() / 30.0;
    let std = (results.iter().map(|r| (r - mean) * (r - mean)).sum::<f64>() / results.len() as f64)
        .sqrt();

    FunctionResult {
        min,
        mean,
        std,
        time: end,
    }
}

fn run_functions<const SIZE: usize>(function: function::Function<SIZE>) -> FunctionResult {
    let mut results = Vec::with_capacity(30);
    let start = Instant::now();
    for r in 0..30 {
        // println!("Run: {r}");
        let train = [(
            (
                &function.func as &Box<dyn for<'a> Fn(&'a Vector<SIZE>) -> f64 + Send>,
                function.bounds.as_slice(),
            ),
            function.minima,
        )];
        let mut ge =
            GE::<(&Box<dyn Fn(&Vector<SIZE>) -> f64 + Send>, &[Bound]), f64, Velocity<SIZE>>::new(
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
        // println!("{:?}", velocity);
        // dbg!(&velocity);
        let func = |current: &_, best: &_| velocity.runner(current, best);

        // running the pso
        let particle = pso(100, 100, &function.bounds, func, &function.func);
        let minima = (function.func)(&particle.coordinates());
        results.push(minima)
    }
    let end = start.elapsed();

    let mut min = f64::MAX;
    for result in &results {
        if result < &min {
            min = *result
        }
    }

    let mean = results.iter().sum::<f64>() / 30.0;
    let std = (results.iter().map(|r| (r - mean) * (r - mean)).sum::<f64>() / results.len() as f64)
        .sqrt();

    FunctionResult {
        min,
        mean,
        std,
        time: end,
    }
}

fn main() {
    run_all_functions()
    // let func = Function {
    //     func: Box::new(|coords: &Vector<30>| {
    //         0.26 * (coords[0] * coords[0] + coords[1] * coords[1]) - 0.48 * coords[0] * coords[1]
    //     }),
    //     minima: 0.0,
    //     bounds: vec![Bound::from((-10.0, 10.0)); 30],
    // };
    // let f = |current: &Particle<30>, best: &Particle<30>| {
    //     current.best() - (best.coordinates() + current.best())
    // };
    // let particle = pso(100, 100, &func.bounds, &f, &func.func);
    // println!("{particle:?}");
}
