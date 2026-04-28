use std::fmt;

#[derive(Debug)]
pub enum ChurnLensError {
    ParseError(String),
    GitError(String),
    IoError(String),
}

impl fmt::Display for ChurnLensError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChurnLensError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ChurnLensError::GitError(msg) => write!(f, "Git error: {}", msg),
            ChurnLensError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ChurnLensError {}
