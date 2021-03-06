/// Module for flattening all of the polygons out of a Geometry collection
/// into a single MultiPolygon. Very useful before hatching a group of geometries,
/// or for determining just the outlines of shapes.

use std::error::Error;
use geo_types::{Geometry, MultiPolygon, Polygon};

/// Return a single MultiPolygon which contains ALL of the polygons in a given geometry.
/// This includes all nested polygons, even in a complex Geo with nested/recursive
/// GeometryCollections.
pub trait FlattenPolygons{
    fn flatten_polys(&self) -> Result<MultiPolygon<f64>, Box<dyn Error>>;
}

impl FlattenPolygons for Geometry<f64>{
    fn flatten_polys(&self) -> Result<MultiPolygon<f64>, Box<dyn Error>> {
        match self {
            geo_types::Geometry::Polygon(_poly) => Ok(MultiPolygon::<f64>::new(vec![_poly.clone()])),
            geo_types::Geometry::MultiPolygon(_polys) => Ok(MultiPolygon::<f64>::new(_polys.0.clone())),
            geo_types::Geometry::GeometryCollection(gc) => {
                let foo: Vec<Vec<Polygon<f64>>> = gc.iter().map(|g| match g {
                    geo_types::Geometry::Polygon(poly) => vec![poly.clone()],
                    geo_types::Geometry::MultiPolygon(polys) => polys.0.clone(),
                    geo_types::Geometry::GeometryCollection(gc) => {
                        geo_types::Geometry::GeometryCollection(gc.clone())
                            .flatten_polys()
                            .unwrap_or(MultiPolygon::new(vec![]))
                            .0
                    },
                    _ => vec![] // Some part didn't buffer <shrug/>
                }).collect();

                Ok(MultiPolygon::new(foo.iter().map(|polys| polys.clone()).flatten().collect())) // .flatten().collect()))

            },
            _ => Ok(MultiPolygon::new(vec![])),
        }
    }

}

