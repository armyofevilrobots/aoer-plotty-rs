use std::path::Path;
use geo_types::{coord, Rect};
use aoer_plotty_rs::context::Context;
use aoer_plotty_rs::geo_types::svg::Arrangement;


fn main(){

    let mut ctx = Context::new();
    ctx.stroke("black")
        .fill("red")
        .pen(0.5)
        .hatch(Some(45.0))
        // .outline(Some(5.0))
        .poly(vec![(0.0,0.0),
                   (25.0,0.0),
                   (25.0,25.0),
                   (0.0,25.0)],
              vec![]);
    let svg = ctx.to_svg(
        &Arrangement::<f64>::Center(
            Rect::<f64>::new(coord!{x:0.0, y:0.0}, coord!{x:100.0, y:100.0}),
            false))
        .unwrap();
    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();


}