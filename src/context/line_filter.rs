use std::sync::Arc;

use geo::Geometry;
use geo::algorithm::Densify;
use geo::line_measures::Densifiable;
use geo::{Coord, GeometryCollection, MapCoordsInPlace, MultiLineString};
use geo::{Euclidean, Polygon};
// TODO: Switch this to using the parent crate instead.
use nannou::noise::{NoiseFn, Perlin};
use rand::{Rng, thread_rng};

pub trait LineFilter: std::fmt::Debug {
    fn apply(&self, lines: &MultiLineString<f64>) -> MultiLineString<f64>;
}

#[derive(Clone, Debug)]
pub struct SketchyLineFilter {
    perlin: Perlin,
    pub noise_prescale: f64,
    pub deviation: f64,
}

#[derive(Clone, Debug)]
pub struct PerlinFilter {
    perlin: Perlin,
    pub noise_prescale: f64,
    pub deviation: f64,
}

impl PerlinFilter {
    pub fn new(noise_prescale: f64, deviation: f64) -> PerlinFilter {
        PerlinFilter {
            perlin: Perlin::new(),
            noise_prescale,
            deviation,
        }
    }

    pub fn noise_prescale(self, val: f64) -> PerlinFilter {
        PerlinFilter {
            noise_prescale: val,
            ..self
        }
    }

    pub fn deviation(self, val: f64) -> PerlinFilter {
        PerlinFilter {
            deviation: val,
            ..self
        }
    }

    pub fn finish(self) -> Arc<Box<dyn LineFilter>> {
        Arc::new(Box::new(self))
    }
}

impl SketchyLineFilter {
    pub fn new(noise_prescale: f64, deviation: f64) -> SketchyLineFilter {
        SketchyLineFilter {
            perlin: Perlin::new(),
            noise_prescale,
            deviation,
        }
    }

    pub fn noise_prescale(self, val: f64) -> SketchyLineFilter {
        SketchyLineFilter {
            noise_prescale: val,
            ..self
        }
    }

    pub fn deviation(self, val: f64) -> SketchyLineFilter {
        SketchyLineFilter {
            deviation: val,
            ..self
        }
    }

    pub fn finish(self) -> Arc<Box<dyn LineFilter>> {
        Arc::new(Box::new(self))
    }
}

pub fn geo_densify(geo: &Geometry<f64>, density: f64) -> Geometry<f64> {
    match geo {
        Geometry::Point(p) => Geometry::Point(p.clone()),
        Geometry::Line(line) => {
            Geometry::LineString(Euclidean::densify(&Euclidean {}, line, density))
        }
        Geometry::LineString(line_string) => {
            Geometry::LineString(Euclidean::densify(&Euclidean {}, line_string, density))
        }
        Geometry::Polygon(polygon) => {
            Geometry::Polygon(Euclidean::densify(&Euclidean {}, polygon, density))
        }
        Geometry::MultiPoint(multi_point) => Geometry::MultiPoint(multi_point.clone()),
        Geometry::MultiLineString(multi_line_string) => Geometry::MultiLineString(
            Euclidean::densify(&Euclidean {}, multi_line_string, density),
        ),
        Geometry::MultiPolygon(multi_polygon) => {
            Geometry::MultiPolygon(Euclidean::densify(&Euclidean {}, multi_polygon, density))
        }
        Geometry::GeometryCollection(geometry_collection) => {
            Geometry::GeometryCollection(GeometryCollection::new_from(
                geometry_collection
                    .into_iter()
                    .map(|g| geo_densify(g, density))
                    .collect(),
            ))
        }
        Geometry::Rect(rect) => {
            let poly: Polygon = rect.to_polygon();
            Geometry::Polygon(Euclidean::densify(&Euclidean {}, &poly, density))
        }
        Geometry::Triangle(triangle) => {
            let poly: Polygon = triangle.to_polygon();
            Geometry::Polygon(Euclidean::densify(&Euclidean {}, &poly, density))
        }
    }
}

impl LineFilter for PerlinFilter {
    fn apply(&self, mls: &MultiLineString<f64>) -> MultiLineString<f64> {
        let mut mls = mls.densify(&Euclidean {}, self.noise_prescale / 10.);
        for line in &mut mls {
            line.map_coords_in_place(move |coord| {
                let dx = self.deviation
                    * self.perlin.get([
                        coord.x / self.noise_prescale,
                        coord.y / self.noise_prescale,
                        0.,
                    ]);
                let dy = self.deviation
                    * self.perlin.get([
                        coord.x / self.noise_prescale,
                        coord.y / self.noise_prescale,
                        10000.,
                    ]);
                Coord {
                    x: coord.x + dx,
                    y: coord.y + dy,
                }
            });
        }
        mls
    }
}

impl LineFilter for SketchyLineFilter {
    fn apply(&self, mls: &MultiLineString<f64>) -> MultiLineString<f64> {
        let mut mls = mls.densify(&Euclidean {}, self.noise_prescale / 10.);
        let mut rng = thread_rng();
        for line in &mut mls {
            let depth = rng.gen_range(0.0f64..100000.0);
            line.map_coords_in_place(move |coord| {
                let dx = self.deviation
                    * self.perlin.get([
                        coord.x / self.noise_prescale,
                        coord.y / self.noise_prescale,
                        0. + depth,
                    ]);
                let dy = self.deviation
                    * self.perlin.get([
                        coord.x / self.noise_prescale,
                        coord.y / self.noise_prescale,
                        10000. + depth,
                    ]);
                Coord {
                    x: coord.x + dx,
                    y: coord.y + dy,
                }
            });
        }
        mls
    }
}

#[cfg(test)]
pub mod test {
    use geo::LineString;
    use geo_types::Coord;

    use super::*;

    #[test]
    pub fn test_trait() {
        let foo = SketchyLineFilter::new(10., 3.);
        // let geo: Geometry<f64> =
        //     Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 50.0, y: 50.0 }).into();
        let mls = MultiLineString(vec![LineString::new(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 50.0, y: 50.0 },
        ])]);
        let new_mls = foo.apply(&mls);
        println!("New MLS: {:?}", &new_mls);
    }
}
