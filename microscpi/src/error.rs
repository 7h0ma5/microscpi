/// SCPI error
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// A custom error, consisting of an error number and a name.
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
    ///
    /// The parser was expecting a separator and encountered an illegal
    /// character; for example, the semicolon was omitted after a program
    /// message unit, `*EMC 1:CH1:VOLTS 5.`
    InvalidSeparator,

    /// Data type error (-104)
    ///
    /// The parser recognized a data element different than one allowed; for
    /// example, numeric or string data was expected but block data was
    /// encountered.
    DataTypeError,

    /// Parameter not allowed (-108)
    ///
    /// More parameters were received than expected for the header; for example,
    /// the *EMC common command only accepts one parameter, so receiving `*EMC
    /// 0,1` is not allowed.
    ParameterNotAllowed,

    /// Missing parameter (-109)
    /// Fewer parameters were recieved than required for the header; for
    ///
    /// example, the `*EMC` common command requires one parameter, so
    /// receiving `*EMC` is not allowed.
    MissingParameter,

    /// Command header error (-110)
    ///
    /// An error was detected in the header. This error message should be used
    /// when the device cannot detect the more specific errors described for
    /// errors -111 through -119.
    CommandHeaderError,

    /// Header separator error (-111)
    ///
    /// A character which is not a legal header separator was encountered while
    /// parsing the header; for example, no white shace followed the header,
    /// thus `*GMC"MACRO"` is an error.
    HeaderSeparatorError,

    /// Program mnemonic too long (-112)
    ///
    /// The header contains more that twelve characters (see IEEE 488.2,
    /// 7.6.1.4.1).
    ProgramMnemonicTooLong,

    /// Undefined header (-113)
    ///
    /// The header is syntactically correct, but it is undefined for this
    /// specific device; for example, `*XYZ` is not defined for any device.
    UndefinedHeader,

    /// Header suffix out of range (-114)
    ///
    /// The value of a numeric suffix attached to a program mnemonic, see Syntax
    /// and Style section 6.2.5.2, makes the header invalid.
    HeaderSuffixOutOfRange,

    /// Unexpected number of parameters (-115)
    ///
    /// The number of parameters received does not correspond to the number of
    /// parameters expected.
    UnexpectedNumberOfParameters,

    /// Numeric data error (-120)
    ///
    /// This error, as well as errors -121 through -129, are generated when
    /// parsing a data element which apprears to be numeric, including the
    /// nondecimal numeric types. This particular error message should be
    /// used if the device cannot detect a more specific error.
    NumericDataError,

    /// Invalid character in number (-121)
    ///
    /// An invalid character for the data type being parsed was encountered; for
    /// example, an alpha in a decimal numeric or a `9` in octal data.
    InvalidCharacterInNumber,

    /// Exponent too large (-123)
    ///
    /// The magnitude of the exponent was larger than 32000 (see IEEE 488.2,
    /// 7.7.2.4.1).
    ExponentTooLarge,

    /// Too many digits (-124)
    ///
    /// The mantissa of a decimal numeric data element contained more than 255
    /// digits excluding leading zeros (see IEEE 488.2, 7.7.2.4.1).
    TooManyDigits,

    /// Numeric data not allowed (-128)
    ///
    /// A legal numeric data element was received, but the device does not
    /// accept one in this position for the header.
    NumericDataNotAllowed,

    /// Invalid character data (-141)
    ///
    /// Either the character data element contains an invalid character or the
    /// particular element received is not valid for the header.
    InvalidCharacterData,

    /// Execution error (-200)
    ///
    /// This is the generic syntax error for devices that cannot detect more
    /// specific errors. This code indicates only that an Execution Error as
    /// defined in IEEE 488.2, 11.5.1.1.5 has occurred.
    ExecutionError,

    /// Invalid while in local (-201)
    ///
    /// Indicates that a command is not executable while the device is in local
    /// due to a hard local control (see IEEE 488.2, 5.6.1.5); for example,
    /// a device with a rotary switch receives a message which would change
    /// the switches state, but the device is in local so the message can
    /// not be executed.
    InvalidWhileInLocal,

    /// Command protected (-203)
    ///
    /// Indicates that a legal password-protected program command or query could
    /// not be executed because the command was disabled.
    CommandProtected,

    /// Trigger error (-210)
    ///
    /// Indicates that a GET, *TRG, or triggering signal was received and
    /// recognized by the device but was ignored because of device timing
    /// considerations; for example, the device was not ready to respond.
    /// Note: a DT0 device always ignores GET and treats *TRG as a Command
    /// Error.
    TriggerError,

    /// Parameter error (-220)
    ///
    /// Indicates that a program data element related error occurred. This error
    /// message should be used when the device cannot detect the more
    /// specific errors described for errors -221 through -229.
    ParameterError,

    /// Settings conflict (-221)
    ///
    /// Indicates that a legal program data element was parsed but could not be
    /// executed due to the current device state (see IEEE 488.2, 6.4.5.3
    /// and 11.5.1.1.5.)
    SettingsConflict,

    /// Data out of range (-222)
    ///
    /// Indicates that a legal program data element was parsed but could not be
    /// executed due to the current device state (see IEEE 488.2, 6.4.5.3
    /// and 11.5.1.1.5.)
    DataOutOfRange,

    /// Too much data (-223)
    ///
    /// Indicates that a legal program data element of block, expression, or
    /// string type was received that contained more data than the device
    /// could handle due to memory or related device-specific requirements.
    TooMuchData,

    /// Illegal parameter value (-224)
    ///
    /// Used where exact value, from a list of possibles, was expected.
    IllegalParameterValue,

    /// Hardware Error (-240)
    ///
    /// Indicates that a legal program command or query could not be executed
    /// because of a hardware problem in the device. Definition of what
    /// constitutes a hardware problem is completely device-specific. This
    /// error message should be used when the device cannot detect the more
    /// specific errors described for errors -241 through -249.
    HardwareError,

    /// Device specific error (-300)
    ///
    /// This is the generic device-dependent error for devices that cannot
    /// detect more specific errors. This code indicates only that a
    /// Device-Dependent Error as defined in IEEE 488.2, 11.5.1.1.6 has
    /// occurred.
    DeviceSpecificError,

    /// System error (-310)
    ///
    /// Indicates that some error, termed "system error" by the device, has
    /// occurred. This code is device-dependent.
    SystemError,

    /// Storage fault (-320)
    ///
    /// Indicates that the firmware detected a fault when using data storage.
    /// This error is not an indication of physical damage or failure of any
    /// mass storage element.
    StorageFault,

    /// Self-test failed (-330)
    SelfTestFailed,

    /// Calibration failed (-340)
    CalibrationFailed,

    /// Queue overflow (-350)
    ///
    /// A specific code entered into the queue in lieu of the code that caused
    /// the error. This code indicates that there is no room in the queue
    /// and an error occurred but was not recorded.
    QueueOverflow,

    /// Communication error (-360)
    ///
    /// This is the generic communication error for devices that cannot detect
    /// the more specific errors described for errors -361 through -363.
    CommunicationError,

    /// Input buffer overrun (-363)
    ///
    /// Software or hardware input buffer on serial port overflows with data
    /// caused by improper or nonexistent pacing.
    InputBufferOverrun,

    /// Timeout error (-365)
    ///
    /// This is a generic device-dependent error.
    TimeoutError,

    /// Query error (-400)
    ///
    /// This is the generic query error for devices that cannot detect more
    /// specific errors. This code indicates only that a Query Error as
    /// defined in IEEE 488.2, 11.5.1.1.7 and 6.3 has occurred.
    QueryError,
}

impl Error {
    /// Get the error number as defined in IEEE 488.2.
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
            Error::CommandHeaderError => -110,
            Error::HeaderSeparatorError => -111,
            Error::ProgramMnemonicTooLong => -112,
            Error::UndefinedHeader => -113,
            Error::HeaderSuffixOutOfRange => -114,
            Error::UnexpectedNumberOfParameters => -115,
            Error::NumericDataError => -120,
            Error::InvalidCharacterInNumber => -121,
            Error::ExponentTooLarge => -123,
            Error::TooManyDigits => -124,
            Error::NumericDataNotAllowed => -128,
            Error::InvalidCharacterData => -141,
            Error::ExecutionError => -200,
            Error::InvalidWhileInLocal => -201,
            Error::CommandProtected => -203,
            Error::ParameterError => -220,
            Error::TriggerError => -210,
            Error::SettingsConflict => -221,
            Error::DataOutOfRange => -222,
            Error::TooMuchData => -223,
            Error::IllegalParameterValue => -224,
            Error::HardwareError => -240,
            Error::DeviceSpecificError => -300,
            Error::SystemError => -310,
            Error::StorageFault => -320,
            Error::SelfTestFailed => -330,
            Error::CalibrationFailed => -340,
            Error::QueueOverflow => -350,
            Error::CommunicationError => -360,
            Error::InputBufferOverrun => -363,
            Error::TimeoutError => -365,
            Error::QueryError => -400,
        }
    }
}

impl Into<&'static str> for Error {
    fn into(self) -> &'static str {
        match self {
            Error::Custom(_, name) => name,
            Error::CommandError => "Command error",
            Error::InvalidCharacter => "Invalid character",
            Error::SyntaxError => "Syntax Error",
            Error::UndefinedHeader => "Undefined header",
            Error::HeaderSuffixOutOfRange => "Header suffix out of range",
            Error::InvalidCharacterInNumber => "Invalid character in number",
            Error::InvalidCharacterData => "Invalid character data",
            Error::ExecutionError => "Execution error",
            Error::QueryError => "Formatter error",
            Error::UnexpectedNumberOfParameters => "Argument overflow",
            Error::InvalidSeparator => "Invalid separator",
            Error::DataTypeError => "Data type error",
            Error::ParameterNotAllowed => "Parameter not allowed",
            Error::MissingParameter => "Missing parameter",
            Error::SystemError => "System error",
            Error::QueueOverflow => "Queue overflow",
            Error::CommandHeaderError => "Command header error",
            Error::HeaderSeparatorError => "Header separator error",
            Error::ProgramMnemonicTooLong => "Program mnemonic too long",
            Error::NumericDataError => "Numeric data error",
            Error::ExponentTooLarge => "Exponent too large",
            Error::TooManyDigits => "Too many digits",
            Error::NumericDataNotAllowed => "Numeric data not allowed",
            Error::InvalidWhileInLocal => "Invalid while in local",
            Error::CommandProtected => "Command protected",
            Error::TriggerError => "Trigger error",
            Error::ParameterError => "Parameter error",
            Error::SettingsConflict => "Settings conflict",
            Error::DataOutOfRange => "Data out of range",
            Error::TooMuchData => "Too much data",
            Error::IllegalParameterValue => "Illegal parameter value",
            Error::HardwareError => "Hardware error",
            Error::DeviceSpecificError => "Device specific error",
            Error::StorageFault => "Storage fault",
            Error::SelfTestFailed => "Self test failed",
            Error::CalibrationFailed => "Calibration failed",
            Error::CommunicationError => "Communication error",
            Error::InputBufferOverrun => "Input buffer overrun",
            Error::TimeoutError => "Timeout error",
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

impl core::error::Error for Error {}

impl From<core::fmt::Error> for Error {
    fn from(_value: core::fmt::Error) -> Self {
        Error::QueryError
    }
}
