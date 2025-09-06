#![warn(clippy::pedantic)]

use std::path::PathBuf;

use crate::{ant::Ant, params::Parameters, pheromone_trail::PheromoneTrails};

pub mod ant;
pub mod coordinates;
pub mod params;
pub mod pheromone_trail;
pub mod tsp;

fn main() {
    let path: PathBuf = "./data/a280.tsp".into();
    let t = tsp::SymmetricTSP::from_file(&path).unwrap();
    let parameters = Parameters {
        ants: 1000,
        initial_pheromone_level: 1e-10,
        alpha: 0.3,
        beta: 0.8,
        tau0: 0.8,
        p_of_take_best_path: 0.1,
    };
    let mut pt = PheromoneTrails::new(&parameters);

    loop {
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
        println!("{}", best_ant.path_lenght)
    }
}
