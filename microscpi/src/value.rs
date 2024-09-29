use core::str;

use crate::Error;

/// SCPI argument value
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<'a> {
    String(&'a str),
    Mnemonic(&'a str),
    /// A number that has not been completely parsed yet.
    ///
    /// The integer or float type this number will get converted to depends on
    /// the command that is called with this value.
    Decimal(&'a str),
    Bool(bool),
}

impl<'a> TryInto<&'a str> for &Value<'a> {
    type Error = Error;

    fn try_into(self) -> Result<&'a str, Self::Error> {
        match self {
            Value::String(data) => Ok(data),
            _ => Err(Error::DataTypeError),
        }
    }
}

impl<'a> TryInto<&'a str> for Value<'a> {
    type Error = Error;

    fn try_into(self) -> Result<&'a str, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<u32> for &Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<u32, Self::Error> {
        match self {
            Value::Decimal(data) => u32::from_str_radix(data, 10).or(Err(Error::NumericDataError)),
            _ => Err(Error::DataTypeError),
        }
    }
}

impl TryInto<u32> for Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<u32, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<i32> for &Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            Value::Decimal(data) => i32::from_str_radix(data, 10).or(Err(Error::NumericDataError)),
            _ => Err(Error::DataTypeError),
        }
    }
}

impl TryInto<i32> for Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<i32, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<u64> for &Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            Value::Decimal(data) => u64::from_str_radix(data, 10).or(Err(Error::NumericDataError)),
            _ => Err(Error::DataTypeError),
        }
    }
}

impl TryInto<u64> for Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<u64, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<i64> for &Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            Value::Decimal(data) => i64::from_str_radix(data, 10).or(Err(Error::NumericDataError)),
            _ => Err(Error::DataTypeError),
        }
    }
}

impl TryInto<i64> for Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<i64, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<bool> for &Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Value::Mnemonic("ON" | "on")
            | Value::Mnemonic("TRUE" | "true")
            | Value::Decimal("1") => Ok(true),
            Value::Mnemonic("OFF" | "off")
            | Value::Mnemonic("FALSE" | "false")
            | Value::Decimal("0") => Ok(false),
            _ => Err(Error::IllegalParameterValue),
        }
    }
}

impl TryInto<bool> for Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<bool, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<f32> for &Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<f32, Self::Error> {
        match self {
            Value::Decimal(data) => data.parse().or(Err(Error::NumericDataError)),
            _ => Err(Error::DataTypeError),
        }
    }
}

impl TryInto<f32> for Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<f32, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<f64> for &Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            Value::Decimal(data) => data.parse().or(Err(Error::NumericDataError)),
            _ => Err(Error::DataTypeError),
        }
    }
}

impl TryInto<f64> for Value<'_> {
    type Error = Error;

    fn try_into(self) -> Result<f64, Self::Error> {
        (&self).try_into()
    }
}

#[test]
pub fn test_bool() {
    assert_eq!(Value::Mnemonic("ON").try_into(), Ok(true));
    assert_eq!(Value::Mnemonic("on").try_into(), Ok(true));
    assert_eq!(Value::Decimal("1").try_into(), Ok(true));

    assert_eq!(Value::Mnemonic("OFF").try_into(), Ok(false));
    assert_eq!(Value::Mnemonic("off").try_into(), Ok(false));
    assert_eq!(Value::Decimal("0").try_into(), Ok(false));

    assert_eq!(
        Value::Mnemonic("10").try_into(),
        Err::<bool, Error>(Error::IllegalParameterValue)
    );

    assert_eq!(
        Value::Mnemonic("NO").try_into(),
        Err::<bool, Error>(Error::IllegalParameterValue)
    );
}
