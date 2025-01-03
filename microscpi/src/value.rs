use core::str;

use crate::Error;

/// SCPI argument value
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<'a> {
    /// String
    ///
    /// A string that is enclosed by single or double quotes.
    /// Example: "Hello" or 'Hello'
    String(&'a str),
    /// Characters
    ///
    /// A string that represents a constant value like *ON* or *OFF*.
    /// It is not enclosed by quotes.
    Characters(&'a str),
    /// Decimal number
    ///
    /// The integer or float type this number will get converted to depends on
    /// the command that is called with this value.
    /// Example: 3953.64
    Decimal(&'a str),
    /// Hexadecimal number
    ///
    /// A number in hexadecimal format. Example `#H3A1CE96`
    Hexadecimal(&'a str),
    /// Binary number
    ///  
    /// A number in binary format. Example `#B110101101`
    Binary(&'a str),
    /// Binary number
    ///  
    /// A number in octal format. Example `#Q735102`
    Octal(&'a str),
    /// Arbitrary data
    ///
    /// Raw arbitrary data bytes.
    Arbitrary(&'a [u8]),
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

impl<'a> TryInto<&'a [u8]> for &Value<'a> {
    type Error = Error;

    fn try_into(self) -> Result<&'a [u8], Self::Error> {
        match self {
            Value::Arbitrary(data) => Ok(data),
            _ => Err(Error::DataTypeError),
        }
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
            Value::Characters("ON" | "on")
            | Value::Characters("TRUE" | "true")
            | Value::Decimal("1") => Ok(true),
            Value::Characters("OFF" | "off")
            | Value::Characters("FALSE" | "false")
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
        assert_eq!(Value::Characters("ON").try_into(), Ok(true));
        assert_eq!(Value::Characters("on").try_into(), Ok(true));
        assert_eq!(Value::Decimal("1").try_into(), Ok(true));

        assert_eq!(Value::Characters("OFF").try_into(), Ok(false));
        assert_eq!(Value::Characters("off").try_into(), Ok(false));
        assert_eq!(Value::Decimal("0").try_into(), Ok(false));

        assert_eq!(
            Value::Characters("10").try_into(),
            Err::<bool, Error>(Error::IllegalParameterValue)
        );

        assert_eq!(
            Value::Characters("NO").try_into(),
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
