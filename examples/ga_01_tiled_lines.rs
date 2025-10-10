use aoer_plotty_rs::geo_types::svg::{Arrangement, ToSvg};
use geo_types::{LineString, MultiLineString, Rect, coord};
use std::path::Path;

/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/tiled-lines/
fn main() {
    let size = 16;
    let tilesize = 20;

    // Define our viewbox/canvas (in mm)
    let viewbox = Rect::new(
        coord! {
        x:0f64,
        y:0f64},
        coord! {
        x: f64::from(size) * f64::from(tilesize),
        y: f64::from(size) * f64::from(tilesize)},
    );

    // The arrangement chooses the way we "arrange" the SVG on the page.
    // In this case, fit it, center it, and then DON'T flip the coordinate
    // system upside down (SVG has top left as 0,0, whereas mathematically
    // 0,0 is the center, and on a CNC machine, 0,0 is bottom left... usually).
    let arrangement = Arrangement::FitCenter(viewbox, false);

    // Create a mutable empty MultiLineString, which can contain any number of lines.
    let mut lines = MultiLineString::<f64>::new(vec![]);

    // I tweaked the code a bit here to use Rust iterators, but it's close
    // in spirit to the original. We iterate columns and rows instead of
    // directly setting coordinates.
    //  For each row, draw some columns of tiles
    for y in 0..size {
        // For each column in the row, draw a tile
        for x in 0..size {
            // Pick a random number. Basically flipping a coin.
            let r: f64 = rand::random();
            // Heads < 0.5 < Tails
            if r < 0.5f64 {
                // Heads? From top left to bottom right
                lines.0.push(LineString::new(vec![
                    coord! {x: f64::from(x), y: f64::from(y)},
                    coord! {x: f64::from(x+1), y: f64::from(y+1)},
                ]));
            } else {
                // Or Tails? From bottom left to top right.
                lines.0.push(LineString::new(vec![
                    coord! {x: f64::from(x), y: f64::from(y+1)},
                    coord! {x: f64::from(x+1), y: f64::from(y)},
                ]));
            }
        }
    }

    // Use a shortcut to create an SVG scaffold from our arrangement.
    let svg = arrangement
        .create_svg_document()
        .unwrap()
        .add(lines.to_path(&arrangement))
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("stroke-linejoin", "round")
        .set("stroke-linecap", "round");

    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
