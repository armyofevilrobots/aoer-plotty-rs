pub mod halton;
use geo::{Point, Rect};
pub use halton::*;
use rand::prelude::*;
pub mod perlin;
pub use perlin::*;
use std::fmt::Debug;
use voronoice::Voronoi;

pub trait PointField: Debug + Send + Sync + Iterator {
    fn bounds(&self) -> geo::Rect;
}

pub trait FieldToVoronoi: PointField {
    fn to_voronoi(&mut self, point_count: usize) -> Voronoi;
}

#[derive(Debug)]
pub struct RandomPointField {
    bounds: Rect<f64>,
    rng: SmallRng,
}

pub struct RandomPointFieldBuilder {
    bounds: Option<Rect<f64>>,
    seed: u64,
}

impl PointField for RandomPointField {
    fn bounds(&self) -> geo::Rect {
        self.bounds.clone()
    }
}

impl RandomPointFieldBuilder {
    pub fn new() -> RandomPointFieldBuilder {
        RandomPointFieldBuilder {
            bounds: None,
            seed: 0,
        }
    }

    pub fn seed(self, seed: u64) -> RandomPointFieldBuilder {
        RandomPointFieldBuilder { seed: seed, ..self }
    }

    pub fn build(self) -> RandomPointField {
        RandomPointField {
            bounds: self.bounds.expect("Didn't configure bounds"),
            rng: SmallRng::seed_from_u64(self.seed),
        }
    }

    pub fn bounds(self, rect: Rect) -> RandomPointFieldBuilder {
        RandomPointFieldBuilder {
            bounds: Some(rect),
            ..self
        }
    }
}

impl Iterator for RandomPointField {
    type Item = Point<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Point::new(
            self.rng.gen::<f64>() * self.bounds.width() + self.bounds.min().x,
            self.rng.gen::<f64>() * self.bounds.height() + self.bounds.min().y,
        ))
    }
}
