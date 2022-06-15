extern crate geos;

use std::collections::HashMap;
use aoer_plotty_rs::geo_types::nannou::NannouDrawer;
use geo_types::MultiLineString;
use nannou::prelude::*;
use aoer_plotty_rs::turtle::{Turtle, TurtleTrait, degrees};
use aoer_plotty_rs::l_system::LSystem;
use geo::prelude::{BoundingRect, Translate};
use nannou::color;
use nannou::lyon::lyon_tessellation::LineJoin;
use nannou::lyon::tessellation::LineCap;

/// The Model contains just the loop count (number of frames) and the tlines (turtle lines)
/// MultiLineString that contains the gosper curve.
struct Model {
    loops: u32,
    tlines: MultiLineString<f64>
}

/// Creates a new turtle, then a new gosper LSystem. Walks the Gosper path after expanding
/// the LSystem, and then spits out a multiline string which we use to populate the Model.
/// Also centers the resulting MultiLineString on the 0,0 point in the middle of the screen.
fn model(_app: &App) -> Model {
    // Create a turtle
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
    let bc = tlines.bounding_rect().unwrap().bounding_rect().center();

    // Center it
    let tlines = tlines.translate(-bc.x, -bc.y);

    // We're done. Save it in the model.
    Model {
        loops: 0,
        tlines
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // Update the var used to spin the gosper
    _model.loops += 1;
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    // Broilerplate Nannou
    let draw = _app.draw();
    frame.clear(PURPLE);

    // Draw the turtle lines into the draw context
    _model.tlines.iter().for_each(|tline| {
        draw.polyline()
            .stroke_weight(3.0)
            .caps(LineCap::Round)
            .join(LineJoin::Round)
            .polyline_from_linestring(tline)
            .color(color::NAVY);
    });

    // And slowly spin it
    draw.rotate((_model.loops as f32) * PI/180.0)
        // Done. Put it on the screen
        .to_frame(_app, &frame).unwrap();
}

fn main() {
    // Basic Nannou setup.
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}

