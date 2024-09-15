#[derive(Debug)]
pub enum Error {
    InvalidToken,
}

#[derive(Debug)]
pub enum Token<'a> {
    Whitespace,
    Mnemonic(&'a [u8]),
    Number(&'a [u8]),
    Colon,
    Comma,
    SemiColon,
    QuestionMark,
    Terminator,
}

#[derive(Debug)]
pub enum ScanResult<'a> {
    Ok(Token<'a>),
    Incomplete(&'a [u8]),
    Err(Error),
    Done,
}

type InputStream<'a> = &'a [u8];

pub struct Tokenizer<'a> {
    input: &'a [u8],
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: InputStream<'a>) -> Tokenizer<'a> {
        Tokenizer { input }
    }

    fn take_until<F>(&mut self, pred: F) -> Option<&'a [u8]>
    where
        F: FnMut(&u8) -> bool,
    {
        self.input.iter().position(pred).map(|pos| {
            let result = &self.input[..pos];
            self.input = &self.input[pos..];
            result
        })
    }

    /// Read a whitespace token (IEEE 488.2 7.4.1.2).
    fn whitespace(&mut self) -> ScanResult<'a> {
        self.take_until(|c| !matches!(*c, 0u8..=9u8 | 11u8..=32u8))
            .map(|_| ScanResult::Ok(Token::Whitespace))
            .unwrap_or(ScanResult::Incomplete(self.input))
    }

    fn mnemonic(&mut self) -> ScanResult<'a> {
        self.take_until(|c| !matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'_'))
            .map(|result| ScanResult::Ok(Token::Mnemonic(result)))
            .unwrap_or(ScanResult::Incomplete(self.input))
    }

    fn number(&mut self) -> ScanResult<'a> {
        self.take_until(|c| !matches!(c, b'0'..=b'9' | b'.'))
            .map(|result| ScanResult::Ok(Token::Number(result)))
            .unwrap_or(ScanResult::Incomplete(self.input))
    }

    pub fn next_token(&mut self) -> ScanResult<'a> {
        match self.input.first() {
            Some(0u8..=9u8 | 11u8..=32u8) => self.whitespace(),
            Some(b':') => {
                self.input = &self.input[1..];
                ScanResult::Ok(Token::Colon)
            }
            Some(b'?') => {
                self.input = &self.input[1..];
                ScanResult::Ok(Token::QuestionMark)
            }
            Some(b';') => {
                self.input = &self.input[1..];
                ScanResult::Ok(Token::SemiColon)
            }
            Some(b',') => {
                self.input = &self.input[1..];
                ScanResult::Ok(Token::Comma)
            }
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') | Some(b'*') => self.mnemonic(),
            Some(b'0'..=b'9') => self.number(),
            Some(b'\n') => {
                self.input = &self.input[1..];
                ScanResult::Ok(Token::Terminator)
            },
            Some(_) => ScanResult::Err(Error::InvalidToken),
            None => {
                if !self.input.is_empty() {
                    ScanResult::Incomplete(self.input)
                }
                else {
                    ScanResult::Done
                }
            }
        }
    }
}
