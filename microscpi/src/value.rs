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
    Hexadecimal(&'a str),
    Binary(&'a str),
    Octal(&'a str),
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

macro_rules! impl_try_into_int {
    ($type:ty) => {
        impl TryInto<$type> for &Value<'_> {
            type Error = Error;

            fn try_into(self) -> Result<$type, Self::Error> {
                match self {
                    Value::Decimal(data) => {
                        <$type>::from_str_radix(data, 10).or(Err(Error::NumericDataError))
                    }
                    Value::Hexadecimal(data) => {
                        <$type>::from_str_radix(data, 16).or(Err(Error::NumericDataError))
                    }
                    Value::Binary(data) => {
                        <$type>::from_str_radix(data, 2).or(Err(Error::NumericDataError))
                    }
                    Value::Octal(data) => {
                        <$type>::from_str_radix(data, 8).or(Err(Error::NumericDataError))
                    }
                    _ => Err(Error::DataTypeError),
                }
            }
        }

        impl TryInto<$type> for Value<'_> {
            type Error = Error;

            fn try_into(self) -> Result<$type, Self::Error> {
                (&self).try_into()
            }
        }
    };
}

impl_try_into_int!(u8);
impl_try_into_int!(i8);
impl_try_into_int!(u16);
impl_try_into_int!(i16);
impl_try_into_int!(u32);
impl_try_into_int!(i32);
impl_try_into_int!(u64);
impl_try_into_int!(i64);
impl_try_into_int!(usize);
impl_try_into_int!(isize);


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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    pub fn test_string() {
        assert_eq!(Value::String("test").try_into(), Ok("test"));
        assert_eq!(
            Value::Decimal("123").try_into(),
            Err::<&str, Error>(Error::DataTypeError)
        );
    }

    #[test]
    pub fn test_u32() {
        assert_eq!(Value::Decimal("123").try_into(), Ok(123u32));
        assert_eq!(
            Value::Decimal("abc").try_into(),
            Err::<u32, Error>(Error::NumericDataError)
        );
        assert_eq!(
            Value::String("123").try_into(),
            Err::<u32, Error>(Error::DataTypeError)
        );
        assert_eq!(Value::Hexadecimal("7B").try_into(), Ok(123u32));
        assert_eq!(Value::Binary("1111011").try_into(), Ok(123u32));
        assert_eq!(Value::Octal("173").try_into(), Ok(123u32));
    }

    #[test]
    pub fn test_i32() {
        assert_eq!(Value::Decimal("123").try_into(), Ok(123i32));
        assert_eq!(Value::Decimal("-123").try_into(), Ok(-123i32));
        assert_eq!(
            Value::Decimal("abc").try_into(),
            Err::<i32, Error>(Error::NumericDataError)
        );
        assert_eq!(
            Value::String("123").try_into(),
            Err::<i32, Error>(Error::DataTypeError)
        );
        assert_eq!(Value::Hexadecimal("7B").try_into(), Ok(123i32));
        assert_eq!(Value::Binary("1111011").try_into(), Ok(123i32));
        assert_eq!(Value::Octal("173").try_into(), Ok(123i32));
    }

    #[test]
    pub fn test_u64() {
        assert_eq!(Value::Decimal("123").try_into(), Ok(123u64));
        assert_eq!(
            Value::Decimal("abc").try_into(),
            Err::<u64, Error>(Error::NumericDataError)
        );
        assert_eq!(
            Value::String("123").try_into(),
            Err::<u64, Error>(Error::DataTypeError)
        );
        assert_eq!(Value::Hexadecimal("7B").try_into(), Ok(123u64));
        assert_eq!(Value::Binary("1111011").try_into(), Ok(123u64));
        assert_eq!(Value::Octal("173").try_into(), Ok(123u64));
    }

    #[test]
    pub fn test_i64() {
        assert_eq!(Value::Decimal("123").try_into(), Ok(123i64));
        assert_eq!(Value::Decimal("-123").try_into(), Ok(-123i64));
        assert_eq!(
            Value::Decimal("abc").try_into(),
            Err::<i64, Error>(Error::NumericDataError)
        );
        assert_eq!(
            Value::String("123").try_into(),
            Err::<i64, Error>(Error::DataTypeError)
        );
        assert_eq!(Value::Hexadecimal("7B").try_into(), Ok(123i64));
        assert_eq!(Value::Binary("1111011").try_into(), Ok(123i64));
        assert_eq!(Value::Octal("173").try_into(), Ok(123i64));
    }

    #[test]
    pub fn test_f32() {
        assert_eq!(Value::Decimal("123.45").try_into(), Ok(123.45f32));
        assert_eq!(
            Value::Decimal("abc").try_into(),
            Err::<f32, Error>(Error::NumericDataError)
        );
        assert_eq!(
            Value::String("123.45").try_into(),
            Err::<f32, Error>(Error::DataTypeError)
        );
    }

    #[test]
    pub fn test_f64() {
        assert_eq!(Value::Decimal("123.45").try_into(), Ok(123.45f64));
        assert_eq!(
            Value::Decimal("abc").try_into(),
            Err::<f64, Error>(Error::NumericDataError)
        );
        assert_eq!(
            Value::String("123.45").try_into(),
            Err::<f64, Error>(Error::DataTypeError)
        );
    }
}
