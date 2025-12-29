use aoer_plotty_rs::context::Context;
use aoer_plotty_rs::context::typography::Typography;
use aoer_plotty_rs::elements::{ContourField, ContourFieldBuilder};
use aoer_plotty_rs::elements::{ToVoronoi, point_field::*};
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::prelude::NoHatch;
use aoer_plotty_rs::util::AnythingToGeo;
use geo::Coord;
use geo_types::{Rect, coord};
use noise::{
    Abs, BasicMulti, Billow, Fbm, OpenSimplex, Perlin, RidgedMulti, Seedable, Terrace, Worley,
};
use std::path::Path;

fn main() {
    let mut fields = Vec::new();
    let mut field_count = 0;
    let rect = Rect::new(Coord { x: 10., y: 10. }, Coord { x: 190., y: 190. });
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.001)
        .noise(Box::new(Billow::<Perlin>::new(5)))
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();
    fields.push((rect, contour_field, "Billow"));

    // Perlin
    let rect = Rect::new(Coord { x: 200., y: 10. }, Coord { x: 390., y: 190. });
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.01)
        .noise(Box::new(Perlin::new(5)))
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();
    fields.push((rect, contour_field, "Perlin"));

    //Worley (which is %@# cool).
    let rect = Rect::new(Coord { x: 400., y: 10. }, Coord { x: 590., y: 190. });
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.01)
        .noise(Box::new(Worley::new(5)))
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();
    fields.push((rect, contour_field, "Worley"));

    //OpenSimplex
    let rect = Rect::new(Coord { x: 10., y: 220. }, Coord { x: 190., y: 410. });
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.01)
        .noise(Box::new(OpenSimplex::new(5)))
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();
    fields.push((rect, contour_field, "OpenSimplex"));

    //BasicMulti (makes nice clouds)
    let rect = Rect::new(Coord { x: 200., y: 220. }, Coord { x: 390., y: 410. });
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.01)
        .noise(Box::new(BasicMulti::<Perlin>::new(5)))
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();
    fields.push((rect, contour_field, "BasicMulti"));

    //FBM
    let rect = Rect::new(Coord { x: 400., y: 220. }, Coord { x: 590., y: 410. });
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.01)
        .noise(Box::new(Fbm::<Perlin>::new(5)))
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();
    fields.push((rect, contour_field, "FBM"));

    //Abs
    let rect = Rect::new(Coord { x: 10., y: 440. }, Coord { x: 190., y: 630. });
    let perlin = Box::leak(Box::new(Perlin::default()));
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.01)
        .noise(Box::new(Abs::new(Perlin::new(5))))
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();
    fields.push((rect, contour_field, "ABS Perlin"));

    //RidgedMulti
    let rect = Rect::new(Coord { x: 200., y: 440. }, Coord { x: 390., y: 630. });
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.01)
        .noise(Box::new(RidgedMulti::<Perlin>::new(5)))
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();
    fields.push((rect, contour_field, "RidgedMulti"));

    //Terrace
    let rect = Rect::new(Coord { x: 400., y: 440. }, Coord { x: 590., y: 630. });
    let perlin = Box::leak(Box::new(Perlin::default()));
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.01)
        .noise(Box::new(
            Terrace::new(Perlin::new(5))
                .add_control_point(-1.0)
                .add_control_point(-0.5)
                .add_control_point(0.1)
                .add_control_point(1.0),
        ))
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();
    fields.push((rect, contour_field, "Terrace(Perlin)"));

    let mut ctx = Context::new();
    for (rect, mut field, name) in fields {
        println!("Calculating: {}", name);
        let cf_contours = field.contours();
        let cf_isobands = field.isobands();
        let cf_isolines = field.isolines();
        ctx.no_fill()
            .stroke("black")
            .pen(1.)
            .pattern(NoHatch::gen())
            .no_fill()
            .set_mask(&None)
            .geometry(&rect.into())
            .pen(0.2)
            .typography(
                &format!("{}", name).to_string(),
                rect.min().x,
                rect.max().y + 15.,
                &Typography::new().size(5.),
            )
            .set_mask(&Some(rect.to_polygon().into()));
        for mls in cf_isolines {
            ctx.geometry(&mls.into());
        }
    }

    /*
    for geo_pf in cf_contours {
        for poly in geo_pf {
            ctx.geometry(&geo::Geometry::Polygon(poly));
        }
    }
    */

    let svg = ctx
        .to_svg(&Arrangement::<f64>::unit(&Rect::<f64>::new(
            coord! {x:0.0, y:0.0},
            coord! {x:600.0, y:1250.0},
        )))
        .unwrap();
    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
