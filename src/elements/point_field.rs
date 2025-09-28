use geo_types::{Point, Rect};
use noise::{NoiseFn, Perlin, Seedable};
use rand::prelude::*;
use std::fmt::{Debug, Formatter};

use crate::util::HaltonSequence;

pub trait PointField: Debug + Send + Sync + Iterator {}

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

pub struct PerlinPointField {
    seed: u32,
    coord_scale: f64,
    point_prob: f64,
    // density: f64,
    bounds: Option<Rect<f64>>,
    iter_count: usize,
    // iter_limit: usize,
    iter_perlin: Option<Perlin>,
    iter_rng: Option<SmallRng>,
}

pub struct PerlinPointFieldBuilder {
    field: Box<PerlinPointField>,
}

impl Default for PerlinPointField {
    fn default() -> PerlinPointField {
        PerlinPointField {
            seed: 0u32,
            coord_scale: 1.,
            point_prob: 0.5,
            // density: 0.1,
            bounds: None,
            iter_count: 0,
            // iter_limit: 1,
            iter_perlin: None,
            iter_rng: None,
        }
    }
}

impl Iterator for PerlinPointField {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bounds.is_none() || self.iter_rng.is_none() || self.iter_perlin.is_none()
        // || self.iter_count >= self.iter_limit
        {
            return None;
        }
        let min = self.bounds.unwrap().min();

        loop {
            let (x, y) = self.iter_rng.as_mut().unwrap().gen::<(f64, f64)>();
            let x = x * self.bounds.unwrap().width() + min.x;
            let y = y * self.bounds.unwrap().height() + min.y;
            let prob = self
                .iter_perlin
                .unwrap()
                .get([x * self.coord_scale, y * self.coord_scale]);
            self.iter_count += 1;
            if prob >= self.point_prob {
                return Some(Point::new(x, y));
            }
            /*else if self.iter_count > self.iter_limit {
                return None;
            }*/
        }
    }
}

impl PerlinPointFieldBuilder {
    pub fn new() -> Self {
        PerlinPointFieldBuilder {
            field: Box::new(PerlinPointField::default()),
        }
    }
    pub fn bounds(self, bounds: Rect) -> Self {
        let new = Self {
            field: Box::new(PerlinPointField {
                bounds: Some(bounds),
                ..*self.field
            }),
        };
        new
    }

    pub fn seed(self, seed: u32) -> Self {
        let new = Self {
            field: Box::new(PerlinPointField {
                seed,
                ..*self.field
            }),
        };
        new
    }

    pub fn coord_scale(self, coord_scale: f64) -> Self {
        let new = Self {
            field: Box::new(PerlinPointField {
                coord_scale,
                ..*self.field
            }),
        };
        new
    }

    pub fn point_prob(self, point_prob: f64) -> Self {
        let new = Self {
            field: Box::new(PerlinPointField {
                point_prob,
                ..*self.field
            }),
        };
        new
    }

    /*
    pub fn density(self, density: f64) -> Self {
        let new = Self {
            field: Box::new(PerlinPointField {
                density,
                ..*self.field
            }),
        };
        new
    }
    */

    pub fn build(self) -> PerlinPointField {
        PerlinPointField {
            iter_count: 0,
            /*
            iter_limit: if self.field.bounds.is_some() {
                (self.field.density
                    * self
                        .field
                        .bounds
                        .expect("Bounds are not configured")
                        .width()
                    * self
                        .field
                        .bounds
                        .expect("Bounds are not configured")
                        .height()
                        .ceil()) as usize
            } else {
                0
            },
            */
            iter_perlin: Some(Perlin::new().set_seed(self.field.seed)),
            iter_rng: Some(SmallRng::seed_from_u64(self.field.seed as u64)),
            ..*self.field
        }
    }
}

impl Debug for PerlinPointField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerlinPointField")
            .field("seed", &self.seed)
            .field("coord_scale", &self.coord_scale)
            .field("prob_scale", &self.point_prob)
            // .field("density", &self.density)
            // .field("#points", &self.iter_limit)
            .field("bounds", &self.bounds)
            .finish()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_perlin_field() {
        let pf = PerlinPointFieldBuilder::new()
            .seed(1)
            .coord_scale(100.)
            .point_prob(0.5)
            // .density(0.01)
            .bounds(Rect::new(
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 100., y: 100. },
            ))
            .build();

        println!("PF IS {:?}", &pf);
        println!("PF: {:?}", pf.take(100).collect::<Vec<Point>>());
    }

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
