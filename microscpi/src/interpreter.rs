use heapless::Vec;
#[cfg(feature = "embedded-io-async")]
use embedded_io_async::{BufRead, Write};

use crate::parser::{ParseResult, Parser};
use crate::tokens::Tokenizer;
use crate::{Context, Error, Interface, Node, Value, MAX_ARGS};
#[cfg(feature = "embedded-io-async")]
use crate::OUTPUT_BUFFER_SIZE;

pub struct Interpreter<I: Interface> {
    pub interface: I,
    pub context: Context,
    current_node: &'static Node,
    parser: Parser,
}

#[derive(Default)]
struct CommandCall<'a> {
    pub query: bool,
    pub args: Vec<Value<'a>, MAX_ARGS>,
}

impl<I: Interface> Interpreter<I> {
    pub fn new(interface: I) -> Interpreter<I> {
        Interpreter {
            context: Context::new(),
            current_node: interface.root_node(),
            parser: Parser::new(),
            interface,
        }
    }

    pub fn reset(&mut self) {
        self.parser = Parser::new();
        self.current_node = self.interface.root_node();
    }

    pub async fn parse_and_execute<'a>(
        &mut self, input: &'a [u8], response: &mut impl core::fmt::Write,
    ) -> Option<&'a [u8]> {
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
                        self.context.push_error(Error::UndefinedHeader);
                    }
                }
                ParseResult::Argument(value) => {
                    if call.args.push(value).is_err() {
                        // Too many arguments.
                        self.context.push_error(Error::UnexpectedNumberOfParameters);
                    }
                }
                ParseResult::Query => {
                    call.query = true;
                }
                ParseResult::Terminator => {
                    let command = if call.query {
                        self.current_node.query
                    }
                    else {
                        self.current_node.command
                    };

                    if let Some(command) = command {
                        let result = self.interface.execute_command(&mut self.context, command, &call.args, response).await;

                        if let Err(error) = result {
                            self.context.push_error(error);
                        }
                        else {
                            if let Err(error) = response.write_char('\n') {
                                self.context.push_error(error.into());
                            }
                        }

                        call = Default::default();
                        self.reset();
                    }
                    else {
                        self.context.push_error(Error::UndefinedHeader);
                    }
                }
                ParseResult::Incomplete(remaining) => {
                    return Some(remaining);
                }
                ParseResult::Done => {
                    return None;
                }
                ParseResult::Err(error) => {
                    self.context.push_error(error);
                    return None;
                }
            }
        }
    }

    #[cfg(feature = "embedded-io-async")]
    pub async fn process<R, W>(&mut self, mut input: R, mut output: W) -> Result<(), R::Error>
    where
        R: BufRead,
        W: Write,
    {
        let mut next_index: usize = 0;
        let mut output_buffer: heapless::Vec<u8, OUTPUT_BUFFER_SIZE> = Vec::new();

        loop {
            let buf = input.fill_buf().await?;
            let read_to = buf.len();
            let mut read_from: usize = next_index;

            #[cfg(feature = "defmt")]
            defmt::trace!("Read from: {}, Read to: {}", read_from, read_to);

            while let Some(offset) = buf[read_from..read_to].iter().position(|b| *b == b'\n') {
                let terminator_index = read_from + offset;

                let data = &buf[read_from..=terminator_index];
                self.parse_and_execute(data, &mut output_buffer).await;

                #[cfg(feature = "defmt")]
                defmt::trace!("Data: {}", data);

                if !output_buffer.is_empty() {
                    output.write_all(&output_buffer).await.unwrap();
                    output.flush().await.unwrap();
                    output_buffer.clear();
                }

                read_from = terminator_index + 1;
            }

            input.consume(read_from);

            next_index = read_to - read_from;
        }
    }
}
