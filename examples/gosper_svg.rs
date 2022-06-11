use std::collections::HashMap;
use geo::lines_iter::LinesIter;
use geo::prelude::{BoundingRect, EuclideanDistance, HasDimensions, Translate};
use geo_types::{Coordinate, CoordNum, LineString, MultiLineString, Point, Polygon, Rect};
use geos::GeometryTypes::MultiPolygon;
use nalgebra::{Affine2, RealField, Similarity2, Point2 as NPoint2};
use num_traits::real::Real;
use aoer_plotty_rs::geo_types::hatch::{Hatch, LineHatch};
use aoer_plotty_rs::geo_types::nannou::NannouDrawer;
use aoer_plotty_rs::geo_types::PointDistance;
use aoer_plotty_rs::turtle::{Turtle, TurtleTrait, degrees};
use aoer_plotty_rs::l_system::LSystem;

use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;


/// An arrangement is a plan for transformation of an SVG
pub enum Arrangement<T>
    where T: Real,
          T: CoordNum,
          T: RealField {
    Center,
    FitCenter,
    Transform(Affine2<T>),
}

pub trait ToSvg<T>
    where T: CoordNum,
          T: Real,
          T: RealField {
    /// Given an [Arrangement] as a transformation strategy, transform the geometry to
    /// fit the bounds, or just run the transformation without bounds if None
    fn arrange(self, arrangement: Arrangement<T>, bounds: Option<Rect<T>>) -> Self;

    /// Utility function returns a viewbox tuple for the geometry.
    /// Should be used AFTER calling arrange on the geo.
    fn viewbox(&self) -> (T, T, T, T);

    /// Convert the Geometry into an SVG Path
    fn to_path(&self) -> Path;
}

impl<T> ToSvg<T> for MultiLineString<T>
    where T: CoordNum,
          T: Real,
          T: RealField {

    fn arrange(self, arrangement: Arrangement<T>, bounds: Option<Rect<T>>) -> Self {
        let transformation = match arrangement{
            Arrangement::Transform(affine) => affine,
            _=> todo!()
        };
        let linestrings: Vec<LineString<T>> = self.iter().map(|linestring|{
            // linestring.clone()
            linestring.coords().map(|coord|{
                let pt = transformation * NPoint2::<T>::new(coord.x, coord.y);
                Coordinate::<T>::from((pt.x, pt.y))
            }).collect()
        }).collect();
        MultiLineString::<T>::new(linestrings)
    }

    fn viewbox(&self) -> (T, T, T, T) {
        todo!()
    }

    fn to_path(&self) -> Path {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::PI as PI_F32;
    use std::str::FromStr;
    use geo_types::{coord, LineString, MultiLineString};
    use nalgebra::{Vector2, Affine2, Matrix3};
    use wkt::Wkt;
    use super::*;

    #[test]
    fn test_load_wkt(){
        let geoms: Polygon::<f64> = Polygon::try_from(
            Wkt::<f64>::from_str("POLYGON ((350 100, 450 450, 150 400, 100 200, 350 100), (200 300, 350 350, 300 200, 200 300))")
                .expect("Failed to load WKT"))
            .expect("Failed to load box");

    }

    #[test]
    fn test_arrange_mls_arbitrary(){
        let mls = MultiLineString::new(
            vec![LineString::new(
                vec![
                    coord!{x: 0.0f64, y: 0.0f64},
                    coord!{x: 0.0f64, y: 100.0f64},
                    coord!{x: 100.0f64, y: 100.0f64},
                    coord!{x: 100.0f64, y: 0.0f64},
                    coord!{x: 0.0f64, y: 0.0f64},
                ])]);
        let txmls = mls.arrange(
            Arrangement::Transform(Affine2::from_matrix_unchecked(
                Matrix3::new(
                    1.0, 0.0, 0.0,
                    0.0, 1.0, 0.0,
                    0.0, 0.0, 1.0,
                )
            )), None);
        println!("TXMLS IS {:?}", txmls);

    }
}

fn main() {
    let mut t = Turtle::new();
    // And put its pen down so that it is drawing.
    t = t.pen_down();

    // Create a new LSystem, which defines a Gosper curve. We'll be expanding this
    // into a path next.
    let gosper = LSystem {
        axiom: "A".to_string(),
        rules: HashMap::from([
            ('A', "A-B--B+A++AA+B-".to_string()),
            ('B', "+A-BB--B-A++A+B".to_string())]),
    };

    // Create a MultiLineString via the Turtle
    let tlines = t
        // Use the turtle's TurtleTrait to walk an LPath, which is given by...
        .walk_lpath(
            // Expanding the gosper system we just created, on the 4th order
            &gosper.expand(4), degrees(60.0), 8.0)
        // And convert to multiline
        .to_multiline();

    // Find the center of the drawing
    let bounds = tlines.bounding_rect().unwrap();
    let bc = tlines.bounding_rect().unwrap().bounding_rect().center();


    // Center it
    let tlines = tlines.translate(-bc.x, -bc.y);


    let mut svg_data = Data::new();

    // Draw the turtle lines into the SVG context
    // tlines.iter().for_each(move |tline| {
    for tline in tlines {
        for point in tline.points().take(1) {
            svg_data = svg_data.move_to((point.x(), point.y()));
        }
        for point in tline.points().skip(1) {
            svg_data = svg_data.line_to((point.x(), point.y()));
        }
    }
    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("d", svg_data);

    let svg = Document::new()
        .set("viewBox", (bounds.min().x, bounds.min().y, bounds.max().x, bounds.max().y))
        .add(path);

    svg::save("images/gosper-to-svg-example.svg", &svg).unwrap();
}

