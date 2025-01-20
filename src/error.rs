use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Ini(ini::Error),
    Parse(String),
    Serde(serde_json::Error),
    Rusqlite(rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO Error: {}", err),
            Error::Ini(ref err) => write!(f, "INI Error: {}", err),
            Error::Parse(ref desc) => write!(f, "Parse Error: {}", desc),
            Error::Serde(ref err) => write!(f, "Serde Error: {}", err),
            Error::Rusqlite(ref err) => write!(f, "Rusqlite Error: {}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Serde(err)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        Error::Rusqlite(err)
    }
}

impl From<ini::Error> for Error {
    fn from(err: ini::Error) -> Error {
        Error::Ini(err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Ini(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::Parse(_) => None,
            Error::Serde(ref err) => Some(err),
            Error::Rusqlite(ref err) => Some(err),
        }
    }
}
