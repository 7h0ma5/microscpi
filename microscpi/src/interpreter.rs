#[cfg(feature = "embedded-io-async")]
use embedded_io_async::{BufRead, Write};

use crate::parser::{self, CommandCall};
#[cfg(feature = "embedded-io-async")]
use crate::OUTPUT_BUFFER_SIZE;
use crate::{Context, Error, Interface, Node};

pub struct Interpreter<I: Interface> {
    pub interface: I,
    pub context: Context,
    current_node: &'static Node,
}

impl<I: Interface> Interpreter<I> {
    pub fn new(interface: I) -> Interpreter<I> {
        Interpreter {
            context: Context::new(),
            current_node: interface.root_node(),
            interface,
        }
    }

    pub fn reset(&mut self) {
        self.current_node = self.interface.root_node();
    }

    pub async fn execute(
        &mut self, call: &CommandCall<'_>, response: &mut impl core::fmt::Write,
    ) -> Result<(), Error> {
        let command = if call.query {
            call.node.query
        }
        else {
            call.node.command
        };

        if let Some(command) = command {
            self.interface
                .execute_command(&mut self.context, command, &call.args, response)
                .await?;

            if call.query {
                response.write_char('\n')?;
            }
        }
        else {
            return Err(Error::UndefinedHeader);
        }

        Ok(())
    }

    pub async fn parse_and_execute<'a>(
        &mut self, mut input: &'a [u8], response: &mut impl core::fmt::Write,
    ) -> &'a [u8] {
        while !input.is_empty() {
            let result = parser::parse(self.current_node, input);

            if let Err(error) = result {
                self.context.push_error(error.into());
                return input;
            }

            let (i, call) = result.unwrap();

            if let Err(error) = self.execute(&call, response).await {
                self.context.push_error(error);
            }

            self.reset();

            input = i;
        }
        &[][..]
    }

    #[cfg(feature = "embedded-io-async")]
    pub async fn process<R, W>(&mut self, mut input: R, mut output: W) -> Result<(), R::Error>
    where
        R: BufRead,
        W: Write,
    {
        let mut next_index: usize = 0;
        let mut output_buffer: heapless::Vec<u8, OUTPUT_BUFFER_SIZE> = heapless::Vec::new();

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
