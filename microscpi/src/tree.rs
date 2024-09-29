use crate::CommandId;

/// SCPI Command Tree
///
/// This struct represents a node in the SCPI command tree. Each node can hold a
/// reference to a command, a query or both. The tree contains all possible
/// command paths including short, long and optional path components.
#[derive(Debug)]
pub struct Node {
    pub children: &'static [(&'static str, &'static Node)],
    pub command: Option<CommandId>,
    pub query: Option<CommandId>,
}

impl Node {
    /// Searches for a path component in this node.
    ///
    /// The search is *case-insensitive*.
    ///
    /// # Returns
    /// The [Node] with the specified name if found.
    pub fn child(&self, name: &str) -> Option<&'static Node> {
        for child in self.children {
            if child.0.eq_ignore_ascii_case(name) {
                return Some(child.1);
            }
        }
        None
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static ROOT_NODE: Node = Node {
        children: &[("LEAF", &LEAF_NODE)],
        command: None,
        query: None,
    };

    static LEAF_NODE: Node = Node {
        children: &[],
        command: Some(1),
        query: None,
    };

    #[test]
    fn test_child_found() {
        assert_eq!(ROOT_NODE.child("LEAF"), Some(&LEAF_NODE));
    }

    #[test]
    fn test_child_not_found() {
        assert_eq!(ROOT_NODE.child("NON_EXISTENT"), None);
    }

    #[test]
    fn test_child_case_insensitive() {
        assert_eq!(ROOT_NODE.child("leaf"), Some(&LEAF_NODE));
        assert_eq!(ROOT_NODE.child("Leaf"), Some(&LEAF_NODE));
        assert_eq!(ROOT_NODE.child("lEaF"), Some(&LEAF_NODE));
    }

    #[test]
    fn test_node_equality() {
        assert_eq!(&LEAF_NODE, &LEAF_NODE);
        assert_ne!(&ROOT_NODE, &LEAF_NODE);
    }
}
