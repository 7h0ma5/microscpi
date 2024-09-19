#[cfg(feature = "embedded-io-async")]
use embedded_io_async::{BufRead, Write};
use heapless::{Deque, Vec};

use crate::parser::{ParseResult, Parser};
use crate::tokens::Tokenizer;
use crate::{Error, Interface, ScpiTreeNode, Value};

const MAX_ERRORS: usize = 10;
const MAX_ARGS: usize = 10;
#[cfg(feature = "embedded-io-async")]
const OUTPUT_BUFFER_SIZE: usize = 100;

#[derive(Default)]
struct CommandCall<'a> {
    pub query: bool,
    pub args: Vec<Value<'a>, MAX_ARGS>,
}

/// SCPI Context
///
/// The SCPI context contains the current state of the SCPI interface including
/// the parser state, the current selected node in the SCPI command tree and the
/// error queue.
pub struct Context<I: for<'i> Interface<'i>> {
    pub interface: I,
    current_node: &'static ScpiTreeNode,
    parser: Parser,
    errors: heapless::Deque<Error, MAX_ERRORS>,
}

impl<I: for<'i> Interface<'i>> Context<I> {
    pub fn new(interface: I) -> Context<I> {
        Context {
            interface,
            current_node: I::root_node(),
            parser: Parser::new(),
            errors: Deque::new(),
        }
    }

    pub fn pop_error(&mut self) -> Option<Error> {
        self.errors.pop_front()
    }

    pub fn push_error(&mut self, error: Error) {
        #[cfg(feature = "defmt")]
        defmt::trace!("Push Error: {}", error);
        if self.errors.push_back(error).is_err() {
            // If the queue is full, change the most recent added item to an *Queue
            // Overflow* error, as specified in IEEE 488.2, 21.8.1.
            if let Some(value) = self.errors.back_mut() {
                *value = Error::QueueOverflow;
            }
        }
    }

    fn reset(&mut self) {
        self.current_node = I::root_node();
        self.parser = Parser::new();
    }

    pub async fn process_buffer<'a>(
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
                        self.push_error(Error::UndefinedHeader);
                    }
                }
                ParseResult::Argument(value) => {
                    if call.args.push(value).is_err() {
                        // Too many arguments.
                        self.push_error(Error::UnexpectedNumberOfParameters);
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
                        let result = self.interface.run_command(command, &call.args).await;
                        if let Err(error) = result {
                            self.push_error(error);
                        }
                        else if result != Ok(Value::Void) {
                            writeln!(response, "{}", result.unwrap()).unwrap();
                        }

                        call = Default::default();
                        self.reset();
                    }
                    else {
                        self.push_error(Error::UndefinedHeader);
                    }
                }
                ParseResult::Incomplete(remaining) => {
                    return Some(remaining);
                }
                ParseResult::Done => {
                    return None;
                }
                ParseResult::Err(error) => {
                    self.push_error(error);
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
                self.process_buffer(data, &mut output_buffer).await;

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
