use std::path::PathBuf;

use clap::{Parser, Subcommand};

const DEFAULT_ANTS: usize = 190;
const DEFAULT_ALPHA: f64 = 0.6858;
const DEFAULT_BETA: f64 = 2.4499;
const DEFAULT_TAU0: f64 = 0.0;
const DEFAULT_Q0: f64 = 0.231443;
const DEFAULT_ITERATIONS: usize = 2000;

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub filepath: PathBuf,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Finetuning {},
    Run {
        #[arg(short, long, default_value_t = DEFAULT_ANTS)]
        ants: usize,
        #[arg(short, long, default_value_t = DEFAULT_ALPHA)]
        alpha: f64,
        #[arg(short, long, default_value_t = DEFAULT_BETA)]
        beta: f64,
        #[arg(short, long, default_value_t = DEFAULT_TAU0)]
        tau0: f64,
        #[arg(short, long, default_value_t = DEFAULT_Q0)]
        p_of_take_best_path: f64,
        #[arg(short, long, default_value_t = DEFAULT_ITERATIONS)]
        iterations: usize,
    },
}
