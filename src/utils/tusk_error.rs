use std::{
    fmt::{self},
    io,
};

#[derive(Debug)]
pub enum TuskError {
    IndexOutOfRange { index: usize, max: usize },
    InvalidInput { message: String },
    Io(io::Error),
}

impl TuskError {
    pub fn code(&self) -> &'static str {
        match self {
            TuskError::IndexOutOfRange { .. } => "index_out_of_range",
            TuskError::InvalidInput { .. } => "invalid_input",
            TuskError::Io(_) => "io_error"
        }
    }
}

impl fmt::Display for TuskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TuskError::IndexOutOfRange { index, max } => {
                write!(f, "Index {} is out of range (max: {})", index, max)
            }
            TuskError::InvalidInput { message } => {
                write!(f, "Invalid input: {}", message)
            }
            TuskError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for TuskError {}

impl From<io::Error> for TuskError {
    fn from(err: io::Error) -> Self {
        TuskError::Io(err)
    }
}