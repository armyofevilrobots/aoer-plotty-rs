use anyhow::Result;
use std::collections::HashMap;

use geo::{Coord, Rect};

#[derive(Debug, Default, Clone)]
pub struct HilbertSpatialHash {
    bounds: Option<Rect<f64>>,
    entries: HashMap<(u32, u32), Vec<Coord<f64>>>,
    order: Option<Vec<Coord>>,
    i: usize,
}

impl HilbertSpatialHash {
    pub fn new() -> HilbertSpatialHash {
        HilbertSpatialHash {
            bounds: None,
            order: None,
            i: 0,
            entries: HashMap::new(),
        }
    }

    pub fn with_bounds(self, bounds: Rect) -> Self {
        HilbertSpatialHash {
            bounds: Some(bounds),
            entries: HashMap::new(), // Clears entries when rect is changed.
            order: None,
            ..self
        }
    }

    pub fn calc_bucket_key(&self, coord: &Coord) -> Result<(u32, u32)> {
        if self.bounds.is_none() {
            return Err(anyhow::anyhow!("No bounds set yet!"));
        };
        let min = self.bounds.unwrap().min();
        let max = self.bounds.unwrap().max();
        // Calculate the span that fits the entire bounds rect into
        // the u32xu32 span 'window'
        let span_mul: f64 = (self.bounds.unwrap().width() / u32::MAX as f64)
            .min(self.bounds.unwrap().height() / u32::MAX as f64);
        let bucket_x = ((coord.x - min.x) / span_mul).trunc().min(u32::MAX as f64) as u32;
        let bucket_y = ((coord.y - min.y) / span_mul).trunc().min(u32::MAX as f64) as u32;
        // println!(
        //     "Span_mul: {} for coord: {:?} has hash key {},{}",
        //     span_mul, coord, bucket_x, bucket_y
        // );
        Ok((bucket_x, bucket_y))
    }

    pub fn add(&mut self, coord: &Coord) -> Result<()> {
        let key = self.calc_bucket_key(coord)?;
        // let tmp_vec = vec![];
        if let Some(val) = self.entries.get_mut(&key) {
            if val.contains(coord) {
                Ok(())
            } else {
                Ok(val.push(coord.clone()))
            }
        } else {
            self.entries.insert(key, vec![coord.clone()]);
            Ok(())
        }
    }

    pub fn calculate(&mut self) -> Result<()> {
        let mut coords: Vec<(u64, Coord)> = vec![];
        for ((keyx, keyy), entry) in &self.entries {
            let id = fast_hilbert::xy2h(*keyx, *keyy, 32);
            for coord in entry {
                coords.push((id, coord.clone()))
            }
        }
        coords.sort_by(|a, b| a.0.cmp(&b.0));
        self.order = Some(coords.iter().map(|(_i, coord)| *coord).collect());
        Ok(())
    }
}

impl Iterator for HilbertSpatialHash {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.order.is_none() {
            if let Err(_) = self.calculate() {
                return None;
            } else {
                self.i = 0;
            };
        }
        let order = self.order.as_ref().unwrap();
        if self.i >= order.len() {
            return None;
        };
        let out = order.get(self.i).unwrap();
        self.i += 1;
        Some(out.clone())
    }
}

#[cfg(test)]
pub mod test {
    use std::time;

    use geo::{Coord, Rect};
    use rand::{Rng, SeedableRng, rngs::SmallRng};

    use crate::geo_types::hilbert_spatial_hash::HilbertSpatialHash;
    #[test]
    pub fn test_coord_ordering() {
        let rect = Rect::new(Coord { x: 0., y: 0. }, Coord { x: 100., y: 100. });
        let mut h = HilbertSpatialHash::new().with_bounds(rect);
        let coords = vec![
            Coord { x: 2., y: 2. },
            Coord { x: 12., y: 3. },
            Coord { x: 80., y: 37. },
            Coord {
                x: 80.000001,
                y: 37.,
            },
            Coord { x: 100., y: 100. },
        ];
        for coord in coords {
            h.add(&coord).expect("Failed to insert");
        }

        for coord in h.into_iter() {
            println!("COORD: {:?}", coord);
        }
    }

    #[test]
    pub fn test_coord_clamping() {
        let rect = Rect::new(Coord { x: 0., y: 0. }, Coord { x: 100., y: 100. });
        let mut h = HilbertSpatialHash::new().with_bounds(rect);
        let coords = vec![
            Coord { x: 2., y: 2. },
            Coord { x: 12., y: 3. },
            Coord { x: 80., y: 37. },
            Coord {
                x: 80.000001,
                y: 37.,
            },
            Coord { x: 100., y: 100. },
            Coord { x: 110., y: 110. },
            Coord { x: -100., y: -100. },
            Coord { x: -100., y: 90. },
        ];
        for coord in coords {
            h.add(&coord).expect("Failed to insert");
        }
        println!("After building hsh: {:?}", h);
    }

    #[test]
    pub fn risky_benching_coord_ordering() {
        let rect = Rect::new(Coord { x: 0., y: 0. }, Coord { x: 100., y: 100. });
        let now = time::Instant::now();
        let mut h = HilbertSpatialHash::new().with_bounds(rect);
        let mut rng = SmallRng::seed_from_u64(0);
        for coord in 0..100000 {
            h.add(&Coord {
                x: rng.gen_range(0.0..100.0),
                y: rng.gen_range(0.0..100.0),
            })
            .expect("Failed to insert");
        }
        let elapsed = time::Instant::now() - now;
        // for coord in &mut h.into_iter() {
        //     println!("\t{:?}", coord);
        // }
        println!("Took {:?} to calculate 100k items", elapsed);

        // println!("After building hsh: {:?}", h);
    }
}
