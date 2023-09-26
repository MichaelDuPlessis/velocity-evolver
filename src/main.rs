mod function;

use mikes_ge::{ge::GE, grammer::Grammer};
use mikes_pso::{bounds::Bound, particle::Particle, pso::pso, vector::Vector};
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

#[derive(Debug)]
enum Velocity<'a> {
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

impl<'a> Grammer for Velocity<'a> {
    type Input = (&'a Box<dyn Fn(&Vector<2>) -> f64>, &'a [Bound]);
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

impl<'a> Velocity<'a> {
    fn runner(&self, current: &Particle<2>, best: &Particle<2>) -> Vector<2> {
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
    let functions = function::functions();

    let mut file = File::create("./results/result.csv").unwrap();
    file.write(b"x, y, minima, time(s)\n").unwrap();

    // unique solution
    for function in functions.iter() {
        let res = run_function(function);
        file.write(res.to_csv().as_bytes()).unwrap();
    }

    // general solution
    let train = functions
        .iter()
        .map(|function| {
            (
                (
                    &function.func as &Box<dyn for<'a> Fn(&'a Vector<2>) -> f64>,
                    function.bounds.as_slice(),
                ),
                function.minima,
            )
        })
        .collect::<Vec<_>>();

    let start = Instant::now();
    let mut ge = GE::<(&Box<dyn Fn(&Vector<2>) -> f64>, &[Bound]), f64, Velocity>::new(
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
    let end = start.elapsed();

    // creating the velocity equation
    let velocity = Velocity::generate(&chromosome);
    // dbg!(&velocity);
    let func = |current: &_, best: &_| velocity.runner(current, best);

    // running the pso
    for function in functions {
        let particle = pso(100, 100, &function.bounds, func, &function.func);
        let coords = particle.coordinates();
        let mimma = (function.func)(&particle.coordinates());
        let _ = file.write(
            format!(
                "{}, {}, {}, {:.4}",
                coords[0],
                coords[1],
                mimma,
                end.as_secs_f64()
            )
            .as_bytes(),
        );
    }
}

struct FunctionResult {
    coords: Vector<2>,
    minima: f64,
    time: Duration,
}

impl FunctionResult {
    fn to_json(&self) -> String {
        format!(
            "{{
            \"coords\": {{
                \"x\": {},
                \"y\": {}
            }},
            \"minima\": {}
        }}",
            self.coords[0], self.coords[1], self.minima
        )
    }

    fn to_csv(&self) -> String {
        format!(
            "{}, {}, {}, {:.4}\n",
            self.coords[0],
            self.coords[1],
            self.minima,
            self.time.as_secs_f64()
        )
    }
}

fn run_function(function: &function::Function) -> FunctionResult {
    let train = &[(
        (
            &function.func as &Box<dyn for<'a> Fn(&'a Vector<2>) -> f64>,
            function.bounds.as_slice(),
        ),
        function.minima,
    )];
    let start = Instant::now();
    let mut ge = GE::<(&Box<dyn Fn(&Vector<2>) -> f64>, &[Bound]), f64, Velocity>::new(
        100,
        (0.5, 0.5, 0.0),
        3,
        7,
        100,
        4,
        1,
        train,
    );
    let chromosome = ge.start();
    let end = start.elapsed();

    // crearing the velocity equation
    let velocity = Velocity::generate(&chromosome);
    // dbg!(&velocity);
    let func = |current: &_, best: &_| velocity.runner(current, best);

    // running the pso
    let particle = pso(100, 100, &function.bounds, func, &function.func);
    // println!("Function: {}", (function.func)(&particle.coordinates()));

    FunctionResult {
        coords: particle.coordinates(),
        minima: (function.func)(&particle.coordinates()),
        time: end,
    }
}

fn main() {
    run_all_functions()
}
