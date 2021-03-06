use std::f64::consts::PI;
use std::path::Path;
use aoer_plotty_rs::prelude::{Arrangement, LineHatch};
use aoer_plotty_rs::context::Context;

/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/hours-of-dark/


fn main() {
    let pen_width = 0.3;
    let cols = 23;
    let rows = 16;
    let size = (cols * rows) / 2;
    let days = 365;
    let cellw = f64::from(size) / f64::from(cols);
    let cellh = f64::from(size) / f64::from(rows);

    let mut ctx = Context::new();

    // Define our viewbox/canvas (in mm)
    let viewbox = Context::viewbox(
        0.0, 0.0,
        f64::from(size), f64::from(size));


    // Set the default stroke/hatch/pen.
    ctx.stroke("black")
        .hatch(45.0)
        .pen(pen_width)
        .pattern(LineHatch::gen())
        .fill("black");


    for day in 0..days {
        let col = f64::from(day / rows);
        let row = f64::from(day % rows);
        let x = col * cellw;
        let y = row * cellh;
        let w = 2.0f64;
        let h = 30.0f64;

        let phi = (f64::from(day) / f64::from(days)) * PI;
        let theta = phi.sin() * 80.0 + 45.0;

        // We scale w and h by the scale here because of implementation differences
        // between JS canvas context and our context.
        let scale = phi.cos().abs() * 2.0 + 1.0;
        let w = scale * w;
        let h = scale * h;



        ctx.push()
            .transform(Some(&(Context::translate_matrix(x, y))))
            .mask_box(cellw * -0.5, cellh * -0.5,
                      cellw * 0.5, cellh * 0.5)
            .transform(Some(&(Context::translate_matrix(x, y)*Context::rotate_matrix(theta))))
            .hatch(-90.0 + theta)
            .rect(w * -0.5, h * -0.5, w * 0.5, h * 0.5)
            .pop().expect("Somehow lost track of my internal stack");
    }


    // The unit arrangement just means that we'll draw what we mean, where we mean to.
    let arrangement = Arrangement::unit(&viewbox);
    // let arrangement = Arrangement::FitCenterMargin(235.0-f64::from(size)/2.0,
    //                                                viewbox.clone(), false);

    let document = ctx.to_svg(&arrangement).unwrap();

    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &document).unwrap();
}
