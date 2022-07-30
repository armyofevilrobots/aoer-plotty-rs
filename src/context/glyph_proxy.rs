pub use kurbo::BezPath;
pub use kurbo::Point as BezPoint;
use kurbo::PathEl;
use font_kit::font::Font;
use font_kit::outline::OutlineSink;
use pathfinder_geometry::line_segment::LineSegment2F;
use pathfinder_geometry::vector::Vector2F;


pub struct GlyphProxy{
    path: BezPath,
    last_pos: Vector2F,
    close: bool
}

impl GlyphProxy{

    pub fn new(close: bool) -> GlyphProxy{
        GlyphProxy { path: BezPath::new() , last_pos: Vector2F::zero(), close: close}
    }

    pub fn path(&self) -> BezPath{
        // println!("Getting path: {:?}", &self.path);
        self.path.clone()
    }

}

impl OutlineSink for GlyphProxy{
    fn move_to(&mut self, to: Vector2F) {
        // println!("MOVE TO {:?}", &to);
        self.path.move_to(BezPoint::new(f64::from(to.x()), f64::from(to.y())));
        self.last_pos = to.clone()
    }

    fn line_to(&mut self, to: Vector2F) {
        // println!("LINE TO {:?}", &to);
        self.path.line_to(BezPoint::new(f64::from(to.x()), f64::from(to.y())));
    }

    fn quadratic_curve_to(&mut self, ctrl: Vector2F, to: Vector2F) {
        // println!("QUAD TO {:?}, {:?}", &ctrl, &to);
        self.path.quad_to(
            BezPoint::new(f64::from(ctrl.x()), f64::from(ctrl.y())),
            BezPoint::new(f64::from(to.x()), f64::from(to.y()))
        );
    }

    fn cubic_curve_to(&mut self, ctrl: LineSegment2F, to: Vector2F) {
        // println!("CUBIC TO {:?}, {:?}", &ctrl, &to);
        self.path.curve_to(
            BezPoint::new(f64::from(ctrl.from().x()), f64::from(ctrl.from().y())),
            BezPoint::new(f64::from(ctrl.to().x()), f64::from(ctrl.to().y())),
            BezPoint::new(f64::from(to.x()), f64::from(to.y()))
        );
    }

    fn close(&mut self) {
        // println!("CLOSE");
        if ! self.close{
            self.move_to(self.last_pos.clone());
        }else{
            self.line_to(self.last_pos.clone());
        }

        self.path.close_path();
    }


}