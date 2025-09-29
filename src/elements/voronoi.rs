use crate::{
    elements::{FieldToVoronoi, PointField},
    util::AnythingToGeo,
};
use geo::{Coord, MultiPolygon, Polygon};
pub use voronoice::*;

pub struct BBWrapper(pub BoundingBox);

impl<F> FieldToVoronoi for F
where
    F: PointField,
    F: Iterator<Item = geo::Point<f64>>,
{
    fn to_voronoi(&mut self, point_count: usize) -> Voronoi {
        let sites = self
            .take(point_count)
            .map(|pt| voronoice::Point {
                x: pt.x(),
                y: pt.y(),
            })
            .collect::<Vec<voronoice::Point>>();

        let bb = BBWrapper::from(self.bounds().clone());
        VoronoiBuilder::default()
            .set_sites(sites)
            .set_bounding_box(bb.0)
            .set_lloyd_relaxation_iterations(5)
            .build()
            .unwrap()
    }
}

impl From<geo::Rect> for BBWrapper {
    fn from(value: geo::Rect) -> Self {
        let (x, y) = value.center().x_y();
        BBWrapper(BoundingBox::new(
            Point { x, y },
            value.width(),
            value.height(),
        ))
    }
}

impl AnythingToGeo for Voronoi {
    fn to_geo(&self) -> geo::Geometry {
        geo_types::Geometry::MultiPolygon(MultiPolygon::new(
            self.iter_cells()
                .map(|cell| {
                    Polygon::new(
                        cell.iter_vertices()
                            .map(|vert| Coord {
                                x: vert.x,
                                y: vert.y,
                            })
                            .collect(),
                        vec![],
                    )
                })
                .collect(),
        ))
    }
}
