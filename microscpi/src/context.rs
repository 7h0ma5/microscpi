#[cfg(feature = "embedded-io-async")]
use embedded_io_async::{BufRead, Write};
use heapless::Vec;

use crate::parser::{ParseResult, Parser};
use crate::tokens::Tokenizer;
use crate::{Error, Interface, ScpiTreeNode, Value};

const MAX_ARGS: usize = 10;
#[cfg(feature = "embedded-io-async")]
const OUTPUT_BUFFER_SIZE: usize = 100;

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

impl<I: for<'i> Interface<'i>> Context<I> {
    pub fn new(interface: I) -> Context<I> {
        Context {
            interface,
            current_node: I::root_node(),
            parser: Parser::new()
        }
    }

    fn reset(&mut self) {
        self.current_node = I::root_node();
        self.parser = Parser::new();
    }

    pub async fn process_buffer<'a>(
        &mut self, input: &'a [u8], response: &mut impl core::fmt::Write,
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
                        return Err(Error::UndefinedHeader);
                    }
                }
                ParseResult::Argument(value) => {
                    call.args.push(value).or(Err(Error::UnexpectedNumberOfParameters))?;
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
                        let result = self.interface.run_command(command, &call.args).await?;
                        if result != Value::Void {
                            writeln!(response, "{}", result)?;
                        }
                        call = Default::default();
                        self.reset();
                    }
                    else {
                        return Err(Error::UndefinedHeader);
                    }
                }
                ParseResult::Incomplete(remaining) => {
                    return Ok(remaining);
                }
                ParseResult::Done => {
                    return Ok(&[]);
                }
                ParseResult::Err(error) => {
                    return Err(error);
                }
            }
        }
    }

    #[cfg(feature = "embedded-io-async")]
    pub async fn process<R, W>(
        &mut self, mut input: R, mut output: W,
    ) -> Result<(), R::Error> where R: BufRead, W: Write {
        let mut next_index: usize = 0;
        let mut output_buffer: heapless::Vec<u8, OUTPUT_BUFFER_SIZE> = Vec::new();

        loop {
            let buf = input.fill_buf().await?;
            let read_to = buf.len();
            let mut read_from: usize = next_index;

            #[cfg(feature = "defmt")]
            defmt::info!("Read from: {}, Read to: {}", read_from, read_to);

            while let Some(offset) = buf[read_from..read_to].iter().position(|b| *b == b'\n') {
                let terminator_index = read_from + offset;

                let data = &buf[read_from..=terminator_index];
                self.process_buffer(data, &mut output_buffer).await.unwrap();

                #[cfg(feature = "defmt")]
                defmt::info!("Data: {}", data);

                if !output_buffer.is_empty() {
                    output.write_all(&output_buffer).await.unwrap();
                    output.flush().await.unwrap();
                    output_buffer.clear();
                }

                read_from = terminator_index + 1;
            }

            #[cfg(feature = "defmt")]
            defmt::info!("Consume {}", read_from);

            input.consume(read_from);

            next_index = read_to - read_from;
        }
    }
}
