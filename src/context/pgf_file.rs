use serde;
// use std::default;
use std::io::Read;
use std::path::PathBuf;

use anyhow::Result;
use geo::prelude::MapCoords;
use geo::{Coord, Geometry};
use nalgebra::{Affine2, Point2};
use serde::{Deserialize, Serialize};

use crate::plotter::pen::PenDetail;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Copy)]
pub enum KeepdownStrategy {
    None,
    #[default]
    PenWidthAuto,
    PenWidthMultiple(f64),
    Static(f64),
}
impl KeepdownStrategy {
    pub fn threshold(&self, penwidth: f64) -> f64 {
        match self {
            KeepdownStrategy::None => 0.1,
            KeepdownStrategy::PenWidthAuto => 1.414f64 * penwidth,
            KeepdownStrategy::PenWidthMultiple(mul) => mul * penwidth,
            KeepdownStrategy::Static(val) => *val,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlotGeometry {
    // pub id: u64, // This is gonna be a pain in the ass later :()
    pub geometry: Geometry,
    // pub hatch: Option<HatchDetail>,
    pub stroke: Option<PenDetail>,
    // #[serde(default)]
    pub keepdown_strategy: KeepdownStrategy,
    // pub meta: HashMap<String, String>,
}

impl Default for PlotGeometry {
    fn default() -> Self {
        Self {
            // id: 0,
            geometry: Geometry::GeometryCollection(geo::GeometryCollection::new_from(vec![])),
            // hatch: Default::default(),
            stroke: Default::default(),
            keepdown_strategy: Default::default(),
            // meta: Default::default(),
        }
    }
}

impl PlotGeometry {
    pub fn transformed(&self, transformation: &Affine2<f64>) -> PlotGeometry {
        let mut new_geo = self.clone();
        new_geo.geometry = new_geo.geometry.map_coords(|coord| {
            let out = transformation * Point2::new(coord.x, coord.y);
            Coord { x: out.x, y: out.y }
        });
        new_geo
    }
}

/// Plotter Geometry Format -> Just lines and pens, that's all.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PGF {
    geometries: Vec<PlotGeometry>,
}

impl PGF {
    pub fn new() -> PGF {
        PGF {
            geometries: Vec::new(),
        }
    }

    pub fn geometries(&self) -> Vec<PlotGeometry> {
        self.geometries.clone()
    }

    pub fn add(&mut self, geom: PlotGeometry) {
        self.geometries.push(geom);
    }

    pub fn to_string(&self) -> String {
        ron::to_string(self).expect("Somehow we mangled our own PGF datastructure?!")
    }

    pub fn to_file(&self, path: &PathBuf) -> Result<()> {
        let path = path.with_extension("pgf");
        let tmp_path = path.with_added_extension(format!("tmp-{}", rand::random::<usize>()));
        // let content = self.to_string();
        let writer = std::fs::File::create(tmp_path.clone())?;
        // ron::ser::to_io_writer_pretty(writer, self, PrettyConfig::default())?;
        // ron::Options::default().to_io_writer_pretty(writer, &self, PrettyConfig::default())?;
        ron::Options::default().to_io_writer(writer, &self)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    pub fn from_file(path: &PathBuf) -> Result<PGF> {
        let mut reader = std::fs::File::open(path)?;
        let mut data = String::new();
        reader.read_to_string(&mut data)?;
        let pgf = ron::from_str(data.as_str())?;
        Ok(pgf)
    }
}
