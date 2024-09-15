use core::fmt::Write;

use heapless::Vec;

use crate::parser::{ParseResult, Parser};
use crate::tokens::Tokenizer;
use crate::{Error, Interface, ScpiTreeNode, Value};

const MAX_ARGS: usize = 10;

#[derive(Default)]
pub struct CommandCall<'a> {
    pub query: bool,
    pub args: Vec<Value<'a>, MAX_ARGS>,
}

pub struct Context<I: for<'i> Interface<'i>> {
    pub interface: I,
    current_node: &'static ScpiTreeNode,
    parser: Parser,
}

impl<'c, I: for<'i> Interface<'i>> Context<I> {
    pub fn new(interface: I) -> Context<I> {
        Context {
            interface,
            current_node: I::root_node(),
            parser: Parser::new(),
        }
    }

    fn reset(&mut self) {
        self.current_node = I::root_node();
        self.parser = Parser::new();
    }

    pub async fn process<'a>(
        &mut self, input: &'a [u8], response: &mut impl Write,
    ) -> Result<&'a [u8], Error> {
        let mut tokenizer = Tokenizer::new(input);

        let mut call = CommandCall::default();

        loop {
            match self.parser.parse_next(&mut tokenizer) {
                ParseResult::Path(path) => {
                    if let Some(child) = self.current_node.child(path) {
                        self.current_node = child;
                    }
                    else {
                        self.reset();
                        return Err(Error::InvalidCommand);
                    }                    
                },
                ParseResult::Argument(value) => {
                    call.args.push(value).or(Err(Error::ArgumentOverflow))?;
                },
                ParseResult::Query => {
                    call.query = true;
                },
                ParseResult::Terminator => {
                    let command = if call.query {
                        self.current_node.query
                    }
                    else {
                        self.current_node.command
                    };

                    if let Some(command) = command {
                        let result = self.interface.run_command(command, &call.args).await?;
                        if result != Value::Void {
                            writeln!(response, "{}", result)?;
                        }
                        call = Default::default();
                        self.reset();
                    }
                    else {
                        return Err(Error::InvalidCommand)
                    }
                },
                ParseResult::Incomplete(remaining) => {
                    return Ok(remaining);
                },
                ParseResult::Done => {
                    return Ok(&[]);
                }
                ParseResult::Err(error) => {
                    return Err(error);
                }
            }
        }
    }
}
