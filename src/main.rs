use std::ops;

use mikes_ge::{ge::GE, grammer::Grammer};
use mikes_pso::{particle::Particle, pso::PSO, vector::Vector, VelocityFunction};

enum Velocity {
    Current,
    Best,
    Mul(Box<Self>, Box<ScalarOps>),
    Add(Box<Self>, Box<Self>),
}

enum ScalarOps {
    Cognitive,
    Social,
    InertiaWeight,
    Rand,
    Mul(Box<ScalarOps>, Box<ScalarOps>),
    Add(Box<ScalarOps>, Box<ScalarOps>),
}

impl Grammer for Velocity {
    type Input = ();
    type Output = f64;

    fn run(&self, _input: &Self::Input) -> Self::Output {
        let obj_func = |_coords: &Vector<2>| 7.0;
        let func = |current: &_, best: &_| self.runner(current, best);
        let mut pso = PSO::new(100, [(-10.0, 10.0), (-10.0, 10.0)], func);
        let particle = pso.optimize(100, obj_func);
        obj_func(&particle.coordinates())
    }

    fn generate(chromosome: &[u8]) -> Self {
        Self::generate_helper(&mut 0, chromosome)
    }
}

impl Velocity {
    fn runner(&self, current: &Particle<2>, best: &Particle<2>) -> Vector<2> {
        match self {
            Velocity::Current => current.coordinates(),
            Velocity::Best => best.coordinates(),
            Velocity::Mul(x, y) => x.runner(current, best) * y.runner(),
            Velocity::Add(x, y) => x.runner(current, best) + y.runner(current, best),
        }
    }

    fn generate_helper(pos: &mut usize, chromosome: &[u8]) -> Self {
        let p = *pos % chromosome.len();
        let modulos = if *pos / chromosome.len() > 3 { 2 } else { 4 };
        *pos += 1;
        match chromosome[p] % modulos {
            0 => Self::Current,
            1 => Self::Best,
            2 => Self::Mul(
                Box::new(Self::generate_helper(pos, chromosome)),
                Box::new(ScalarOps::generate_helper(pos, chromosome)),
            ),
            3 => Self::Add(
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
            ScalarOps::Rand => 0.5,
            ScalarOps::Mul(x, y) => x.runner() * y.runner(),
            ScalarOps::Add(x, y) => x.runner() + y.runner(),
        }
    }

    fn generate_helper(pos: &mut usize, chromosome: &[u8]) -> Self {
        let p = *pos % chromosome.len();
        let modulos = if *pos / chromosome.len() > 3 { 4 } else { 6 };
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
            _ => panic!("Cannot get here"),
        }
    }
}

fn main() {
    let mut ge = GE::<(), f64, Velocity>::new(100, (0.5, 0.5, 0.0), 3, 7, 100, 4, 1, &[((), 7.0)]);
    ge.start();
}
