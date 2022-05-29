use crate::turtle::*;
use std::collections::HashMap;

/// # LSystem
///
/// What it says on the box; a simple L-system implementation for use with plotter
/// based fractal art.
///
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

