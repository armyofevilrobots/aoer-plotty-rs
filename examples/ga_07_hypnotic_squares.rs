use aoer_plotty_rs::prelude::{Arrangement, ToSvg};
use geo_types::{LineString, MultiLineString, Rect, coord};
use rand::prelude::*;
use std::path::Path;

/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/hypnotic-squares/
fn draw(
    x: f64,
    y: f64,
    size: f64,
    orig_size: f64,
    min_size: f64,
    xofs: f64,
    yofs: f64,
    steps: i32,
    orig_steps: i32,
) -> MultiLineString<f64> {
    let mut mls = MultiLineString::new(vec![LineString::new(vec![
        coord! {x: x-size/2.0, y: y-size/2.0},
        coord! {x: x-size/2.0, y: y+size/2.0},
        coord! {x: x+size/2.0, y: y+size/2.0},
        coord! {x: x+size/2.0, y: y-size/2.0},
        coord! {x: x-size/2.0, y: y-size/2.0},
    ])]);

    if steps > 0 {
        let spacing = (orig_size - min_size) / f64::from(orig_steps);
        let new_size = size - spacing;
        mls.0.append(
            &mut draw(
                x - xofs,
                y - yofs,
                new_size,
                orig_size,
                min_size,
                xofs,
                yofs,
                steps - 1,
                orig_steps,
            )
            .0,
        );
    }
    mls
}

fn main() {
    let size = 224;
    let box_count = 8;
    let box_margin = 1.0f64;
    let box_spacing = f64::from(size) / f64::from(box_count);
    let box_size = box_spacing - box_margin * 2.0;
    let min_size = 2.0f64;
    let pen_width = 0.5;
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

    // Create a container we'll stuff all our lines in
    let mut all_lines = MultiLineString::new(vec![]);

    // Iterate over the X/Y positions our boxes will be in
    for y in 0..box_count {
        for x in 0..box_count {
            // How many steps on this box (2-5)
            let the_steps = rng.gen_range(2i32..5i32);
            // How far apart is each square?
            let ofs_spacing = (box_size - min_size) / f64::from(the_steps);
            // How much to offset randomly by? Not that ofs_spacing is /4.0 instead of /2.0
            // because we're already using half that offset to shrink each subsequent box.
            let xofs = f64::from(rng.gen_range(-1i32..2i32)) * ofs_spacing / 4.0;
            let yofs = f64::from(rng.gen_range(-1i32..2i32)) * ofs_spacing / 4.0;

            // OK, dump all the lines in THIS hypnotic square into the top level squares list.
            all_lines.0.append(
                &mut draw(
                    f64::from(x) * box_spacing,
                    f64::from(y) * box_spacing,
                    box_size,
                    box_size,
                    2.0,
                    xofs,
                    yofs,
                    the_steps,
                    the_steps,
                )
                .0,
            );
        }
    }

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
