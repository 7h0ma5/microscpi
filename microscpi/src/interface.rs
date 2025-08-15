use crate::parser::{self, CommandCall, ParseError};
use crate::{CommandId, Error, Value, tree};

pub trait ErrorHandler {
    fn handle_error(&mut self, _error: Error);
}

pub trait Adapter {
    type Error;

    async fn read(&mut self, dst: &mut [u8]) -> Result<usize, Self::Error>;
    async fn write(&mut self, src: &[u8]) -> Result<(), Self::Error>;
    async fn flush(&mut self) -> Result<(), Self::Error>;
}

pub trait Interface: ErrorHandler {
    /// Returns the root node of the SCPI command tree of this interface.
    #[doc(hidden)]
    fn root_node(&self) -> &'static tree::Node;

    /// Executes the command with the specified command id and the supplied
    /// arguments.
    #[doc(hidden)]
    async fn execute_command<'a>(
        &'a mut self,
        command_id: CommandId,
        args: &[Value<'a>],
        response: &mut impl crate::Write,
    ) -> Result<(), Error>;

    #[doc(hidden)]
    async fn execute(
        &mut self,
        call: &CommandCall<'_>,
        response: &mut impl crate::Write,
    ) -> Result<(), Error> {
        let command = if call.query {
            call.node.query
        } else {
            call.node.command
        };

        if let Some(command) = command {
            self.execute_command(command, &call.args, response).await?;

            if call.query {
                response.write_char('\n')?;
            }
        } else {
            return Err(Error::UndefinedHeader);
        }

        Ok(())
    }

    /// Parses and executes the commands in the input buffer.
    ///
    /// The result is written to the response buffer. Any remaining input that
    /// was not parsed is returned. If an error occurs, the remaining input
    /// is returned and the error is passed to the error handler.
    async fn run<'a>(&mut self, mut input: &'a [u8], response: &mut impl crate::Write) -> &'a [u8] {
        let mut header = self.root_node();

        while !input.is_empty() {
            let result = parser::parse(self.root_node(), header, input);

            #[cfg(feature = "defmt")]
            defmt::trace!("Run: {:?}", input);

            let (i, call) = match result {
                Ok(result) => result,
                Err(ParseError::Incomplete) => {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("Incomplete Input");
                    return input;
                }
                Err(error) => {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("Parse error");
                    self.handle_error(error.into());

                    // Try to continue with the rest of the input after a command terminator.
                    if let Some((end, terminator)) = input
                        .iter()
                        .enumerate()
                        .find(|(_i, c)| **c == b';' || **c == b'\n')
                    {
                        if *terminator == b'\n' {
                            return &input[end + 1..];
                        } else {
                            input = &input[end + 1..];
                            continue;
                        }
                    } else {
                        return &[];
                    }
                }
            };

            if let Some(call) = call {
                if let Err(error) = self.execute(&call, response).await {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("Execution error");
                    self.handle_error(error);
                }

                if call.terminated {
                    // Reset the header to the root node if a call is ended with a terminator.
                    header = self.root_node();
                } else if let Some(call_header) = call.header {
                    // Update the current header, if the current command is not a common command.
                    header = call_header;
                }
            }

            input = i;
        }
        &[]
    }

    async fn process<const N: usize, A: Adapter>(
        &mut self,
        adapter: &mut A,
    ) -> Result<(), A::Error> {
        let mut cmd_buf = [0u8; N];
        let mut res_buf: heapless::Vec<u8, N> = heapless::Vec::new();

        let mut proc_offset = 0;
        let mut read_offset = 0;

        loop {
            let count = adapter.read(&mut cmd_buf[read_offset..]).await?;
            let read_end = read_offset + count;

            // Find the first terminator in the buffer starting from the last read position.
            while let Some(position) = cmd_buf[read_offset..read_end]
                .iter()
                .position(|b| *b == b'\n')
            {
                let terminator_pos = read_offset + position;
                let data = &cmd_buf[proc_offset..=terminator_pos];

                let remaining = self.run(data, &mut res_buf).await;

                if !res_buf.is_empty() {
                    adapter.write(&res_buf).await?;
                    adapter.flush().await?;
                    res_buf.clear();
                }

                // Update the offset to the position up to where the data has been processed.
                if !remaining.is_empty() {
                    proc_offset = proc_offset + data.len() - remaining.len();
                    read_offset = terminator_pos + 1;
                } else {
                    proc_offset = terminator_pos + 1;
                    read_offset = proc_offset;
                }
            }

            read_offset = read_end;

            // Ensure `read_from` does not exceed the buffer length
            if read_offset >= cmd_buf.len() {
                #[cfg(feature = "defmt")]
                defmt::warn!("SCPI buffer overflow, resetting buffer");
                read_offset = 0;
                proc_offset = 0;
            }
            // If there is unprocessed data, shift it to the beginning of the buffer.
            else if proc_offset > 0 {
                cmd_buf.copy_within(proc_offset..read_end, 0);
                read_offset -= proc_offset;
                proc_offset = 0;
            }
        }
    }
}
