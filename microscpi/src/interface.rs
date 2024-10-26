use crate::parser::{self, CommandCall};
use crate::{tree, CommandId, Error, Value};

pub trait ErrorHandler {
    fn handle_error(&mut self, _error: Error);
}

pub trait Interface: ErrorHandler {
    /// Returns the root node of the SCPI command tree of this interface.
    #[doc(hidden)]
    fn root_node(&self) -> &'static tree::Node;

    /// Executes the command with the specified command id and the supplied
    /// arguments.
    #[doc(hidden)]
    async fn execute_command<'a>(
        &'a mut self, command_id: CommandId, args: &[Value<'a>],
        response: &mut impl core::fmt::Write,
    ) -> Result<(), Error>;

    #[doc(hidden)]
    async fn execute(
        &mut self, call: &CommandCall<'_>, response: &mut impl core::fmt::Write,
    ) -> Result<(), Error> {
        let command = if call.query {
            call.node.query
        }
        else {
            call.node.command
        };

        if let Some(command) = command {
            self.execute_command(command, &call.args, response).await?;

            if call.query {
                response.write_char('\n')?;
            }
        }
        else {
            return Err(Error::UndefinedHeader);
        }

        Ok(())
    }

    /// Parses and executes the commands in the input buffer.
    ///
    /// The result is written to the response buffer. Any remaining input that
    /// was not parsed is returned. If an error occurs, the remaining input
    /// is returned and the error is passed to the error handler.
    async fn run<'a>(
        &mut self, mut input: &'a [u8], response: &mut impl core::fmt::Write,
    ) -> &'a [u8] {
        let node = self.root_node();

        while !input.is_empty() {
            let result = parser::parse(node, input);

            if let Err(error) = result {
                self.handle_error(error.into());
                return input;
            }

            let (i, call) = result.unwrap();

            if let Err(error) = self.execute(&call, response).await {
                self.handle_error(error);
            }

            input = i;
        }
        &[][..]
    }
}
