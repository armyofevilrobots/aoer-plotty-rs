use aoer_plotty_rs::prelude::{Arrangement, Hatch, Hatches, ToSvg};
use geo_types::{coord, Coordinate, LineString, MultiLineString, MultiPolygon, Polygon, Rect};
use nalgebra::{Affine2, Matrix3};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use std::path::Path;

/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/cubic-disarray/
fn main() {
    let size = 224;
    let steps = 8; // Note that this HAS to be EVEN
    let gap = size / steps;
    let pen_width = 0.3;
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

    let mut dots: Vec<Coordinate<f64>> = vec![];
    let mut lines = MultiLineString::<f64>::new(vec![]);
    let mut polygons = MultiPolygon::<f64>::new(vec![]);

    // First we generate the actual lines of 'dots'
    for ys in 0..steps {
        let mut line: LineString<f64> = LineString::new(vec![]);
        let y = (gap / 2) + ys * gap;
        for xs in 0..steps {
            let x = (gap / 4) + xs * gap;
            let dot = coord! {
            x: if ys % 2 != 0 {
                f64::from(x) + (rng.gen::<f64>()*0.8f64 - 0.4f64) * f64::from(gap)
            } else {
                f64::from(x+gap/2) + (rng.gen::<f64>()*0.8f64 - 0.4f64)*f64::from(gap)
            },
            y: f64::from(y) + (rng.gen::<f64>()*0.8f64 - 0.4) * f64::from(gap)};
            dots.push(dot.clone());
            line.0.push(dot.clone());
        }
        lines.0.push(line);
    }

    // Then we iterate those lines and generate the triangles (polygons)
    // The odd and even lines have different strategies to generate those
    // triangles since each other row is offset by 1/2 the width.
    for yi in 0..(lines.0.len() - 1) {
        for xi in 0..(lines.0.len() - 1) {
            if yi % 2 == 0 {
                // If it's even
                polygons.0.push(Polygon::new(
                    LineString::new(vec![
                        lines.0[yi].0[xi].clone(),
                        lines.0[yi].0[xi + 1].clone(),
                        lines.0[yi + 1].0[xi + 1].clone(),
                    ]),
                    vec![],
                ));
                polygons.0.push(Polygon::new(
                    LineString::new(vec![
                        lines.0[yi].0[xi].clone(),
                        lines.0[yi + 1].0[xi + 1].clone(),
                        lines.0[yi + 1].0[xi].clone(),
                    ]),
                    vec![],
                ));
            } else {
                polygons.0.push(Polygon::new(
                    LineString::new(vec![
                        lines.0[yi].0[xi].clone(),
                        lines.0[yi + 1].0[xi].clone(),
                        lines.0[yi].0[xi + 1].clone(),
                    ]),
                    vec![],
                ));
                polygons.0.push(Polygon::new(
                    LineString::new(vec![
                        lines.0[yi].0[xi + 1].clone(),
                        lines.0[yi + 1].0[xi].clone(),
                        lines.0[yi + 1].0[xi + 1].clone(),
                    ]),
                    vec![],
                ));
            }
        }
    }

    // OK, make the actual perimeter lines
    let all_lines = MultiLineString::<f64>::new(
        polygons
            .0
            .iter()
            .map(|poly| poly.exterior().clone())
            .collect(),
    );

    // Generate all the hatches
    let hatches: Vec<MultiLineString<f64>> = polygons
        .0
        .iter()
        .map(|p| {
            p.hatch(
                Hatches::line(),
                rng.gen::<f64>() * 90.0,
                rng.gen::<f64>() * 1.0 + pen_width,
                pen_width,
            )
            .unwrap()
        })
        .collect();

    // Merge them into a single layer
    let mut hatches_out: MultiLineString<f64> = MultiLineString::new(vec![]);
    for hatch in hatches {
        hatches_out.0.append(&mut hatch.0.clone());
    }

    // The arrangement chooses the way we "arrange" the SVG on the page.
    // In this case, we have a couple layers, so we set a standard xform
    // which is used for BOTH the hatches and the lines, so they "line up"
    // as it were :D
    let arrangement = Arrangement::Transform(
        viewbox,
        Affine2::from_matrix_unchecked(Matrix3::<f64>::new(
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
        )),
    );

    // Use a shortcut to create an SVG scaffold from our arrangement.
    let svg = arrangement
        .create_svg_document()
        .unwrap()
        .add(
            all_lines
                .to_path(&arrangement)
                .set("fill", "none")
                .set("stroke", "red")
                .set("stroke-width", pen_width)
                .set("stroke-linejoin", "square")
                .set("stroke-linecap", "butt"),
        )
        .add(
            hatches_out
                .to_path(&arrangement)
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", pen_width)
                .set("stroke-linejoin", "square")
                .set("stroke-linecap", "butt"),
        );

    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
    println!("DONE")
}
