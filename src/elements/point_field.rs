use crate::geo_types::boolean::BooleanOp;
use crate::geo_types::buffer::Buffer;
use crate::geo_types::clip::LineClip;
use crate::geo_types::shapes::arc_center;
use crate::prelude::{HatchPattern, TruchetHatch};
use geo::algorithm::rotate::Rotate;
use geo::{Area, Coord, MultiLineString, MultiPolygon, Scale, Translate};
use geo_offset::Offset;
use geo_types::{coord, point, Geometry, GeometryCollection, LineString, Point, Rect};
use noise::{NoiseFn, Perlin, Seedable};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use std::sync::Arc;

pub trait PointField: Debug + Send + Sync {
    fn points(&self) -> Option<Vec<Point>> {
        Some(Vec::new())
    }

    fn bounds(&self) -> Option<Rect<f64>>;

    fn generate(&mut self, rec: &Rect<f64>) {}
}

#[derive(Serialize, Deserialize, Default)]
pub struct PerlinPointField {
    seed: u32,
    coord_scale: f64,
    prob_scale: f64,
    density: f64,
    points: Option<Vec<Point<f64>>>,
    bounds: Option<Rect<f64>>,
}

impl PerlinPointField {
    pub fn new(seed: u32, coord_scale: f64, prob_scale: f64, density: f64) -> Self {
        Self {
            seed,
            coord_scale,
            prob_scale,
            density,
            points: None,
            bounds: None,
        }
    }
}

impl Debug for PerlinPointField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pointlen = if self.points.is_some() {
            (&self.points).clone().unwrap().len()
        } else {
            0usize
        };
        f.debug_struct("PerlinPointField")
            .field("seed", &self.seed)
            .field("coord_scale", &self.coord_scale)
            .field("prob_scale", &self.prob_scale)
            .field("density", &self.density)
            .field("#points", &pointlen)
            .field("bounds", &self.bounds)
            .finish()
    }
}

impl PointField for PerlinPointField {
    fn points(&self) -> Option<Vec<Point>> {
        self.points.clone()
    }

    fn generate(&mut self, rect: &Rect<f64>) {
        let mut points: Vec<Point<f64>> = Vec::new();
        let mut perlin = Perlin::new().set_seed(self.seed);
        let mut rng = SmallRng::seed_from_u64(self.seed as u64);
        let min = rect.min();
        let max = rect.max();
        let mut x = min.x;
        let mut y = min.y;
        // println!("RECT: {:?}", &rect);
        let point_count = (self.density * rect.width() * rect.height().ceil()) as usize;
        for _i in 0..point_count {
            let (x, y) = rng.gen::<(f64, f64)>();
            let x = x * rect.width() + min.x;
            let y = y * rect.height() + min.y;
            let noiseval = perlin.get([x * self.coord_scale, y * self.coord_scale]);
            // println!("Testing {},{}:{}", x, y, noiseval);

            if noiseval > self.prob_scale {
                // println!("Adding");
                points.push(Point::new(x, y))
            } else {
                // println!("Fail to add.");
            }
        }
        self.bounds = Some(rect.clone());
        self.points = Some(points);
        // println!("Points: {:?}", &self.points);
    }

    fn bounds(&self) -> Option<Rect<f64>> {
        self.bounds.clone()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_perlin_field() {
        let mut pf = PerlinPointField::new(12, 15., 0.5, 0.5);
        pf.generate(&Rect::new(
            Coord { x: 0., y: 0. },
            Coord { x: 100., y: 100. },
        ));
        println!("PF: {:?}", pf);
    }
}
