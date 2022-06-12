use std::collections::HashMap;
use geo::lines_iter::LinesIter;
use geo::prelude::{BoundingRect, EuclideanDistance, HasDimensions, Translate};
use geo_types::{Coordinate, CoordNum, LineString, MultiLineString, Point, Polygon, Rect};
use geos::GeometryTypes::MultiPolygon;
use num_traits::real::Real;
use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;
use aoer_plotty_rs::geo_types::hatch::{Hatch, LineHatch};
use aoer_plotty_rs::geo_types::nannou::NannouDrawer;
use aoer_plotty_rs::geo_types::PointDistance;
use aoer_plotty_rs::turtle::{Turtle, TurtleTrait, degrees};
use aoer_plotty_rs::l_system::LSystem;


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

