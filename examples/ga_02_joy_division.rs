use geo_types::{coord, Coordinate, LineString, MultiLineString, Rect, Geometry};
use aoer_plotty_rs::geo_types::svg::{Arrangement, ToSvg};
use aoer_plotty_rs::geo_types::clip::LineClip;
use std::path::Path;
use cubic_spline::{Points as CSPoints, Point as CSPoint, SplineOpts, TryFrom};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};


/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/joy-division/
fn main() {
    let step: usize = 10;
    let size: usize = 320;
    // We're using a static random generator here so that our SVG files
    // don't get regenerated every time we run the examples.
    let mut rng = SmallRng::seed_from_u64(12345);

    // Define our viewbox/canvas (in mm)
    let viewbox = Rect::new(
        coord! {
            x:0f64,
            y:0f64},
        coord! {
            x: f64::from(size as i32),
            y: f64::from(size as i32)});

    // The arrangement chooses the way we "arrange" the SVG on the page.
    // In this case, fit it, center it, and then DON'T flip the coordinate
    // system upside down (SVG has top left as 0,0, whereas mathematically
    // 0,0 is the center, and on a CNC machine, 0,0 is bottom left... usually).
    let arrangement = Arrangement::FitCenterMargin(20.0, viewbox, false);

    // Create a mutable empty MultiLineString, which can contain any number of lines.
    let mut lines = MultiLineString::<f64>::new(vec![]);

    // Alright, the meat of it. We'll iterate over "each line, starting from step
    // (skipping the top line) and upto but not including size (the bottom line).
    // Then we do the same horizontally but generating the "height" of each line
    // point.
    let spoints: Vec<Vec<(f64, f64)>> = (step..size).step_by(step).map(|y| {
        // (step..size).step_by(step).map(|x| {
        (step..size).step_by(step).map(|x| {
            let r: f64 = rng.gen();// * f64::from(step as i32);
            let dts: f64 = ((x as i32 - (size as i32 / 2)) as f64).abs();
            let variance = (((size as i32 / 2) - 50) as f64 - dts).max(0.0_f64);
            (f64::from(x as i32), f64::from(y as i32) + (r * variance / 2.0 * -1.0))
        }).collect::<Vec<(f64, f64)>>()
    }).collect();

    // Now we have the line points, we re-imagine them as splines.
    // First the spline options
    let spline_opts = SplineOpts::new()
        .num_of_segments(step as u32)
        .tension(0.5);
    // Then the data gets iterated for each separate line...
    for spline_data in spoints {
        // Turn those points into a spline
        let points = <CSPoints as cubic_spline::TryFrom<&Vec<(f64, f64)>>>::try_from(&spline_data).expect("Invalid spline points.");
        // Then interpolate with the spline tools
        let fine_points = points.calc_spline(&spline_opts)
            .expect("Could not generate interpolated points.");
        // Iterate them back into a vec of coordinates
        let mpts: Vec<Coordinate<f64>> = fine_points.get_ref().iter().map(|spt| { coord! {x: spt.x, y: spt.y} }).collect();
        // Finally, push a new line into the lines, by turning those coords into a LineString
        lines.0.push(LineString::new(mpts));
    }

    // OK, next we wanna clip these things so that each subsequent line clips the ones
    // that came before it, to give the illusion of a 3d effect from bottom=front to top=back.

    // First we create an output multilinestring as our destination
    let mut newlines: MultiLineString<f64> = MultiLineString::new(vec![]);

    // For each line, clip any existing lines, then add it to the new lines list
    for i in 0..(lines.0.len()) {
        // This needs to be a a dupe
        let mut clipline = lines.0[i].clone();
        // because we extend it to the bottom of the SVG to clip EVERYTHING behind it.
        clipline.0.push(coord! {x: size as f64, y: size as f64});
        clipline.0.push(coord! {x: 0.0, y: size as f64});

        // Now, do the clip, match on Ok, and just keep the existing list if something breaks.
        newlines = match geo_types::Geometry::MultiLineString(newlines.clone())
            .clipwith(&Geometry::LineString(clipline)) {
            Ok(newnewlines) => newnewlines,
            Err(_) => newlines
        };
        // Finally, add the line we just clipped with on to the newlines list, and repeat!
        newlines.0.push(lines.0[i].clone());
    }


    // Use a shortcut to create an SVG scaffold from our arrangement.
    let svg = arrangement.create_svg_document().unwrap()
        .add(newlines.to_path(&arrangement)
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("stroke-linejoin", "round")
        .set("stroke-linecap", "round"));

    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
