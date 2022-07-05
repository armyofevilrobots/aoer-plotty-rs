use std::ops::Div;
use geo::bounding_rect::BoundingRect;
use geo_types::{Coordinate, CoordNum, LineString, MultiLineString, Point, Rect};
use nalgebra::{Affine2, RealField, Point2 as NPoint2, Matrix3};
use num_traits::{AsPrimitive, Float, FromPrimitive, ToPrimitive};
use num_traits::real::Real;
use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;

/// Generic error
#[derive(Debug)]
pub enum SvgCreationError {
    UndefinedViewBox
}

/// An arrangement is a plan for transformation of an SVG
pub enum Arrangement<T>
    where T: Real,
          T: CoordNum,
          T: RealField {
    Center(Rect<T>, bool),
    FitCenter(Rect<T>, bool),
    FitCenterMargin(T, Rect<T>, bool),
    Transform(Rect<T>, Affine2<T>),
}

impl<T: RealField + Float> Arrangement<T> {
    pub fn unit(window: &Rect<T>) -> Arrangement<T> {
        Arrangement::Transform(
            window.clone(),
            Affine2::from_matrix_unchecked(
                Matrix3::<T>::new(
                    T::one(), T::zero(), T::zero(),
                    T::zero(), T::one(), T::zero(),
                    T::zero(), T::zero(), T::one(),
                )))
    }

    pub fn create_svg_document(&self) -> Result<Document, SvgCreationError>
        where T: Real,
              T: CoordNum,
              T: RealField,
              T: ToPrimitive,
              T: FromPrimitive,
              f64: From<T> {
        match self {
            Arrangement::Transform(viewbox, _affine) => Ok(Document::new()
                .set("viewBox", (f64::from(viewbox.min().x.into()), f64::from(viewbox.min().y.into()),
                                 f64::from(viewbox.max().x.into()), f64::from(viewbox.max().y.into())))
                .set("width", format!("{}mm", viewbox.width()))
                .set("height", format!("{}mm", viewbox.height()))
            ),// Err(SvgCreationError::UndefinedViewBox),
            Arrangement::Center(viewbox, _invert) => Ok(Document::new()
                .set("viewBox", (f64::from(viewbox.min().x.into()), f64::from(viewbox.min().y.into()),
                                 f64::from(viewbox.max().x.into()), f64::from(viewbox.max().y.into())))
                .set("width", format!("{}mm", viewbox.width()))
                .set("height", format!("{}mm", viewbox.height()))
            ),
            Arrangement::FitCenter(viewbox, _invert) => Ok(Document::new()
                .set("viewBox", (f64::from(viewbox.min().x.into()), f64::from(viewbox.min().y.into()),
                                 f64::from(viewbox.max().x.into()), f64::from(viewbox.max().y.into())))
                .set("width", format!("{}mm", viewbox.width()))
                .set("height", format!("{}mm", viewbox.height()))
            ),
            Arrangement::FitCenterMargin(_margin, viewbox, _invert) => Ok(Document::new()
                .set("viewBox", (
                    f64::from(viewbox.min().x.into()),
                    f64::from(viewbox.min().y.into()),
                    f64::from(viewbox.max().x.into()),
                    f64::from(viewbox.max().y.into())))
                .set("width", format!("{}mm", viewbox.width()))
                .set("height", format!("{}mm", viewbox.height()))
            ),
        }
    }
}

pub trait ToSvg<T>
    where T: CoordNum,
          T: Real,
          T: RealField {
    /// Given an [Arrangement] as a transformation strategy, transform the geometry to
    /// fit the bounds, or just run the transformation without bounds if None
    fn arrange(&self, arrangement: &Arrangement<T>) -> Result<Self, SvgCreationError> where Self: Sized;

    /// Utility function returns a viewbox tuple for the geometry.
    /// Should be used AFTER calling arrange on the geo.
    fn viewbox(&self) -> Option<(T, T, T, T)>;

    /// Convert the Geometry into an SVG PathData item
    fn to_path_data(&self) -> Data;

    /// Convert the Geometry into an SVG Path, using the arrangement to Center/Fit/Transform it
    fn to_path(&self, arrangement: &Arrangement<T>) -> Path;
}

impl<T> ToSvg<T> for MultiLineString<T>
    where T: CoordNum,
          T: Real,
          T: RealField,
          T: Float,
          T: AsPrimitive<T>,
          T: ToPrimitive,
          T: FromPrimitive,
          f64: From<T> {
    fn arrange(&self, arrangement: &Arrangement<T>) -> Result<Self, SvgCreationError> {
        let gbox = match self.bounding_rect() {
            Some(gbox) => gbox,
            None => return Err(SvgCreationError::UndefinedViewBox),
        };
        let transformation = match arrangement {
            Arrangement::Transform(_viewbox, affine) => affine.clone(),
            Arrangement::Center(bounds, invert) => {
                let bcenter = bounds.min() + (bounds.max() - bounds.min()).div(T::from(2.0).unwrap()); // / (2.0 as T);
                let gcenter = gbox.min() + (gbox.max() - gbox.min()).div(T::from(2.0).unwrap());
                let delta = bcenter - gcenter;
                let tx = Affine2::from_matrix_unchecked(
                    Matrix3::<T>::new(
                        T::from(1.0).unwrap(), T::zero(), delta.x as T,
                        T::zero(), T::one(), delta.y as T,
                        T::zero(), T::zero(), T::one(),
                    )
                );
                if *invert {
                    Affine2::from_matrix_unchecked(Matrix3::<T>::new(
                        T::from(1.0).unwrap(), T::zero(), T::zero(),
                        T::zero(), -T::one(), bounds.height(),
                        T::zero(), T::zero(), T::one(),
                    )) * tx
                } else {
                    tx
                }
            }
            Arrangement::FitCenter(bounds, invert) => {
                let scale = <T as Real>::min(bounds.width() / gbox.width(), bounds.height() / gbox.height());
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
                if *invert {
                    Affine2::from_matrix_unchecked(Matrix3::<T>::new(
                        T::from(1.0).unwrap(), T::zero(), T::zero(),
                        T::zero(), -T::one(), bounds.height(),
                        T::zero(), T::zero(), T::one(),
                    )) * tx
                } else {
                    tx
                }
            }
            Arrangement::FitCenterMargin(margin, bounds, invert) => {
                let scale = <T as Real>::min(
                    (bounds.width() - T::from(2.0).unwrap() * *margin) / gbox.width(),
                    (bounds.height() - T::from(2.0).unwrap() * *margin) / gbox.height());
                let bcenter = bounds.min() +
                    (bounds.max() - bounds.min())
                        .div(T::from(2.0).unwrap()); // / (2.0 as T);
                let gcenter = gbox.center() * scale; // This is post scaling now.
                let delta = bcenter - gcenter;
                let tx = Affine2::from_matrix_unchecked(
                    Matrix3::new(
                        scale, T::zero(), delta.x,
                        T::zero(), scale, delta.y,
                        T::zero(), T::zero(), T::one(),
                    )
                );
                if *invert {
                    Affine2::from_matrix_unchecked(Matrix3::<T>::new(
                        T::from(1.0).unwrap(), T::zero(), T::zero(),
                        T::zero(), -T::one(), bounds.height(),
                        T::zero(), T::zero(), T::one(),
                    )) * tx
                } else {
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
        Ok(MultiLineString::<T>::new(linestrings))
    }

    fn viewbox(&self) -> Option<(T, T, T, T)> {
        let bounds = self.bounding_rect()?;
        Some((bounds.min().x, bounds.min().y, bounds.max().x, bounds.max().y))
    }

    fn to_path_data(&self) -> Data {
        let mut svg_data = Data::new();
        for tline in self {
            for point in tline.points().take(1) {
                let point = Point::<f64>::new(point.x().into(), point.y().into());
                svg_data = svg_data.move_to((point.x(), point.y()));
            }
            for point in tline.points().skip(1) {
                let point = Point::<f64>::new(point.x().into(), point.y().into());
                svg_data = svg_data.line_to((point.x(), point.y()));
            }
        }
        svg_data
    }

    fn to_path(&self, arrangement: &Arrangement<T>) -> Path {
        let path_result = (&self).arrange(arrangement);
        match path_result {
            Ok(pathval) =>
                Path::new()
                    .set("d", pathval.to_path_data()),
            Err(_) =>
                Path::new()
                    .set("d", "")
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use geo_types::{coord, LineString, MultiLineString, Polygon};
    use nalgebra::{Affine2, Matrix3};
    use wkt::Wkt;
    use super::*;

    #[test]
    fn test_load_wkt() {
        let _geoms: Polygon::<f64> = Polygon::try_from(
            Wkt::<f64>::from_str("POLYGON ((350 100, 450 450, 150 400, 100 200, 350 100), (200 300, 350 350, 300 200, 200 300))")
                .expect("Failed to load WKT"))
            .expect("Failed to load box");
    }

    #[test]
    fn test_arrange_center() {
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
            &Arrangement::Center(
                Rect::new(coord! {x:0f64, y:0f64}, coord! {x:400f64, y:400f64}),
                false));
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect")
                       .center(),
                   coord! {x: 200.0f64, y:200.0f64});
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect for mlines")
                       .width(), 100.0f64);
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect for mlines")
                       .height(), 100.0f64);
        assert_eq!(txmls.bounding_rect()
                       .expect("Couldn't get bounding rect on second attempt?")
                       .center(), coord! {x: 200.0, y:200.0});
    }

    #[test]
    fn test_arrange_fit_center() {
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
            &Arrangement::FitCenter(
                Rect::new(coord! {x:0f64, y:0f64}, coord! {x:400f64, y:400f64}),
                false));
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect")
                       .center(),
                   coord! {x: 200.0f64, y:200.0f64});
    }

    #[test]
    fn test_arrange_fit_center_invert() {
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
            &Arrangement::FitCenter(
                Rect::new(coord! {x:0f64, y:0f64}, coord! {x:400f64, y:400f64}),
                true));
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect")
                       .center(),
                   coord! {x: 200.0f64, y:200.0f64});
    }

    #[test]
    fn test_arrange_fit_center_invert_2() {
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
            &Arrangement::FitCenter(
                Rect::new(coord! {x:0f64, y:0f64}, coord! {x:400f64, y:400f64}),
                true));
        assert_eq!(txmls.bounding_rect()
                       .expect("Should have been able to get brect")
                       .center(),
                   coord! {x: 200.0f64, y:200.0f64});
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
            &Arrangement::Transform(
                Rect::new(coord! {x:0f64, y:0f64}, coord! {x:400f64, y:400f64}),
                Affine2::from_matrix_unchecked(
                    Matrix3::<f64>::new(
                        1.0, 0.0, 300.0,
                        0.0, 1.0, 0.0,
                        0.0, 0.0, 1.0,
                    )
                )));
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
                    left == right
                })
                .count(),
            5);
    }
}
