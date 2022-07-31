use std::path::Path;
use geo_types::{coord, Rect};
use aoer_plotty_rs::context::Context;
use aoer_plotty_rs::context::typography::{TextAlignment, Typography};
use aoer_plotty_rs::geo_types::svg::Arrangement;
use aoer_plotty_rs::prelude::NoHatch;


fn main(){

    let mut ctx = Context::new();
    let mut typ = Typography::new();
    typ.size(2.0);
    typ.close(true);

    ctx.stroke("black")
        .fill("red")
        .pen(0.5)
        .pattern(NoHatch::gen())
        // .pattern(LineHatch::gen())
        // .typography(&"i".to_string(), 50.0, 50.0, &typ);
        .typography(&"Left".to_string(), 50.0, 50.0, &typ);
    typ.align(TextAlignment::Right);
    ctx.typography(&"Right".to_string(), 50.0, 90.0, &typ);
    typ.align(TextAlignment::Center);
    ctx.typography(&"Center".to_string(), 50.0, 70.0, &typ);

    let svg = ctx.to_svg(
        &Arrangement::<f64>::unit(
            &Rect::<f64>::new(coord!{x:0.0, y:0.0}, coord!{x:100.0, y:100.0})))
        .unwrap();
    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();


}
