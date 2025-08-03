use geo::coord;
use geo::map_coords::MapCoords;
use geo::Coord;
use geo_types::CoordNum;
use geo_types::Geometry;
use nalgebra::Matrix3;
use nalgebra::{Affine2, Point2 as NPoint2, RealField};
use num_traits::real::Real;
use num_traits::{Float, Num};

/// Helper to create a scaling matrix
pub fn scale_matrix<T>(sx: T, sy: T) -> Affine2<T>
where
    T: RealField,
    T: Float,
{
    Affine2::from_matrix_unchecked(Matrix3::<T>::new(
        sx,
        T::zero(),
        T::zero(),
        T::zero(),
        sy,
        T::zero(),
        T::zero(),
        T::zero(),
        T::one(),
    ))
}

/// Helper to create a translation matrix
pub fn translate_matrix<T>(tx: T, ty: T) -> Affine2<T>
where
    T: RealField,
    T: Float,
{
    Affine2::from_matrix_unchecked(Matrix3::<T>::new(
        T::one(),
        T::zero(),
        tx,
        T::zero(),
        T::one(),
        ty,
        T::zero(),
        T::zero(),
        T::one(),
    ))
}

/// Unlike in operations/context, this is standard coordinate
/// and orientation system.
pub fn rotate_matrix<T>(radians: T) -> Affine2<T>
where
    T: RealField,
    T: Float,
{
    Affine2::from_matrix_unchecked(Matrix3::<T>::new(
        Float::cos(radians),
        Float::sin(radians).neg(),
        T::zero(),
        Float::sin(radians),
        Float::cos(radians),
        T::zero(),
        T::zero(),
        T::zero(),
        T::one(),
    ))
}

/// Unit matrix. Basically a no-op
pub fn unit_matrix() -> Affine2<f64> {
    Affine2::from_matrix_unchecked(Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0))
}

pub trait TransformGeometry<T: CoordNum>
where
    T: RealField,
    T: Copy,
    T: Real,
    T: CoordNum,
    T: Num,
    T: Float,
    T: RealField,
{
    //fn xform_coord(xy: &(T, T), affine: &Affine2<T>) -> (T, T);
    fn xform_coord(xy: &Coord<T>, affine: &Affine2<T>) -> Coord<T>;
    fn transformed(&self, affine: &Affine2<T>) -> Geometry<T>;
}

impl<T> TransformGeometry<T> for Geometry<T>
where
    T: Num,
    T: RealField,
    T: Float,
{
    /// Helper to transform geometry when we have an affine transform set.
    /*
     * fn xform_coord((x, y): &(T, T), affine: &Affine2<T>) -> (T, T) {
        let out = affine * NPoint2::new(*x, *y);
        (out.x, out.y)
    }
    */
    fn xform_coord(xy: &Coord<T>, affine: &Affine2<T>) -> Coord<T> {
        let out = affine * NPoint2::new(xy.x, xy.y);
        coord!(x: out.x, y: out.y)
    }

    fn transformed(&self, affine: &Affine2<T>) -> Geometry<T> {
        self.map_coords(|xy| Self::xform_coord(&xy, affine))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::PointDistance;
    use std::f64::consts::PI;

    use super::*;

    #[test]
    fn test_translate() {
        let tx = translate_matrix(10., 5.);
        let source = geo_types::Point::<f64>::new(25., 25.);
        let dest = geo_types::Point::<f64>::new(35., 30.);
        let result = Geometry::Point(source).transformed(&tx);
        if let Geometry::Point(out) = result {
            assert!((dest - out).length() < 1e-8);
        } else {
            todo!()
        }
    }

    #[test]
    fn test_simple_rotate() {
        for (radians, source, destination) in vec![
            (
                PI / 2.,
                geo_types::Point::new(1., 0.),
                geo_types::Point::new(0., 1.),
            ),
            (
                PI,
                geo_types::Point::new(1., 0.),
                geo_types::Point::new(-1., 0.),
            ),
            (
                PI / 2.,
                geo_types::Point::new(0., 1.),
                geo_types::Point::new(-1., 0.),
            ),
            (
                PI / 4.,
                geo_types::Point::new(1., 0.),
                geo_types::Point::new(0.7071067811865476, 0.7071067811865476),
            ),
            (
                -PI / 4.,
                geo_types::Point::new(1., 0.),
                geo_types::Point::new(0.7071067811865476, -0.7071067811865476),
            ),
        ] {
            let rotation_matrix = rotate_matrix(radians);
            let result = Geometry::Point(source);
            if let Geometry::Point(out) = result.transformed(&rotation_matrix) {
                let distance = (destination - out).length();
                // println!(
                //     "Rotation of pt {:?} by {}\nshould have destination {:?} to actual {:?}\nFound distance {}\n\n",
                //     &source, &radians, &destination, &out, &distance
                // );
                assert!(distance < 1e-8);
            } else {
                todo!()
            }
        }
    }
}
