/// SCPI error
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    ParseError,
    ExecutionError,
    InvalidCommand,
    FormatterError,
    ArgumentOverflow
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::ParseError => write!(f, "Parse error"),
            Error::ExecutionError => write!(f, "Execution error"),
            Error::InvalidCommand => write!(f, "Invalid command"),
            Error::FormatterError => write!(f, "Formatter error"),
            Error::ArgumentOverflow => write!(f, "Argument overflow"),
        }
    }
}

impl core::error::Error for Error {}

impl From<core::fmt::Error> for Error {
    fn from(_value: core::fmt::Error) -> Self {
        Error::FormatterError
    }
}
