use geo_types::{Point, CoordNum};
use num_traits::real::Real;

pub trait PointDistance<T: CoordNum> {
    fn distance(&self, other: &Point<T>) -> T;
    fn length(&self) -> T;
}

impl<T> PointDistance<T> for Point<T>
    where T: CoordNum,
          T: Real {
    fn distance(&self, other: &Point<T>) -> T {
        let p = *self - *other;
        p.length()
    }

    fn length(&self) -> T {
        (self.x().powi(2) + self.y().powi(2)).sqrt()
    }
}

#[cfg(test)]

mod tests{
    use super::PointDistance;
    use geo_types::Point;
    use num_traits::abs;

    #[test]
    fn test_length(){
        let p = Point::new(10.0f64, 0.0f64);
        assert!(abs(p.length()-10.0) < 0.0001)
    }

    #[test]
    fn test_distance(){
        let d = Point::new(10.0, 0.0).distance(&Point::new(0.0, 10.0));
        assert!(abs(d-(10.0f64.powi(2)+10.0f64.powi(2)).sqrt()) < 0.0001)
    }
}