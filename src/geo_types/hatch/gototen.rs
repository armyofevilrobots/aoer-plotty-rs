use super::HatchPattern;
use geo_types::{coord, LineString, MultiLineString, Rect};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
/// This is a hatch pattern that is reminiscent of the old C64 program
/// that looked kinda like this:
/// \/\\\/\
/// /\//\/\
/// \\\//\/
/// ie:
/// 10 PRINT CHR$(205.5+RND(1)); : GOTO 10
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct GotoTenHatch {
    seed: u64,
}

impl GotoTenHatch {
    pub fn gen(seed: u64) -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(GotoTenHatch { seed }))
    }
}

impl HatchPattern for GotoTenHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString<f64> {
        let min = bbox.min();
        let max = bbox.max();
        let mut y = min.y - scale;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(self.seed);
        // MultiLineString::<T>::new(
        let mut lines: Vec<geo_types::LineString<f64>> = vec![];

        while y < max.y + scale {
            let mut x = min.x - scale;
            while x < max.x + scale {
                if rng.gen_bool(0.5) {
                    lines.push(geo_types::LineString::<f64>::new(vec![
                        coord! {x: x, y: y},
                        coord! {x: x+scale, y: y+scale},
                    ]));
                } else {
                    lines.push(geo_types::LineString::<f64>::new(vec![
                        coord! {x: x, y: y+scale},
                        coord! {x: x+scale, y: y},
                    ]));
                }
                x += scale
            }
            y += scale;
        }
        MultiLineString::<f64>::new(lines)
    }
}
