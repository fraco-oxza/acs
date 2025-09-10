use std::{
    collections::{BTreeMap, HashSet},
    sync::{LazyLock, Mutex},
};

use rand::{
    Rng, SeedableRng,
    rngs::{self, SmallRng},
    seq::{IndexedRandom, IteratorRandom},
};

use crate::{params::Parameters, pheromone_trail::PheromoneTrails, tsp::SymmetricTSP};

pub type AntSet = HashSet<usize>;
static NOT_VISITED_CACHE: LazyLock<Mutex<BTreeMap<usize, AntSet>>> =
    LazyLock::new(|| Mutex::default());

fn get_set_of_not_visited(size: usize) -> AntSet {
    NOT_VISITED_CACHE
        .lock()
        .expect("failed to lock")
        .entry(size)
        .or_insert_with(|| (0..size).collect())
        .clone()
}

pub struct Ant<'a, 'b> {
    rng: rngs::SmallRng,
    params: &'b Parameters,
    tsp: &'a SymmetricTSP,
    current_coordinate: usize,
    pub path_arr: Vec<usize>,
    not_visited: HashSet<usize>,
    path_lenght: f64,
}

impl<'a, 'b> Ant<'a, 'b> {
    pub fn get_path_lenght(&self) -> f64 {
        let mut lenght = self.path_lenght;

        if let Some(start_idx) = self.path_arr.first() {
            lenght += self
                .tsp
                .distance_between(self.current_coordinate, *start_idx);
        }

        lenght
    }

    pub fn with_random_start(tsp: &'a SymmetricTSP, params: &'b Parameters) -> Self {
        let mut rng = SmallRng::from_os_rng();
        let current_coordinate = rng.random_range(0..tsp.coordinates.len());

        let mut not_visited = get_set_of_not_visited(tsp.coordinates.len());
        not_visited.remove(&current_coordinate);
        let mut path_arr = Vec::with_capacity(tsp.coordinates.len());
        path_arr.push(current_coordinate);

        Self {
            rng,
            tsp,
            current_coordinate,
            path_arr,
            not_visited,
            path_lenght: 0.0,
            params,
        }
    }

    fn get_step_score(&self, next_coord: usize, pt: &mut PheromoneTrails) -> f64 {
        self.tsp
            .distance_between(self.current_coordinate, next_coord)
            * pt.get_or_create((self.current_coordinate, next_coord))
                .powf(self.params.beta)
    }

    fn get_all_path_scores(&self, pt: &mut PheromoneTrails) -> Vec<(usize, f64)> {
        self.not_visited
            .iter()
            .map(|idx| (*idx, self.get_step_score(*idx, pt)))
            .collect()
    }

    fn choose_the_best(&self, pt: &mut PheromoneTrails) -> Option<usize> {
        self.get_all_path_scores(pt)
            .iter()
            .max_by(|(_, score1), (_, score2)| score1.total_cmp(score2))
            .map(|v| v.0)
    }

    fn choose_probabilistic(&mut self, pt: &mut PheromoneTrails) -> usize {
        self.get_all_path_scores(pt)
            .choose_weighted(&mut self.rng, |(_, w)| *w)
            .map(|v| v.0)
            .unwrap_or_else(|_| *self.not_visited.iter().choose(&mut self.rng).unwrap())
    }

    fn choose_next_coord(&mut self, pt: &mut PheromoneTrails) -> Option<usize> {
        if self.rng.random_bool(self.params.p_of_take_best_path) {
            self.choose_the_best(pt)
        } else {
            Some(self.choose_probabilistic(pt))
        }
    }

    pub fn move_ant(&mut self, pheromone_trail: &mut PheromoneTrails) -> Result<(), &'static str> {
        let next_coord = self
            .choose_next_coord(pheromone_trail)
            .ok_or("no more coords")?;
        pheromone_trail.local_update(self.current_coordinate, next_coord);

        self.not_visited.remove(&next_coord);
        self.path_lenght += self
            .tsp
            .distance_between(self.current_coordinate, next_coord);
        self.current_coordinate = next_coord;
        self.path_arr.push(self.current_coordinate);

        Ok(())
    }
}
