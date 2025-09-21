use super::HatchPattern;
use geo_types::{coord, LineString, MultiLineString, Rect};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

/// The basic built in parallel LineHatch.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct LineHatch {}

impl LineHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for LineHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString<f64> {
        let min = bbox.min();
        let max = bbox.max();
        let mut y = min.y;
        let mut count = 0u32;
        // MultiLineString::<T>::new(
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
        let out = MultiLineString::<f64>::new(lines);
        out
    }
}
