use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rand::{
    Rng, SeedableRng,
    rngs::SmallRng,
    seq::{IndexedRandom, IteratorRandom},
};
use rayon::prelude::*;
use std::{collections::HashMap, time::Duration};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, RwLock},
};

use crate::{
    ant::Ant,
    params::{Parameters, ParametersRange},
    pheromone_trail::PheromoneTrails,
    tsp::SymmetricTSP,
};

pub struct GeneticSelector {
    parameter_range: ParametersRange,
    generation: Vec<Parameters>,
    top_n: usize,
    tsp: SymmetricTSP,
    target_population: usize,
    // GA hyperparameters
    tournament_k: usize,
    mutation_rate: f64,
    mutation_sigma: f64,
    blx_alpha: f64,
    // book-keeping
    generation_idx: usize,
    eval_cache: HashMap<u64, f64>,
}

pub fn run_one(parameters: &Parameters, t: &SymmetricTSP) -> Option<f64> {
    let pt = PheromoneTrails::new(parameters, t.coordinates.len());
    let mut last_run = None;

    for _ in 0..parameters.iterations {
        let mut ants: Vec<Ant> = (0..parameters.ants)
            .map(|_| Ant::with_random_start(&t, parameters, &pt))
            .collect();

        for _ in 1..t.coordinates.len() {
            ants.par_iter_mut()
                .for_each(|ant| ant.move_ant().expect("Error moving ant"));
        }

        let best_ant = ants
            .iter()
            .min_by(|a, b| a.get_path_lenght().total_cmp(&b.get_path_lenght()))
            .unwrap();

        pt.global_update(&best_ant.path_arr, best_ant.get_path_lenght());

        last_run = Some(best_ant.get_path_lenght());
        println!("{last_run:?}");
    }

    last_run
}

impl GeneticSelector {
    pub fn new(
        parameter_range: ParametersRange,
        top_n: usize,
        target_population: usize,
        tsp: SymmetricTSP,
    ) -> Self {
        Self {
            parameter_range,
            generation: Vec::default(),
            top_n,
            tsp,
            target_population,
            tournament_k: 3,
            mutation_rate: 0.25,
            mutation_sigma: 0.1,
            blx_alpha: 0.3,
            generation_idx: 0,
            eval_cache: HashMap::with_capacity(target_population * 2),
        }
    }

    pub fn create_first_generation(&mut self) {
        let mut rng = SmallRng::from_os_rng();
        self.generation = (0..self.target_population)
            .map(|_| self.parameter_range.clone().random(&mut rng))
            .collect();
    }

    pub fn evaluate_generation(&mut self) -> Vec<Option<f64>> {
        let style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:80.cyan/blue} {pos:>7}/{len:7} ETA: {eta_precise}",
        )
        .expect("invalid progress bar template");
        let bar = ProgressBar::new(self.generation.len() as u64).with_style(style);
        bar.enable_steady_tick(Duration::from_secs(1));

        self.generation
            .par_iter()
            .map(|p| {
                let key = Self::hash_params(p);
                if let Some(v) = self.eval_cache.get(&key) {
                    return Some(*v);
                }
                let sc = run_one(p, &self.tsp);
                // caching skipped inside parallel loop to avoid locks
                sc
            })
            .progress_with(bar)
            .collect()
    }

    fn get_random_one(&self, rng: &mut impl Rng) -> &Parameters {
        self.generation.choose(rng).expect("Generation is empty")
    }

    pub fn sex(&mut self) {
        // Elitism already handled in kill_dump by truncating; here we refill the population.
        let needed = self.target_population.saturating_sub(self.generation.len());
        if needed > 0 {
            let children: Vec<Parameters> = (0..needed)
                .into_par_iter()
                .map(|_| {
                    let mut rng = SmallRng::from_os_rng();
                    if rng.random_bool(0.85) {
                        let p1 = self.tournament_select(&mut rng).clone();
                        let p2 = self.tournament_select(&mut rng).clone();
                        let mut c = self.blx_crossover(&p1, &p2, self.blx_alpha, &mut rng);
                        self.mutate(&mut c, &mut rng);
                        c
                    } else {
                        self.parameter_range.clone().random(&mut rng)
                    }
                })
                .collect();
            self.generation.extend(children);
        }
        self.generation_idx += 1;
    }

    pub fn kill_dump(&mut self, scores: &[Option<f64>]) {
        let mut v: Vec<_> = self
            .generation
            .iter()
            .zip(scores.iter())
            .filter(|(_, s)| s.is_some())
            .map(|(a, b)| (a.clone(), b.unwrap()))
            .collect();

        v.par_sort_unstable_by(|a, b| a.1.total_cmp(&b.1));

        if let Some((best_p, best_s)) = v.first().map(|(p, s)| (p.clone(), *s)) {
            println!(
                "Gen {:04} | Best length: {:.5} | params: {:#?}",
                self.generation_idx, best_s, best_p
            );
        }

        // Elitism: keep top_n and refresh cache for elites
        self.eval_cache.clear();
        self.generation = v
            .into_iter()
            .map(|(a, s)| {
                let key = Self::hash_params(&a);
                self.eval_cache.insert(key, s);
                a
            })
            .take(self.top_n)
            .collect();
    }

    // ---- GA operators ----
    fn tournament_select(&self, rng: &mut impl Rng) -> &Parameters {
        // Uniform random K candidates, pick best score among current elites (front of vector)
        // If we haven't evaluated, fallback to random
        if self.generation.is_empty() {
            return self.generation.choose(rng).expect("empty generation");
        }
        let k = self.tournament_k.max(1);
        let candidates = self
            .generation
            .iter()
            .take(self.top_n.min(self.generation.len()))
            .choose_multiple(rng, k);
        let best = candidates
            .iter()
            .min_by(|a, b| {
                let ka = Self::hash_params(a);
                let kb = Self::hash_params(b);
                let ca = self.eval_cache.get(&ka).copied().unwrap_or(f64::INFINITY);
                let cb = self.eval_cache.get(&kb).copied().unwrap_or(f64::INFINITY);
                ca.total_cmp(&cb)
            })
            .copied();
        best.unwrap_or_else(|| self.get_random_one(rng))
    }

    fn blx_crossover(
        &self,
        p1: &Parameters,
        p2: &Parameters,
        alpha: f64,
        rng: &mut impl Rng,
    ) -> Parameters {
        fn lerp_f(rng: &mut impl Rng, a: f64, b: f64, alpha: f64) -> f64 {
            let (min, max) = (a.min(b), a.max(b));
            let d = max - min;
            let low = min - alpha * d;
            let high = max + alpha * d;
            rng.random_range(low..=high)
        }
        let ants = {
            let a = p1.ants.min(p2.ants);
            let b = p1.ants.max(p2.ants);
            let span = (b - a).max(1) as i64;
            let low = a as i64 - ((alpha * span as f64).round() as i64);
            let high = b as i64 + ((alpha * span as f64).round() as i64);
            rng.random_range(low..=high).max(0) as usize
        };
        let mut child = Parameters {
            ants,
            initial_pheromone_level: lerp_f(
                rng,
                p1.initial_pheromone_level,
                p2.initial_pheromone_level,
                alpha,
            ),
            alpha: lerp_f(rng, p1.alpha, p2.alpha, alpha),
            beta: lerp_f(rng, p1.beta, p2.beta, alpha),
            tau0: lerp_f(rng, p1.tau0, p2.tau0, alpha),
            p_of_take_best_path: lerp_f(rng, p1.p_of_take_best_path, p2.p_of_take_best_path, alpha),
            iterations: lerp_f(rng, p1.iterations as f64, p2.iterations as f64, alpha) as usize,
        };
        self.parameter_range.clamp(&mut child);
        child
    }

    fn mutate(&self, p: &mut Parameters, rng: &mut impl Rng) {
        let rate = self.mutation_rate;
        if rng.random_bool(rate) {
            // integer mutation: +/- step scaled by span
            let (ants_span, _, _, _, _, _) = self.parameter_range.spans();
            let step = ((ants_span as f64).sqrt().max(1.0)).round() as i64;
            let delta = rng.random_range(-step..=step);
            let new_val = p.ants as i64 + delta;
            p.ants = new_val.max(*self.parameter_range.ants.start() as i64) as usize;
        }
        let mut n = |x: &mut f64, span: f64| {
            if rng.random_bool(rate) {
                let sigma = (self.mutation_sigma * span).max(1e-9);
                // Box-Muller approx via rand normal not available; use small uniform jitter as fallback
                let jitter = rng.random_range(-sigma..=sigma);
                *x += jitter;
            }
        };
        let (_, ip_span, a_span, b_span, t_span, p_span) = self.parameter_range.spans();
        n(&mut p.initial_pheromone_level, ip_span);
        n(&mut p.alpha, a_span);
        n(&mut p.beta, b_span);
        n(&mut p.tau0, t_span);
        n(&mut p.p_of_take_best_path, p_span);
        self.parameter_range.clamp(p);
    }

    fn hash_params(p: &Parameters) -> u64 {
        let mut h = DefaultHasher::new();
        // Quantize floats to bits (stable) and hash fields
        p.ants.hash(&mut h);
        p.initial_pheromone_level.to_bits().hash(&mut h);
        p.alpha.to_bits().hash(&mut h);
        p.beta.to_bits().hash(&mut h);
        p.tau0.to_bits().hash(&mut h);
        p.p_of_take_best_path.to_bits().hash(&mut h);
        h.finish()
    }
}
