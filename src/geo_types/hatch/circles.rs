use super::HatchPattern;
use crate::geo_types::shapes::circle;
use geo::Geometry as GeoGeometry;
use geo_types::{LineString, MultiLineString, Rect};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CircleHatch {}

impl CircleHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for CircleHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, pen: f64) -> MultiLineString {
        let (x1, y1) = bbox.min().x_y();
        let (x2, y2) = bbox.max().x_y();
        let mut lines: Vec<LineString> = vec![];
        let mut ix: usize = 0;
        let mut x = x1 - 2. * scale;
        let r2 = 2.0_f64.sqrt();
        while x < x2 + 2. * (scale + pen) {
            let (ofsx, ofsy) = if ix % 2 == 0 {
                (scale - pen, -(scale + pen * r2) * r2)
            } else {
                //(-(scale - pen), 0.)
                (-scale + pen / 2., 0.)
            };

            let mut y = y1 - scale;
            while y < y2 + 2. * (scale + pen) {
                // println!("Adding circle at x:{}, y:{} @ofs:{:?}", x, y, (ofsx, ofsy));
                let c = circle(x + ofsx, y + ofsy, scale / 2.);
                if let GeoGeometry::Polygon(tmp_lines) = c.into() {
                    // Tricky. We reverse every other circle for better back and forth
                    // optimization of hatch drawing.
                    if ix % 2 == 0 {
                        lines.push(tmp_lines.exterior().clone());
                    } else {
                        let mut tmp_lines = tmp_lines.exterior().clone();
                        tmp_lines.0.reverse();
                        lines.push(tmp_lines)
                    }
                }
                y += scale + pen;
            }
            ix += 1;
            x += scale - (pen * r2 / 2.);
        }
        // println!("Lines for radius fill are: {:?}", &lines);

        MultiLineString::<f64>::new(lines)
    }
}
