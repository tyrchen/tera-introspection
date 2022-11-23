use std::error::Error as StdError;
use std::fmt;

/// The kind of an error (non-exhaustive)
#[non_exhaustive]
#[derive(Debug)]
pub enum ErrorKind {
    /// Generic error
    Msg(String),
    /// An error happened while serializing JSON
    Json(serde_json::Error),
    /// An IO error occurred
    Io(std::io::ErrorKind),
}

/// The Error type
#[derive(Debug)]
pub struct Error {
    /// Kind of error
    pub kind: ErrorKind,
    source: Option<Box<dyn StdError + Sync + Send>>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Msg(ref message) => write!(f, "{}", message),
            ErrorKind::Json(ref e) => write!(f, "{}", e),
            ErrorKind::Io(ref io_error) => {
                write!(
                    f,
                    "Io error while writing rendered value to output: {:?}",
                    io_error
                )
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|c| &**c as &(dyn StdError + 'static))
    }
}

impl Error {
    /// Creates generic error
    pub fn msg(value: impl ToString) -> Self {
        Self {
            kind: ErrorKind::Msg(value.to_string()),
            source: None,
        }
    }

    /// Creates JSON error
    pub fn json(value: serde_json::Error) -> Self {
        Self {
            kind: ErrorKind::Json(value),
            source: None,
        }
    }

    /// Creates an IO error
    pub fn io_error(error: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(error.kind()),
            source: Some(Box::new(error)),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::io_error(error)
    }
}
impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Self::msg(e)
    }
}
impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::msg(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::json(e)
    }
}
/// Convenient wrapper around std::Result.
pub type Result<T> = ::std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    #[test]
    fn test_error_is_send_and_sync() {
        fn test_send_sync<T: Send + Sync>() {}

        test_send_sync::<super::Error>();
    }
}
