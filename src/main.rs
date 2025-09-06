#![warn(clippy::pedantic)]

use std::path::PathBuf;

use crate::{natural_selection::GeneticSelector, params::ParametersRange};

pub mod ant;
pub mod coordinates;
pub mod natural_selection;
pub mod params;
pub mod pheromone_trail;
pub mod tsp;

fn main() {
    let path: PathBuf = "./data/a280.tsp".into();
    let t = tsp::SymmetricTSP::from_file(&path).unwrap();

    let mut gs = GeneticSelector::new(
        ParametersRange {
            ants: 1..=30,
            initial_pheromone_level: 0.0..=1.0,
            alpha: 0.0..=1.0,
            beta: 0.0..=20.0,
            tau0: 0.0..=5.0,
            p_of_take_best_path: 0.0..=1.0,
        },
        100,
        200,
        t,
        2000,
    );

    gs.create_first_generation();
    loop {
        let scores = gs.evaluate_generation();
        gs.kill_dump(&scores);
        gs.sex();
    }
}
