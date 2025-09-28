use geo_types::{Point, Rect};
use noise::{NoiseFn, Perlin, Seedable};
use rand::prelude::*;
use std::fmt::{Debug, Formatter};

use super::PointField;
use crate::util::HaltonSequence;

pub struct HaltonPointField {
    seed: usize,
    bounds: Option<Rect<f64>>,
    halton_x: HaltonSequence,
    halton_y: HaltonSequence,
}

impl Debug for HaltonPointField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HaltonPointField")
            .field("seed", &self.seed)
            .field("bounds", &self.bounds)
            .finish()
    }
}

impl Iterator for HaltonPointField {
    type Item = Point<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        let bounds = self
            .bounds
            .expect("Failed to get bounds. Improperly initialized HaltonPointField?");
        let x = self
            .halton_x
            .next()
            .expect("Failed to get X coord from Halton iterator.")
            * bounds.width()
            + bounds.min().x;
        let y = self
            .halton_y
            .next()
            .expect("Failed to get Y coord from Halton iterator.")
            * bounds.height()
            + bounds.min().y;
        Some(Point::new(x, y))
    }
}

impl Default for HaltonPointField {
    fn default() -> Self {
        Self {
            seed: 0,
            // num_points: 1,
            bounds: None,
            halton_x: HaltonSequence::with_base(2),
            halton_y: HaltonSequence::with_base(3),
        }
    }
}

pub struct HaltonPointFieldBuilder {
    hpf: HaltonPointField,
}

impl HaltonPointFieldBuilder {
    pub fn new() -> HaltonPointFieldBuilder {
        HaltonPointFieldBuilder {
            hpf: HaltonPointField::default(),
        }
    }

    /// Note that seeding the halton is O(n), so use small seeds.
    pub fn seed(self, seed: usize) -> Self {
        let halton_x = self.hpf.halton_x;
        let halton_x = halton_x.seed(seed);
        let halton_y = self.hpf.halton_y;
        let halton_y = halton_y.seed(seed);
        HaltonPointFieldBuilder {
            hpf: HaltonPointField {
                seed,
                halton_x,
                halton_y,
                ..self.hpf
            },
        }
    }

    pub fn bounds(self, bounds: Rect) -> Self {
        HaltonPointFieldBuilder {
            hpf: HaltonPointField {
                bounds: Some(bounds),
                ..self.hpf
            },
        }
    }

    // pub fn num_points(self, num_points: usize) -> Self {
    //     HaltonPointFieldBuilder {
    //         hpf: HaltonPointField {
    //             num_points,
    //             ..self.hpf
    //         },
    //     }
    // }

    pub fn build(self) -> HaltonPointField {
        self.hpf
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_halton_field() {
        let hf = HaltonPointFieldBuilder::new()
            .seed(1)
            .bounds(Rect::new(
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 100., y: 100. },
            ))
            .build();

        println!("PF IS {:?}", &hf);
        println!("PF: {:?}", hf.take(100).collect::<Vec<Point>>());
    }
}
