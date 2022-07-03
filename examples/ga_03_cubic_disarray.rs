use std::path::Path;
use geo::centroid::Centroid;
use geo::rotate::RotatePoint;
use geo::translate::Translate;
use geo_types::{coord, LineString, MultiLineString, Polygon, Rect};
use aoer_plotty_rs::prelude::{Arrangement, ToSvg};
use rand::prelude::{SmallRng, SeedableRng, Rng};

/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/cubic-disarray/
fn main() {
    // The squares are 25mm each (smaller than the original tutorial to fit on 8.5/11)
    let square_size: i32 = 25;
    let square_count: i32 = 9;
    let rotate_mul = 30.0;
    let displace_mul = f64::from(square_size / 2);
    // We're using a static random generator here so that our SVG files
    // don't get regenerated every time we run the examples.
    let mut rng = SmallRng::seed_from_u64(12345);

    // Define our viewbox/canvas (in mm)
    let viewbox = Rect::new(
        coord! {
            x:0f64,
            y:0f64},
        coord! {
            x: f64::from((square_size*square_count) as i32),
            y: f64::from((square_size*square_count) as i32)});

    let mut boxes: Vec<Polygon<f64>> = vec![];
    for y in 0..square_count {
        for x in 0..square_count {
            let rotate_amt = (f64::from(y) / f64::from(square_count))
                * ((2.0 * rng.gen::<f64>()) - 1.0) * rotate_mul; // -1.0 to 1.0
            let translate_amt = (f64::from(y) / f64::from(square_count))
                * ((2.0 * rng.gen::<f64>()) - 1.0) * displace_mul;
            let r = Rect::new(
                coord! {
                    x: f64::from(x * square_size),
                    y: f64::from(y * square_size)},
                coord! {
                    x: f64::from((x * square_size) + square_size),
                    y: f64::from((y * square_size) + square_size)
                })
                .to_polygon();
            let r = r.rotate_around_point(rotate_amt, r.centroid().unwrap());
            let r = r.translate(translate_amt, 0.0);
            boxes.push(r);
        }
    }

    // Get the outlines for those boxes as a MultiLineString
    let box_lines: Vec<LineString<f64>> = boxes.iter().map(|b| b.exterior().clone()).collect();
    let box_lines = MultiLineString::new(box_lines);

    // The arrangement chooses the way we "arrange" the SVG on the page.
    // In this case, fit it, center it, and then DON'T flip the coordinate
    // system upside down (SVG has top left as 0,0, whereas mathematically
    // 0,0 is the center, and on a CNC machine, 0,0 is bottom left... usually).
    let arrangement = Arrangement::FitCenterMargin(10.0, viewbox, false);

    // Use a shortcut to create an SVG scaffold from our arrangement.
    let svg = arrangement.create_svg_document().unwrap()
        .add(box_lines.to_path(&arrangement)
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1)
            .set("stroke-linejoin", "square")
            .set("stroke-linecap", "square"));

    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
