use aoer_plotty_rs::prelude::{Arrangement, Hatches, OutlineFillStroke, ToSvg};
use geo::{translate::Translate, Rotate};
use geo_types::{coord, point, LineString, MultiLineString, Rect};
use rand::prelude::*;
use std::path::Path;
// use aoer_plotty_rs::geo_types::buffer::{Buffer, OutlineStroke};

/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/un-deux-trois/

/// Utility function for drawing, translating, and rotating the lines
fn draw(
    line_positions: Vec<f64>,
    size: f64,
    xc: f64,
    yc: f64,
    rotation: f64,
) -> MultiLineString<f64> {
    let mut lines: Vec<LineString<f64>> = vec![];
    for position in line_positions {
        let p1 = coord! {
            x: position*size,
            y: 0.0
        };
        let p2 = coord! {
            x: position*size,
            y: size
        };
        let line = LineString::new(vec![p1, p2])
            .rotate_around_point(
                rotation,
                point! {
                x: f64::from(size)/2.0,
                y: f64::from(size)/2.0},
            )
            .translate(xc - f64::from(size) / 2.0, yc - f64::from(size) / 2.0);
        lines.push(line);
    } // i in 0..count
    MultiLineString::new(lines)
}

fn main() {
    let size = 224;
    let steps = 14; // Matching the tute.
    let step = size / steps;
    let pen_width = 0.3;
    let stroke_mm = f64::from(step) / 6.0;
    // We're using a static random generator here so that our SVG files
    // don't get regenerated every time we run the examples.
    let mut rng = SmallRng::seed_from_u64(12345);

    // Define our viewbox/canvas (in mm)
    let viewbox = Rect::new(
        coord! {
        x:0f64,
        y:0f64},
        coord! {
        x: f64::from(size),
        y: f64::from(size)},
    );

    let mut lines_list: Vec<MultiLineString<f64>> = vec![];
    let mut line_positions: Vec<f64>; // = vec![];
    for yc in 0..steps {
        for xc in 0..steps {
            let rot_angle = rng.gen::<f64>() * 180.0;
            if yc < steps / 3 {
                line_positions = vec![0.5]
            } else if yc < ((2 * steps) / 3) {
                line_positions = vec![0.2, 0.8]
            } else {
                line_positions = vec![0.1, 0.5, 0.9]
            }

            // Render the thing, including fill
            lines_list.push(
                draw(
                    line_positions,
                    f64::from(step),
                    f64::from(xc * step),
                    f64::from(yc * step),
                    rot_angle,
                )
                // This method is cool. It fills a perimeter with a hatch, and turns the
                // perimeter into lines as well, returning the whole shebang as a
                // multilinestring.
                .outline_fill_stroke_with_hatch(
                    stroke_mm,
                    pen_width,
                    Hatches::line(),
                    rot_angle + 90.0,
                )
                .unwrap(),
            )
        }
    }

    // I've maybe spent a little too much time doing functional wannabe stuff...
    let all_lines = MultiLineString::new(
        lines_list
            .iter()
            .map(|mls| mls.0.clone())
            .flatten()
            .collect(),
    );

    // The arrangement chooses the way we "arrange" the SVG on the page.
    // In this case, fit it, center it, and then DON'T flip the coordinate
    // system upside down (SVG has top left as 0,0, whereas mathematically
    // 0,0 is the center, and on a CNC machine, 0,0 is bottom left... usually).
    let arrangement = Arrangement::FitCenterMargin(10.0, viewbox, false);

    // Use a shortcut to create an SVG scaffold from our arrangement.
    let svg = arrangement.create_svg_document().unwrap().add(
        all_lines
            .to_path(&arrangement)
            .set("fill", "none")
            .set("stroke", "red")
            .set("stroke-width", pen_width)
            .set("stroke-linejoin", "round")
            .set("stroke-linecap", "round"),
    );

    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
