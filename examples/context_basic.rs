use std::path::Path;
use geo_types::{coord, Rect};
use nalgebra::Affine2;
use aoer_plotty_rs::context::Context;
use aoer_plotty_rs::geo_types::svg::Arrangement;


fn main(){

    let mut ctx = Context::new();
    ctx.stroke("black")
        .fill("red")
        .pen(0.5)
        .outline(Some(5.0))
        // .hatch(Some(45.0))
        .poly(vec![(0.0,0.0),
                   (25.0,0.0),
                   (25.0,25.0),
                   (0.0,25.0)],
              vec![])
        .outline(None)
        .hatch(Some(135.0))
        .stroke("blue")
        .fill("yellow")
        .circle(12.5,12.5, 5.0)
        .push()
        .hatch(Some(180.0))
        .stroke("red")
        .fill("green")
        .circle(17.5,12.5,2.5)
        .pop().unwrap()
        .hatch(None)
        .clip(true)
        .circle(7.5,12.5,2.5)
        .clip(false)
        .stroke("brown")
        .pen(1.0)
        .line(0.0, 0.0, 3.0, 3.0)
        .pen(0.1)
        .outline(Some(1.0))
        .stroke("pink")
        .line(3.0, 0.0, 0.0, 3.0)
        .stroke("purple")
        .spline(&vec![(0.0, 25.0), (0.0, 25.0), (10.0, 20.0), (20.0,25.0), (25.0, 25.0)],
        8, 0.5)
        .push()  // Prepare for this transformation stuff...
        .transform(Some(
            &(Context::translate_matrix(25.0, 25.0)
                * Context::rotate_matrix(45.0)
                * Context::scale_matrix(1.0, 0.5)
            ))) // Holy crap we can multiply these?! ;)
        .stroke("cyan")
        .circle(0.0, 0.0, 8.0)
        .pop().unwrap() // We're back to purple and regular coords
    ;

    let svg = ctx.to_svg(
        &Arrangement::<f64>::unit(
            &Rect::<f64>::new(coord!{x:0.0, y:0.0}, coord!{x:100.0, y:100.0})))
        .unwrap();
    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();


}