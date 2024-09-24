use core::fmt::{Error, Write};

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

impl Response for crate::Error {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        let result: (i16, &str) = (self.number(), (*self).into());
        result.scpi_fmt(f)
    }
}

impl<A, B> Response for (A, B) where A: Response, B: Response {
    fn scpi_fmt(&self, f: &mut impl Write) -> Result<(), Error> {
        self.0.scpi_fmt(f)?;
        f.write_char(',')?;
        self.1.scpi_fmt(f)
    }
}