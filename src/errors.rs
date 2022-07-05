use std::fmt;


#[derive(Debug)]
pub enum SvgCreationError {
    NullGeometry,
}

impl std::error::Error for SvgCreationError {}

impl fmt::Display for SvgCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SvgCreationError::NullGeometry =>
                write!(f, "Empty/Invalid/Dimensionless geometry"),
        }
    }
}

#[derive(Debug)]
pub enum ContextError {
    PoppedEmptyStack,
    SvgGenerationError(String)
}

impl std::error::Error for ContextError {}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContextError::PoppedEmptyStack =>
                write!(f, "Popping from an empty context stack."),
            ContextError::SvgGenerationError(msg) =>
                write!(f, "Svg generation error: {}", msg),
        }
    }
}