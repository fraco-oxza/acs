#[derive(Debug)]
pub struct Coordinate {
    x: f64,
    y: f64,
}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Coordinate) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}
