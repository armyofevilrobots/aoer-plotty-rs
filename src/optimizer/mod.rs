use std::collections::HashMap;

#[allow(deprecated)]
use geo::{HasDimensions, prelude::EuclideanDistance};
use geo_types::{Coord as Coordinate, LineString, MultiLineString};
use rstar::{AABB, PointDistance, RTree, RTreeObject};

#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationStrategy {
    Greedy,
}

#[derive(Debug, Clone, PartialEq)]
/// Optimization strategy utility class.
pub struct Optimizer {
    max_keepdown: f64,
    strategy: OptimizationStrategy,
}

impl Optimizer {
    pub fn new(max_keepdown: f64, strategy: OptimizationStrategy) -> Optimizer {
        Optimizer {
            max_keepdown,
            strategy,
        }
    }

    pub fn build_rtree_from_hashmap(
        &self,
        hashmap: &HashMap<usize, LineString<f64>>,
    ) -> RTree<LineRef> {
        // Build out the RTree data
        let mut linerefs: Vec<LineRef> = vec![];
        for (i, line) in hashmap.iter() {
            // Only add lines that are valid (2+ points)
            if line.0.len() < 2 {
                continue;
            }
            linerefs.push(LineRef::new(
                *i,
                line.0
                    .first()
                    .expect("We already bounds checked this?!")
                    .clone(),
                line.0
                    .last()
                    .expect("We already bounds checked this too!?")
                    .clone(),
                self.max_keepdown * 2., //.clone(),
                true,
            ));
            linerefs.push(LineRef::new(
                *i,
                line.0
                    .last()
                    .expect("We already bounds checked this?!")
                    .clone(),
                line.0
                    .first()
                    .expect("We already bounds checked this too!?")
                    .clone(),
                self.max_keepdown * 2., //.clone(),
                false,
            ));
        }

        RTree::bulk_load(linerefs)
    }

    /// Merges lines who have endpoints at most pen_width*pi.sqrt() apart
    pub fn merge(&self, mls: &MultiLineString<f64>) -> MultiLineString<f64> {
        let mut lines_out = MultiLineString::new(vec![]); // Just one empty line in it.
        let mut current_line: LineString<f64> = LineString::new(vec![]);
        for source_line in mls.0.iter() {
            if source_line.0.len() == 0 {
                continue;
            } // Skip blank/dot lines
            let source_start = source_line.0.first().unwrap();
            #[allow(deprecated)]
            if current_line.0.len() == 0 {
                current_line.0.append(&mut source_line.0.clone());
            } else if current_line
                .0
                .last()
                .unwrap()
                .euclidean_distance(source_start)
                <= self.max_keepdown
            {
                let mut tmpline = source_line.0.clone();
                current_line.0.append(&mut tmpline);
            } else {
                lines_out.0.push(current_line.clone());
                current_line = source_line.clone();
            }
        }
        if current_line.0.len() > 0 {
            lines_out.0.push(current_line)
        }
        lines_out
    }

    /// Optimizes lines by finding the nearest neighbor to each endpoint
    /// using an rtree as a spatial index. Fast, but just greedy for now.
    pub fn optimize(&self, mls: &MultiLineString<f64>) -> MultiLineString<f64> {
        assert_eq!(self.strategy, OptimizationStrategy::Greedy);
        let mut lines_out = MultiLineString::new(vec![]);
        // if mls.0.len() == 0 {
        if mls.0.len() == 0 {
            return lines_out;
        };
        if mls.0.len() == 1 {
            return mls.clone();
        }
        let mut line_count: usize = 0;
        let mut lines_hash: HashMap<usize, LineString<f64>> =
            HashMap::from_iter(mls.0.iter().map(|line| {
                let pair = (line_count, line.clone());
                line_count = line_count + 1;
                pair
            }));
        let rtree = self.build_rtree_from_hashmap(&lines_hash);

        while lines_hash.len() > 0 {
            if let Some(tmpline) = lines_hash.remove(&0) {
                if tmpline.is_empty() {
                    continue;
                }
                if tmpline.0.len() > 1 {
                    lines_out.0.push(tmpline); //.clone());
                    break;
                } else {
                }
            } else {
                break;
            }
        }
        while lines_hash.len() > 0 {
            let line = lines_out
                .0
                .last()
                .expect("Cannot pull line from list I just pushed to?!");
            let last = match line.0.last() {
                Some(point) => point,
                None => {
                    eprintln!("Failed to get last point for line: {:?}", &line);
                    continue;
                }
            };
            let mut found = false;
            for neighbor_ref in rtree.nearest_neighbor_iter(&[last.x, last.y]) {
                if let Some(mut neighbor_line) = lines_hash.remove(&neighbor_ref.line_id) {
                    found = true;
                    if neighbor_ref.fwd {
                        lines_out.0.push(neighbor_line);
                    } else {
                        neighbor_line.0.reverse();
                        lines_out.0.push(neighbor_line);
                    }
                    break;
                }
            }
            if !found {
                break;
            }
        } // We don't want to iterate the same item forever
        let remaining_keys: Vec<usize> = lines_hash.keys().map(|k| k.clone()).collect();
        for k in remaining_keys {
            if let Some(line) = lines_hash.remove(&k) {
                lines_out.0.push(line);
            }
        }

        lines_out
    }
}

/// A reference to a line which provides the line id, it's coordinates,
/// and whether it's a forward or reverse traversal of the given line,
/// as both are valid entries.
#[derive(Clone, Debug, PartialEq)]
pub struct LineRef {
    line_id: usize,
    start: Coordinate<f64>,
    end: Coordinate<f64>,
    pen_width: f64,
    fwd: bool,
}

impl LineRef {
    pub fn new(
        line_id: usize,
        start: Coordinate<f64>,
        end: Coordinate<f64>,
        pen_width: f64,
        fwd: bool,
    ) -> LineRef {
        LineRef {
            line_id,
            start,
            end,
            pen_width,
            fwd,
        }
    }
}

impl RTreeObject for LineRef {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.start.x - self.pen_width, self.start.y - self.pen_width],
            [self.start.x + self.pen_width, self.start.y + self.pen_width],
        )
    }
}

impl PointDistance for LineRef {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let d_x = self.start.x - point[0];
        let d_y = self.start.y - point[1];
        let distance_to_start = (d_x * d_x + d_y * d_y).sqrt();
        let distance_to_ring = distance_to_start - self.pen_width / 2.;
        let distance_to_circle = f64::max(0.0, distance_to_ring);
        // We must return the squared distance!
        distance_to_circle * distance_to_circle
    }

    // This implementation is not required but more efficient since it
    // omits the calculation of a square root
    fn contains_point(&self, point: &[f64; 2]) -> bool {
        let d_x = self.start.x - point[0];
        let d_y = self.start.y - point[1];
        let distance_to_start_2 = d_x * d_x + d_y * d_y;
        let radius_2 = self.pen_width / 2. * self.pen_width / 2.;
        distance_to_start_2 <= radius_2
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::prelude::LineHatch;

    use super::*;
    use geo_types::{Polygon, coord};
    use rstar::RTree;
    use wkt::ToWkt;

    #[test]
    fn test_optimizer_poly_nest() {
        let mut ctx = crate::context::Context::new();
        ctx.pen(1.)
            .stroke("#000")
            .fill("#000")
            .hatch(0.)
            .pen(0.5)
            .pattern(Arc::new(Box::new(LineHatch {})));
        let outer = LineString::new(vec![
            coord! {x:0., y:10.},
            coord! {x:14., y: 0.},
            coord! {x:-14., y:0.},
            coord! {x:0., y:10.},
        ]);
        let inner = vec![LineString::new(vec![
            coord! {x:0., y:6.},
            coord! {x:4.,y:4.},
            coord! {x:-4., y:4.},
            coord! {x:0., y:6.},
        ])];
        let test_poly = Polygon::<f64>::new(outer, inner);
        ctx.geometry(&geo_types::Geometry::Polygon(test_poly));
        for layer in ctx.to_layers() {
            println!("LAYER:");
            println!("STROKE: {}", layer.stroke_lines.to_wkt());
            println!("FILL: {}", layer.fill_lines.to_wkt());
        }
    }

    #[test]
    fn test_optimizer_infill_premerged() {
        let mut ctx = crate::context::Context::new();
        ctx.pen(1.)
            .stroke("#000")
            .fill("#000")
            .hatch(0.)
            .pattern(Arc::new(Box::new(LineHatch {})))
            .circle(0., 0., 10.);
        let layers = ctx.to_layers();
        for layer in layers {
            let opt = Optimizer::new(layer.stroke_width * 1.5, OptimizationStrategy::Greedy);
            let opt_stroke = opt.optimize(&layer.stroke_lines);
            let merged_fill = opt.merge(&layer.fill_lines);
            let opt_fill = opt.optimize(&merged_fill);
            println!("LAYER:");
            println!("STROKE: {}", opt_stroke.to_wkt());
            println!("FILL: {}", opt_fill.to_wkt());
        }
    }

    #[test]
    fn test_optimizer_infill() {
        let mut ctx = crate::context::Context::new();
        ctx.pen(1.)
            .stroke("#000")
            .fill("#000")
            .hatch(0.)
            .pattern(Arc::new(Box::new(LineHatch {})))
            .circle(0., 0., 10.);
        let layers = ctx.to_layers();
        for layer in layers {
            let opt = Optimizer::new(layer.stroke_width, OptimizationStrategy::Greedy);
            let opt_stroke = opt.optimize(&layer.stroke_lines);
            let opt_fill = opt.optimize(&layer.fill_lines);
            println!("LAYER:");
            println!("STROKE: {}", opt_stroke.to_wkt());
            println!("FILL: {}", opt_fill.to_wkt());
        }
    }

    #[test]
    fn test_lineref_basic() {
        let line_0f = LineRef {
            line_id: 0,
            start: coord! {x:0., y:0.},
            end: coord! {x:20., y:0.},

            pen_width: 0.5,
            fwd: true,
        };
        let line_0r = LineRef {
            line_id: 1,
            start: coord! {x:20., y:0.},
            end: coord! {x:0., y:0.},
            pen_width: 0.5,
            fwd: false,
        };
        let line_1f = LineRef {
            line_id: 2,
            start: coord! {x:10., y:10.},
            end: coord! {x:20., y:5.},
            pen_width: 1.0,
            fwd: true,
        };
        let line_1r = LineRef {
            line_id: 3,
            start: coord! {x:20., y:5.},
            end: coord! {x:10., y:10.},
            pen_width: 1.0,
            fwd: false,
        };
        let mut tree = RTree::new();
        tree.insert(line_0f.clone());
        tree.insert(line_0r.clone());
        tree.insert(line_1f.clone());
        tree.insert(line_1r.clone());
        let e = AABB::from_point([10., 10.]);
        println!("e: {:?}", e);
        println!(
            "Closest neighbor to e is {:?}",
            tree.nearest_neighbor(&[10., 10.])
        );

        for line in tree.nearest_neighbor_iter(&[10., 10.]) {
            println!("Found line endpoint for line id: {}", line.line_id);
        }
    }

    #[test]
    fn test_optimizer_basic() {
        let lines: MultiLineString<f64> = MultiLineString::new(vec![
            LineString::new(vec![coord! {x: 0.0, y:20.0}, coord! {x:0.0, y:0.0}]),
            LineString::new(vec![coord! {x: 0.0, y:0.0}, coord! {x:20.0, y:20.0}]),
            LineString::new(vec![coord! {x: 20.0, y:20.5}, coord! {x:40.0, y:20.0}]),
            LineString::new(vec![coord! {x: 20.0, y:0.5}, coord! {x:20.0, y:20.0}]),
            LineString::new(vec![coord! {x:40.0, y:20.0}, coord! {x:40.5,y:40.5}]),
            LineString::new(vec![coord! {x:0.0, y:0.0}, coord! {x:40.5,y:20.5}]),
        ]);
        let mut distance_unopt = 0.;
        for i in 0..lines.0.len() - 1 {
            let pt0 = lines.0[i].0.last().unwrap();
            let pt1 = lines.0[i + 1].0.first().unwrap();
            let ls0 = pt0.euclidean_distance(pt1);
            distance_unopt = distance_unopt + ls0;
        }
        println!("UNOPT TRAVEL: {}", distance_unopt);
        let opt = Optimizer::new(0.7, OptimizationStrategy::Greedy);
        let out = opt.optimize(&lines);
        println!("OUT OPT: ");
        for pt in &out.0 {
            println!("PT: {:?}", pt);
        }
        let mut distance_opt = 0.;
        for i in 0..out.0.len() - 1 {
            let pt0 = out.0[i].0.last().unwrap();
            let pt1 = out.0[i + 1].0.first().unwrap();
            let ls0 = pt0.euclidean_distance(pt1);
            distance_opt = distance_opt + ls0;
        }
        println!("OPT TRAVEL: {}", distance_opt);
    }
}
