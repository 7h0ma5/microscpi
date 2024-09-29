use core::str::{self, Utf8Error};

use heapless::Vec;

use crate::tree::Node;
use crate::{Error, Value, MAX_ARGS};

/// Enum to handle both recoverable and fatal errors
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Recoverable error (continue trying other paths)
    SoftError(Option<Error>),
    /// Unrecoverable syntax error
    FatalError(Error),
    /// Incomplete data
    Incomplete,
}

impl From<()> for ParseError {
    fn from(_: ()) -> Self {
        ParseError::SoftError(None)
    }
}

impl From<Error> for ParseError {
    fn from(e: Error) -> Self {
        match e {
            Error::UndefinedHeader => ParseError::FatalError(e),
            _ => ParseError::SoftError(Some(e)),
        }
    }
}

impl From<Utf8Error> for ParseError {
    fn from(_e: Utf8Error) -> Self {
        Error::InvalidCharacter.into()
    }
}

/// A Result type that propagates ParseError
type ParseResult<'a, T> = Result<(&'a [u8], T), ParseError>;

#[derive(Debug, PartialEq)]
pub struct CommandCall<'a> {
    pub node: &'static Node,
    pub query: bool,
    pub args: Vec<Value<'a>, MAX_ARGS>,
}

fn take_while<F>(pred: F) -> impl Fn(&[u8]) -> ParseResult<&[u8]>
where
    F: Fn(u8) -> bool,
{
    move |input: &[u8]| match input.iter().position(|&byte| !pred(byte)) {
        Some(pos) => Ok((&input[pos..], &input[..pos])),
        None => Ok((&[], input)),
    }
}

fn satisfy<F>(pred: F) -> impl Fn(&[u8]) -> ParseResult<u8>
where
    F: Fn(u8) -> bool,
{
    move |i: &[u8]| match i.first() {
        Some(&byte) if pred(byte) => Ok((&i[1..], byte)),
        Some(_) => Err(Error::InvalidCharacter)?,
        None => Err(ParseError::Incomplete),
    }
}

fn optional<'a, F, G>(parser: F) -> impl Fn(&'a [u8]) -> ParseResult<'a, Option<G>>
where
    F: Fn(&'a [u8]) -> ParseResult<'a, G>,
    G: 'a,
{
    move |input: &[u8]| {
        Ok(parser(input)
            .map(|(i, o)| (i, Some(o)))
            .unwrap_or((input, None)))
    }
}

fn is_whitespace(input: u8) -> bool {
    matches!(input, 0u8..=9u8 | 11u8..=32u8)
}

/// Reads a whitespace token (IEEE 488.2 7.4.1.2)
fn whitespace(input: &[u8]) -> ParseResult<&[u8]> {
    match take_while(is_whitespace)(input) {
        // If no input is remaning, the input is incomplete.
        Ok((&[], &[])) => Err(ParseError::Incomplete),
        // There is only something other than whitespace.
        Ok((_, &[])) => Err(Error::InvalidCharacter)?,
        // There is at least some whitespace.
        Ok(res) => Ok(res),
        // There was another error.
        Err(error) => Err(error),
    }
}

fn tag(tag: u8) -> impl Fn(&[u8]) -> ParseResult<u8> {
    satisfy(move |byte| byte == tag)
}

fn terminator(input: &[u8]) -> ParseResult<u8> {
    tag(b'\n')(input)
}

fn digits(input: &[u8]) -> ParseResult<&[u8]> {
    let (i1, _) = satisfy(|c| c.is_ascii_digit())(input)?;
    let (i2, res) = take_while(|c| c.is_ascii_digit())(i1)?;
    Ok((i2, &input[..res.len() + 1]))
}

/// Parses a program mnemonic
fn program_mnemonic(input: &[u8]) -> ParseResult<&[u8]> {
    let (i1, _) = satisfy(|c| c.is_ascii_alphabetic())(input)?;
    let (i2, res) = take_while(|c| c.is_ascii_alphanumeric() || c == b'_')(i1)?;
    Ok((i2, &input[..res.len() + 1]))
}

fn sign(input: &[u8]) -> ParseResult<u8> {
    tag(b'+')(input).or_else(|_| tag(b'-')(input))
}

/// Parses a mnemonic value
fn mnemonic(input: &[u8]) -> ParseResult<Value<'_>> {
    let (input, res) = program_mnemonic(input)?;
    let mnemonic_str = str::from_utf8(res)?;
    Ok((input, Value::Mnemonic(mnemonic_str)))
}

fn mantissa(input: &[u8]) -> ParseResult<&[u8]> {
    let (i1, _sign) = optional(sign)(input)?;
    let (i2, d1) = optional(digits)(i1)?;
    let (i3, _decimal) = optional(tag(b'.'))(i2)?;
    let (i4, _d2) = if d1.is_some() {
        optional(digits)(i3)?
    }
    else {
        digits(i3).map(|(i, o)| (i, Some(o)))?
    };
    Ok((i4, &input[..input.len() - i4.len()]))
}

fn exponent(input: &[u8]) -> ParseResult<&[u8]> {
    let (i1, _) = satisfy(|c| c == b'E' || c == b'e')(input)?;
    let (i2, _) = optional(sign)(i1)?;
    let (i3, _) = digits(i2)?;
    Ok((i3, &input[..input.len() - i3.len()]))
}

fn decimal_numeric_program_data(input: &[u8]) -> ParseResult<Value<'_>> {
    let (i1, _) = mantissa(input)?;
    let (i2, _) = optional(exponent)(i1)?;
    let res = str::from_utf8(&input[..input.len() - i2.len()])?;
    Ok((i2, Value::Decimal(res)))
}

fn header_separator(input: &[u8]) -> ParseResult<()> {
    let (input, _) = optional(whitespace)(input)?;
    let (input, _) = tag(b':')(input).map_err(|_| Error::HeaderSeparatorError)?;
    let (input, _) = optional(whitespace)(input)?;
    Ok((input, ()))
}

/// Parses a common command program header (e.g., "*IDN")
fn common_command_program_header(
    root: &'static Node,
) -> impl Fn(&[u8]) -> ParseResult<&'static Node> {
    move |input: &[u8]| {
        let (i1, _) = tag(b'*')(input).map_err(|_| Error::UndefinedHeader)?;
        let (i2, res) = program_mnemonic(i1)?;
        let name = &input[0..res.len() + 1]; // Include the asterisk in the name
        let node = root
            .child(str::from_utf8(name)?)
            .ok_or(Error::UndefinedHeader)?;

        Ok((i2, node))
    }
}

/// Parses a compound command program header (e.g., "SYST:ERR")
fn compound_command_program_header(
    root: &'static Node,
) -> impl Fn(&[u8]) -> ParseResult<&'static Node> {
    move |mut input: &[u8]| {
        let mut node = root;
        let (i, res) = program_mnemonic(input)?;
        let name = str::from_utf8(res)?;
        node = node.child(name).ok_or(Error::UndefinedHeader)?;
        input = i;

        loop {
            let i = match header_separator(input) {
                Ok((input, _)) => input,
                Err(ParseError::SoftError(_)) => break,
                Err(e) => return Err(e),
            };

            let (i, res) = program_mnemonic(i)?;
            let name = str::from_utf8(res)?;
            node = node.child(name).ok_or(Error::UndefinedHeader)?;
            input = i;
        }

        Ok((input, node))
    }
}

/// Parses the command program header (both common and compound)
fn command_program_header(root: &'static Node) -> impl Fn(&[u8]) -> ParseResult<&'static Node> {
    move |input: &[u8]| {
        compound_command_program_header(root)(input)
            .or_else(|_| common_command_program_header(root)(input))
    }
}

/// Parses an argument separator (comma with optional whitespace)
fn argument_separator(input: &[u8]) -> ParseResult<()> {
    let (input, _) = optional(whitespace)(input)?;
    let (input, _) = tag(b',')(input).map_err(|_| Error::InvalidSeparator)?;
    let (input, _) = optional(whitespace)(input)?;
    Ok((input, ()))
}

/// Parses an argument (mnemonic or number)
fn argument(input: &[u8]) -> ParseResult<Value<'_>> {
    mnemonic(input).or_else(|_| decimal_numeric_program_data(input))
}

/// Parses multiple arguments separated by commas
fn arguments<'a, 'b>(
    args: &'b mut Vec<Value<'a>, MAX_ARGS>,
) -> impl 'b + FnMut(&'a [u8]) -> ParseResult<'a, ()> {
    move |mut input: &'a [u8]| {
        let (i, arg) = argument(input)?;
        args.push(arg).unwrap();
        input = i;

        loop {
            let i = match argument_separator(input) {
                Ok((input, _)) => input,
                Err(ParseError::SoftError(_)) => break,
                e => return e,
            };

            let (i, arg) = argument(i)?;
            args.push(arg).unwrap();
            input = i;
        }

        Ok((input, ()))
    }
}

/// The main parsing function
pub fn parse<'a>(root: &'static Node, input: &'a [u8]) -> ParseResult<'a, CommandCall<'a>> {
    // Skip optional whitespace
    let (input, _) = optional(whitespace)(input)?;

    let (input, node) = command_program_header(root)(input)?;

    let (input, query) = tag(b'?')(input)
        .map(|(i, _)| (i, true))
        .unwrap_or_else(|_| (input, false));

    let (input, has_args) = match whitespace(input) {
        Ok((input, _)) => (input, true),
        Err(ParseError::SoftError(_)) => (input, false),
        Err(e) => return Err(e),
    };

    let mut args = Vec::new();
    let (input, _) = if has_args {
        arguments(&mut args)(input).unwrap_or((input, ()))
    }
    else {
        (input, ())
    };

    // Skip optional whitespace
    let (input, _) = optional(whitespace)(input)?;

    let (input, _) = terminator(input)?;

    Ok((input, CommandCall { node, query, args }))
}

#[cfg(test)]
mod tests {
    use super::*;

    static ROOT_NODE: Node = Node {
        children: &[("*IDN", &IDN_NODE), ("SYST", &SYST_NODE)],
        command: None,
        query: None,
    };

    static IDN_NODE: Node = Node {
        children: &[],
        command: None,
        query: None,
    };

    static SYST_NODE: Node = Node {
        children: &[("ERR", &ERR_NODE)],
        command: None,
        query: None,
    };

    static ERR_NODE: Node = Node {
        children: &[],
        command: None,
        query: None,
    };

    #[test]
    pub fn test_take_while() {
        assert_eq!(
            take_while(|c| c == b'z')(b"zzzzx"),
            Ok((&b"x"[..], &b"zzzz"[..]))
        );

        assert_eq!(
            take_while(|c| c == b'x')(b"zzzzx"),
            Ok((&b"zzzzx"[..], &b""[..]))
        );
    }

    #[test]
    pub fn test_whitespace() {
        assert_eq!(
            whitespace(b" \t \r xyz"),
            Ok((&b"xyz"[..], &b" \t \r "[..]))
        );
        assert_eq!(whitespace(b"abc"), Err(Error::InvalidCharacter.into()));
    }

    #[test]
    pub fn test_digits() {
        assert_eq!(digits(b""), Err(ParseError::Incomplete));
        assert_eq!(digits(b"abc"), Err(Error::InvalidCharacter.into()));
        assert_eq!(digits(b"1"), Ok((&b""[..], &b"1"[..])));
        assert_eq!(digits(b"1a"), Ok((&b"a"[..], &b"1"[..])));
        assert_eq!(digits(b"12"), Ok((&b""[..], &b"12"[..])));
        assert_eq!(digits(b"12a"), Ok((&b"a"[..], &b"12"[..])));
        assert_eq!(digits(b"01234567890"), Ok((&b""[..], &b"01234567890"[..])));
        assert_eq!(
            digits(b"01234567890abc"),
            Ok((&b"abc"[..], &b"01234567890"[..]))
        );
    }

    #[test]
    pub fn test_mnemonic() {
        assert_eq!(
            mnemonic(b"a1b2_c3 uvw"),
            Ok((&b" uvw"[..], Value::Mnemonic("a1b2_c3")))
        );

        assert_eq!(
            mnemonic(b"142b"),
            Err(ParseError::SoftError(Some(Error::InvalidCharacter)))
        );
    }

    #[test]
    pub fn test_decimal() {
        assert_eq!(
            decimal_numeric_program_data(b"089582"),
            Ok((&b""[..], Value::Decimal("089582")))
        );

        assert_eq!(
            decimal_numeric_program_data(b"23.439"),
            Ok((&b""[..], Value::Decimal("23.439")))
        );

        assert_eq!(
            decimal_numeric_program_data(b"42.439E-29"),
            Ok((&b""[..], Value::Decimal("42.439E-29")))
        );
    }

    #[test]
    pub fn test_arguments() {
        let mut args: Vec<Value<'_>, MAX_ARGS> = Vec::new();
        assert_eq!(arguments(&mut args)(b"123, 456\n"), Ok((&b"\n"[..], ())));
        assert_eq!(&args[..], &[Value::Decimal("123"), Value::Decimal("456")]);
    }

    #[test]
    pub fn test_satisfy() {
        assert_eq!(satisfy(|c| c == b'a')(b"abc"), Ok((&b"bc"[..], b'a')));
        assert_eq!(
            satisfy(|c| c == b'a')(b"bca"),
            Err(Error::InvalidCharacter.into())
        );
        assert_eq!(satisfy(|c| c == b'a')(b""), Err(ParseError::Incomplete));
    }

    #[test]
    pub fn test_optional() {
        assert_eq!(
            optional(satisfy(|c| c == b'a'))(b"abc"),
            Ok((&b"bc"[..], Some(b'a')))
        );
        assert_eq!(
            optional(satisfy(|c| c == b'a'))(b"bca"),
            Ok((&b"bca"[..], None))
        );
        assert_eq!(optional(satisfy(|c| c == b'a'))(b""), Ok((&b""[..], None)));
    }

    #[test]
    pub fn test_tag() {
        assert_eq!(tag(b'a')(b"abc"), Ok((&b"bc"[..], b'a')));
        assert_eq!(tag(b'a')(b"bca"), Err(Error::InvalidCharacter.into()));
        assert_eq!(tag(b'a')(b""), Err(ParseError::Incomplete));
    }

    #[test]
    pub fn test_terminator() {
        assert_eq!(terminator(b"\n"), Ok((&b""[..], b'\n')));
        assert_eq!(terminator(b"\nabc"), Ok((&b"abc"[..], b'\n')));
        assert_eq!(terminator(b"abc"), Err(Error::InvalidCharacter.into()));
    }

    #[test]
    pub fn test_mantissa() {
        assert_eq!(mantissa(b"123.456"), Ok((&b""[..], &b"123.456"[..])));
        assert_eq!(mantissa(b"-123.456"), Ok((&b""[..], &b"-123.456"[..])));
        assert_eq!(mantissa(b"123"), Ok((&b""[..], &b"123"[..])));
        assert_eq!(mantissa(b".456"), Ok((&b""[..], &b".456"[..])));
        assert_eq!(mantissa(b"abc"), Err(Error::InvalidCharacter.into()));
    }

    #[test]
    pub fn test_exponent() {
        assert_eq!(exponent(b"E123"), Ok((&b""[..], &b"E123"[..])));
        assert_eq!(exponent(b"e-123"), Ok((&b""[..], &b"e-123"[..])));
        assert_eq!(exponent(b"E"), Err(ParseError::Incomplete));
        assert_eq!(exponent(b"abc"), Err(Error::InvalidCharacter.into()));
    }

    #[test]
    pub fn test_header_separator() {
        assert_eq!(header_separator(b": "), Ok((&b""[..], ())));
        assert_eq!(header_separator(b":"), Ok((&b""[..], ())));
        assert_eq!(
            header_separator(b"abc"),
            Err(Error::HeaderSeparatorError.into())
        );
    }

    #[test]
    pub fn test_common_command_program_header() {
        assert_eq!(
            common_command_program_header(&ROOT_NODE)(b"*IDN"),
            Ok((&b""[..], &IDN_NODE))
        );

        assert_eq!(
            common_command_program_header(&ROOT_NODE)(b"*XYZ"),
            Err(Error::UndefinedHeader.into())
        );
    }

    #[test]
    pub fn test_compound_command_program_header() {
        assert_eq!(
            compound_command_program_header(&ROOT_NODE)(b"SYST:ERR"),
            Ok((&b""[..], &ERR_NODE))
        );

        assert_eq!(
            compound_command_program_header(&ROOT_NODE)(b"SYST:XYZ"),
            Err(Error::UndefinedHeader.into())
        );
    }

    #[test]
    pub fn test_command_program_header() {
        assert_eq!(
            command_program_header(&ROOT_NODE)(b"*IDN"),
            Ok((&b""[..], &IDN_NODE))
        );

        assert_eq!(
            command_program_header(&ROOT_NODE)(b"SYST:ERR"),
            Ok((&b""[..], &ERR_NODE))
        );

        assert_eq!(
            command_program_header(&ROOT_NODE)(b"*XYZ"),
            Err(Error::UndefinedHeader.into())
        );
    }

    #[test]
    pub fn test_argument_separator() {
        assert_eq!(argument_separator(b", "), Ok((&b""[..], ())));
        assert_eq!(argument_separator(b","), Ok((&b""[..], ())));
        assert_eq!(
            argument_separator(b"abc"),
            Err(Error::InvalidSeparator.into())
        );
    }

    #[test]
    pub fn test_parse() {
        assert_eq!(
            parse(&ROOT_NODE, b"*IDN?\n"),
            Ok((&b""[..], CommandCall {
                node: &IDN_NODE,
                query: true,
                args: Vec::new()
            }))
        );

        assert_eq!(
            parse(&ROOT_NODE, b"SYST:ERR 123, 456\n"),
            Ok((&b""[..], CommandCall {
                node: &ERR_NODE,
                query: false,
                args: heapless::Vec::from_slice(&[Value::Decimal("123"), Value::Decimal("456")])
                    .unwrap()
            }))
        );

        assert_eq!(
            parse(&ROOT_NODE, b"*XYZ\n"),
            Err(Error::UndefinedHeader.into())
        );
    }
}
