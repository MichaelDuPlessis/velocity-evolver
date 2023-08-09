use std::ops;

use mikes_ge::grammer::Grammer;
use mikes_pso::{particle::Particle, pso::PSO, vector::Vector, VelocityFunction};

enum Velocity {
    Mul(VectorOps, ScalarOps),
    Add(VectorOps, VectorOps),
}

enum VectorOps {
    Current,
    Best,
    Mul(Box<VectorOps>, ScalarOps),
    Add(Box<VectorOps>, Box<VectorOps>),
}

enum ScalarOps {
    Cognitive,
    Social,
    InertiaWeight,
    Rand,
    Mul(VectorOps, Box<ScalarOps>),
    MulS(Box<ScalarOps>, Box<ScalarOps>),
    Add(Box<ScalarOps>, Box<ScalarOps>),
}

impl Grammer for Velocity {
    type Input = ();
    type Output = f64;

    fn run(&self, _input: &Self::Input) -> Self::Output {
        let obj_func = |_coords: &Vector<2>| 7.0;
        let Some(func) = self.create_func() else {
            return f64::MAX;
        };
        let mut pso = PSO::new(100, [(-10.0, 10.0), (-10.0, 10.0)], func);
        let particle = pso.optimize(100, obj_func);
        obj_func(&particle.coordinates())
    }

    fn generate(chromosome: &[u8]) -> Self {
        Self::generate_helper(&mut 0, chromosome)
    }
}

impl Velocity {
    fn create_func<'a>(&'a self) -> Option<Box<dyn VelocityFunction<2> + 'a>> {
        if !self.check_evaluation() {
            None
        } else {
            let func = Box::new(|current: &_, best: &_| self.calc_vector(current, best));
            Some(func)
        }
    }

    fn calc_vector(&self, current: &Particle<2>, best: &Particle<2>) -> Vector<2> {
        match self {
            Velocity::Current => current.coordinates(),
            Velocity::Best => best.coordinates(),
            Velocity::Mul(x, y) => match (x.check_evaluation(), y.check_evaluation()) {
                (true, true) => x.calc_vector(current, best) * y.calc_vector(current, best),
                (true, false) => todo!(),
                (false, true) => todo!(),
                (false, false) => todo!(),
            },
            Velocity::Add(_, _) => todo!(),
            _ => panic!("Scalar in calc_vector"),
        }
    }

    fn calc_scalar(&self) -> f64 {
        todo!()
    }

    fn check_evaluation(&self) -> bool {
        match self {
            Velocity::Rand => false,
            Velocity::Current => true,
            Velocity::Best => true,
            Velocity::Cognitive => false,
            Velocity::Social => false,
            Velocity::InertiaWeight => false,
            Velocity::Mul(x, y) => x.check_evaluation() || y.check_evaluation(),
            Velocity::Add(x, y) => x.check_evaluation() && y.check_evaluation(),
        }
    }

    fn generate_helper(pos: &mut usize, chromosome: &[u8]) -> Self {
        if pos == 0 {
            *pos += 1;
            if chromosome[0] % 2 == 0 {
                Self::Mul(Self::generate_helper(pos, chromosome))
            }
        }
        let p = *pos % chromosome.len();
        let modulus = if *pos / chromosome.len() > 3 { 6 } else { 8 };
        *pos += 1;
        match chromosome[p] % modulus {
            0 => Self::Rand,
            1 => Self::Current,
            2 => Self::Best,
            3 => Self::Cognitive,
            4 => Self::Social,
            5 => Self::InertiaWeight,
            6 => Self::Mul(
                Box::new(Self::generate_helper(pos, chromosome)),
                Box::new(Self::generate_helper(pos, chromosome)),
            ),
            7 => Self::Add(
                Box::new(Self::generate_helper(pos, chromosome)),
                Box::new(Self::generate_helper(pos, chromosome)),
            ),
            _ => panic!("Cannot get here"),
        }
    }
}

fn main() {}
