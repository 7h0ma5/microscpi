use core::str::{self, Utf8Error};

use heapless::Vec;

use crate::{Error, Node, Value, MAX_ARGS};

/// Enum to handle both recoverable and fatal errors
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Recoverable error (continue trying other paths)
    SoftError(Option<Error>),
    /// Unrecoverable syntax error
    FatalError(Error),
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

pub struct CommandCall<'a> {
    pub node: &'a Node,
    pub query: bool,
    pub args: Vec<Value<'a>, MAX_ARGS>,
}

/// Reads a whitespace token (IEEE 488.2 7.4.1.2)
fn whitespace(input: &[u8]) -> ParseResult<&[u8]> {
    let pos = input
        .iter()
        .position(|&c| !(matches!(c, 0u8..=9u8 | 11u8..=32u8)));

    match pos {
        Some(pos) if pos > 0 => Ok((&input[pos..], &input[..pos])),
        None if !input.is_empty() => Ok((&[], input)),
        _ => Err(Error::InvalidCharacter)?,
    }
}

fn tag(tag: u8) -> impl Fn(&[u8]) -> ParseResult<u8> {
    move |i: &[u8]| {
        if i.first() == Some(&tag) {
            Ok((&i[1..], tag))
        }
        else {
            Err(Error::InvalidCharacter)?
        }
    }
}

fn terminator(input: &[u8]) -> ParseResult<u8> {
    tag(b'\n')(input)
}

/// Parses a program mnemonic
fn program_mnemonic(input: &[u8]) -> ParseResult<&[u8]> {
    let first = input.first().ok_or(Error::InvalidCharacter)?;
    if !(*first).is_ascii_alphabetic() {
        Err(Error::InvalidCharacter)?;
    }

    let pos = input
        .iter()
        .enumerate()
        .skip(1)
        .take_while(|(_, &c)| c.is_ascii_alphanumeric() || c == b'_')
        .map(|(i, _)| i)
        .last()
        .map_or(1, |i| i + 1);

    Ok((&input[pos..], &input[..pos]))
}

/// Parses a mnemonic value
fn mnemonic(input: &[u8]) -> ParseResult<Value<'_>> {
    let (input, res) = program_mnemonic(input)?;
    let mnemonic_str = str::from_utf8(res)?;
    Ok((input, Value::Mnemonic(mnemonic_str)))
}

/// Parses a number value (digits and possibly a decimal point)
fn number(input: &[u8]) -> ParseResult<Value<'_>> {
    let pos = input
        .iter()
        .enumerate()
        .take_while(|(_, &c)| c.is_ascii_digit() || c == b'.')
        .map(|(i, _)| i)
        .last()
        .map_or(0, |i| i + 1);

    if pos > 0 {
        let num_str = str::from_utf8(&input[..pos])?;
        Ok((&input[pos..], Value::Number(num_str)))
    }
    else {
        Err(().into())
    }
}

/// Parses a header separator
fn header_separator(input: &[u8]) -> ParseResult<()> {
    let (input, _) = whitespace(input).unwrap_or((input, &[]));
    let (input, _) = tag(b':')(input).map_err(|_| Error::HeaderSeparatorError)?;
    let (input, _) = whitespace(input).unwrap_or((input, &[]));
    Ok((input, ()))
}

/// Parses a common command program header (e.g., "*IDN")
fn common_command_program_header(
    root: &'static Node,
) -> impl Fn(&[u8]) -> ParseResult<&'static Node> {
    move |input: &[u8]| {
        let (i1, _) = tag(b'*')(input).map_err(|_| Error::CommandHeaderError)?;
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
        common_command_program_header(root)(input)
            .or_else(|_| compound_command_program_header(root)(input))
    }
}

/// Parses an argument separator (comma with optional whitespace)
fn argument_separator(input: &[u8]) -> ParseResult<()> {
    let (input, _) = whitespace(input).unwrap_or((input, &[]));
    let (input, _) = tag(b',')(input).map_err(|_| Error::InvalidSeparator)?;
    let (input, _) = whitespace(input).unwrap_or((input, &[]));
    Ok((input, ()))
}

/// Parses an argument (mnemonic or number)
fn argument(input: &[u8]) -> ParseResult<Value<'_>> {
    mnemonic(input).or_else(|_| number(input))
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
    let (input, _) = whitespace(input).unwrap_or((input, &[]));

    let (input, node) = command_program_header(root)(input)?;

    let (input, query) = tag(b'?')(input)
        .map(|(i, _)| (i, true))
        .unwrap_or_else(|_| (input, false));

    // Skip optional whitespace
    let (input, _) = whitespace(input).unwrap_or((input, &[]));

    let mut args = Vec::new();
    let (input, _) = arguments(&mut args)(input).unwrap_or((input, ()));

    // Skip optional whitespace
    let (input, _) = whitespace(input).unwrap_or((input, &[]));

    let (input, _) = terminator(input)?;

    Ok((input, CommandCall { node, query, args }))
}

#[test]
pub fn parse_whitespace() {
    assert_eq!(
        whitespace(b" \t \r xyz"),
        Ok((&b"xyz"[..], &b" \t \r "[..]))
    );
    assert_eq!(whitespace(b"abc"), Err(Error::InvalidCharacter.into()));
}

#[test]
pub fn parse_mnemonic() {
    assert_eq!(
        mnemonic(b"a1b2_c3 uvw"),
        Ok((&b" uvw"[..], Value::Mnemonic("a1b2_c3")))
    );
}

#[test]
pub fn parse_arguments() {
    let mut args: Vec<Value<'_>, MAX_ARGS> = Vec::new();
    assert_eq!(arguments(&mut args)(b"123, 456\n"), Ok((&b"\n"[..], ())));
    assert_eq!(&args[..], &[Value::Number("123"), Value::Number("456")]);
}
