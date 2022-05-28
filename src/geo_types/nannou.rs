use geo_types::{CoordNum, LineString};
use nannou::draw::Drawing;
use nannou::draw::primitive::{Path, PathStroke};
use nannou::geom::Point2;
use num_traits::real::Real;
// use num::traits::AsPrimitive;

pub trait NannouDrawer<'a, T> {
    fn draw_from_linestring(self, line: LineString<T>) -> Drawing<'a, Path>
        where T: CoordNum, T: Real;
}

impl<'a, T> NannouDrawer<'a, T> for Drawing<'a, PathStroke>
{
    fn draw_from_linestring(self, line: LineString<T>) -> Drawing<'a, Path>
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