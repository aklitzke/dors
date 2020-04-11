use std::fmt;

// All gracefully handled errors
#[derive(Debug)]
pub enum DorsError {
    CouldNotParseDorsfile(toml::de::Error),
    NoDorsfile,
    NoMemberDorsfile,
    NoTask(String),
    Unknown(Box<dyn std::error::Error>),
}

pub trait Error: std::error::Error {
    fn kind(&self) -> &DorsError;
}

impl std::error::Error for DorsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DorsError::Unknown(inner) => Some(inner.as_ref()),
            _ => None,
        }
    }
}

impl Error for DorsError {
    fn kind(&self) -> &Self {
        self
    }
}

impl fmt::Display for DorsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            DorsError::CouldNotParseDorsfile(e) => write!(f, "Could not parse dorsfile: {}", e),
            DorsError::NoDorsfile => {
                // TODO offer to create one
                write!(f, "Expected `Dorsfile.toml`")
            }
            DorsError::NoMemberDorsfile => write!(
                f,
                "Need `Dorsfile.toml` at either member or workspace root."
            ),
            DorsError::NoTask(task) => write!(f, "No task named: `{}`", task),
            DorsError::Unknown(e) => write!(f, "Error: {}", e),
        }
    }
}

impl<'a, T: 'a + Error> From<T> for Box<dyn Error + 'a> {
    fn from(error: T) -> Self {
        Box::new(error)
    }
}
