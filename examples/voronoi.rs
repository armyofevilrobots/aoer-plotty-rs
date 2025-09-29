use aoer_plotty_rs::context::Context;
use aoer_plotty_rs::elements::{point_field::*, BBWrapper, FieldToVoronoi};
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::util::AnythingToGeo;
use geo::{Coord, Geometry};
use geo_types::{coord, Rect};
use std::path::Path;
// use voronoice::VoronoiBuilder;

fn main() {
    let rect = Rect::new(Coord { x: 10., y: 10. }, Coord { x: 490., y: 490. });
    let mut pf = HaltonPointFieldBuilder::new().bounds(rect.clone()).build();
    let vn = pf.to_voronoi(3000);
    let geo = vn.to_geo();
    let mut ctx = Context::new();
    ctx.no_fill()
        .geometry(&rect.into())
        .pen(0.5)
        .no_fill()
        .stroke("black");
    if let geo::Geometry::MultiPolygon(polys) = geo {
        for poly in polys {
            ctx.geometry(&geo::Geometry::Polygon(poly));
        }
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
