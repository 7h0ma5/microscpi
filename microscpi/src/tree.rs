/// Reference identifier of a command or query
///
/// Due to current limitations with async function pointers, the references to
/// the command handler functions are stored as integers.
pub type CommandId = usize;

/// SCPI Command Tree
///
/// This struct represents a node in the SCPI command tree. Each node can hold a
/// reference to a command, a query or both. The tree contains all possible
/// command paths including short, long and optional path components.
pub struct ScpiTreeNode {
    pub children: &'static [(&'static str, &'static ScpiTreeNode)],
    pub command: Option<CommandId>,
    pub query: Option<CommandId>,
}

impl ScpiTreeNode {
    /// Searches for a path component in this node.
    ///
    /// The search is *case-insensitive*.
    ///
    /// # Returns
    /// The ScpiTreeNode with the specified name if found.
    pub fn child(&self, name: &str) -> Option<&'static ScpiTreeNode> {
        for child in self.children {
            if child.0.eq_ignore_ascii_case(name) {
                return Some(child.1);
            }
        }
        None
    }
}

impl PartialEq for &'static ScpiTreeNode {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self, other)
    }
}
