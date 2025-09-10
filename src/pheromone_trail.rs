use std::{
    collections::HashMap,
    mem::swap,
    sync::{Arc, RwLock},
};

use crate::params::Parameters;

pub type Edge = (usize, usize);

#[derive(Debug)]
pub struct PheromoneTrails<'a> {
    params: &'a Parameters,
    levels: HashMap<Edge, Arc<RwLock<f64>>>,
}

impl<'a> PheromoneTrails<'a> {
    pub fn new(params: &'a Parameters, nodes: usize) -> Self {
        let mut levels = HashMap::with_capacity(nodes.pow(2).div_ceil(2));

        for i in 0..nodes {
            for j in (i + 1)..nodes {
                levels.insert(
                    (i, j),
                    Arc::new(RwLock::new(params.initial_pheromone_level)),
                );
            }
        }

        Self { params, levels }
    }

    pub fn global_update(&self, path: &[usize], lenght: f64) {
        for edge in path.windows(2).map(|arr| (arr[0], arr[1])) {
            let pa = self.get(edge);
            let mut p = pa.write().expect("Failed to get mut");

            *p *= 1.0 - self.params.alpha;
            *p += self.params.alpha * lenght.recip();
        }
    }

    pub fn local_update(&self, current_coordinate: usize, next_coord: usize) {
        let pa = self.get((current_coordinate, next_coord));
        let mut p = pa.write().expect("Failed to get mut");

        *p *= 1.0 - self.params.alpha;
        *p += self.params.alpha * self.params.tau0;
    }

    pub fn get(&self, mut edge: Edge) -> Arc<RwLock<f64>> {
        if edge.0 > edge.1 {
            swap(&mut edge.0, &mut edge.1);
        }
        Arc::clone(&self.levels.get(&edge).expect("Not registered"))
    }
}
