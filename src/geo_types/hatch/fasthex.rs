use super::HatchPattern;
use geo_types::{coord, LineString, MultiLineString, Rect};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct FastHexHatch {}

impl FastHexHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for FastHexHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString {
        let (x1, y1) = bbox.min().x_y();
        let (x2, y2) = bbox.max().x_y();
        let mut lines: Vec<LineString> = vec![];
        let mut ix: usize = 0;
        let mut x = x1 - 2. * scale;
        let sidelen = scale / 2.;
        let rin = scale * (PI / 6.).cos() / 2.;
        // println!("SIDELEN: {} , RIN: {} , ", sidelen, rin);

        while x <= x2 + scale {
            let (inc, mul) = if ix % 2 == 0 {
                (rin * 2., 1.)
            } else {
                (0.0, -1.)
            };
            let mut y = y1 - 2. * scale;
            let mut aline: LineString<f64> = LineString::new(vec![]);
            while y <= y2 + scale {
                // y2 + 2. * scale {
                aline.0.push(coord! {x:x, y:y});
                aline.0.push(coord! {x:x+mul*rin, y: y+sidelen/2.});
                aline.0.push(coord! {x:x+mul*rin, y: y+sidelen/2.+sidelen});
                aline.0.push(coord! {x:x, y: y+scale});
                aline.0.push(coord! {x:x, y: y+scale+sidelen});
                y = y + scale + sidelen;
            }
            // println!("ALINE: {:?}", &aline);
            if ix % 2 != 0 {
                aline.0.reverse();
            }
            lines.push(aline);
            ix += 1;
            x += inc + 0.00001; // We do slightly over the inc, to ensure no overlapping lines.
        }
        // println!("Lines for radius fill are: {:?}", &lines);

        MultiLineString::<f64>::new(lines)
    }
}
