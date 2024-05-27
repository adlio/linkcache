use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(String),
    Serde(serde_json::Error),
    Tantivy(tantivy::TantivyError),
    TantivyQueryParser(tantivy::query::QueryParserError),
    Rusqlite(rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO Error: {}", err),
            Error::Parse(ref desc) => write!(f, "Parse Error: {}", desc),
            Error::Tantivy(ref err) => write!(f, "Tantivy Error: {}", err),
            Error::Serde(ref err) => write!(f, "Serde Error: {}", err),
            Error::TantivyQueryParser(ref err) => {
                write!(f, "Tantivy Query Parser Error: {}", err)
            }
            Error::Rusqlite(ref err) => write!(f, "Rusqlite Error: {}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<tantivy::TantivyError> for Error {
    fn from(err: tantivy::TantivyError) -> Error {
        Error::Tantivy(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Serde(err)
    }
}

impl From<tantivy::query::QueryParserError> for Error {
    fn from(err: tantivy::query::QueryParserError) -> Error {
        Error::TantivyQueryParser(err)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        Error::Rusqlite(err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Parse(_) => None,
            Error::Tantivy(ref err) => Some(err),
            Error::Serde(ref err) => Some(err),
            Error::TantivyQueryParser(ref err) => Some(err),
            Error::Rusqlite(ref err) => Some(err),
        }
    }
}
