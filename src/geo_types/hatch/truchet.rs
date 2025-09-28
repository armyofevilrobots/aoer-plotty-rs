use super::HatchPattern;
use geo::{Coord, MapCoords, MapCoordsInPlace};
use geo_types::{LineString, MultiLineString, Rect};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// A repeating Truchet hatch, optionally with multiple tiles which
/// are statistically chosen based on their weight.
#[derive(Deserialize, Clone, PartialEq, Default)]
pub struct TruchetHatch {
    /// The native scale of the hatch, ie: if scale is 0.5, and the fill
    /// is invoked at 0.75 scale, then the tiles will be scaled up 1.5x.
    /// Note that multi-tile TruchetHatch instances MUST have matching
    /// sized tiles. If they are different sizes, they'll overlap or do
    /// other unexpected things.
    pub scale: f64,
    pub seed: u64,
    pub tile_size: (f64, f64),
    pub tiles: Vec<(u32, MultiLineString)>,
}

impl Debug for TruchetHatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TruchetHatch")
            .field("scale", &self.scale)
            .field("seed", &self.seed)
            .field("tile_size", &self.tile_size)
            .field("#tiles", &self.tiles.len())
            .finish()
    }
}

impl TruchetHatch {
    /// This is a very simple diagonal line hatch. Mostly for example/testing
    /// purposes. You should use LineHatch directly instead for better
    /// performance.
    pub fn minimal_line(scale: f64) -> Arc<Box<dyn HatchPattern>> {
        let tile = LineString::new(vec![Coord { x: 0.0, y: 0.0 }, Coord { x: scale, y: scale }]);
        let tile = MultiLineString::new(vec![tile]);
        let hatch = TruchetHatch {
            scale: scale,
            seed: 0,
            tile_size: (scale, scale),
            tiles: vec![(1, tile)],
        };
        Arc::new(Box::new(hatch))
    }

    /*
    pub fn carlson_smith_truchet(scale: f64) -> Arc<Box<dyn HatchPattern>> {
        let hatch = TruchetHatch {
            scale: scale,
            seed: 0,
            tile_size: (scale, scale),
            tiles: vec![(1, tile)],
        };
        Arc::new(Box::new(hatch))
    }
    */
}

impl HatchPattern for TruchetHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString<f64> {
        let min = bbox.min();
        let max = bbox.max();
        let scale_mul = scale / self.scale;
        let mut lines: Vec<geo_types::LineString<f64>> = vec![];
        if self.tiles.len() > 0 {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(self.seed);
            let mut new_tiles = self.tiles.clone();
            for tile in &mut new_tiles {
                tile.1.map_coords_in_place(|coord| Coord {
                    x: coord.x * scale_mul,
                    y: coord.y * scale_mul,
                });
            }
            let tile_list: Vec<&MultiLineString> = new_tiles.iter().map(|x| &x.1).collect();
            let weights: Vec<u32> = new_tiles.iter().map(|x| x.0).collect();
            let dist = WeightedIndex::new(&weights).unwrap();
            println!("Tile size: {:?}", self.tile_size);

            let mut y = min.y - self.tile_size.1;
            while y < max.y + self.tile_size.1 {
                let mut x = min.x - self.tile_size.0;
                while x < max.x + self.tile_size.0 {
                    // println!("XY:{},{}", x, y);
                    let tile = tile_list
                        .get(dist.sample(&mut rng))
                        .expect("Failure in random selection algorithm");
                    lines.extend(tile.map_coords(|coord| Coord {
                        x: x + coord.x,
                        y: y + coord.y,
                    }));
                    x += self.tile_size.0 * scale_mul;
                    // break;
                }
                y += self.tile_size.1 * scale_mul;
                // break;
            }
            // println!("End of tiling.");
        }
        let out = MultiLineString::<f64>::new(lines);
        // println!("DONE");
        out
    }
}

#[cfg(test)]
mod test {
    use super::HatchPattern;
    use super::TruchetHatch;
    use geo::{Coord, LineString, MultiLineString, Rect};

    #[test]
    pub fn test_simple() {
        let tile = LineString::new(vec![Coord { x: 0.0, y: 0.0 }, Coord { x: 10., y: 10. }]);
        let tile = MultiLineString::new(vec![tile]);

        let hatch = TruchetHatch {
            tiles: vec![(1, tile)],
            scale: 10.,
            seed: 0,
            tile_size: (10., 10.),
        };

        let _out = hatch.generate(
            &Rect::new(Coord { x: -10., y: -10. }, Coord { x: 30., y: 30. }),
            10.,
            0.5,
        );
    }
}
