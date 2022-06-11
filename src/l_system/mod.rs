//! The l_system module provides a simple Lindenmeyer fractal generator for use with
//! plotted line-art. Take a look at the [`crate::l_system::LSystem`] struct for
//! more details, and examples.

use std::collections::HashMap;
use embed_doc_image::embed_doc_image;

/// # LSystem
///
/// What it says on the box; a simple L-system implementation for use with plotter
/// based fractal art.
///
/// # Example
///
/// ```rust
/// use aoer_plotty_rs::turtle::{TurtleTrait, Turtle, degrees};
/// use aoer_plotty_rs::l_system::LSystem;
/// use aoer_plotty_rs::geo_types::nannou::NannouDrawer;
/// use std::collections::HashMap;
/// use nannou::lyon::tessellation::{LineCap, LineJoin};
/// use nannou::Draw;
///
/// let draw = Draw::new();
/// let gosper = LSystem{
///     axiom: "A".to_string(),
///     rules: HashMap::from([
///         ('A', "A-B--B+A++AA+B-".to_string()),
///         ('B', "+A-BB--B-A++A+B". to_string())])
///     };
///
/// let tlines = Turtle::new()
///     .pen_down()
///     .walk_lpath(&gosper.expand(4), degrees(60.0), 8.0)
///     .to_multiline();
/// for line in tlines {
///     draw.polyline()
///         .stroke_weight(3.0)
///         .caps(LineCap::Round)
///         .join(LineJoin::Round)
///         .polyline_from_linestring(&line)
///         .color(nannou::color::NAVY);
/// }
/// ```
/// ![gosper-4][gosper-4]
#[embed_doc_image("gosper-4", "images/gosper-4.png")]

#[derive(Clone, Debug)]
pub struct LSystem{
    pub axiom: String,
    pub rules: HashMap<char, String>,
}

impl LSystem{

    fn recur(&self, state: String, order: u32)->String{
        let new_state = state.chars().map(|c|{
            match self.rules.get(&c){
                Some(replacement) => replacement.clone(),
                None => String::from(c)
            }
        }).collect();
        if order == 0{
            state
        }else{
            self.recur(new_state, order-1)
        }
    }

    /// #expand
    ///
    /// Expands the L-system by the requested "order" of iterations. Returns a string
    /// representing the state of the L-system. Useful with
    /// [`crate::turtle::TurtleTrait::walk_lpath`]
    pub fn expand(&self, order: u32) -> String{
        self.recur(self.axiom.clone(), order)
    }

}


#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_expand_simple(){
        let system = LSystem {
            axiom: "A".to_string(),
            rules: HashMap::from([
                ('A', "AB".to_string()),
                ('B', "A". to_string())]),
        };
        assert!(system.expand(2) == "ABA".to_string());
        assert!(system.expand(5) == "ABAABABAABAAB".to_string());
    }
}

