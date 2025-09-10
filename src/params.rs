use std::ops::RangeInclusive;

use rand::Rng;

#[derive(Default, Debug, Clone)]
pub struct Parameters {
    pub ants: usize,
    pub initial_pheromone_level: f64,
    pub alpha: f64,
    pub beta: f64,
    pub tau0: f64,
    pub p_of_take_best_path: f64,
    pub iterations: usize,
}

#[derive(Clone)]
pub struct ParametersRange {
    pub ants: RangeInclusive<usize>,
    pub initial_pheromone_level: RangeInclusive<f64>,
    pub alpha: RangeInclusive<f64>,
    pub beta: RangeInclusive<f64>,
    pub tau0: RangeInclusive<f64>,
    pub p_of_take_best_path: RangeInclusive<f64>,
    pub iterations: RangeInclusive<usize>,
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
            iterations: rng.random_range(self.iterations),
        }
    }

    pub fn clamp(&self, p: &mut Parameters) {
        // integers
        let (min_a, max_a) = (*self.ants.start(), *self.ants.end());
        if p.ants < min_a {
            p.ants = min_a;
        } else if p.ants > max_a {
            p.ants = max_a;
        }

        // floats helper
        fn clamp_f(x: &mut f64, r: &RangeInclusive<f64>) {
            let min = *r.start();
            let max = *r.end();
            if *x < min {
                *x = min;
            } else if *x > max {
                *x = max;
            }
        }

        clamp_f(
            &mut p.initial_pheromone_level,
            &self.initial_pheromone_level,
        );
        clamp_f(&mut p.alpha, &self.alpha);
        clamp_f(&mut p.beta, &self.beta);
        clamp_f(&mut p.tau0, &self.tau0);
        clamp_f(&mut p.p_of_take_best_path, &self.p_of_take_best_path);
    }

    pub fn spans(&self) -> (usize, f64, f64, f64, f64, f64) {
        let ants_span = self.ants.end() - self.ants.start();
        let fspan = |r: &RangeInclusive<f64>| r.end() - r.start();
        (
            ants_span,
            fspan(&self.initial_pheromone_level),
            fspan(&self.alpha),
            fspan(&self.beta),
            fspan(&self.tau0),
            fspan(&self.p_of_take_best_path),
        )
    }
}
