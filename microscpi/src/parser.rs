use core::str::{self, Utf8Error};

use heapless::Vec;

use crate::tree::Node;
use crate::{Error, Value, MAX_ARGS};

/// Enum to handle both recoverable and fatal errors.
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

/// Type alias for the parser result.
type ParseResult<'a, T> = Result<(&'a [u8], T), ParseError>;

#[derive(Debug, PartialEq)]
pub struct CommandCall<'a> {
    pub node: &'static Node,
    pub query: bool,
    pub args: Vec<Value<'a>, MAX_ARGS>,
}

/// Takes bytes while the predicate function is true.
///
/// Returns a tuple with the remaining input and the slice of bytes that were
/// taken.
fn take_while<F>(pred: F) -> impl Fn(&[u8]) -> ParseResult<&[u8]>
where
    F: Fn(u8) -> bool,
{
    move |input: &[u8]| match input.iter().position(|&byte| !pred(byte)) {
        Some(pos) => Ok((&input[pos..], &input[..pos])),
        None => Ok((&[], input)),
    }
}

/// Takes a single byte that satisfies the predicate function.
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

/// Makes a parser optional.
///
/// If the parser fails, the result is None.
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

/// Checks if a byte is a whitespace character according to IEEE 488.2)
fn is_whitespace(input: u8) -> bool {
    matches!(input, 0u8..=9u8 | 11u8..=32u8)
}

/// Parses whitespace characters.
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

/// Parses a single specific byte.
fn tag(tag: u8) -> impl Fn(&[u8]) -> ParseResult<u8> {
    satisfy(move |byte| byte == tag)
}

/// Parses a terminator (newline character).
fn terminator(input: &[u8]) -> ParseResult<u8> {
    tag(b'\n')(input)
}

/// Parses a sequence of digits.
fn digits(input: &[u8]) -> ParseResult<&[u8]> {
    let (i1, _) = satisfy(|c| c.is_ascii_digit())(input)?;
    let (i2, res) = take_while(|c| c.is_ascii_digit())(i1)?;
    Ok((i2, &input[..res.len() + 1]))
}

/// Parses a program mnemonic (e.g., "SYSTEM").
fn program_mnemonic(input: &[u8]) -> ParseResult<&[u8]> {
    let (i1, _) = satisfy(|c| c.is_ascii_alphabetic())(input)?;
    let (i2, res) = take_while(|c| c.is_ascii_alphanumeric() || c == b'_')(i1)?;
    Ok((i2, &input[..res.len() + 1]))
}

/// Parses a sign character (`+` or `-`).
fn sign(input: &[u8]) -> ParseResult<u8> {
    tag(b'+')(input).or_else(|_| tag(b'-')(input))
}

/// Parses a mnemonic value.
fn mnemonic(input: &[u8]) -> ParseResult<Value<'_>> {
    let (input, res) = program_mnemonic(input)?;
    let mnemonic_str = str::from_utf8(res)?;
    Ok((input, Value::Mnemonic(mnemonic_str)))
}

/// Parses the mantissa part of a decimal number.
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

/// Parses the exponent part of a decimal number.
fn exponent(input: &[u8]) -> ParseResult<&[u8]> {
    let (i1, _) = satisfy(|c| c == b'E' || c == b'e')(input)?;
    let (i2, _) = optional(sign)(i1)?;
    let (i3, _) = digits(i2)?;
    Ok((i3, &input[..input.len() - i3.len()]))
}

/// Parses a decimal number.
fn decimal_numeric_program_data(input: &[u8]) -> ParseResult<Value<'_>> {
    let (i1, _) = mantissa(input)?;
    let (i2, _) = optional(exponent)(i1)?;
    let res = str::from_utf8(&input[..input.len() - i2.len()])?;
    Ok((i2, Value::Decimal(res)))
}

/// Parses a hexadecimal number.
fn hexadecimal_numeric_program_data(input: &[u8]) -> ParseResult<Value<'_>> {
    let (i1, _) = tag(b'#')(input)?;
    let (i2, _) = satisfy(|c| c == b'H' || c == b'h')(i1)?;
    let (i3, _) = satisfy(|c| c.is_ascii_hexdigit())(i2)?;
    let (i4, _) = take_while(|c| c.is_ascii_hexdigit())(i3)?;
    let res = str::from_utf8(&i2[..i2.len() - i4.len()])?;
    Ok((i4, Value::Hexadecimal(res)))
}

/// Parses a binary number.
fn binary_numeric_program_data(input: &[u8]) -> ParseResult<Value<'_>> {
    let (i1, _) = tag(b'#')(input)?;
    let (i2, _) = satisfy(|c| c == b'B' || c == b'b')(i1)?;
    let (i3, _) = satisfy(|c| c == b'0' || c == b'1')(i2)?;
    let (i4, _) = take_while(|c| c == b'0' || c == b'1')(i3)?;
    let res = str::from_utf8(&i2[..i2.len() - i4.len()])?;
    Ok((i4, Value::Binary(res)))
}

/// Parses an octal number.
fn octal_numeric_program_data(input: &[u8]) -> ParseResult<Value<'_>> {
    let (i1, _) = tag(b'#')(input)?;
    let (i2, _) = satisfy(|c| c == b'Q' || c == b'q')(i1)?;
    let (i3, _) = satisfy(|c| (b'0'..b'8').contains(&c))(i2)?;
    let (i4, _) = take_while(|c| (b'0'..b'8').contains(&c))(i3)?;
    let res = str::from_utf8(&i2[..i2.len() - i4.len()])?;
    Ok((i4, Value::Octal(res)))
}

/// Parses a header separator (colon with optional whitespace).
fn header_separator(input: &[u8]) -> ParseResult<()> {
    let (input, _) = optional(whitespace)(input)?;
    let (input, _) = tag(b':')(input).map_err(|_| Error::HeaderSeparatorError)?;
    let (input, _) = optional(whitespace)(input)?;
    Ok((input, ()))
}

/// Parses a common command program header (e.g., "*IDN").
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

/// Parses a compound command program header (e.g., "SYST:ERR").
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

/// Parses the command program header (both common and compound).
fn command_program_header(root: &'static Node) -> impl Fn(&[u8]) -> ParseResult<&'static Node> {
    move |input: &[u8]| {
        compound_command_program_header(root)(input)
            .or_else(|_| common_command_program_header(root)(input))
    }
}

/// Parses an argument separator (comma with optional whitespace).
fn argument_separator(input: &[u8]) -> ParseResult<()> {
    let (input, _) = optional(whitespace)(input)?;
    let (input, _) = tag(b',')(input).map_err(|_| Error::InvalidSeparator)?;
    let (input, _) = optional(whitespace)(input)?;
    Ok((input, ()))
}

/// Parses an argument value.
fn argument(input: &[u8]) -> ParseResult<Value<'_>> {
    mnemonic(input)
        .or_else(|_| decimal_numeric_program_data(input))
        .or_else(|_| hexadecimal_numeric_program_data(input))
        .or_else(|_| binary_numeric_program_data(input))
        .or_else(|_| octal_numeric_program_data(input))
}

/// Parses multiple arguments separated by commas.
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
            args.push(arg).or(Err(Error::UnexpectedNumberOfParameters))?;
            input = i;
        }

        Ok((input, ()))
    }
}

/// Parses a SCPI command call.
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

    #[test]
    pub fn test_hexadecimal() {
        assert_eq!(
            hexadecimal_numeric_program_data(b"#H1A2B"),
            Ok((&b""[..], Value::Hexadecimal("1A2B")))
        );

        assert_eq!(
            hexadecimal_numeric_program_data(b"#h1a2b"),
            Ok((&b""[..], Value::Hexadecimal("1a2b")))
        );

        assert_eq!(
            hexadecimal_numeric_program_data(b"#HXYZ"),
            Err(Error::InvalidCharacter.into())
        );
    }

    #[test]
    pub fn test_binary() {
        assert_eq!(
            binary_numeric_program_data(b"#B1010"),
            Ok((&b""[..], Value::Binary("1010")))
        );

        assert_eq!(
            binary_numeric_program_data(b"#b1101"),
            Ok((&b""[..], Value::Binary("1101")))
        );

        assert_eq!(
            binary_numeric_program_data(b"#B102"),
            Ok((&b"2"[..], Value::Binary("10")))
        );
    }

    #[test]
    pub fn test_octal() {
        assert_eq!(
            octal_numeric_program_data(b"#Q123"),
            Ok((&b""[..], Value::Octal("123")))
        );

        assert_eq!(
            octal_numeric_program_data(b"#q456"),
            Ok((&b""[..], Value::Octal("456")))
        );

        assert_eq!(
            octal_numeric_program_data(b"#Q89"),
            Err(Error::InvalidCharacter.into())
        );
    }

    #[test]
    pub fn test_parse_with_whitespace() {
        assert_eq!(
            parse(&ROOT_NODE, b"  *IDN?  \n"),
            Ok((&b""[..], CommandCall {
                node: &IDN_NODE,
                query: true,
                args: Vec::new()
            }))
        );

        assert_eq!(
            parse(&ROOT_NODE, b"  SYST:ERR  123,  456  \n"),
            Ok((&b""[..], CommandCall {
                node: &ERR_NODE,
                query: false,
                args: heapless::Vec::from_slice(&[Value::Decimal("123"), Value::Decimal("456")])
                    .unwrap()
            }))
        );
    }

    #[test]
    pub fn test_parse_incomplete() {
        assert_eq!(
            parse(&ROOT_NODE, b"*IDN?"),
            Err(ParseError::Incomplete)
        );
    }

    #[test]
    pub fn test_parse_invalid_character() {
        assert_eq!(
            parse(&ROOT_NODE, b"*IDN?abc\n"),
            Err(Error::InvalidCharacter.into())
        );

        assert_eq!(
            parse(&ROOT_NODE, b"SYST:ERR 123, 456abc\n"),
            Err(Error::InvalidCharacter.into())
        );
    }
}
