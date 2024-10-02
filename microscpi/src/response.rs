use core::fmt::{Error, Write};

/// An SCPI mnemonic.
///
/// This represents a defined value in SCPI
pub struct Mnemonic<'a>(pub &'a str);

pub trait Response {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error>;
}

impl Response for bool {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        match self {
            true => f.write_char('1'),
            false => f.write_char('0'),
        }
    }
}

impl Response for () {
    fn scpi_fmt(&self, _f: &mut impl Write) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> Response for Mnemonic<'a> {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_str(self.0)
    }
}

impl Response for &str {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_char('"')?;
        f.write_str(self)?;
        f.write_char('"')
    }
}

impl Response for i8 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for u8 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for i16 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for u16 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for i32 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for u32 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for i64 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for u64 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for isize {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for usize {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_fmt(format_args!("{self}"))
    }
}

impl Response for f32 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        if self.is_nan() {
            f.write_str("9.91E+37")
        }
        else if self.is_infinite() {
            if self.is_sign_negative() {
                f.write_str("-9.9E+37")
            }
            else {
                f.write_str("9.9E+37")
            }
        }
        else {
            f.write_fmt(format_args!("{}", self))
        }
    }
}

impl Response for f64 {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        if self.is_nan() {
            f.write_str("9.91E+37")
        }
        else if self.is_infinite() {
            if self.is_sign_negative() {
                f.write_str("-9.9E+37")
            }
            else {
                f.write_str("9.9E+37")
            }
        }
        else {
            f.write_fmt(format_args!("{}", self))
        }
    }
}

#[cfg(feature = "std")]
impl Response for std::string::String {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}")
    }
}

impl Response for crate::Error {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        let result: (i16, &str) = (self.number(), (*self).into());
        result.scpi_fmt(f)
    }
}

impl<A, B> Response for (A, B)
where
    A: Response,
    B: Response,
{
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        self.0.scpi_fmt(f)?;
        f.write_char(',')?;
        self.1.scpi_fmt(f)
    }
}

impl<T> Response for &[T] where T: Response {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                f.write_char(',')?;
            }
            item.scpi_fmt(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_response() {
        let mut buffer = String::new();
        true.scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "1");

        let mut buffer = String::new();
        false.scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "0");
    }

    #[test]
    fn test_mnemonic_response() {
        let mut buffer = String::new();
        Mnemonic("TEST").scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "TEST");
    }

    #[test]
    fn test_str_response() {
        let mut buffer = String::new();
        "hello".scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "\"hello\"");
    }

    #[test]
    fn test_i8_response() {
        let mut buffer = String::new();
        (-121 as i8).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "-121");
    }

    #[test]
    fn test_u8_response() {
        let mut buffer = String::new();
        (83 as u8).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "83");
    }

    #[test]
    fn test_i16_response() {
        let mut buffer = String::new();
        (-23502 as i16).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "-23502");
    }

    #[test]
    fn test_u16_response() {
        let mut buffer = String::new();
        (54968 as u16).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "54968");
    }

    #[test]
    fn test_i32_response() {
        let mut buffer = String::new();
        (-3895783 as i32).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "-3895783");
    }

    #[test]
    fn test_u32_response() {
        let mut buffer = String::new();
        (9437838 as u32).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "9437838");
    }

    #[test]
    fn test_i64_response() {
        let mut buffer = String::new();
        (-128945978592 as i64).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "-128945978592");
    }

    #[test]
    fn test_u64_response() {
        let mut buffer = String::new();
        (39048530499456 as u64).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "39048530499456");
    }

    #[test]
    fn test_isize_response() {
        let mut buffer = String::new();
        (-3451512 as isize).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "-3451512");
    }

    #[test]
    fn test_usize_response() {
        let mut buffer = String::new();
        (49684793 as usize).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "49684793");
    }

    #[test]
    fn test_f32_response() {
        let mut buffer = String::new();
        (1.23 as f32).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "1.23");

        let mut buffer = String::new();
        (f32::NAN).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "9.91E+37");

        let mut buffer = String::new();
        (f32::INFINITY).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "9.9E+37");

        let mut buffer = String::new();
        (f32::NEG_INFINITY).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "-9.9E+37");
    }

    #[test]
    fn test_f64_response() {
        let mut buffer = String::new();
        (4.56 as f64).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "4.56");

        let mut buffer = String::new();
        (f64::NAN).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "9.91E+37");

        let mut buffer = String::new();
        (f64::INFINITY).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "9.9E+37");

        let mut buffer = String::new();
        (f64::NEG_INFINITY).scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "-9.9E+37");
    }

    #[test]
    fn test_tuple_response() {
        let mut buffer = String::new();
        (123, "world").scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "123,\"world\"");
    }

    #[test]
    fn test_slice_response() {
        let mut buffer = String::new();
        let slice: &[i32] = &[1, 2, 3, 4, 5];
        slice.scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "1,2,3,4,5");

        let mut buffer = String::new();
        let slice: &[&str] = &["one", "two", "three"];
        slice.scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "\"one\",\"two\",\"three\"");

        let mut buffer = String::new();
        let slice: &[bool] = &[true, false, true];
        slice.scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "1,0,1");

        let mut buffer = String::new();
        let slice: &[f64] = &[1.1, 2.2, 3.3];
        slice.scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "1.1,2.2,3.3");

        let mut buffer = String::new();
        let slice: &[Mnemonic] = &[Mnemonic("CMD1"), Mnemonic("CMD2")];
        slice.scpi_fmt(&mut buffer).unwrap();
        assert_eq!(buffer, "CMD1,CMD2");
    }
}
