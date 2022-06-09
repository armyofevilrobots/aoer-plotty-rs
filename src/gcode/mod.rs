//! Module which provides Line->GCode post-processing
use geo_types::{CoordNum, MultiLineString};
use tera::{Context, Tera};
use std::error::Error;
use num_traits::real::Real;

/// List of all available machines as an Enum
pub enum AoerPostMachines {
    BAPv1,
    CustomMachine(Tera),
}

#[derive(Debug)]
pub enum PostTemplateError {
    NoSuchTemplateError,
    TemplateStructureError,
}

#[derive(Debug,Clone)]
pub enum PostGeometrySource<T>
    where T: CoordNum, T: Real{
    MultiLineString(MultiLineString<T>),
}


/// # AoerPostMachines
impl AoerPostMachines {
    /// Getter for machine templates for the gcode processor. You'll need one of these
    /// templates to generate gcode, and these are the lookups tables for the various
    /// commands the post-processor needs.
    pub fn get_machine(machine: AoerPostMachines) -> Result<Tera, PostTemplateError> {
        let mut bap_post_template = Tera::default();
        match machine {
            AoerPostMachines::BAPv1 => {
                bap_post_template.add_raw_templates(vec![
                    ("prelude", "M280 S5\nG4 P150\nG28 X Y\nG90\n G92 X0 Y0 ; HOME"),
                    ("epilog", "M280 S5\nG4 P150\nG0 X0 Y230\nM281 ; FINISHED"),
                    ("penup", "M400\nM280 S9\nG4 P150\nM400\nM281 ; PENUP"),
                    ("pendown", "M400\nM280 S12\nG4 P250\nM400 ; PENDOWN"),
                    ("moveto", "G0 X{{xmm|round(precision=2)}} Y{{ymm|round(precision=2)}} ; NEW LINE START"),
                    ("lineto", "G01 F1200 X{{xmm|round(precision=2)}} Y{{ymm|round(precision=2)}}"),
                ]).unwrap();
                Ok(bap_post_template)
            }
            _ => Err(PostTemplateError::NoSuchTemplateError)
        }
    }
}


/// Given a set of lines, gcode-process and generate GCode
/// Returns either a list of gcode lines, or a box'd dyn error
/// for what went wrong.
pub fn post<T>(lines: &PostGeometrySource<T>, post_template: &Tera)
               -> Result<Vec<String>, Box<dyn Error>>
    where T: CoordNum, T: Real {
    let mut program: Vec<String> = Vec::new();
    // This currently only allows for a single enum subtype, but that's all we have
    // for now. Eventually we'll expand this to include a variety of subtypes, like
    // svg2polyline lines, and even multi-layers with separate tools.
    let lines = match lines{
        PostGeometrySource::MultiLineString(lines) => lines
    };
    program.extend(
        post_template.render("prelude", &Context::new())?
            .split("\n").map(|s| s.to_string()));
    for line in lines.iter() {
        program.extend(post_template.render("penup", &Context::new())?
            .split("\n")
            .map(|s| s.to_string()));
        let mut context = Context::new();
        context.insert("xmm", &line[0].x.to_f64().unwrap());
        context.insert("ymm", &line[0].y.to_f64().unwrap());
        program.extend(
            post_template.render("moveto", &context)?
                .split("\n")
                .map(|s| s.to_string()));

        program.extend(post_template.render("pendown", &Context::new())?
            .split("\n")
            .map(|s| s.to_string()));
        for point in line.points().skip(1) {
            let mut context = Context::new();
            context.insert("xmm", &point.x().to_f64().unwrap());
            context.insert("ymm", &point.y().to_f64().unwrap());
            program.extend(
                post_template.render("lineto", &context)?
                    .split("\n").map(|s| s.to_string()));
        }
    }
    program.extend(
        post_template.render("epilog", &Context::new())?
            .split("\n").map(|s| s.to_string()));
    Ok(program)
}


#[cfg(test)]
mod test {
    use std::iter::zip;
    use geo_types::{coord, LineString, MultiLineString};
    use crate::gcode::{AoerPostMachines, post, PostGeometrySource};

    #[test]
    fn test_post() {
        let post_template = AoerPostMachines::get_machine(AoerPostMachines::BAPv1)
            .unwrap();
        let lines = MultiLineString::new(vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0}])]);
        let program = post(&PostGeometrySource::MultiLineString(lines), &post_template)
            .unwrap();
        let pairs: Vec<(String, String)> = zip(program, vec!["M280 S5", "G4 P150", "G28 X Y",
                                                             "G90", " G92 X0 Y0 ; HOME", "M400",
                                                             "M280 S9", "G4 P150", "M400",
                                                             "M281 ; PENUP",
                                                             "G0 X0 Y0 ; NEW LINE START", "M400",
                                                             "M280 S12", "G4 P250",
                                                             "M400 ; PENDOWN", "G01 F1200 X10 Y0",
                                                             "M280 S5", "G4 P150", "G0 X0 Y230",
                                                             "M281 ; FINISHED"].iter()
            .map(|s| { s.to_string() })).collect();
        for (left, right) in pairs {
            assert!(left == right);
        }
    }
}