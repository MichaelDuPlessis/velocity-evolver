use mikes_ge::grammar::Grammar;
use mikes_pso::{bounds::Bound, particle::Particle, pso::pso, vector::Vector};
use rand::Rng;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum Velocity<'a, const SIZE: usize> {
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
pub enum ScalarOps {
    Cognitive,
    Social,
    InertiaWeight,
    Rand,
    Mul(Box<ScalarOps>, Box<ScalarOps>),
    Add(Box<ScalarOps>, Box<ScalarOps>),
    Sub(Box<ScalarOps>, Box<ScalarOps>),
}

impl<'a, const SIZE: usize> Grammar for Velocity<'a, SIZE> {
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
    pub fn runner(&self, current: &Particle<SIZE>, best: &Particle<SIZE>) -> Vector<SIZE> {
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
