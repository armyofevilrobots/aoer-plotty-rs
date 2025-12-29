use aoer_plotty_rs::geo_types::hilbert_spatial_hash::HilbertSpatialHash;
use aoer_plotty_rs::geo_types::spline::hobby_points;
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::prelude::NoHatch;
use aoer_plotty_rs::{context::Context, prelude::LineHatch};
use cubic_spline::SplineOpts;
use geo::{Coord, LineString};
use geo_offset::Offset;
use geo_types::{Rect, coord};
use noise::{NoiseFn, Perlin, Seedable};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::path::Path;
use std::sync::Arc;

fn main() {
    let mut ctx = Context::new();

    let perlx = Perlin::new(0);
    let perly = Perlin::new(2);
    let pscale = 0.005f64;
    let pdisplace = 3.0f64;

    let rect = Rect::new(Coord { x: 0., y: 0. }, Coord { x: 200., y: 200. });
    let mut h = HilbertSpatialHash::new().with_bounds(rect.clone());
    let mut rng = SmallRng::from_entropy(); //seed_from_u64(16);
    for _i in 0..8 {
        h.add(&Coord {
            x: rng.gen_range(40.0..160.0),
            y: rng.gen_range(40.0..160.0),
        })
        .expect("Failed to insert");
    }

    let mut spoints = h.into_iter().collect();
    for i in 0..30 {
        let curve: Vec<(f64, f64)> = hobby_points(&spoints, 0.5)
            .unwrap()
            .iter()
            .map(|c| (c.x, c.y))
            .collect();
        ctx.stroke("black")
            .pattern(NoHatch::gen())
            .pen(0.35)
            .spline(&curve, 32, 0.5);
        spoints = spoints
            .iter()
            .map(|c| Coord {
                x: c.x + pdisplace * perlx.get([c.x * pscale, c.y * pscale]),
                y: c.y + pdisplace * perly.get([c.x * pscale, c.y * pscale]),
            })
            .collect();

        // h = HilbertSpatialHash::new().with_bounds(rect.clone());
        // for p in spoints {
        //     if p.x > 200. || p.x < 0. || p.y > 200. || p.y < 0. {
        //         println!("Point out of bounds: {:?}", &p);
        //     }
        //     h.add(&Coord { x: p.x, y: p.y }).unwrap()
        // }
    }

    let svg = ctx
        .to_svg(&Arrangement::<f64>::unit(&Rect::<f64>::new(
            coord! {x:0.0, y:0.0},
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
