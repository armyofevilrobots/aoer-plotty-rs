use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use geo::bounding_rect::BoundingRect;
use geo::map_coords::MapCoords;
use geo::translate::Translate;
use geo_types::{coord, Geometry, GeometryCollection, Rect};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::context::glyph_proxy::GlyphProxy;
use crate::geo_types::ToGTGeometry;
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::vector::Vector2F;

type TypographicBounds = RectF;

#[derive(Debug)]
pub enum TypographyError {
    FontError,
    NoFontSet,
    GlyphNotFound(u32),
}

impl Display for TypographyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TypographyError {}

#[derive(Clone, Debug)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
pub struct RenderedGlyph {
    geo: Geometry<f64>,
    pub bounds: TypographicBounds,
    pub advance: Vector2F,
}

#[derive(Clone, Debug)]
pub struct Typography {
    font: Option<Font>,
    hinting: HintingOptions,
    em: f64,
    close: bool,
    align: TextAlignment, // TODO: Add text-align, other stuff.
}

impl Typography {
    /// Default font
    pub fn default_font() -> Font {
        let font_data =
            include_bytes!("../../resources/fonts/ReliefSingleLineOutline-Regular.otf").to_vec();
        Font::from_bytes(Arc::new(font_data), 0).unwrap() // We know this font is OK
    }

    /// Generate a blank Typography instance
    pub fn new() -> Self {
        Typography {
            font: Some(Self::default_font()),
            hinting: HintingOptions::None,
            em: 1.0,
            close: false,
            align: TextAlignment::Left,
        }
    }

    pub fn mm_per_em() -> f64 {
        8.0f64 * 0.3527777778f64
    }

    pub fn align(&mut self, align: TextAlignment) -> &mut Self {
        self.align = align;
        self
    }

    pub fn close(&mut self, close: bool) -> &mut Self {
        self.close = close;
        self
    }

    pub fn hint(&mut self, hint: HintingOptions) -> &mut Self {
        self.hinting = hint;
        self
    }

    pub fn font(&mut self, font: &Font) -> &mut Self {
        self.font = Some(font.clone());
        self
    }

    pub fn size(&mut self, em: f64) -> &mut Self {
        self.em = em;
        self
    }

    pub fn render(&self, text: &String, accuracy: f64) -> Result<Geometry<f64>, Box<dyn Error>> {
        let font = match &self.font {
            None => return Err(Box::new(TypographyError::NoFontSet)),
            Some(font) => font.clone(),
        };
        let metrics = font.metrics();
        let mut glyphs: Vec<RenderedGlyph> = vec![];
        let mut advance = Vector2F::new(0.0, 0.0);
        for char in text.chars() {
            let mut gp = GlyphProxy::new(self.close);
            let glyph = font.glyph_for_char(char).or(Some(32)).unwrap();
            font.outline(glyph, self.hinting, &mut gp)?;
            let thisadvance = font.advance(glyph)?;
            let gtgeo = gp.path().to_gt_geometry(accuracy)?;
            // let gbounds = font.typographic_bounds(glyph)?;
            // println!("Advancing: {:?} for {:?} which has bounds: {:?} and self-advance of {:?}", advance.x(), char, gbounds.0.x(), thisadvance.x());
            let rglyph = RenderedGlyph {
                geo: gtgeo.translate(advance.x().into(), advance.y().into()),
                bounds: font.typographic_bounds(glyph)?,
                advance: advance.clone(),
            };
            advance = advance + thisadvance;
            //println!("GLYPH PUSHED: {:?}", &rglyph);
            glyphs.push(rglyph);
        }
        // Ok, now we have a collection of glyphs, we have to scale and transform (center, etc)
        // println!("OK, now we do a metrics calculation for scaling: {:?}", &metrics.units_per_em);
        let units_per_em: f64 = &self.em / f64::from(metrics.units_per_em);
        let output_geometries: Vec<Geometry<f64>> = glyphs
            .iter()
            .map(|g| {
                g.geo.map_coords(|xy| {
                    coord!(
                        x: units_per_em * xy.x * Self::mm_per_em(),
                        y: units_per_em * xy.y * Self::mm_per_em(),
                    )
                }) //.clone()
            })
            .collect();
        let output_geo_collection = GeometryCollection::new_from(output_geometries);
        let bounds = output_geo_collection
            .bounding_rect()
            .unwrap_or(Rect::new(coord! {x: 0.0, y:0.0}, coord! {x:0.0, y:0.0}));
        // println!("After scasling: {:?}", output_geo_collection);
        // println!("Scaled bounds: {:?}", bounds);
        let output_geo_collection = match &self.align {
            TextAlignment::Left => output_geo_collection,
            TextAlignment::Right => {
                output_geo_collection.translate(-(bounds.max().x - bounds.min().x), 0.0)
            }
            TextAlignment::Center => {
                output_geo_collection.translate(-(bounds.max().x - bounds.min().x) / 2.0, 0.0)
            }
        };
        Ok(Geometry::GeometryCollection(output_geo_collection))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::context::typography::Typography;
    use font_kit::font::Font;
    use std::sync::Arc;

    #[test]
    fn test_simple() {
        let fdata = include_bytes!("../../resources/fonts/ReliefSingleLine-Regular.ttf").to_vec();
        let f = Font::from_bytes(Arc::new(fdata), 0).unwrap(); // We know this font is OK
        let mut t = Typography::new();
        let _geo = t
            .size(2.0)
            .font(&f)
            .render(&"YES: This is some text XXX".to_string(), 0.1);
    }
}
