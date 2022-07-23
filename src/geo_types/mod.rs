use std::error::Error;
use geo_types::{Point, CoordNum, Geometry, Polygon, coord, LineString};
use geos::CoordSeq;
use num_traits::real::Real;

/// Helper module for converting geo-types geometry into something useful
/// in Nannou.
pub mod nannou;

/// Crosshatching! You can fill your polys with lines, which is really useful
/// for plotters, because all you can draw is lines (or dots if you're a *monster*).
pub mod hatch;

/// Trait to convert geometry into an SVG object (or specifically, SVG components)
pub mod svg;

/// Trait to clip geometry by another geometry. Useful for clipping lines behind
/// another object to emulate 3D without the... 3D?
pub mod clip;

/// Trait to buffer various geo_types geometries. Basically a helper built on
/// the geo utilities
pub mod buffer;

/// Helper to flatten all the polygons from a Geometry into a MultiPolygon
pub mod flatten;

/// Boolean ops for geo_types
pub mod boolean;

/// Trait that implements a distance function between two [`geo_types::Point`] structs.
/// Also includes a length function which returns the length of a [`geo_types::Point`]
/// as if it were a Vector.
pub trait PointDistance<T: CoordNum> {
    /// Return the scalar distance between two [`geo_types::Point`]s.
    fn distance(&self, other: &Point<T>) -> T;

    /// Treat a [`geo_types::Point`] as a Vector and return its scalar length.
    fn length(&self) -> T;
}

/// Kinda weird that arc features are missing from geo_types, but ok, here is one.
pub mod shapes {
    use geo_types::{coord, Geometry, LineString, Point, Polygon};
    use std::f64::consts::PI;
    use num_traits::FromPrimitive;

    pub fn regular_poly(sides: usize, x: f64, y: f64, radius: f64, rotation: f64) -> Geometry<f64> {
        // all the way around to the start again, and hit the first point twice to close it.
        if sides < 3 {
            return Geometry::Point(Point::new(x, y));
        }

        Geometry::Polygon(Polygon::new(LineString::new((0..(sides + 2))
            .map(|i| {
                let angle = rotation - PI / 2.0 +
                    (f64::from(i as i32) / f64::from(sides as i32)) * (2.0 * PI);
                coord! {x: x+angle.cos() * radius, y: y+angle.sin() * radius}
            }).collect()
        ), vec![]))
    }

    pub fn circle(x0: f64, y0: f64, radius: f64) -> Geometry<f64> {
        let radius = radius.abs();
        let sides = 1000.min(32.max(usize::from_f64(radius).unwrap_or(1000) * 4));
        regular_poly(sides, x0, y0, radius, 0.0)
    }

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
            let arc = arc_center(0.0f64, 0.0f64, 10.0f64, 90.0f64, 135f64);
            println!("ARC: {:?}", &arc);
        }
    }
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

pub trait ToGeos {
    fn to_geos(&self) -> Result<geos::Geometry, Box<dyn Error>>;
}

impl ToGeos for geo_types::Geometry<f64> {
    fn to_geos(&self) -> Result<geos::Geometry, Box<dyn Error>> {
        if let Geometry::GeometryCollection(collection) = self{
            let geomap: Vec<geos::Geometry> = collection.iter()
                .map(|item|
                    item
                        .to_geos()
                        .or(
                            geos::Geometry::create_empty_collection(
                                geos::GeometryTypes::GeometryCollection)
                        )
                        .unwrap()
                )
                // .map_ok(|x|x)
                .collect();
            if let Ok(geosmap) = geos::Geometry::create_geometry_collection(geomap){
                return Ok(geosmap);
            } else {
                return Err(Box::new(geos::Error::InvalidGeometry("Wrong type of geometry".into())))
            }
        }
        Ok(match self {
            Geometry::Point(p) => geos::Geometry::try_from(p),
            Geometry::Line(line) => {
                geos::Geometry::create_line_string(CoordSeq::new_from_vec(
                    &vec![
                        vec![line.start.x, line.start.y],
                        vec![line.end.x, line.end.y]]
                ).expect("Unexpected failure of create_line_string"))
            }
            Geometry::Rect(rect) => geos::Geometry::try_from(
                Polygon::new(LineString::from(
                    vec![
                        rect.min(),
                        coord!{x: rect.max().x, y: rect.min().y},
                        rect.max(),
                        coord!{x: rect.min().x, y: rect.max().y},
                        rect.min(),
                    ]),
                             vec![])),
            Geometry::LineString(line) => geos::Geometry::try_from(line),
            Geometry::Polygon(poly) => geos::Geometry::try_from(poly),
            Geometry::MultiPolygon(polys) => geos::Geometry::try_from(polys),
            Geometry::MultiLineString(mls) => {
                geos::Geometry::create_multiline_string(mls.0
                    .clone()
                    .iter()
                    .map(|line| {
                        geos::Geometry::try_from(line)
                            .unwrap_or(geos::Geometry::create_empty_line_string().unwrap())
                    })
                    .collect())
            },
            _ => Err(geos::Error::InvalidGeometry("Wrong type of geometry".into()))
        }?)
    }
}

#[cfg(test)]
mod tests {
    use super::PointDistance;
    use geo_types::Point;
    use num_traits::abs;

    #[test]
    fn test_length() {
        let p = Point::new(10.0f64, 0.0f64);
        assert!(abs(p.length() - 10.0) < 0.0001)
    }

    #[test]
    fn test_distance() {
        let d = Point::new(10.0, 0.0).distance(&Point::new(0.0, 10.0));
        assert!(abs(d - (10.0f64.powi(2) + 10.0f64.powi(2)).sqrt()) < 0.0001)
    }
}