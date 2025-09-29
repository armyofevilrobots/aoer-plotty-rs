use aoer_plotty_rs::context::typography::Typography;
use aoer_plotty_rs::context::Context;
use aoer_plotty_rs::elements::{point_field::*, FieldToVoronoi};
use aoer_plotty_rs::elements::{ContourField, ContourFieldBuilder};
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::prelude::NoHatch;
use aoer_plotty_rs::util::AnythingToGeo;
use geo::Coord;
use geo_types::{coord, Rect};
use std::path::Path;

fn main() {
    let rect = Rect::new(Coord { x: 10., y: 10. }, Coord { x: 490., y: 490. });
    let mut contour_field = ContourFieldBuilder::new()
        .bounds(rect.clone())
        .xy_step(1.123)
        .seed(5)
        .perlin_scale(0.01)
        .thresholds(
            (-20..20)
                .map(|x| x as f64 / 20. + 0.001)
                .collect::<Vec<f64>>(),
        )
        .build();

    let cf_contours = contour_field.contours();
    let cf_isobands = contour_field.isobands();
    let cf_isolines = contour_field.isolines();

    let mut ctx = Context::new();
    ctx.no_fill()
        .stroke("black")
        .pen(0.5)
        .pattern(NoHatch::gen())
        .no_fill()
        .geometry(&rect.into())
        .pen(0.2)
        .typography(
            &"Contour Field".to_string(),
            510.,
            510.,
            &Typography::new().size(5.),
        );

    /*
    for geo_pf in cf_contours {
        for poly in geo_pf {
            ctx.geometry(&geo::Geometry::Polygon(poly));
        }
    }
    */

    for mls in cf_isolines {
        ctx.geometry(&mls.into());
    }

    let svg = ctx
        .to_svg(&Arrangement::<f64>::unit(&Rect::<f64>::new(
            coord! {x:0.0, y:0.0},
            coord! {x:500.0, y:550.0},
        )))
        .unwrap();
    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
