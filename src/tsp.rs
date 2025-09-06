use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    rc::Rc,
    sync::{Arc, LazyLock},
};

use regex::Regex;

use crate::coordinates::Coordinate;

static SPACES_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" +").expect("invalid regex"));

#[derive(Debug, Clone)]
pub struct SymmetricTSP {
    // name: String,
    // comment: String,
    // stype: String,
    // dimension: u32,
    // edge_weight_type: String,
    pub coordinates: Arc<[Coordinate]>,
}

impl SymmetricTSP {
    pub fn from_file(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);
        let mut coordinates = Vec::default();

        // TODO: Parse other parameters
        for line in buf_reader.lines().skip(6) {
            let line = line?;
            if line.trim() == "EOF" {
                break;
            }

            let mut splited = SPACES_REGEX.split(line.trim()).skip(1);
            coordinates.push(Coordinate::new(
                splited.next().unwrap().trim().parse().unwrap(),
                splited.next().unwrap().trim().parse().unwrap(),
            ));
        }

        Ok(Self {
            coordinates: coordinates.into(),
        })
    }

    pub fn distance_between(&self, c1_idx: usize, c2_idx: usize) -> f64 {
        self.coordinates[c1_idx].distance_to(&self.coordinates[c2_idx])
    }
}

#[cfg(test)]
mod tests {
    use crate::tsp::Coordinate;

    const EPSILON: f64 = 1e-30;

    #[test]
    fn coordinate_distance() {
        let p1 = Coordinate::new(-15.0, 23.0);
        let p2 = Coordinate::new(25.0, -7.0);

        assert!((p1.distance_to(&p2) - 50.0).abs() < EPSILON);
    }
}
