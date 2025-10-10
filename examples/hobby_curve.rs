use aoer_plotty_rs::geo_types::spline::hobby_points;
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::prelude::NoHatch;
use aoer_plotty_rs::{context::Context, prelude::LineHatch};
use geo::{Coord, LineString};
use geo_types::{coord, Rect};
use std::path::Path;
use std::sync::Arc;

fn main() {
    let mut ctx = Context::new();

    let spoints = vec![
        Coord { x: 50.0, y: 30.0 },
        Coord { x: 30.0, y: 50.0 },
        Coord { x: 10.0, y: 30.0 },
        Coord { x: 30.0, y: 10.0 },
    ];

    let curve: Vec<(f64, f64)> = hobby_points(&spoints, 1.0)
        .unwrap()
        .iter()
        .map(|c| (c.x, c.y))
        .collect();

    println!("POINTS: {:?}", curve);

    ctx.stroke("black")
        .pattern(NoHatch::gen())
        .stroke("green")
        .pen(0.5)
        .geometry(&LineString::new(spoints.clone()).into())
        .stroke("red")
        .pen(1.0)
        .geometry(&LineString::new(curve.iter().map(|c| Coord { x: c.0, y: c.1 }).collect()).into())
        .stroke("black")
        .pen(0.5)
        .spline(&curve, 32, 0.5)
        .stroke("blue")
        .pen(0.2)
        .spline(&spoints.iter().map(|c| (c.x, c.y)).collect(), 16, 0.5);

    let svg = ctx
        .to_svg(&Arrangement::<f64>::unit(&Rect::<f64>::new(
            coord! {x:-20.0, y:-20.0},
            coord! {x:200.0, y:200.0},
        )))
        .unwrap();
    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();

    // .join("images")
    // .join(fname)
    // .to_str()
    // .unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
