use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum AppError {
    MissingFile {
        path: PathBuf,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Yaml {
        path: PathBuf,
        message: String,
        line: Option<usize>,
    },
    Validation {
        path: PathBuf,
        message: String,
    },
    Internal {
        message: String,
    },
}

impl AppError {
    pub fn missing_file(path: &Path) -> Self {
        Self::MissingFile {
            path: path.to_path_buf(),
        }
    }

    pub fn io(path: &Path, source: std::io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    pub fn yaml(path: &Path, message: String, line: Option<usize>) -> Self {
        Self::Yaml {
            path: path.to_path_buf(),
            message,
            line,
        }
    }

    pub fn validation(path: &Path, message: String) -> Self {
        Self::Validation {
            path: path.to_path_buf(),
            message,
        }
    }

    pub fn internal(message: String) -> Self {
        Self::Internal { message }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingFile { path } => write!(f, "missing file: {}", path.display()),
            AppError::Io { path, source } => {
                write!(f, "io error reading {}: {}", path.display(), source)
            }
            AppError::Yaml {
                path,
                message,
                line,
            } => {
                if let Some(line) = line {
                    write!(
                        f,
                        "malformed YAML at {}:{}: {}",
                        path.display(),
                        line,
                        message
                    )
                } else {
                    write!(f, "malformed YAML at {}: {}", path.display(), message)
                }
            }
            AppError::Validation { path, message } => {
                write!(f, "validation error in {}: {}", path.display(), message)
            }
            AppError::Internal { message } => write!(f, "internal error: {}", message),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
