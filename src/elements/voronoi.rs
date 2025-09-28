use voronoice::*;

pub struct VoronoiDiagram {
    voronoi: Option<Voronoi>,
}

impl Default for VoronoiDiagram {
    fn default() -> Self {
        Self { voronoi: None }
    }
}

pub struct VoronoiDiagramBuilder {}

/*
impl VoronoiDiagram {
    pub fn from_noise_field(noise_field: impl super::PointField) -> VoronoiDiagram {
        let dia = VoronoiBuilder::default()
            .sites(noise_field.)
    }

}
*/
