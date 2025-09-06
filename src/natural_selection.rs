use indicatif::ParallelProgressIterator;
use rand::{Rng, SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    ant::Ant,
    params::{Parameters, ParametersRange},
    pheromone_trail::PheromoneTrails,
    tsp::SymmetricTSP,
};

pub struct GeneticSelector {
    parameter_range: ParametersRange,
    generation: Vec<Parameters>,
    runs: usize,
    top_n: usize,
    tsp: SymmetricTSP,
    target_population: usize,
}

pub fn run_one(iterations: usize, parameters: &Parameters, t: &SymmetricTSP) -> Option<f64> {
    let mut pt = PheromoneTrails::new(parameters, t.coordinates.len().pow(2).div_ceil(2));
    let mut last_run = None;

    for _ in 0..iterations {
        let mut ants: Vec<Ant> = (0..parameters.ants)
            .map(|_| Ant::with_random_start(&t, parameters))
            .collect();

        for _ in 1..t.coordinates.len() {
            for ant in &mut ants {
                ant.move_ant(&mut pt).unwrap();
            }
        }

        let best_ant = ants
            .iter()
            .min_by(|a, b| a.path_lenght.total_cmp(&b.path_lenght))
            .unwrap();

        pt.global_update(&best_ant.path_arr, best_ant.path_lenght);

        last_run = Some(best_ant.path_lenght);
    }

    last_run
}

impl GeneticSelector {
    pub fn new(
        parameter_range: ParametersRange,
        runs: usize,
        top_n: usize,
        tsp: SymmetricTSP,
        target_population: usize,
    ) -> Self {
        Self {
            parameter_range,
            runs,
            generation: Vec::default(),
            top_n,
            tsp,
            target_population,
        }
    }

    pub fn create_first_generation(&mut self) {
        let mut rng = SmallRng::from_os_rng();
        self.generation = (0..self.target_population)
            .map(|_| self.parameter_range.clone().random(&mut rng))
            .collect();
    }

    pub fn evaluate_generation(&self) -> Vec<Option<f64>> {
        self.generation
            .par_iter()
            .progress()
            .map(|p| run_one(20, &p, &self.tsp))
            .collect()
    }

    fn get_random_one(&mut self, rng: &mut impl Rng) -> &Parameters {
        self.generation.choose(rng).expect("Generation is empty")
    }

    pub fn sex(&mut self) {
        let mut rng = SmallRng::from_os_rng();
        while self.generation.len() < self.target_population {
            if rng.random_bool(0.8) {
                let p1 = self.get_random_one(&mut rng).clone();
                let p2 = self.get_random_one(&mut rng).clone();

                let generator = ParametersRange {
                    ants: p1.ants.min(p2.ants)..=p1.ants.max(p2.ants),
                    initial_pheromone_level: p1
                        .initial_pheromone_level
                        .min(p2.initial_pheromone_level)
                        ..=p1.initial_pheromone_level.max(p2.initial_pheromone_level),
                    alpha: p1.alpha.min(p2.alpha)..=p1.alpha.max(p2.alpha),
                    beta: p1.beta.min(p2.beta)..=p1.beta.max(p2.beta),
                    tau0: p1.tau0.min(p2.tau0)..=p1.tau0.max(p2.tau0),
                    p_of_take_best_path: p1.p_of_take_best_path.min(p2.p_of_take_best_path)
                        ..=p1.p_of_take_best_path.max(p2.p_of_take_best_path),
                };

                self.generation.push(generator.random(&mut rng));
            } else {
                self.generation
                    .push(self.parameter_range.clone().random(&mut rng));
            }
        }
    }

    pub fn kill_dump(&mut self, scores: &[Option<f64>]) {
        let mut v: Vec<_> = self
            .generation
            .iter()
            .zip(scores.iter())
            .filter(|(_, s)| s.is_some())
            .map(|(a, b)| (a.clone(), b.unwrap()))
            .collect();

        v.sort_unstable_by(|a, b| a.1.total_cmp(&b.1));

        // for a in &v {
        //     println!("Score: {} with {:?}", a.1, a.0);
        // }
        let best = v.first().expect("First");
        println!("Best: {} with {:#?}", best.1, best.0);

        self.generation = v.into_iter().map(|(a, _)| a).take(self.top_n).collect();
    }
}
