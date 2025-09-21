use super::HatchPattern;
use geo::Coord;
use geo_types::{LineString, MultiLineString, Rect};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum SpiralDirection {
    #[default]
    Deasil,
    Widdershins,
}

pub const CLOCKWISE: SpiralDirection = SpiralDirection::Deasil;
pub const COUNTERCLOCKWISE: SpiralDirection = SpiralDirection::Widdershins;

#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct SpiralHatch {
    pub x: f64,
    pub y: f64,
    pub direction: SpiralDirection,
}

impl SpiralHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl Debug for SpiralHatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.debug_struct("SpiralHatch")
        //     .field("x", &self.x)
        //     .field("y", &self.y)
        //     .field("direction", &self.direction)
        //     .finish()
        write!(f, "SpiralHatch({},{}-{:?})", self.x, self.y, self.direction)?;
        std::fmt::Result::Ok(())
    }
}

impl HatchPattern for SpiralHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString {
        let (x1, y1) = bbox.min().x_y();
        let (x2, y2) = bbox.max().x_y();
        // println!("Center for bbox is: {:?}", bbox.center());
        let mut max_radius = 0.0f64;
        let mut min_radius = scale / 2.; //f64::MAX;
        for (x, y) in vec![(x1, y1), (x2, y1), (x2, y2), (x1, y2)] {
            let tmp_rad = ((x - self.x).powi(2) + (y - self.y).powi(2)).sqrt();
            if tmp_rad > max_radius {
                max_radius = tmp_rad;
            }
            if tmp_rad < min_radius {
                min_radius = tmp_rad;
            }
        }
        let mut points: Vec<Coord> = vec![];
        let mut r = min_radius;
        let mut theta = 0.0_f64;

        while r < max_radius {
            // This keeps segment length around PI mm or less.
            let ainc = (2. * PI / 16.).min(PI / r);
            // println!("AINC: {}, YAINC: {}", ainc, ainc.sin() * r);
            let r1 = r + (ainc / (2. * PI)) * scale;
            let x = self.x + theta.cos() * r;
            let y = self.y + theta.sin() * r;
            theta = theta
                + if self.direction == SpiralDirection::Widdershins {
                    ainc
                } else {
                    -ainc
                };
            points.push(Coord { x: x, y: y });
            r = r1;
        }
        // println!("Lines for radius fill are: {:?}", &lines);

        MultiLineString::<f64>::new(vec![LineString::new(points)])
    }
}
