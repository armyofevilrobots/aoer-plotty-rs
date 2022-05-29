use geo_types::{CoordNum, LineString};
use nannou::draw::Drawing;
use nannou::draw::primitive::{Path, PathStroke};
use nannou::geom::Point2;
use num_traits::real::Real;

/// #NannouDrawer trait
///
/// Useful for taking a polyline and turning it into a Nannou Drawing.
///
/// # Example
/// ```
/// use geo_types::{Coordinate, LineString};
/// use nannou::Draw;
/// use nannou::draw::Drawing;
/// use nannou::lyon::tessellation::{LineCap, LineJoin};
/// use aoer_plotty_rs::prelude::NannouDrawer;
///
/// let line = LineString::new(vec![Coordinate::zero(),
///                            Coordinate{x: 10.0f64, y: 10.0f64}]);
/// let draw = Draw::new();
/// draw.polyline()
///     .stroke_weight(3.0)
///     .caps(LineCap::Round)
///     .join(LineJoin::Round)
///     .polyline_from_linestring(line)
///     .color(nannou::color::NAVY);
/// ```
pub trait NannouDrawer<'a, T> {
    fn polyline_from_linestring(self, line: LineString<T>) -> Drawing<'a, Path>
        where T: CoordNum, T: Real;
}

impl<'a, T> NannouDrawer<'a, T> for Drawing<'a, PathStroke>
{
    fn polyline_from_linestring(self, line: LineString<T>) -> Drawing<'a, Path>
        where T: CoordNum {
        self.points(
            line.coords()
                .map(|p| {
                    Point2::new(p.x.to_f32().unwrap(), p.y.to_f32().unwrap())
                })
                .collect::<Vec<Point2>>())
    }
}

#[cfg(test)]
mod test {
    // use super::*;
}