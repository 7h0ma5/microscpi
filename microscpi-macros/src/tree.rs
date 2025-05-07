use std::collections::HashMap;
use std::rc::Rc;

use crate::CommandDefinition;

/// Error types for the SCPI command tree operations.

#[derive(Debug)]
pub enum Error {
    /// Error when attempting to register a command that already exists at the same path
    CommandExists { path: String },
    /// Error when attempting to register a query that already exists at the same path
    QueryExists { path: String },
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CommandExists { path } => write!(f, "Command {} is already defined", path),
            Error::QueryExists { path } => write!(f, "Query {} is already defined", path),
        }
    }
}

/// Identifier for nodes in the command tree
type NodeId = usize;

/// Represents a tree of SCPI commands.
/// 
/// The tree is used to efficiently look up command handlers based on the parsed command string.
/// It maps each component of a command path to the appropriate handler function.
pub struct Tree {
    /// All nodes in the tree, indexed by their NodeId
    pub items: HashMap<NodeId, TreeNode>,
}

/// A node in the SCPI command tree.
/// 
/// Each node can have:
/// - Children nodes representing the next part of a command
/// - An optional command handler for when this node is a command endpoint
/// - An optional query handler for when this node is a query endpoint
#[derive(Default)]
pub struct TreeNode {
    /// Maps command part names to child node IDs
    pub children: HashMap<String, NodeId>,
    /// Command handler for this node (if this node is a command endpoint)
    pub command: Option<Rc<CommandDefinition>>,
    /// Query handler for this node (if this node is a query endpoint)
    pub query: Option<Rc<CommandDefinition>>,
}

impl Tree {
    /// Creates a new empty command tree with just a root node.
    pub fn new() -> Tree {
        Tree {
            items: HashMap::from([(0, TreeNode::default())]),
        }
    }

    /// Inserts a command definition into the tree.
    ///
    /// This will insert the command under all valid paths generated from the command definition.
    /// For example, a command like "[STATus]:EVENt?" will be inserted under paths:
    /// - "STATUS:EVENT?"
    /// - "STAT:EVENT?"
    /// - "EVENT?" (since STATUS is optional)
    /// 
    /// # Arguments
    /// * `cmd` - The command definition to insert
    ///
    /// # Returns
    /// * `Ok(())` - If the command was successfully inserted
    /// * `Err(Error)` - If the command conflicts with an existing one
    pub fn insert(&mut self, cmd: Rc<CommandDefinition>) -> Result<(), Error> {
        cmd.command
            .paths()
            .iter()
            .try_for_each(|path| self.insert_at(0, path, cmd.clone()))
    }

    /// Inserts a command definition at a specific path in the tree.
    ///
    /// # Arguments
    /// * `id` - The ID of the current node
    /// * `path` - The remaining parts of the command path to insert
    /// * `cmd` - The command definition to insert
    ///
    /// # Returns
    /// * `Ok(())` - If the command was successfully inserted
    /// * `Err(Error)` - If the command conflicts with an existing one
    fn insert_at(
        &mut self,
        id: NodeId,
        path: &[String],
        cmd: Rc<CommandDefinition>,
    ) -> Result<(), Error> {
        use std::collections::hash_map::Entry;

        if let Some(part) = path.first() {
            // We're still traversing the path
            let next_id = self.items.len();

            // Get or create the child node for this path part
            let entry = self
                .items
                .get_mut(&id)
                .expect("Node ID must exist in the tree")
                .children
                .entry(part.clone());

            let node_id = match entry {
                Entry::Occupied(o) => *o.get(),
                Entry::Vacant(v) => {
                    v.insert(next_id);
                    next_id
                }
            };

            // A new node has to be inserted if we just created a new ID
            if node_id == next_id {
                self.items.insert(next_id, TreeNode::default());
            }

            // Continue recursively with the rest of the path
            self.insert_at(node_id, &path[1..], cmd)?;
        } else {
            // We've reached the end of the path, register the command here
            let node = self.items.get_mut(&id).expect("Node ID must exist in the tree");
            
            if cmd.command.is_query() {
                // This is a query command (ends with '?')
                if let Some(existing) = &node.query {
                    return Err(Error::QueryExists {
                        path: existing.command.canonical_path(),
                    });
                } else {
                    node.query = Some(cmd)
                }
            } else {
                // This is a regular command (no '?')
                if let Some(existing) = &node.command {
                    return Err(Error::CommandExists {
                        path: existing.command.canonical_path(),
                    });
                } else {
                    node.command = Some(cmd)
                }
            }
        }
        Ok(())
    }
}
