use crate::{CommandId, Context, Error, Node, Value};

#[doc(hidden)]
pub trait Interface {
    /// Returns the root node of the SCPI command tree of this interface.
    fn root_node(&self) -> &'static Node;

    /// Executes the command with the specified command id and the supplied
    /// arguments.
    async fn execute_command<'a>(
        &'a mut self, context: &mut Context, command_id: CommandId, args: &[Value<'a>],
    ) -> Result<Value<'a>, Error>;
}