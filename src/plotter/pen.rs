pub use csscolorparser::Color as CssColor;
pub use csscolorparser::parse as parse_css_color;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PenDetail {
    #[serde(default)]
    pub tool_id: usize,
    #[serde(default)]
    pub name: String,
    pub stroke_width: f64,
    pub stroke_density: f64,
    pub feed_rate: Option<f64>, //mm/min
    // pub color: String,          // A CSS string color.
    pub color: CssColor,
}

impl Default for PenDetail {
    fn default() -> Self {
        Self {
            tool_id: 1,
            name: "Default Pen".to_string(),
            stroke_width: 0.5,
            stroke_density: 0.5,
            feed_rate: None,
            color: CssColor::from_rgba8(0, 0, 0, 255),
        }
    }
}
