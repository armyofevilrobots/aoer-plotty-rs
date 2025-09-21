use aoer_plotty_rs::context::{typography::Typography, Context};
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::prelude::{
    CircleHatch, CrossHatch, FastHexHatch, GotoTenHatch, HatchPattern, LineHatch, NoHatch,
    RadiusHatch, SpiralDirection, SpiralHatch,
};
use geo_types::{coord, Rect};
use std::path::Path;
use std::sync::Arc;

fn main() {
    let mut ctx = Context::new();

    let fills: Vec<Arc<Box<dyn HatchPattern>>> = vec![
        Arc::new(Box::new(LineHatch {})),
        (Arc::new(Box::new(CrossHatch {}))),
        (Arc::new(Box::new(RadiusHatch { x: 160., y: 32.0 }))),
        (Arc::new(Box::new(CircleHatch {}))),
        (Arc::new(Box::new(FastHexHatch {}))),
        (Arc::new(Box::new(RadiusHatch { x: 160., y: 32.0 }))),
        (Arc::new(Box::new(SpiralHatch {
            x: 32.0,
            y: 3. * 60.0,
            direction: SpiralDirection::Widdershins,
        }))),
        (Arc::new(Box::new(SpiralHatch {
            x: 64. + 32.0,
            y: 3. * 60.0,
            direction: SpiralDirection::Deasil,
        }))),
        GotoTenHatch::gen(0),
    ];

    for (i, pattern) in fills.iter().enumerate() {
        ctx.stroke("black")
            .fill("black")
            .pen(0.25)
            .pattern(pattern.clone())
            .hatch_scale(Some(3.))
            .hatch(0.)
            .poly(
                vec![
                    ((i % 3) as f64 * 64. + 4., (i / 3) as f64 * 72. + 4.),
                    ((i % 3) as f64 * 64. + 4. + 56., (i / 3) as f64 * 72. + 4.),
                    (
                        (i % 3) as f64 * 64. + 56. + 4.,
                        (i / 3) as f64 * 72. + 56. + 4.,
                    ),
                    ((i % 3) as f64 * 64. + 4., (i / 3) as f64 * 72. + 4. + 56.),
                ],
                vec![],
            )
            .pen(0.2)
            .pattern(Arc::new(Box::new(NoHatch {})))
            .typography(
                &format!("{:?}", pattern),
                (i % 3) as f64 * 64. + 4.,
                (i / 3) as f64 * 72. + 8. + 56.,
                &Typography::new().size(1.2),
            );
    }

    let svg = ctx
        .to_svg(&Arrangement::<f64>::unit(&Rect::<f64>::new(
            coord! {x:0.0, y:0.0},
            coord! {x:500.0, y:500.0},
        )))
        .unwrap();
    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
