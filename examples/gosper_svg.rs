use std::collections::HashMap;
use geo_types::{coord, Rect};
use aoer_plotty_rs::geo_types::svg::{Arrangement, ToSvg};
use aoer_plotty_rs::turtle::{Turtle, TurtleTrait, degrees};
use aoer_plotty_rs::l_system::LSystem;


fn main() {
    // First, we create a new Turtle, which is capable of drawing things.
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
            &gosper.expand(3), degrees(60.0), 8.0)
        // And convert to multiline
        .to_multiline();

    // Define our viewbox/canvas (in mm)
    let viewbox = Rect::new(
        coord! {x:0f64, y:0f64},
        coord! {x: 300f64, y:400f64});

    // Define an arrangement where we center the Gosper Curve in the center
    // of the page, with the viewbox of 300mmx400mm as the canvas.
    let arrangement = Arrangement::FitCenter(viewbox, true);

    // Now we create the base SVG document from our arrangement,
    // and use the to_path feature to create a path for the lines.
    // Note that you can call that add multiple times, but if you
    // do, you should use a set transformation instead of just fit/center
    // which could misalign separate path sets (or just use MultiLineString
    // with several LineStrings inside to consistently draw and align).
    let svg = arrangement.create_svg_document().unwrap()
        .add(tlines.to_path(&arrangement))
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("stroke-linejoin", "round")
        .set("stroke-linecap", "round");

    // Write it to the images folder, so we can use it as an example!
    svg::save("images/gosper-to-svg-example.svg", &svg).unwrap();
}

