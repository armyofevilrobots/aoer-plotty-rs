use crate::geo_types::ToGeos;
use geo_types::{Geometry, MultiPolygon, Polygon};
use geos::Geom;
use geos::Geometry as GGeometry;
/// Module for flattening all of the polygons out of a Geometry collection
/// into a single MultiPolygon. Very useful before hatching a group of geometries,
/// or for determining just the outlines of shapes.
use std::error::Error;

/// Return a single MultiPolygon which contains ALL of the polygons in a given geometry.
/// This includes all nested polygons, even in a complex Geo with nested/recursive
/// GeometryCollections.
pub trait FlattenPolygons {
    fn flatten_polys(&self) -> Result<MultiPolygon<f64>, Box<dyn Error>>;
    fn flatten(&self) -> Self;
}

impl FlattenPolygons for Geometry<f64> {
    /// Just simplify some geometry as much as possible
    fn flatten(&self) -> Self {
        let ggeom: GGeometry = match self.to_geos() {
            Ok(ggeom) => ggeom,
            Err(_err) => return self.clone(),
        };
        match ggeom.unary_union() {
            Ok(ggeom) => geo_types::Geometry::<f64>::try_from(ggeom).unwrap_or(self.clone()),
            Err(_err) => self.clone(),
        }
    }

    fn flatten_polys(&self) -> Result<MultiPolygon<f64>, Box<dyn Error>> {
        match self {
            geo_types::Geometry::Polygon(_poly) => {
                Ok(MultiPolygon::<f64>::new(vec![_poly.clone()]))
            }
            geo_types::Geometry::MultiPolygon(_polys) => {
                Ok(MultiPolygon::<f64>::new(_polys.0.clone()))
            }
            geo_types::Geometry::GeometryCollection(gc) => {
                let foo: Vec<Vec<Polygon<f64>>> = gc
                    .iter()
                    .map(|g| match g {
                        geo_types::Geometry::Polygon(poly) => vec![poly.clone()],
                        geo_types::Geometry::MultiPolygon(polys) => polys.0.clone(),
                        geo_types::Geometry::GeometryCollection(gc) => {
                            geo_types::Geometry::GeometryCollection(gc.clone())
                                .flatten_polys()
                                .unwrap_or(MultiPolygon::new(vec![]))
                                .0
                        }
                        _ => vec![], // Some part didn't buffer <shrug/>
                    })
                    .collect();

                Ok(MultiPolygon::new(
                    foo.iter().map(|polys| polys.clone()).flatten().collect(),
                )) // .flatten().collect()))
            }
            _ => Ok(MultiPolygon::new(vec![])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo_types::{
        boolean::BooleanOp,
        buffer::Buffer,
        shapes::{arc_center, circle, regular_poly_native},
    };
    use geo::coord;
    use geo_types::{Geometry, GeometryCollection, LineString};
    use std::f64::consts::PI;

    #[test]
    fn test_diff_flattened_hex_truchet() {
        let p0 = coord! {x: 1., y: 0.};
        let p1 = coord! {x: (PI/3.).cos(), y: (PI/3.).sin()};
        let p2 = coord! {x: (2.*PI/3.).cos(), y: (2.*PI/3.).sin()};
        let p3 = coord! {x: PI.cos(), y: PI.sin()};
        let p4 = coord! {x: (4.*PI/3.).cos(), y: (4.*PI/3.).sin()};
        let p5 = coord! {x: (5.*PI/3.).cos(), y: (5.*PI/3.).sin()};
        println!(
            "Points are: {:?}, \n {:?}, \n {:?}, \n {:?}, \n {:?}, \n {:?},",
            p0, p1, p2, p3, p4, p5
        );
        let hex_invert_base = Geometry::GeometryCollection(GeometryCollection::new_from(vec![
            regular_poly_native(6, 0., 0., 1., 0.0),
            circle(p0.x, p0.y, 1. / 3.),
            circle(p1.x, p1.y, 1. / 3.),
            circle(p2.x, p2.y, 1. / 3.),
            circle(p3.x, p3.y, 1. / 3.),
            circle(p4.x, p4.y, 1. / 3.),
            circle(p5.x, p5.y, 1. / 3.),
        ]))
        .flatten();
        let hex_long_division = Geometry::GeometryCollection(GeometryCollection::new_from(vec![
            Geometry::MultiPolygon(
                Geometry::LineString(arc_center(
                    p0.x,
                    p0.y,
                    1. / 2.,
                    210.0, // These are degrees because they're from the operations lib
                    330.,  // These are degrees because they're from the operations lib
                ))
                .buffer(1. / 6.)
                .unwrap(),
            ),
            Geometry::MultiPolygon(
                Geometry::LineString(arc_center(
                    p3.x,
                    p3.y,
                    1. / 2.,
                    30.0, // These are degrees because they're from the operations lib
                    150., // These are degrees because they're from the operations lib
                ))
                .buffer(1. / 6.)
                .unwrap(),
            ),
            Geometry::MultiPolygon(
                Geometry::LineString(LineString::new(vec![(p1 + p2) / 2., (p4 + p5) / 2.]))
                    .buffer(1. / 6.)
                    .unwrap(),
            ),
            // regular_poly_native(6, 0., 0., 1., 0.0),
        ]))
        .flatten();

        // If we fucked up, this catches fire.
        let _foo = hex_invert_base.difference(&hex_long_division).unwrap();
    }
}
