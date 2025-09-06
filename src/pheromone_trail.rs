use std::{collections::HashMap, mem::swap};

use crate::params::Parameters;

pub type Edge = (usize, usize);

#[derive(Debug)]
pub struct PheromoneTrails<'a> {
    params: &'a Parameters,
    levels: HashMap<Edge, f64>,
}

impl<'a> PheromoneTrails<'a> {
    pub fn new(params: &'a Parameters, capacity: usize) -> Self {
        Self {
            params,
            levels: HashMap::with_capacity(capacity),
        }
    }

    pub fn global_update(&mut self, path: &[usize], lenght: f64) {
        for edge in path.windows(2).map(|arr| (arr[0], arr[1])) {
            let mut p = *self.get_or_create(edge);
            p *= 1.0 - self.params.alpha;
            p += self.params.alpha * lenght.recip();
            self.set(edge, p);
        }
    }

    pub fn local_update(&mut self, current_coordinate: usize, next_coord: usize) {
        let mut p = *self.get_or_create((current_coordinate, next_coord));
        p *= 1.0 - self.params.alpha;
        p += self.params.alpha * self.params.tau0;
        self.set((current_coordinate, next_coord), p);
    }

    pub fn get_or_create(&mut self, mut edge: Edge) -> &mut f64 {
        if edge.0 > edge.1 {
            swap(&mut edge.0, &mut edge.1);
        }

        self.levels
            .entry(edge)
            .or_insert(self.params.initial_pheromone_level)
    }

    pub fn set(&mut self, mut edge: Edge, val: f64) {
        if edge.0 > edge.1 {
            swap(&mut edge.0, &mut edge.1);
        }

        self.levels.insert(edge, val);
    }
}
