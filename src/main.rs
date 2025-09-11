#![warn(clippy::pedantic)]

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    ant::{Ant, AntPath},
    client::{Cli, Commands},
    natural_selection::GeneticSelector,
    params::{Parameters, ParametersRange},
    pheromone_trail::PheromoneTrails,
    tsp::SymmetricTSP,
};

pub mod ant;
pub mod client;
pub mod coordinates;
pub mod natural_selection;
pub mod params;
pub mod pheromone_trail;
pub mod tsp;

fn command_run(parameters: &Parameters, t: &SymmetricTSP, limit_without_improvement: usize) {
    let pt = PheromoneTrails::new(parameters, t.coordinates.len());
    let mut best_run_ant: Option<AntPath> = None;
    let bar = ProgressBar::new_spinner().with_style(
        ProgressStyle::with_template("[{elapsed}] {spinner} {msg}").expect("Bad template"),
    );
    let mut with_out_improvement = 0;

    for _ in 0..parameters.iterations {
        let mut ants: Vec<_> = (0..parameters.ants)
            .map(|_| Ant::with_random_start(t, parameters, &pt))
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

        if let Some(ref prev_best) = best_run_ant
            && prev_best.lenght > best_ant.get_path_lenght()
        {
            best_run_ant = Some(best_ant.into());
            with_out_improvement = 0;
        } else if best_run_ant.is_none() {
            best_run_ant = Some(best_ant.into());
        }

        if with_out_improvement > limit_without_improvement {
            break;
        }

        if let Some(ref ant) = best_run_ant {
            bar.set_message(format!("Best path {}", ant.lenght.round()));
        }

        bar.inc(1);
    }
    bar.finish_and_clear();

    println!("{best_run_ant:?}");
}

fn main() {
    let args = Cli::parse();
    let t = tsp::SymmetricTSP::from_file(&args.filepath).unwrap();

    match args.command {
        Commands::Run {
            ants,
            alpha,
            beta,
            tau0,
            p_of_take_best_path,
            iterations,
            limit_without_improvement,
        } => {
            command_run(
                &Parameters {
                    ants,
                    alpha,
                    beta,
                    tau0,
                    p_of_take_best_path,
                    iterations,
                },
                &t,
                limit_without_improvement,
            );
        }
        Commands::Finetuning => {
            let mut gs = GeneticSelector::new(
                ParametersRange {
                    ants: 1..=200,
                    alpha: 0.0..=(1.0 - 1e-1),
                    beta: 0.0..=20.0,
                    tau0: 0.0..=5.0,
                    p_of_take_best_path: 0.0..=1.0,
                    iterations: 100..=1000,
                },
                100,
                1000,
                t,
            );

            gs.create_first_generation();
            loop {
                let scores = gs.evaluate_generation();
                gs.kill_dump(&scores);
                gs.sex();
            }
        }
    }
}
