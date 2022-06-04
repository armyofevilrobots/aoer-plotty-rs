use geo_types::{Point, CoordNum, Polygon, MultiPolygon, MultiLineString, Rect, LineString, coord};
use geo::bounding_rect::BoundingRect;
use num_traits::real::Real;

/// # HatchPattern
/// Returns a MultiLineString which draws a hatch pattern which fills the entire
/// bbox area. Set up as a trait so the developer can add new patterns at their
/// leisure.
pub trait HatchPattern<T>
    where T: CoordNum, T: Real {
    fn generate(&self, bbox: &Rect<T>, scale: T) -> MultiLineString<T>;
}

/// # Hatch
/// Trait which can be implemented for various geo_types, to provide fills
/// on their interiors.
pub trait Hatch<P, T>
    where P: HatchPattern<T>,
          T: CoordNum, T: Real {
    fn hatch(&self, pattern: P, angle: T, scale: T, inset: T) -> MultiLineString<T>;
}

struct LineHatch {}

impl<T> HatchPattern<T> for LineHatch
    where T: CoordNum,
          T: Real,
          T: std::ops::AddAssign {
    fn generate(&self, bbox: &Rect<T>, scale: T) -> MultiLineString<T> {
        let min = bbox.min();
        let max = bbox.max();
        let mut y = min.y;
        let mut count = 0u32;
        // MultiLineString::<T>::new(
        let mut lines: Vec<LineString<T>> = vec![];
        while y < max.y {
            y += scale;
            count += 1;
            if count % 2 == 0 {
                lines.push(LineString::<T>::new(vec![
                    coord! {x: min.x, y: y},
                    coord! {x: max.x, y: y},
                ]));
            } else {
                lines.push(LineString::<T>::new(vec![
                    coord! {x: max.x, y: y},
                    coord! {x: min.x, y: y},
                ]));
            }
        };
        MultiLineString::<T>::new(lines)
    }
}


#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_box_hatch(){
        let rect = Rect::<f64>::new(coord!{x: 0.0, y: 0.0}, coord!{x: 100.0, y: 100.0});
        let hatch_lines = LineHatch{}.generate(&rect, 10.0);
        println!("LINES HATCHED: {:?}", hatch_lines);
    }
}