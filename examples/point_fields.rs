use aoer_plotty_rs::context::{typography::Typography, Context};
use aoer_plotty_rs::elements::point_field::*;
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::prelude::NoHatch;
use geo::{Coord, Geometry, GeometryCollection};
use geo_offset::Offset;
use geo_types::{coord, Rect};
use std::path::Path;

fn main() {
    let mut ctx = Context::new();
    let rect = Rect::new(Coord { x: 10., y: 10. }, Coord { x: 110., y: 110. });
    let pf = PerlinPointFieldBuilder::new()
        .seed(1)
        .coord_scale(0.1)
        .point_prob(0.05)
        .bounds(rect.clone())
        .build();

    let circles: Vec<Geometry> = pf
        .take(3000)
        .map(|point| {
            point
                .offset(0.5)
                .expect("Should always be able to offset a point.")
                .into()
        })
        .collect();
    let circles = Geometry::GeometryCollection(GeometryCollection::new_from(circles));

    ctx.stroke("black")
        .pen(1.)
        .pattern(NoHatch::gen())
        .geometry(&Geometry::Rect(rect))
        .pen(0.2)
        .typography(
            &"PerlinPointField".to_string(),
            // &"FIELD".to_string(),
            10.,
            115.,
            &Typography::new().size(1.2),
        )
        .fill("black")
        .pattern(NoHatch::gen())
        .mask_box(10., 10., 110., 110.)
        .geometry(&circles);

    let rect = Rect::new(Coord { x: 120., y: 10. }, Coord { x: 220., y: 110. });
    let hf = HaltonPointFieldBuilder::new()
        .seed(1)
        .bounds(rect.clone())
        .build();

    let circles: Vec<Geometry> = hf
        .take(3000)
        .map(|point| {
            point
                .offset(0.5)
                .expect("Should always be able to offset a point.")
                .into()
        })
        .collect();
    let circles = Geometry::GeometryCollection(GeometryCollection::new_from(circles));

    ctx.set_mask(&None)
        .stroke("black")
        .pen(1.)
        .pattern(NoHatch::gen())
        .geometry(&Geometry::Rect(rect).clone())
        .pen(0.2)
        .typography(
            &"HaltonPointField".to_string(),
            // &"FIELD".to_string(),
            120.,
            115.,
            &Typography::new().size(1.2),
        )
        .fill("black")
        .pattern(NoHatch::gen())
        .mask_box(120., 10., 220., 110.)
        .geometry(&circles);

    let rect = Rect::new(Coord { x: 230., y: 10. }, Coord { x: 330., y: 110. });
    let rf = RandomPointFieldBuilder::new()
        .seed(1)
        .bounds(rect.clone())
        .build();
    let circles: Vec<Geometry> = rf
        .take(3000)
        .map(|point| {
            point
                .offset(0.5)
                .expect("Should always be able to offset a point.")
                .into()
        })
        .collect();
    let circles = Geometry::GeometryCollection(GeometryCollection::new_from(circles));
    ctx.set_mask(&None)
        .stroke("black")
        .pen(1.)
        .pattern(NoHatch::gen())
        .geometry(&Geometry::Rect(rect).clone())
        .pen(0.2)
        .typography(
            &"RandomPointField".to_string(),
            // &"FIELD".to_string(),
            230.,
            115.,
            &Typography::new().size(1.2),
        )
        .fill("black")
        .pattern(NoHatch::gen())
        .mask_box(230., 10., 330., 110.)
        .geometry(&circles);

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
