use std::error;
use std::fmt::{self, Debug, Display};

use serde::de;
use serde::ser;

use anyhow::format_err;

pub struct Error {
    err: anyhow::Error,
}

impl Error {
    pub fn from_err<T: Debug>(err: T) -> Self {
        Self::from(format_err!("{:?}", err))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        ""
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.err, f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

impl de::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Error {
        Error::from(format_err!("{}", msg))
    }
}

impl ser::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Error {
        Error::from(format_err!("{}", msg))
    }
}

impl std::convert::From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self { err }
    }
}
