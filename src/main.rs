#![warn(clippy::pedantic)]

use std::path::PathBuf;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{ant::Ant, params::Parameters, pheromone_trail::PheromoneTrails, tsp::SymmetricTSP};

pub mod ant;
pub mod coordinates;
pub mod params;
pub mod pheromone_trail;
pub mod tsp;

fn run_one(iterations: usize, parameters: Parameters, t: &SymmetricTSP) -> Option<f64> {
    let mut pt = PheromoneTrails::new(&parameters);
    let mut last_run = None;

    for _ in 0..iterations {
        let mut ants: Vec<Ant> = (0..parameters.ants)
            .map(|_| Ant::with_random_start(&t, &parameters))
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

fn main() {
    let path: PathBuf = "./data/a280.tsp".into();
    let t = tsp::SymmetricTSP::from_file(&path).unwrap();

    let antsa = [10, 100, 1000];
    let iph = [1e-5, 1e-2, 1.0];
    let al = [0.1, 0.25, 0.5, 0.75];
    let be = [0.1, 0.25, 0.5, 0.75, 2.0, 4.0];
    let tau = [1e-2, 1e-1, 1e1, 1e2];
    let ptb = [0.1, 0.25, 0.5, 0.75, 0.9];

    let mut ps = Vec::new();

    for ants in antsa {
        for initial_pheromone_level in iph {
            for alpha in al {
                for beta in be {
                    for tau0 in tau {
                        for p_of_take_best_path in ptb {
                            let parameters = Parameters {
                                ants,
                                initial_pheromone_level,
                                alpha,
                                beta,
                                tau0,
                                p_of_take_best_path,
                            };

                            ps.push(parameters);
                        }
                    }
                }
            }
        }
    }

    let mut results: Vec<(Parameters, f64)> = ps
        .par_iter()
        .map(|p| {
            let result = run_one(100, p.clone(), &t);
            println!("{:?} => {:?}", p, result);
            (p.clone(), result)
        })
        .filter(|(_, r)| r.is_some())
        .map(|(a, b)| (a, b.unwrap()))
        .collect();

    results.sort_unstable_by(|a, b| a.1.total_cmp(&b.1));

    for (para, res) in results.iter().take(20) {
        println!("{:?} => {:?}", para, res);
    }
}
