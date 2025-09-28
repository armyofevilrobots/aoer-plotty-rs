use super::HatchPattern;
use geo_types::{coord, MultiLineString, Rect};
// use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct CrossHatch {}

impl CrossHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for CrossHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString<f64> {
        let min = bbox.min();
        let max = bbox.max();
        let mut y = min.y;
        let mut count = 0u32;
        let mut lines: Vec<geo_types::LineString<f64>> = vec![];
        while y < max.y {
            if count % 2 == 0 {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: min.x, y: y},
                    coord! {x: max.x, y: y},
                ]));
            } else {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: max.x, y: y},
                    coord! {x: min.x, y: y},
                ]));
            }
            y += scale;
            count += 1;
        }
        let mut x = min.x;
        count = 0u32;
        while x < max.x {
            if count % 2 == 0 {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: x, y: min.y},
                    coord! {x: x, y: max.y},
                ]));
            } else {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: x, y: max.y},
                    coord! {x: x, y: min.y},
                ]));
            }
            x += scale;
            count += 1;
        }
        //println!("HATCH LINES ARE: {:?}", &lines);
        MultiLineString::<f64>::new(lines)
    }
}
