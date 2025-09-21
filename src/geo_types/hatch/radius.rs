use super::HatchPattern;
use crate::geo_types::shapes::circle;
use geo::Geometry as GeoGeometry;
use geo_types::{LineString, MultiLineString, Rect};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct RadiusHatch {
    pub x: f64,
    pub y: f64,
}

impl RadiusHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for RadiusHatch {
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
        let mut lines: Vec<LineString> = vec![];
        let mut r = min_radius;
        while r < max_radius {
            let c = circle(self.x, self.y, r);
            if let GeoGeometry::Polygon(tmp_lines) = c.into() {
                lines.push(tmp_lines.exterior().clone());
            }
            r += scale;
        }
        // println!("Lines for radius fill are: {:?}", &lines);

        MultiLineString::<f64>::new(lines)
    }
}
