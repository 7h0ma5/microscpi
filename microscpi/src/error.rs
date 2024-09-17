/// SCPI error
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Custom(i16, &'static str),

    /// Command error (-100)
    ///
    /// This is the generic syntax error for devices that cannot detect more
    /// specific errors. This code indicates only that a Command Error as
    /// defined in IEEE 488.2, 11.5.1.1.4 has occurred.
    CommandError,

    /// Invalid character (-101)
    ///
    /// A syntactic element contains a character which is invalid for that type;
    /// for example, a header containing an ampersand, SETUP&. This error
    /// might be used in place of errors [Error::HeaderSuffixOutOfRange],
    /// [Error::InvalidCharacterInNumber], [Error::InvalidCharacterData], and
    /// perhaps some others.
    InvalidCharacter,

    /// Syntax error (-102)
    ///
    /// An unrecognized command or data type was encountered; for example, a
    /// string was received when the device does not accept strings.
    SyntaxError,

    /// Invalid separator (-103)
    /// The parser was expecting a separator and encountered an illegal
    /// character; for example, the semicolon was omitted after a program
    /// message unit, `*EMC 1:CH1:VOLTS 5.`
    InvalidSeparator,

    /// Data type error (-104)
    /// The parser recognized a data element different than one allowed; for
    /// example, numeric or string data was expected but block data was
    /// encountered.
    DataTypeError,

    /// Parameter not allowed (-108)
    /// More parameters were received than expected for the header; for example,
    /// the *EMC common command only accepts one parameter, so receiving `*EMC
    /// 0,1` is not allowed.
    ParameterNotAllowed,

    /// Missing parameter (-109)
    /// Fewer parameters were recieved than required for the header; for
    /// example, the `*EMC` common command requires one parameter, so
    /// receiving `*EMC` is not allowed.
    MissingParameter,

    /// Undefined header (-113)
    /// The header is syntactically correct, but it is undefined for this
    /// specific device; for example, `*XYZ` is not defined for any device.
    UndefinedHeader,

    /// Header suffix out of range (-114)
    /// The value of a numeric suffix attached to a program mnemonic, see Syntax
    /// and Style section 6.2.5.2, makes the header invalid.
    HeaderSuffixOutOfRange,

    /// Unexpected number of parameters (-115)
    /// The number of parameters received does not correspond to the number of
    /// parameters expected.
    UnexpectedNumberOfParameters,

    /// Invalid character in number (-121)
    /// An invalid character for the data type being parsed was encountered; for
    /// example, an alpha in a decimal numeric or a `9` in octal data.
    InvalidCharacterInNumber,

    /// Invalid character data (-141)
    /// Either the character data element contains an invalid character or the
    /// particular element received is not valid for the header.
    InvalidCharacterData,

    /// Execution error (-200)
    /// This is the generic syntax error for devices that cannot detect more
    /// specific errors. This code indicates only that an Execution Error as
    /// defined in IEEE 488.2, 11.5.1.1.5 has occurred.
    ExecutionError,

    /// System error (-310)
    /// Indicates that some error, termed “system error” by the device, has
    /// occurred. This code is device-dependent.
    SystemError,

    /// Query error (-400)
    /// This is the generic query error for devices that cannot detect more
    /// specific errors. This code indicates only that a Query Error as
    /// defined in IEEE 488.2, 11.5.1.1.7 and 6.3 has occurred.
    QueryError,
}

impl Error {
    pub fn number(&self) -> i16 {
        match self {
            Error::Custom(number, _name) => *number,
            Error::CommandError => -100,
            Error::InvalidCharacter => -101,
            Error::SyntaxError => -102,
            Error::InvalidSeparator => -103,
            Error::DataTypeError => -104,
            Error::ParameterNotAllowed => -108,
            Error::MissingParameter => -109,
            Error::UndefinedHeader => -113,
            Error::HeaderSuffixOutOfRange => -114,
            Error::UnexpectedNumberOfParameters => -115,
            Error::InvalidCharacterInNumber => -121,
            Error::InvalidCharacterData => -141,
            Error::ExecutionError => -200,
            Error::SystemError => -310,
            Error::QueryError => -400,
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Custom(_, name) => write!(f, "{}", name),
            Error::CommandError => write!(f, "Command error"),
            Error::InvalidCharacter => write!(f, "Invalid character"),
            Error::SyntaxError => write!(f, "Syntax Error"),
            Error::UndefinedHeader => write!(f, "Undefined header"),
            Error::HeaderSuffixOutOfRange => write!(f, "Header suffix out of range"),
            Error::InvalidCharacterInNumber => write!(f, "Invalid character in number"),
            Error::InvalidCharacterData => write!(f, "Invalid character data"),
            Error::ExecutionError => write!(f, "Execution error"),
            Error::QueryError => write!(f, "Formatter error"),
            Error::UnexpectedNumberOfParameters => write!(f, "Argument overflow"),
            Error::InvalidSeparator => write!(f, "Invalid separator"),
            Error::DataTypeError => write!(f, "Data type error"),
            Error::ParameterNotAllowed => write!(f, "Parameter not allowed"),
            Error::MissingParameter => write!(f, "Missing parameter"),
            Error::SystemError => write!(f, "System error"),
        }
    }
}

impl core::error::Error for Error {}

impl From<core::fmt::Error> for Error {
    fn from(_value: core::fmt::Error) -> Self {
        Error::QueryError
    }
}
