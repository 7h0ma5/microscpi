use core::fmt::Arguments;

use crate::Error;

/// SCPI characters
///
/// This represents a choice or predefined value in SCPI.
pub struct Characters<'a>(pub &'a str);

/// Arbitrary data
///
/// Contains arbitrary binary data.
pub struct Arbitrary<'a>(pub &'a [u8]);

pub trait Write {
    async fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error>;
    async fn write_char(&mut self, c: char) -> Result<(), Error>;
    async fn write_str(&mut self, str: &str) -> Result<(), Error>;
    async fn write_fmt(&mut self, fmt: Arguments) -> Result<(), Error>;
    async fn flush(&mut self) -> Result<(), Error>;
}

impl<const N: usize> Write for heapless::Vec<u8, N> {
    async fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.extend_from_slice(bytes).or(Err(Error::TooMuchData))?;
        Ok(())
    }

    async fn write_char(&mut self, c: char) -> Result<(), Error> {
        self.push(c as u8).or(Err(Error::TooMuchData))?;
        Ok(())
    }

    async fn write_str(&mut self, s: &str) -> Result<(), Error> {
        self.extend_from_slice(s.as_bytes())
            .or(Err(Error::TooMuchData))?;
        Ok(())
    }

    async fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> Result<(), Error> {
        core::fmt::Write::write_fmt(self, args).or(Err(Error::SystemError))?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(feature = "std")]
impl Write for std::vec::Vec<u8> {
    async fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.extend_from_slice(bytes);
        Ok(())
    }

    async fn write_char(&mut self, c: char) -> Result<(), Error> {
        self.push(c as u8);
        Ok(())
    }

    async fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> Result<(), Error> {
        let buf = format!("{}", args);
        self.extend_from_slice(buf.as_bytes());
        Ok(())
    }

    async fn write_str(&mut self, str: &str) -> Result<(), Error> {
        self.extend_from_slice(str.as_bytes());
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

pub trait Response {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error>;
}

impl Response for bool {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        match self {
            true => f.write_char('1').await,
            false => f.write_char('0').await,
        }
    }
}

impl Response for () {
    async fn write_response(&self, _f: &mut impl Write) -> Result<(), Error> {
        Ok(())
    }
}

impl Response for Characters<'_> {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        f.write_str(self.0).await
    }
}

impl Response for Arbitrary<'_> {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        let len = self.0.len();
        if len > 0 {
            let len_digits = len.ilog10() + 1;

            if len_digits > 9 {
                return Err(Error::TooMuchData);
            }

            write!(f, "#{}{}", len_digits, len).await?;
            f.write_bytes(self.0).await
        }
        else {
            f.write_str("#10").await
        }
    }
}

impl Response for &str {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "\"{self}\"").await
    }
}

impl Response for i8 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for u8 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for i16 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for u16 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for i32 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for u32 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for i64 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for u64 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for isize {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for usize {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "{self}").await
    }
}

impl Response for f32 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        if self.is_nan() {
            f.write_str("9.91E+37").await
        }
        else if self.is_infinite() {
            if self.is_sign_negative() {
                f.write_str("-9.9E+37").await
            }
            else {
                f.write_str("9.9E+37").await
            }
        }
        else {
            write!(f, "{self}").await
        }
    }
}

impl Response for f64 {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        if self.is_nan() {
            f.write_str("9.91E+37").await
        }
        else if self.is_infinite() {
            if self.is_sign_negative() {
                f.write_str("-9.9E+37").await
            }
            else {
                f.write_str("9.9E+37").await
            }
        }
        else {
            write!(f, "{self}").await
        }
    }
}

impl<const N: usize> Response for heapless::String<N> {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "\"{}\"", self.as_str()).await
    }
}

impl<const N: usize, T: Response> Response for heapless::Vec<T, N> {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                f.write_char(',').await?;
            }
            item.write_response(f).await?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl Response for std::string::String {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        write!(f, "\"{}\"", self.as_str()).await
    }
}

impl Response for crate::Error {
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        let result: (i16, &str) = (self.number(), (*self).into());
        result.write_response(f).await
    }
}

impl<A, B> Response for (A, B)
where
    A: Response,
    B: Response,
{
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        self.0.write_response(f).await?;
        f.write_char(',').await?;
        self.1.write_response(f).await
    }
}

impl<A, B, C> Response for (A, B, C)
where
    A: Response,
    B: Response,
    C: Response,
{
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        self.0.write_response(f).await?;
        f.write_char(',').await?;
        self.1.write_response(f).await?;
        f.write_char(',').await?;
        self.2.write_response(f).await
    }
}

impl<A, B, C, D> Response for (A, B, C, D)
where
    A: Response,
    B: Response,
    C: Response,
    D: Response,
{
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        self.0.write_response(f).await?;
        f.write_char(',').await?;
        self.1.write_response(f).await?;
        f.write_char(',').await?;
        self.2.write_response(f).await?;
        f.write_char(',').await?;
        self.3.write_response(f).await
    }
}

impl<T> Response for [T]
where
    T: Response,
{
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                f.write_char(',').await?;
            }
            item.write_response(f).await?;
        }
        Ok(())
    }
}

impl<T> Response for &[T]
where
    T: Response,
{
    async fn write_response(&self, f: &mut impl Write) -> Result<(), Error> {
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                f.write_char(',').await?;
            }
            item.write_response(f).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bool_response() {
        let mut buffer: Vec<u8> = Vec::new();
        true.write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"1");

        let mut buffer: Vec<u8> = Vec::new();
        false.write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"0");
    }

    #[tokio::test]
    async fn test_label_response() {
        let mut buffer: Vec<u8> = Vec::new();
        Characters("TEST")
            .write_response(&mut buffer)
            .await
            .unwrap();
        assert_eq!(buffer, b"TEST");
    }

    #[tokio::test]
    async fn test_str_response() {
        let mut buffer: Vec<u8> = Vec::new();
        "hello".write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"\"hello\"");
    }

    #[tokio::test]
    async fn test_i8_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (-121 as i8).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"-121");
    }

    #[tokio::test]
    async fn test_u8_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (83 as u8).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"83");
    }

    #[tokio::test]
    async fn test_i16_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (-23502 as i16).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"-23502");
    }

    #[tokio::test]
    async fn test_u16_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (54968 as u16).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"54968");
    }

    #[tokio::test]
    async fn test_i32_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (-3895783 as i32).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"-3895783");
    }

    #[tokio::test]
    async fn test_u32_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (9437838 as u32).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"9437838");
    }

    #[tokio::test]
    async fn test_i64_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (-128945978592 as i64)
            .write_response(&mut buffer)
            .await
            .unwrap();
        assert_eq!(buffer, b"-128945978592");
    }

    #[tokio::test]
    async fn test_u64_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (39048530499456 as u64)
            .write_response(&mut buffer)
            .await
            .unwrap();
        assert_eq!(buffer, b"39048530499456");
    }

    #[tokio::test]
    async fn test_isize_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (-3451512 as isize)
            .write_response(&mut buffer)
            .await
            .unwrap();
        assert_eq!(buffer, b"-3451512");
    }

    #[tokio::test]
    async fn test_usize_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (49684793 as usize)
            .write_response(&mut buffer)
            .await
            .unwrap();
        assert_eq!(buffer, b"49684793");
    }

    #[tokio::test]
    async fn test_f32_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (1.23 as f32).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"1.23");

        let mut buffer: Vec<u8> = Vec::new();
        (f32::NAN).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"9.91E+37");

        let mut buffer: Vec<u8> = Vec::new();
        (f32::INFINITY).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"9.9E+37");

        let mut buffer: Vec<u8> = Vec::new();
        (f32::NEG_INFINITY)
            .write_response(&mut buffer)
            .await
            .unwrap();
        assert_eq!(buffer, b"-9.9E+37");
    }

    #[tokio::test]
    async fn test_f64_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (4.56 as f64).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"4.56");

        let mut buffer: Vec<u8> = Vec::new();
        (f64::NAN).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"9.91E+37");

        let mut buffer: Vec<u8> = Vec::new();
        (f64::INFINITY).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"9.9E+37");

        let mut buffer: Vec<u8> = Vec::new();
        (f64::NEG_INFINITY)
            .write_response(&mut buffer)
            .await
            .unwrap();
        assert_eq!(buffer, b"-9.9E+37");
    }

    #[tokio::test]
    async fn test_arbitrary_response() {
        let mut buffer: Vec<u8> = Vec::new();
        Arbitrary(&[0x23, 0x42, 0x85, 0xab, 0xfe, 0xac])
            .write_response(&mut buffer)
            .await
            .unwrap();
        assert_eq!(buffer, b"#16\x23\x42\x85\xab\xfe\xac");

        let mut buffer: Vec<u8> = Vec::new();
        Arbitrary(b"\xb7\x54\x5d\xc8\x60\x10\xa5\x13\x33\x3c\xd0")
            .write_response(&mut buffer)
            .await
            .unwrap();
        assert_eq!(buffer, b"#211\xb7\x54\x5d\xc8\x60\x10\xa5\x13\x33\x3c\xd0");

        let mut buffer: Vec<u8> = Vec::new();
        Arbitrary(&[]).write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"#10");
    }

    #[tokio::test]
    async fn test_tuple_response() {
        let mut buffer: Vec<u8> = Vec::new();
        (123, "world").write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"123,\"world\"");
    }

    #[tokio::test]
    async fn test_slice_response() {
        let mut buffer: Vec<u8> = Vec::new();
        let slice: &[i32] = &[1, 2, 3, 4, 5];
        slice.write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"1,2,3,4,5");

        let mut buffer: Vec<u8> = Vec::new();
        let slice: &[&str] = &["one", "two", "three"];
        slice.write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"\"one\",\"two\",\"three\"");

        let mut buffer: Vec<u8> = Vec::new();
        let slice: &[bool] = &[true, false, true];
        slice.write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"1,0,1");

        let mut buffer: Vec<u8> = Vec::new();
        let slice: &[f64] = &[1.1, 2.2, 3.3];
        slice.write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"1.1,2.2,3.3");

        let mut buffer: Vec<u8> = Vec::new();
        let slice: &[Characters] = &[Characters("CMD1"), Characters("CMD2")];
        slice.write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"CMD1,CMD2");
    }

    #[tokio::test]
    async fn test_heapless_string_response() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test = heapless::String::<16>::new();
        test.push_str("Hello World").unwrap();
        test.write_response(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"\"Hello World\"");
    }
}
