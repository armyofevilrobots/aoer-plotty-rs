use aoer_plotty_rs::context::typography::Typography;
use aoer_plotty_rs::context::Context;
use aoer_plotty_rs::elements::{point_field::*, ToVoronoi};
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::prelude::NoHatch;
use aoer_plotty_rs::util::AnythingToGeo;
use geo::{Coord, Point, Scale};
use geo_types::{coord, Rect};
use image::{DynamicImage, GrayImage};
use std::path::Path;
// use voronoice::VoronoiBuilder;

fn main() {
    let image =
        image::ImageReader::open(Path::new("images/aoer_logo.png")).expect("Failed to open image.");
    let img_raw = image.decode().expect("Failed to decode image.");
    let img_raw =
        image::imageops::resize(&img_raw, 150, 150, image::imageops::FilterType::CatmullRom);
    let img_raw: DynamicImage = img_raw.into();
    let img_raw = img_raw.grayscale();

    let rect_pf = Rect::new(Coord { x: -30., y: -30. }, Coord { x: 330., y: 330. });
    let mut pf = BitmapPointField::new(img_raw.into(), 0.5, 0.5); //.with_dither(DitherType::FloydSteinberg);
    println!("B:{:?}", pf.bounds());
    // Use a shortcut to create the voronoi cells quickly
    println!("Start voronoi calc...");
    let vn_pf = pf.to_voronoi();
    println!("End voronoi calc.");
    // Gimme mul
    println!("Start to_geo");
    let geo_pf = vn_pf.to_geo();
    println!("Start geo scale");
    let geo_pf = geo_pf.scale_around_point(2., 2., Coord { x: 0.0, y: 0.0 });
    println!("OK, drawing!");

    let mut ctx = Context::new();
    // First the voronoi
    ctx.no_fill()
        .geometry(&rect_pf.into())
        .pen(0.5)
        .no_fill()
        .stroke("black")
        .pen(0.2)
        .pattern(NoHatch::gen())
        // .typography(
        //     &"Voronoi over Perlin".to_string(),
        //     510.,
        //     510.,
        //     &Typography::new().size(5.),)
        ;
    if let geo::Geometry::MultiPolygon(polys) = geo_pf {
        for poly in polys {
            ctx.geometry(&geo::Geometry::Polygon(poly));
        }
    }
    println!("Done draw.");
    // for p in pf {
    //     println!("C:{:?}", p);
    //     ctx.circle(p.0.x, p.0.y, 4.);
    // }
    // }

    let svg = ctx
        .to_svg(&Arrangement::<f64>::unit(&Rect::<f64>::new(
            coord! {x:0.0, y:0.0},
            coord! {x:330.0, y:330.0},
        )))
        .unwrap();
    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
