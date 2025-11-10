use geo::Coord;
/// Creates a bitfield out of an image.
///
use geo_types::{Point, Rect};
use image::{GrayImage, Luma, imageops::colorops::invert};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use std::fmt::Debug;

use super::PointField;

#[derive(Clone, Default, Debug, PartialEq)]
pub enum DitherType {
    //[ ... ...  *  1/8 1/8
    //  ... 1/8 1/8 1/8 ...
    //  ... ... 1/8 ... ... ]
    #[default]
    Atkinson,
    FloydSteinberg,
}

impl DitherType {
    pub fn matrix(&self) -> Vec<(i32, i32, i32, i32)> {
        match self {
            DitherType::Atkinson => vec![
                (1, 0, 1_i32, 8_i32),
                (2, 0, 1, 8),
                (-1, 1, 1, 8),
                (0, 1, 1, 8),
                (1, 1, 1, 8),
                (0, 2, 1, 8),
            ],
            DitherType::FloydSteinberg => {
                vec![(1, 0, 7, 16), (-1, 1, 3, 16), (0, 1, 5, 16), (1, 1, 1, 16)]
            }
        }
    }
}

#[derive(Clone, Default)]
pub enum PassStrategy {
    #[default]
    Normal,
    Serpentine,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BitmapPointField {
    bounds: Option<Rect<f64>>,
    bitmap: Box<GrayImage>,
    dither_type: DitherType,

    threshold: f64, // threshold for black/white decision. 0.0->1.0
    gain: f64,      // How much to weight the diffused pixels when distributing.
    scratch: Option<GrayImage>,
    ithresh: u8,
    ix: u32,
    iy: u32,
    rng: Option<SmallRng>,
}

impl BitmapPointField {
    pub fn new(image: GrayImage, gain: f64, threshold: f64) -> BitmapPointField {
        BitmapPointField {
            bounds: Some(Rect::new(
                Coord { x: 0., y: 0. },
                Coord {
                    x: (image.width()) as f64,
                    y: (image.height()) as f64,
                },
            )),
            bitmap: Box::new(image),
            dither_type: DitherType::default(),
            threshold,
            ithresh: (threshold * 255.).max(0.).min(255.) as u8,
            gain,
            scratch: None,
            ix: 0,
            iy: 0,
            rng: None,
        }
    }
    pub fn with_dither(self, dither_type: DitherType) -> Self {
        Self {
            dither_type,
            ..self
        }
    }
}

impl PointField for BitmapPointField {
    fn bounds(&self) -> geo::Rect {
        self.bounds
            .expect("No bounds set. Uninit'd BitmapPointField?")
            .clone()
    }
}

impl Iterator for BitmapPointField {
    type Item = Point<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rng.is_none() {
            self.rng = Some(SmallRng::seed_from_u64(0_u64));
        };

        /*
        let bounds = self
            .bounds
            .expect("Failed to get bounds. Improperly initialized BitmapPointField?");*/
        let img = match &mut self.scratch {
            Some(img) => img,
            None => {
                self.scratch = Some(*self.bitmap.clone());
                let scratch = self.scratch.as_mut().unwrap();
                invert(scratch);

                // Plotters are ink on paper,so subtractive color (generally). We invert the
                // image we work with to match that expectation. We accumulate black until
                // we hit the threshold and yield a point.
                scratch.pixels_mut().for_each(|p| {
                    *p = Luma::<u8>::from([(*p.0.get(0).unwrap() as f64 * self.gain) as u8]);
                });
                scratch
            }
        };
        let min = self.bounds.unwrap().min();
        let max = self.bounds.unwrap().max();
        let min = (min.x as u32, min.y as u32);
        let max = (max.x as u32 - 1, max.y as u32 - 1);
        loop {
            let oldpixel = img.get_pixel(self.ix, self.iy).0.get(0).unwrap();
            // println!("OLDPIXEL:{} ITHRESH:{}", oldpixel, self.ithresh);
            // let norm_pixel = (*oldpixel.0.get(0).unwrap() as f64) / 255.;
            // println!("NORM: {}, THRESH: {}", norm_pixel, self.threshold);
            let (new_pixel, point, q_err) = if oldpixel >= &self.ithresh {
                // println!("Should return a point at {},{}", self.ix, self.iy);
                (
                    255,
                    Some(Point::new(self.ix as f64, self.iy as f64)),
                    (*oldpixel as i32 - 255_i32/*+ self.rng.as_mut().unwrap().gen_range(-1..1)*/), // .min(255)
                                                                                                   // .max(0) as i32,
                )
            } else {
                (0, None, *oldpixel as i32)
            };

            img.put_pixel(self.ix, self.iy, Luma::<u8>::from([new_pixel as u8]));
            for (xofs, yofs, num, denom) in self.dither_type.matrix() {
                let xx = (self.ix as i32 + xofs) as u32;
                let yy = (self.iy as i32 + yofs) as u32;
                if min.0 <= xx && xx < max.0 && min.1 <= yy && yy < max.1 {
                    let pixel = img.get_pixel(xx, yy).0[0] as u16;
                    // println!("GOT PIXEL:{}", pixel);
                    // let new_pixel = pixel + (frac * q_err * 255.).min(255.).max(0.);
                    let new_pixel = pixel + ((q_err * num) / denom).min(255).max(0) as u16;
                    // let _new_pixel =
                    img.put_pixel(xx, yy, Luma::<u8>::from([new_pixel as u8]));
                }
            }

            self.ix += 1;
            if self.ix >= max.0 {
                self.ix = min.0 as u32;
                self.iy += 1;
                if self.iy >= max.1 as u32 {
                    // println!("SCRATCH AFTER: {:?}", img);
                    // img.save("images/test_out.png").unwrap();
                    return None;
                }
            }
            if point.is_some() {
                return point;
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::path::Path;

    use super::*;
    use image::{Luma, Pixel};

    #[test]
    pub fn test_simple_fill() {
        // println!("Test atkinson dither getting coords");
        let img =
            image::GrayImage::from_fn(320, 200, |_x, _y| Luma::<u8>::from_slice(&[123u8]).clone());
        let pf = BitmapPointField::new(img, 0.3, 0.5);
        let out: Vec<Point> = pf.collect();
        println!("LEN: {:?}", out.len());
        assert!(out.len() == 4944);
        // for p in pf {
        // println!("P:{:?}", p);
        // }
    }

    #[test]
    pub fn test_from_bmap() {
        let img = image::ImageReader::open(Path::new("images/aoer_logo.png"))
            .expect("Failed to open image.");
        let img = img.decode().expect("Failed to decode image.");
        let img = img.grayscale();
        let pf = BitmapPointField::new(img.grayscale().into(), 1., 1.);
        let out: Vec<Point> = pf.collect();
    }
}
