use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum WorkflowError {
    Alfrusco(alfrusco::Error),
    Linkcache(linkcache::Error),
    Io(std::io::Error),
}

impl alfrusco::WorkflowError for WorkflowError {}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WorkflowError::Alfrusco(ref err) => err.fmt(f),
            WorkflowError::Linkcache(ref err) => err.fmt(f),
            WorkflowError::Io(ref err) => err.fmt(f),
        }
    }
}

impl Error for WorkflowError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            WorkflowError::Alfrusco(ref err) => Some(err),
            WorkflowError::Linkcache(ref err) => Some(err),
            WorkflowError::Io(ref err) => Some(err),
        }
    }
}

impl From<alfrusco::Error> for WorkflowError {
    fn from(value: alfrusco::Error) -> Self {
        WorkflowError::Alfrusco(value)
    }
}

impl From<linkcache::Error> for WorkflowError {
    fn from(value: linkcache::Error) -> Self {
        WorkflowError::Linkcache(value)
    }
}

impl From<std::io::Error> for WorkflowError {
    fn from(value: std::io::Error) -> Self {
        WorkflowError::Io(value)
    }
}