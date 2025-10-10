/// Kinda weird that arc features are missing from geo_types, but ok, here is one.
use geo_types::{Geometry, LineString, Point, Polygon, coord};
use num_traits::FromPrimitive;
use std::f64::consts::PI;

/// Just a regular polygon, but for some reason using compass degrees
pub fn regular_poly(sides: usize, x: f64, y: f64, radius: f64, rotation: f64) -> Geometry<f64> {
    // all the way around to the start again, and hit the first point twice to close it.
    if sides < 3 {
        return Geometry::Point(Point::new(x, y));
    }

    Geometry::Polygon(Polygon::new(
        LineString::new(
            (0..=sides)
                .map(|i| {
                    let angle = rotation - PI / 2.0
                        + (f64::from(i as i32) / f64::from(sides as i32)) * (2.0 * PI);
                    coord! {x: x+angle.cos() * radius, y: y+angle.sin() * radius}
                })
                .collect(),
        ),
        vec![],
    ))
}

/// Exactly the same, except uses regular mathematical orientation, radians
pub fn regular_poly_native(
    sides: usize,
    x: f64,
    y: f64,
    radius: f64,
    radians: f64,
) -> Geometry<f64> {
    // all the way around to the start again, and hit the first point twice to close it.
    if sides < 3 {
        return Geometry::Point(Point::new(x, y));
    }

    Geometry::Polygon(Polygon::new(
        LineString::new(
            (0..=sides)
                .map(|i| {
                    let angle =
                        radians + (f64::from(i as i32) / f64::from(sides as i32)) * (2.0 * PI);
                    coord! {x: x+angle.cos() * radius, y: y+angle.sin() * radius}
                })
                .collect(),
        ),
        vec![],
    ))
}

/// Draw a regular polygon with enough sides that nobody can tell the difference.
pub fn circle(x0: f64, y0: f64, radius: f64) -> Geometry<f64> {
    let radius = radius.abs();
    let sides = 1000.min(32.max(usize::from_f64(radius).unwrap_or(1000) * 4));
    regular_poly(sides, x0, y0, radius, 0.0)
}

/// Draw an arc, centered on a point. Degrees are compass degrees again, sorry.
pub fn arc_center(x0: f64, y0: f64, radius: f64, deg0: f64, deg1: f64) -> LineString<f64> {
    let radius = radius.abs();
    // Clamp the angle.
    let deg0 = PI * ((deg0 % 360.0) / 180.0);
    let deg1 = PI * ((deg1 % 360.0) / 180.0);
    let (deg0, deg1) = if deg0 > deg1 {
        (deg1, deg0)
    } else {
        (deg0, deg1)
    };
    let sides = 1000.min(32.max(usize::from_f64(radius).unwrap_or(1000) * 4));
    let segments = (deg1 - deg0) * f64::from(sides as i32).floor();
    let seg_size = (deg1 - deg0) / segments;
    let mut ls = LineString::<f64>::new(vec![]);
    let mut angle = deg0;
    for _segment in 0..(segments as i32) {
        ls.0.push(coord! {x: x0+radius*angle.sin(), y: y0+radius*angle.cos()});
        angle += seg_size;
    }
    if deg1 - angle > 0.0 {
        ls.0.push(coord! {x: x0+radius*deg1.sin(), y: y0+radius*deg1.cos()});
    }
    ls
}

#[cfg(test)]
mod test {
    use super::arc_center;

    #[test]
    fn test_arc_c() {
        let _arc = arc_center(0.0f64, 0.0f64, 10.0f64, 90.0f64, 135f64);
        // println!("ARC: {:?}", &arc);
    }
}
