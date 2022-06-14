use std::ops::Div;
use geo::bounding_rect::BoundingRect;
use geo::dimensions::HasDimensions;
use geo::translate::Translate;
use geo_types::{Coordinate, CoordNum, LineString, MultiLineString, Point, Polygon, Rect};
use nalgebra::{Affine2, RealField, Similarity2, Point2 as NPoint2, Matrix3};
use num_traits::{AsPrimitive, Float};
use num_traits::real::Real;
use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;


/// An arrangement is a plan for transformation of an SVG
pub enum Arrangement<T>
    where T: Real,
          T: CoordNum,
          T: RealField {
    Center(Rect<T>, bool),
    FitCenter(Rect<T>, bool),
    Transform(Affine2<T>),
}

pub trait ToSvg<T>
    where T: CoordNum,
          T: Real,
          T: RealField {
    /// Given an [Arrangement] as a transformation strategy, transform the geometry to
    /// fit the bounds, or just run the transformation without bounds if None
    fn arrange(self, arrangement: Arrangement<T>) -> Self;

    /// Utility function returns a viewbox tuple for the geometry.
    /// Should be used AFTER calling arrange on the geo.
    fn viewbox(&self) -> Option<(T, T, T, T)>;

    /// Convert the Geometry into an SVG Path
    fn to_path(&self) -> Path;
}

impl<T> ToSvg<T> for MultiLineString<T>
    where T: CoordNum,
          T: Real,
          T: RealField,
          T: Float,
          T: AsPrimitive<T> {
    fn arrange(self, arrangement: Arrangement<T>) -> Self {
        let gbox = self.bounding_rect()
            .expect("Arranging geometry with no dimensions.");
        let transformation = match arrangement {
            Arrangement::Transform(affine) => affine,
            Arrangement::Center(bounds, invert) => {
                let bcenter = bounds.min() + (bounds.max() - bounds.min()).div(T::from(2.0).unwrap()); // / (2.0 as T);
                let gcenter = gbox.min() + (gbox.max() - gbox.min()).div(T::from(2.0).unwrap());
                let delta = bcenter - gcenter;
                let tx = Affine2::from_matrix_unchecked(
                    Matrix3::<T>::new(
                        T::from(1.0).unwrap() , T::zero(), delta.x as T,
                        T::zero(), T::one(), delta.y as T,
                        T::zero(), T::zero(), T::one(),
                    )
                );
                if invert{
                    Affine2::from_matrix_unchecked(Matrix3::<T>::new(
                        T::from(1.0).unwrap() , T::zero(), T::zero(),
                        T::zero(), -T::one(),  bounds.height(),
                        T::zero(), T::zero(), T::one(),
                    )) * tx
                }else{
                    tx
                }
            }
            Arrangement::FitCenter(bounds, invert) => {
                let scale = <T as Real>::min((bounds.width()/gbox.width()), (bounds.height()/gbox.height()));
                let bcenter = bounds.min() + (bounds.max() - bounds.min()).div(T::from(2.0).unwrap()); // / (2.0 as T);
                let gcenter = gbox.center() * scale; // This is post scaling now.
                let delta = bcenter - gcenter;
                let tx = Affine2::from_matrix_unchecked(
                    Matrix3::new(
                        scale, T::zero(), delta.x,
                        T::zero(), scale, delta.y,
                        T::zero(), T::zero(), T::one(),
                    )
                );
                if invert{
                    Affine2::from_matrix_unchecked(Matrix3::<T>::new(
                        T::from(1.0).unwrap() , T::zero(), T::zero(),
                        T::zero(), -T::one(),  bounds.height(),
                        T::zero(), T::zero(), T::one(),
                    )) * tx
                }else{
                    tx
                }
            }
        };
        let linestrings: Vec<LineString<T>> = self.iter().map(|linestring| {
            // linestring.clone()
            linestring.coords().map(|coord| {
                let pt = transformation * NPoint2::<T>::new(coord.x, coord.y);
                Coordinate::<T>::from((pt.x, pt.y))
            }).collect()
        }).collect();
        MultiLineString::<T>::new(linestrings)
    }

    fn viewbox(&self) -> Option<(T, T, T, T)> {
        let bounds = self.bounding_rect()?;
        Some((bounds.min().x, bounds.min().y, bounds.max().x, bounds.max().y))
    }

    fn to_path(&self) -> Path {
        todo!()
    }
}

#[cfg(test)]
pub mod test {
    use std::f32::consts::PI as PI_F32;
    use std::str::FromStr;
    use geo_types::{coord, LineString, MultiLineString};
    use nalgebra::{Vector2, Affine2, Matrix3};
    use wkt::Wkt;
    use super::*;

    #[test]
    fn test_load_wkt() {
        let geoms: Polygon::<f64> = Polygon::try_from(
            Wkt::<f64>::from_str("POLYGON ((350 100, 450 450, 150 400, 100 200, 350 100), (200 300, 350 350, 300 200, 200 300))")
                .expect("Failed to load WKT"))
            .expect("Failed to load box");
    }

    #[test]
    fn test_arrange_center(){
        let mls = MultiLineString::new(
            vec![LineString::new(
                vec![
                    coord! {x: 0.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 100.0f64},
                    coord! {x: 100.0f64, y: 100.0f64},
                    coord! {x: 100.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 0.0f64},
                ])]);
        let txmls = mls.arrange(
            Arrangement::Center(
                Rect::new(coord!{x:0f64, y:0f64}, coord!{x:400f64, y:400f64}),
                false));
        println!("TXMLS when centered is: {:?}", txmls);
        assert_eq!(txmls.bounding_rect()
            .expect("Should have been able to get brect")
            .center(),
        coord!{x: 200.0f64, y:200.0f64});
        assert_eq!(txmls.bounding_rect()
            .expect("Should have been able to get brect for mlines")
            .width(), 100.0f64);
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect for mlines")
                       .height(), 100.0f64);
        assert_eq!(txmls.bounding_rect()
            .expect("Couldn't get bounding rect on second attempt?")
            .center(), coord!{x: 200.0, y:200.0});
    }

    #[test]
    fn test_arrange_fit_center(){
        let mls = MultiLineString::new(
            vec![LineString::new(
                vec![
                    coord! {x: 0.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 100.0f64},
                    coord! {x: 100.0f64, y: 100.0f64},
                    coord! {x: 100.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 0.0f64},
                ])]);
        let txmls = mls.arrange(
            Arrangement::FitCenter(
                Rect::new(coord!{x:0f64, y:0f64}, coord!{x:400f64, y:400f64}),
                false));
        println!("TXMLS when centered is: {:?}", txmls);
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect")
                       .center(),
                   coord!{x: 200.0f64, y:200.0f64});
    }

    #[test]
    fn test_arrange_fit_center_invert(){
        let mls = MultiLineString::new(
            vec![LineString::new(
                vec![
                    coord! {x: 0.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 100.0f64},
                    coord! {x: 100.0f64, y: 100.0f64},
                    coord! {x: 100.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 0.0f64},
                ])]);
        let txmls = mls.arrange(
            Arrangement::FitCenter(
                Rect::new(coord!{x:0f64, y:0f64}, coord!{x:400f64, y:400f64}),
                true));
        println!("TXMLS when centered is: {:?}", txmls);
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect")
                       .center(),
                   coord!{x: 200.0f64, y:200.0f64});
    }

    #[test]
    fn test_arrange_fit_center_invert_2(){
        let mls = MultiLineString::new(
            vec![LineString::new(
                vec![
                    coord! {x: 0.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 10.0f64},
                    coord! {x: 10.0f64, y: 10.0f64},
                    coord! {x: 10.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 0.0f64},
                ]), LineString::new(
                vec![
                    coord! {x: 370.0f64, y: 380.0f64},
                    coord! {x: 380.0f64, y: 380.0f64},
                    coord! {x: 380.0f64, y: 370.0f64},
                    coord! {x: 370.0f64, y: 370.0f64},
                    coord! {x: 380.0f64, y: 380.0f64},
                ]
            )]);
        let txmls = mls.arrange(
            Arrangement::FitCenter(
                Rect::new(coord!{x:0f64, y:0f64}, coord!{x:400f64, y:400f64}),
                true));
        println!("TXMLS fitcenter2 when centered is: {:?}", txmls);
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect")
                       .center(),
                   coord!{x: 200.0f64, y:200.0f64});
        assert_eq!(txmls.bounding_rect().unwrap().width(), 400_f64);
        assert_eq!(txmls.bounding_rect().unwrap().height(), 400_f64);
    }


    #[test]
    fn test_arrange_mls_arbitrary() {
        let mls = MultiLineString::new(
            vec![LineString::new(
                vec![
                    coord! {x: 0.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 100.0f64},
                    coord! {x: 100.0f64, y: 100.0f64},
                    coord! {x: 100.0f64, y: 0.0f64},
                    coord! {x: 0.0f64, y: 0.0f64},
                ])]);
        let txmls = mls.arrange(
            Arrangement::Transform(Affine2::from_matrix_unchecked(
                Matrix3::new(
                    1.0, 0.0, 300.0,
                    0.0, 1.0, 0.0,
                    0.0, 0.0, 1.0,
                )
            )));
        println!("TXMLS IS {:?}", txmls);
        assert_eq!(
            txmls.0[0].coords()
                .zip(
                    LineString::new(
                        vec![
                            coord! {x: 300.0f64, y: 0.0f64},
                            coord! {x: 300.0f64, y: 100.0f64},
                            coord! {x: 400.0f64, y: 100.0f64},
                            coord! {x: 400.0f64, y: 0.0f64},
                            coord! {x: 300.0f64, y: 0.0f64},
                        ]).coords())
                .filter(|&(left, right)| {
                    println!("LEFT: {:?} RIGHT: {:?}", left, right);
                    left == right
                })
                .count(),
            5);
    }
}
