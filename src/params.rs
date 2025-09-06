use std::ops::{Range, RangeInclusive};

use rand::Rng;

#[derive(Default, Debug, Clone)]
pub struct Parameters {
    pub ants: usize,
    pub initial_pheromone_level: f64,
    pub alpha: f64,
    pub beta: f64,
    pub tau0: f64,
    pub p_of_take_best_path: f64,
}

#[derive(Clone)]
pub struct ParametersRange {
    pub ants: RangeInclusive<usize>,
    pub initial_pheromone_level: RangeInclusive<f64>,
    pub alpha: RangeInclusive<f64>,
    pub beta: RangeInclusive<f64>,
    pub tau0: RangeInclusive<f64>,
    pub p_of_take_best_path: RangeInclusive<f64>,
}

impl ParametersRange {
    pub fn random(self, rng: &mut impl Rng) -> Parameters {
        Parameters {
            ants: rng.random_range(self.ants),
            initial_pheromone_level: rng.random_range(self.initial_pheromone_level),
            alpha: rng.random_range(self.alpha),
            beta: rng.random_range(self.beta),
            tau0: rng.random_range(self.tau0),
            p_of_take_best_path: rng.random_range(self.p_of_take_best_path),
        }
    }
}
