use geo_types::{
    coord, CoordNum, Geometry, GeometryCollection, LineString, MultiLineString, Point, Polygon,
};
use geos::{CoordSeq, Geom};
use kurbo::PathEl;
use num_traits::real::Real;
use std::error::Error;

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

/// Affine Transformations
pub mod matrix;

/// Various shapes
pub mod shapes;

/// Trait that implements a distance function between two [`geo_types::Point`] structs.
/// Also includes a length function which returns the length of a [`geo_types::Point`]
/// as if it were a Vector.
pub trait PointDistance<T: CoordNum> {
    /// Return the scalar distance between two [`geo_types::Point`]s.
    fn distance(&self, other: &Point<T>) -> T;

    /// Treat a [`geo_types::Point`] as a Vector and return its scalar length.
    fn length(&self) -> T;
}

pub trait ToGTGeometry {
    fn to_gt_geometry(&self, accuracy: f64) -> Result<Geometry<f64>, Box<dyn Error>>;
}

impl ToGTGeometry for kurbo::BezPath {
    fn to_gt_geometry(&self, accuracy: f64) -> Result<Geometry<f64>, Box<dyn Error>> {
        let mut segments: MultiLineString<f64> = MultiLineString::new(vec![]);
        let mut lastpoint = kurbo::Point::new(0.0, 0.0);
        let add_segment = |el: PathEl| match el {
            PathEl::MoveTo(pos) => {
                segments
                    .0
                    .push(LineString::new(vec![coord! {x: pos.x, y: pos.y}]));
                lastpoint = pos.clone();
            }
            PathEl::LineTo(pos) => {
                if let Some(line) = segments.0.last_mut() {
                    line.0.push(coord! {x: pos.x, y: pos.y});
                }
            }
            PathEl::ClosePath => {
                if let Some(line) = segments.0.last_mut() {
                    line.0.push(coord! {x: lastpoint.x, y: lastpoint.y});
                }
            }
            _ => panic!("Unexpected/Impossible segment type interpolating a bezier path!"),
        };
        // println!("Pre-flatten geo is: {:?}", self.segments().map(|s| s).collect::<Vec<PathSeg>>());
        // self.segments().for_each(|s| println!("Segment: {:?}", s));
        self.flatten(accuracy.into(), add_segment);
        let tmp_gtgeo = Geometry::MultiLineString(segments);
        let tmp_geos = tmp_gtgeo.to_geos();
        Ok(match tmp_geos {
            Ok(geos_geom) => {
                if let Ok((poly_geo, _cuts_geo, dangles_geo, invalid_geo)) =
                    geos_geom.polygonize_full()
                {
                    // if let Some(cuts) = &cuts_geo {
                    //     println!("Cuts: {:?}", cuts.to_wkt().unwrap());
                    // }
                    // if let Some(invalid) = &invalid_geo {
                    //     println!("Invalid: {:?}", invalid.to_wkt().unwrap());
                    // }
                    // if let Some(dangles) = &dangles_geo {
                    //     println!("Dangles: {:?}", dangles.to_wkt().unwrap());
                    // }
                    // let mut out_gtgeo = match invalid_geo {
                    //     None => Geometry::try_from(&poly_geo).unwrap_or(tmp_gtgeo.clone()),
                    //     Some(invalid) => {
                    //         println!("Invalid: {:?}", invalid.to_wkt().unwrap());
                    //         Geometry::GeometryCollection(GeometryCollection::new_from(vec![
                    //             Geometry::try_from(&poly_geo).unwrap_or(tmp_gtgeo.clone()),
                    //             Geometry::try_from(&invalid).unwrap_or(Geometry::GeometryCollection(GeometryCollection::new_from(vec![]))),
                    //         ]))
                    //     }
                    // };
                    let out_gtgeo =
                        Geometry::GeometryCollection(GeometryCollection::new_from(vec![
                            Geometry::try_from(&poly_geo).unwrap_or(Geometry::GeometryCollection(
                                GeometryCollection::new_from(vec![]),
                            )),
                            match invalid_geo {
                                Some(invalid) => Geometry::try_from(&invalid).unwrap_or(
                                    Geometry::GeometryCollection(GeometryCollection::new_from(
                                        vec![],
                                    )),
                                ),
                                None => Geometry::GeometryCollection(GeometryCollection::new_from(
                                    vec![],
                                )),
                            },
                            match dangles_geo {
                                Some(dangles) => Geometry::try_from(&dangles).unwrap_or(
                                    Geometry::GeometryCollection(GeometryCollection::new_from(
                                        vec![],
                                    )),
                                ),
                                None => Geometry::GeometryCollection(GeometryCollection::new_from(
                                    vec![],
                                )),
                            },
                        ]));
                    // println!("Polygonzed: {:?}", &out_gtgeo);
                    out_gtgeo
                } else {
                    // println!("Couldn't convert to geos polys");
                    tmp_gtgeo.clone()
                }
            }
            Err(_err) => {
                // println!("Couldn't convert to geos at all");
                tmp_gtgeo.clone()
            }
        })
    }
}

impl<T> PointDistance<T> for Point<T>
where
    T: CoordNum,
    T: Real,
{
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
        if let Geometry::GeometryCollection(collection) = self {
            let geomap: Vec<geos::Geometry> = collection
                .iter()
                .map(|item| {
                    item.to_geos()
                        .or(geos::Geometry::create_empty_collection(
                            geos::GeometryTypes::GeometryCollection,
                        ))
                        .unwrap()
                })
                // .map_ok(|x|x)
                .collect();
            if let Ok(geosmap) = geos::Geometry::create_geometry_collection(geomap) {
                return Ok(geosmap);
            } else {
                return Err(Box::new(geos::Error::InvalidGeometry(
                    "Wrong type of geometry".into(),
                )));
            }
        }
        Ok(match self {
            Geometry::Point(p) => geos::Geometry::try_from(p),
            Geometry::Line(line) => geos::Geometry::create_line_string(
                CoordSeq::new_from_vec(&vec![
                    vec![line.start.x, line.start.y],
                    vec![line.end.x, line.end.y],
                ])
                .expect("Unexpected failure of create_line_string"),
            ),
            Geometry::Rect(rect) => geos::Geometry::try_from(Polygon::new(
                LineString::from(vec![
                    rect.min(),
                    coord! {x: rect.max().x, y: rect.min().y},
                    rect.max(),
                    coord! {x: rect.min().x, y: rect.max().y},
                    rect.min(),
                ]),
                vec![],
            )),
            Geometry::LineString(line) => geos::Geometry::try_from(line),
            Geometry::Polygon(poly) => geos::Geometry::try_from(poly),
            Geometry::MultiPolygon(polys) => geos::Geometry::try_from(polys),
            Geometry::MultiLineString(mls) => geos::Geometry::create_multiline_string(
                mls.0
                    .clone()
                    .iter()
                    .map(|line| {
                        geos::Geometry::try_from(line)
                            .unwrap_or(geos::Geometry::create_empty_line_string().unwrap())
                    })
                    .collect(),
            ),
            _ => Err(geos::Error::InvalidGeometry(
                "Wrong type of geometry".into(),
            )),
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
