use core::str;

use crate::tokens::{ScanResult, Token, Tokenizer};
use crate::{Error, Value};

#[derive(PartialEq, Clone, Copy)]
enum ParserState {
    // Expect either a command to begin or infinite terminators
    Start,
    // Expect a command path component as the next token.
    Path,
    // Expect a path seperator (`;`) as the next token. Alternatively
    // continue with parsing the arguments when a question mark or whitespace is encountered.
    PathSeparator,
    // Expect whitespace as the next token. This is the divider between the command path and the
    // arguments.
    Whitespace,
    Argument,
    ArgumentSeparator,
}

pub enum ParseResult<'a> {
    Path(&'a str),
    Argument(Value<'a>),
    Query,
    Terminator,
    Incomplete(&'a [u8]),
    Done,
    Err(Error),
}

pub struct Parser {
    state: ParserState,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            state: ParserState::Start,
        }
    }

    pub fn parse_next<'a>(&mut self, tokenizer: &mut Tokenizer<'a>) -> ParseResult<'a> {
        loop {
            let token = match tokenizer.next_token() {
                ScanResult::Ok(token) => token,
                ScanResult::Err(error) => return ParseResult::Err(error),
                ScanResult::Incomplete(remaining) => return ParseResult::Incomplete(remaining),
                ScanResult::Done => return ParseResult::Done,
            };

            match (self.state, token) {
                (ParserState::PathSeparator, Token::Whitespace) => {
                    self.state = ParserState::Argument;
                }
                (ParserState::Whitespace, Token::Whitespace) => {
                    self.state = ParserState::Argument;
                }
                (_, Token::Whitespace) => {
                    // ignore
                }
                (
                    ParserState::PathSeparator
                    | ParserState::ArgumentSeparator
                    | ParserState::Argument
                    | ParserState::Whitespace,
                    Token::Terminator,
                ) => {
                    self.state = ParserState::Start;
                    return ParseResult::Terminator;
                }
                (ParserState::Start, Token::Terminator) => {
                    // ignore
                },
                (ParserState::PathSeparator, Token::QuestionMark) => {
                    self.state = ParserState::Whitespace;
                    return ParseResult::Query;
                }
                (ParserState::Start | ParserState::Path, Token::Mnemonic(name)) => {
                    self.state = ParserState::PathSeparator;
                    let path = str::from_utf8(name).unwrap();
                    return ParseResult::Path(path);
                }
                (ParserState::PathSeparator, Token::Colon) => {
                    self.state = ParserState::Path;
                }
                (ParserState::PathSeparator, _) => {
                    return ParseResult::Err(Error::InvalidSeparator);
                }
                (ParserState::Argument, Token::Number(value)) => {
                    self.state = ParserState::ArgumentSeparator;
                    let number = str::from_utf8(value).unwrap();
                    return ParseResult::Argument(Value::Number(number));
                }
                (ParserState::Argument, Token::Mnemonic(value)) => {
                    self.state = ParserState::ArgumentSeparator;
                    let name = str::from_utf8(value).unwrap();
                    return ParseResult::Argument(Value::Mnemonic(name));
                }
                (ParserState::ArgumentSeparator, Token::Comma) => {
                    self.state = ParserState::Argument;
                }
                (ParserState::ArgumentSeparator, _) => {
                    return ParseResult::Err(Error::InvalidSeparator);
                }
                _ => {
                    return ParseResult::Err(Error::SyntaxError);
                }
            }
        }
    }
}
