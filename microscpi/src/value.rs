use core::fmt::Display;
use core::str;

/// SCPI value
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Void,
    String(&'a str),
    Mnemonic(&'a str),
    /// A number that has not been parsed yet.
    ///
    /// The integer or float type this  number will get converted to depends on
    /// the command that is called with this value.
    Number(&'a str),
    U64(u64),
    I64(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::Void => Ok(()),
            Value::String(value) => write!(f, "\"{}\"", value),
            Value::Mnemonic(value) => write!(f, "{}", value),
            Value::Number(value) => write!(f, "{}", value),
            Value::U64(value) => write!(f, "{}", value),
            Value::I64(value) => write!(f, "{}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::Double(value) => write!(f, "{}", value),
            Value::Bool(value) => {
                if *value {
                    write!(f, "1")
                }
                else {
                    write!(f, "0")
                }
            }
        }
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Value<'a> {
        Value::String(value)
    }
}

impl From<bool> for Value<'_> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i64> for Value<'_> {
    fn from(value: i64) -> Self {
        Value::I64(value)
    }
}

impl From<u64> for Value<'_> {
    fn from(value: u64) -> Self {
        Value::U64(value)
    }
}

impl From<f64> for Value<'_> {
    fn from(value: f64) -> Self {
        Value::Double(value)
    }
}

impl From<()> for Value<'_> {
    fn from(_: ()) -> Self {
        Value::Void
    }
}

impl<'a> TryInto<&'a str> for &Value<'a> {
    type Error = crate::Error;

    fn try_into(self) -> Result<&'a str, Self::Error> {
        match self {
            Value::String(data) => Ok(data),
            _ => Err(Self::Error::ParseError),
        }
    }
}

impl TryInto<u64> for &Value<'_> {
    type Error = crate::Error;

    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            Value::Number(data) => {
                u64::from_str_radix(data, 10).map_err(|_| Self::Error::ParseError)
            }
            Value::U64(val) => Ok(*val),
            _ => Err(Self::Error::ParseError),
        }
    }
}

impl TryInto<bool> for &Value<'_> {
    type Error = crate::Error;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Value::Mnemonic("ON" | "on") |
            Value::Number("1") => Ok(true),
            Value::Mnemonic("OFF" | "off") |
            Value::Number("0") => Ok(false),
            _ => Err(Self::Error::ParseError),
        }
    }
}