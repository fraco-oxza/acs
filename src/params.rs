#[derive(Default, Debug)]
pub struct Parameters {
    pub ants: usize,
    pub initial_pheromone_level: f64,
    pub alpha: f64,
    pub beta: f64,
    pub tau0: f64,
    pub p_of_take_best_path: f64,
}
