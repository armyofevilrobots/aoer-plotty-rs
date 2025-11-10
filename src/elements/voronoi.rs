use crate::{elements::ToVoronoi, util::AnythingToGeo};
use geo::{Contains, Coord, MultiPolygon, Polygon, Rect};
pub use voronoice::*;

pub struct BBWrapper(pub BoundingBox);

impl<F> ToVoronoi<F> for F
where
    F: Iterator<Item = geo::Point<f64>>,
{
    fn to_voronoi(&mut self) -> Voronoi {
        let mut bounds: Option<Rect> = None;
        let sites = self
            // .take(limit)
            .map(|pt| voronoice::Point {
                x: pt.x(),
                y: pt.y(),
            })
            .collect::<Vec<voronoice::Point>>();

        // println!("BB rect: {:?}", self.bounds());
        // println!("Sites: {:?}", &sites);
        for item in &sites {
            if let Some(b) = &mut bounds {
                let item_coord = Coord {
                    x: item.x,
                    y: item.y,
                };
                if !b.contains(&item_coord) {
                    let min_x = b.min().x.min(item_coord.x);
                    let max_x = b.max().x.max(item_coord.x);
                    let min_y = b.min().y.min(item_coord.y);
                    let max_y = b.max().y.max(item_coord.y);
                    bounds = Some(Rect::new(
                        Coord { x: min_x, y: min_y },
                        Coord { x: max_x, y: max_y },
                    ));
                }
            } else {
                bounds = Some(Rect::new(
                    Coord {
                        x: item.x,
                        y: item.y,
                    },
                    Coord {
                        x: item.x,
                        y: item.y,
                    },
                ));
            }
        }

        let bb = BBWrapper::from(bounds.expect("No bounds found. Empty point field?"));
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
