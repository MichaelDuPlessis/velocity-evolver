mod function;

use mikes_ge::{ge::GE, grammer::Grammer};
use mikes_pso::{bounds::Bound, particle::Particle, pso::pso, vector::Vector};
use rand::Rng;
use std::borrow::Borrow;
use std::fs::File;
use std::io::Write;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

#[derive(Debug)]
enum Velocity<'a, const SIZE: usize> {
    CurrentCoords,
    BestCoords,
    CurrentBestCoords,
    BestBestCoords,
    CurrentVelocity,
    BestVelocity,
    Mul(Box<Self>, Box<ScalarOps>),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    _Unused(PhantomData<&'a ()>),
}

#[derive(Debug)]
enum ScalarOps {
    Cognitive,
    Social,
    InertiaWeight,
    Rand,
    Mul(Box<ScalarOps>, Box<ScalarOps>),
    Add(Box<ScalarOps>, Box<ScalarOps>),
    Sub(Box<ScalarOps>, Box<ScalarOps>),
}

impl<'a, const SIZE: usize> Grammer for Velocity<'a, SIZE> {
    type Input = (&'a Box<dyn Fn(&Vector<SIZE>) -> f64>, &'a [Bound]);
    type Output = f64;

    fn run(&self, input: &Self::Input) -> Self::Output {
        let func = |current: &_, best: &_| self.runner(current, best);
        let particle = pso(100, 100, input.1, func, &input.0);
        (input.0)(&particle.coordinates())
    }

    fn generate(chromosome: &[u8]) -> Self {
        Self::generate_helper(&mut 0, chromosome)
    }
}

impl<'a, const SIZE: usize> Velocity<'a, SIZE> {
    fn runner(&self, current: &Particle<SIZE>, best: &Particle<SIZE>) -> Vector<SIZE> {
        match self {
            Velocity::CurrentCoords => current.coordinates(),
            Velocity::BestCoords => best.coordinates(),
            Velocity::CurrentBestCoords => current.best(),
            Velocity::BestBestCoords => best.best(),
            Velocity::CurrentVelocity => current.velocity(),
            Velocity::BestVelocity => best.velocity(),
            Velocity::Mul(x, y) => x.runner(current, best) * y.runner(),
            Velocity::Add(x, y) => x.runner(current, best) + y.runner(current, best),
            Velocity::Sub(x, y) => x.runner(current, best) - y.runner(current, best),
            Velocity::_Unused(_) => panic!("Cannot get here"),
        }
    }

    fn generate_helper(pos: &mut usize, chromosome: &[u8]) -> Self {
        let p = *pos % chromosome.len();
        let modulos = if *pos / chromosome.len() > 3 { 6 } else { 9 };
        *pos += 1;
        match chromosome[p] % modulos {
            0 => Self::CurrentCoords,
            1 => Self::BestCoords,
            2 => Self::CurrentBestCoords,
            3 => Self::BestBestCoords,
            4 => Self::CurrentVelocity,
            5 => Self::BestVelocity,
            6 => Self::Mul(
                Box::new(Self::generate_helper(pos, chromosome)),
                Box::new(ScalarOps::generate_helper(pos, chromosome)),
            ),
            7 => Self::Add(
                Box::new(Self::generate_helper(pos, chromosome)),
                Box::new(Self::generate_helper(pos, chromosome)),
            ),
            8 => Self::Sub(
                Box::new(Self::generate_helper(pos, chromosome)),
                Box::new(Self::generate_helper(pos, chromosome)),
            ),
            _ => panic!("Cannot get here"),
        }
    }
}

impl ScalarOps {
    fn runner(&self) -> f64 {
        match self {
            ScalarOps::Cognitive => 0.3,
            ScalarOps::Social => 0.3,
            ScalarOps::InertiaWeight => 0.5,
            ScalarOps::Rand => rand::thread_rng().gen(),
            ScalarOps::Mul(x, y) => x.runner() * y.runner(),
            ScalarOps::Add(x, y) => x.runner() + y.runner(),
            ScalarOps::Sub(x, y) => x.runner() - y.runner(),
        }
    }

    fn generate_helper(pos: &mut usize, chromosome: &[u8]) -> Self {
        let p = *pos % chromosome.len();
        let modulos = if *pos / chromosome.len() > 3 { 4 } else { 7 };
        *pos += 1;
        match chromosome[p] % modulos {
            0 => Self::Rand,
            1 => Self::Social,
            2 => Self::Cognitive,
            3 => Self::InertiaWeight,
            4 => Self::Mul(
                Box::new(Self::generate_helper(pos, chromosome)),
                Box::new(Self::generate_helper(pos, chromosome)),
            ),
            5 => Self::Add(
                Box::new(Self::generate_helper(pos, chromosome)),
                Box::new(Self::generate_helper(pos, chromosome)),
            ),
            6 => Self::Sub(
                Box::new(Self::generate_helper(pos, chromosome)),
                Box::new(Self::generate_helper(pos, chromosome)),
            ),
            _ => panic!("Cannot get here"),
        }
    }
}

fn run_all_functions() {
    const SIZE: usize = 30;
    let functions = function::functions::<SIZE>();

    let mut file = File::create(format!("./results/result_{}.csv", SIZE)).unwrap();
    file.write(b"avg_mse, mse, time(s)\n").unwrap();

    // unique solution
    println!("Starting Single Function Runs");
    for function in &functions {
        let res = run_function(&[function]);
        file.write(res.to_csv().as_bytes()).unwrap();
    }

    // general solution
    println!("Starting Multi Function Runs");
    let res = run_function(&functions);
    file.write(res.to_csv().as_bytes()).unwrap();

    let mut file = File::create(format!("./results/canonical_{}.csv", SIZE)).unwrap();
    file.write(b"avg_mse, mse, time(s)\n").unwrap();

    println!("Starting Canoncial PSO");
    println!("Starting Single Function Runs");
    for function in &functions {
        let res = run_canonical_pso(&[function]);
        file.write(res.to_csv().as_bytes()).unwrap();
    }

    // general solution
    println!("Starting Multi Function Runs");
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

fn run_canonical_pso<const SIZE: usize>(
    functions: &[impl Borrow<function::Function<SIZE>>],
) -> FunctionResult {
    let mut total_mse = 0.0;
    let mut best_mse = f64::MAX;
    let start = Instant::now();
    for r in 0..30 {
        println!("Run: {r}");

        // running the pso
        let mut mse = 0.0;
        for function in functions {
            let function = function.borrow();
            let particle = pso(
                100,
                100,
                &function.bounds,
                mikes_pso::canonical_velocity,
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

fn run_function<const SIZE: usize>(
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
        println!("Run: {r}");

        let mut ge = GE::<(&Box<dyn Fn(&Vector<SIZE>) -> f64>, &[Bound]), f64, Velocity<SIZE>>::new(
            100,
            (0.5, 0.5, 0.0),
            3,
            7,
            100,
            4,
            1,
            &train,
        );
        let chromosome = ge.start();

        // crearing the velocity equation
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
