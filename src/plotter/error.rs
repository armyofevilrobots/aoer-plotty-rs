use std::fmt;
use std::fmt::Formatter;
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum PlotterConnectionError {
    #[allow(dead_code)]
    IOError(i32),
    DeviceError(String),
    ParseError(String),
    UnknownError,
}

impl From<url::ParseError> for PlotterConnectionError {
    fn from(error: url::ParseError) -> Self {
        PlotterConnectionError::DeviceError(error.to_string())
    }
}

impl From<ParseIntError> for PlotterConnectionError {
    fn from(error: ParseIntError) -> Self {
        PlotterConnectionError::ParseError(error.to_string())
    }
}

impl From<serialport::Error> for PlotterConnectionError {
    fn from(error: serialport::Error) -> Self {
        PlotterConnectionError::DeviceError(error.to_string())
    }
}
