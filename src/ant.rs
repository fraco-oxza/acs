use std::collections::HashSet;

use rand::{
    Rng, random_bool, rng,
    seq::{IndexedRandom, IteratorRandom},
};

use crate::{params::Parameters, pheromone_trail::PheromoneTrails, tsp::SymmetricTSP};

pub struct Ant<'a, 'b> {
    params: &'b Parameters,
    tsp: &'a SymmetricTSP,
    current_coordinate: usize,
    pub path_arr: Vec<usize>,
    not_visited: HashSet<usize>,
    pub path_lenght: f64,
}

impl<'a, 'b> Ant<'a, 'b> {
    pub fn with_random_start(tsp: &'a SymmetricTSP, params: &'b Parameters) -> Self {
        let mut rng = rng();
        let current_coordinate = rng.random_range(0..tsp.coordinates.len());

        let mut not_visited: HashSet<usize> = (0..tsp.coordinates.len()).collect();
        not_visited.remove(&current_coordinate);

        Self {
            tsp,
            current_coordinate,
            path_arr: vec![current_coordinate],
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

    fn choose_probabilistic(&self, pt: &mut PheromoneTrails) -> usize {
        self.get_all_path_scores(pt)
            .choose_weighted(&mut rng(), |(_, w)| *w)
            .map(|v| v.0)
            .inspect_err(|e| eprintln!("{e}"))
            .unwrap_or_else(|_| *self.not_visited.iter().choose(&mut rng()).unwrap())
    }

    fn choose_next_coord(&self, pt: &mut PheromoneTrails) -> Option<usize> {
        if random_bool(self.params.p_of_take_best_path) {
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
