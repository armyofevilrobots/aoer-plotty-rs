use std::path::Path;
use geo_types::{coord, Coordinate, Geometry, Rect};
use aoer_plotty_rs::prelude::{Arrangement, LineHatch, NoHatch};
use rand::prelude::*;
use aoer_plotty_rs::context::Context;

/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/piet-mondrian/

/// Splits a square at an x value
fn split_on_x(square: &Rect<f64>, x: f64) -> Vec<Rect<f64>> {
    vec![
        Rect::new(
            coord! {x: square.min().x, y: square.min().y},
            coord! {x: x, y: square.max().y}),
        Rect::new(
            coord! {x: x, y: square.min().y},
            coord! {x: square.max().x, y: square.max().y}),
    ]
}

/// Splits a square at a y value
fn split_on_y(square: &Rect<f64>, y: f64) -> Vec<Rect<f64>> {
    vec![
        Rect::new(
            coord! {x: square.min().x, y: square.min().y},
            coord! {x: square.max().x, y: y}),
        Rect::new(
            coord! {x: square.min().x, y: y},
            coord! {x: square.max().x, y: square.max().y}),
    ]
}

/// Iterates the squares, splitting them where appropriate
fn split_squares_at(squares: &mut Vec<Rect<f64>>, coord: Coordinate<f64>, rng: &mut rand::rngs::SmallRng) {
    for i in (0..squares.len()).rev() {
        let square = squares[i].clone();
        if coord.x > square.min().x &&
            coord.x < square.max().x &&
            rng.gen::<f64>() > 0.5
        {
            squares.remove(i);
            squares.append(&mut split_on_x(&square, coord.x));
        }
        if coord.y > square.min().y &&
            coord.y < square.max().y &&
            rng.gen::<f64>() > 0.5 {
            squares.remove(i);
            squares.append(&mut split_on_y(&square, coord.y));
        }
    }
}

fn main() {
    let pen_width = 0.5f64;
    let size: i32 = 224;
    let square_count = 8;
    let white = "#F2F5F1";
    let colors = vec!["#D40920", "#1356A2", "#F7D842"];
    let square_weight = 2.0f64; // How thick the dividing lines are

    // We're using a static random generator here so that our SVG files
    // don't get regenerated every time we run the examples.
    let mut rng = SmallRng::seed_from_u64(1234567);
    let mut ctx = Context::new();

    // Define our viewbox/canvas (in mm)
    let viewbox = Rect::new(
        coord! {
            x:0f64,
            y:0f64},
        coord! {
            x: f64::from(size),
            y: f64::from(size)});

    let mut squares: Vec<Rect<f64>> = vec![
        Rect::new(coord! {x:0.0, y:0.0},
                  coord! {x: f64::from(size), y: f64::from(size)})];

    // split_squares_at(&mut squares, coord!{x: f64::from(size)/2.0, y: -1.0});
    // split_squares_at(&mut squares, coord!{x:-1.0, y: f64::from(size)/2.0});
    for i in (0..(size)).step_by((size / square_count) as usize) {
        split_squares_at(&mut squares, coord! {x: f64::from(i), y: -1.0}, &mut rng);
        split_squares_at(&mut squares, coord! {x:-1.0, y: f64::from(i)}, &mut rng);
    }

    // Create an array of the same length as the squares array, and fill it with "white"
    let mut square_colors: Vec<&str> = squares.iter()
        .map(|_s| white.clone())
        .collect();

    for color in colors.clone() {
        // Note, this might overlap an existing colored square. More often than you'd think actually.
        square_colors[rng.gen_range(0..squares.len())] = color;
    }

    // Set the default stroke/hatch/pen.
    ctx.stroke("black")
        .hatch(45.0)
        .pen(pen_width);

    // for square in squares {
    for i in 0..squares.len() {
        let square = squares[i].clone();
        let color = square_colors[i].clone();

        // Don't bother hatching if it's white.
        if color != white {
            ctx.pattern(LineHatch::gen());
        } else {
            ctx.pattern(NoHatch::gen());
        }

        // And set the fill color.
        ctx.fill(color)
            .rect(
                square.min().x + square_weight, square.min().y + square_weight,
                square.max().x - square_weight, square.max().y - square_weight,
            );

        // Now we just draw the rest of the outlines, inside to outside, no fill.
        ctx.pattern(NoHatch::gen());
        let mut remaining_width = square_weight - (pen_width / 2.0);
        let s = squares[i].clone();
        while remaining_width >= 0.0 {
            ctx.geometry(&Geometry::Rect(Rect::new(
                coord! {x: &s.min().x+remaining_width, y: &s.min().y+remaining_width},
                coord! {x: &s.max().x-remaining_width, y: &s.max().y-remaining_width},
            )));
            remaining_width -= pen_width;
        }
    }


    // The unit arrangement just means that we'll draw what we mean, where we mean to.
    let arrangement = Arrangement::unit(&viewbox);

    let document = ctx.to_svg(&arrangement).unwrap();

    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &document).unwrap();
}
