pub use kurbo::BezPath;
pub use kurbo::Point as BezPoint;
use kurbo::PathEl;
use font_kit::font::Font;
use font_kit::outline::OutlineSink;
use pathfinder_geometry::line_segment::LineSegment2F;
use pathfinder_geometry::vector::Vector2F;


pub struct GlyphProxy{
    path: BezPath
}

impl GlyphProxy{

    pub fn new() -> GlyphProxy{
        GlyphProxy { path: BezPath::new() }
    }

    pub fn path(&self) -> BezPath{
        self.path.clone()
    }

}

impl OutlineSink for GlyphProxy{
    fn move_to(&mut self, to: Vector2F) {
        self.path.move_to(BezPoint::new(f64::from(to.x()), f64::from(to.y())));
    }

    fn line_to(&mut self, to: Vector2F) {
        self.path.line_to(BezPoint::new(f64::from(to.x()), f64::from(to.y())));
    }

    fn quadratic_curve_to(&mut self, ctrl: Vector2F, to: Vector2F) {
        self.path.quad_to(
            BezPoint::new(f64::from(ctrl.x()), f64::from(ctrl.y())),
            BezPoint::new(f64::from(to.x()), f64::from(to.y()))
        );
    }

    fn cubic_curve_to(&mut self, ctrl: LineSegment2F, to: Vector2F) {
        self.path.curve_to(
            BezPoint::new(f64::from(ctrl.from().x()), f64::from(ctrl.from().y())),
            BezPoint::new(f64::from(ctrl.to().x()), f64::from(ctrl.to().y())),
            BezPoint::new(f64::from(to.x()), f64::from(to.y()))
        );
    }

    fn close(&mut self) {
        self.path.close_path();
    }


}