use aoer_plotty_rs::context::Context;
use aoer_plotty_rs::context::typography::Typography;
use aoer_plotty_rs::elements::{ToVoronoi, point_field::*};
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::prelude::NoHatch;
use aoer_plotty_rs::util::AnythingToGeo;
use geo::Coord;
use geo_types::{Rect, coord};
use std::path::Path;
// use voronoice::VoronoiBuilder;

fn main() {
    // The bounds of the voronoi area
    let rect_hf = Rect::new(Coord { x: 10., y: 10. }, Coord { x: 490., y: 490. });
    let rect_pf = Rect::new(Coord { x: 510., y: 10. }, Coord { x: 990., y: 490. });
    // Create the point field that defines the voronoi cell "centers"
    let mut hf = HaltonPointFieldBuilder::new()
        .bounds(rect_hf.clone())
        .build();
    let mut pf = PerlinPointFieldBuilder::new()
        .bounds(rect_pf.clone())
        .coord_scale(0.01)
        .point_prob(0.1)
        .build();
    // Use a shortcut to create the voronoi cells quickly
    let vn_pf = pf.take(3000).to_voronoi();
    let vn_hf = hf.take(3000).to_voronoi();
    // Gimme mul
    let geo_hf = vn_hf.to_geo();
    let geo_pf = vn_pf.to_geo();

    let mut ctx = Context::new();
    // First the voronoi
    ctx.no_fill()
        .geometry(&rect_pf.into())
        .pen(0.5)
        .no_fill()
        .stroke("black")
        .pen(0.2)
        .pattern(NoHatch::gen())
        .typography(
            &"Voronoi over Perlin".to_string(),
            510.,
            510.,
            &Typography::new().size(5.),
        );
    if let geo::Geometry::MultiPolygon(polys) = geo_pf {
        for poly in polys {
            ctx.geometry(&geo::Geometry::Polygon(poly));
        }
    }

    // Then the Halton Field
    ctx.no_fill()
        .pen(0.5)
        .geometry(&rect_hf.into())
        .no_fill()
        .stroke("black")
        .pen(0.2)
        .pattern(NoHatch::gen())
        .typography(
            &"Voronoi over Halton".to_string(),
            10.,
            510.,
            &Typography::new().size(5.),
        );
    if let geo::Geometry::MultiPolygon(polys) = geo_hf {
        for poly in polys {
            ctx.geometry(&geo::Geometry::Polygon(poly));
        }
    }

    let svg = ctx
        .to_svg(&Arrangement::<f64>::unit(&Rect::<f64>::new(
            coord! {x:0.0, y:0.0},
            coord! {x:1000.0, y:600.0},
        )))
        .unwrap();
    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
