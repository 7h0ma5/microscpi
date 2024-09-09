use std::collections::HashMap;
use std::rc::Rc;

use crate::CommandDefinition;

#[derive(Debug)]
pub enum Error {
    CommandExists,
    QueryExists,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::CommandExists => write!(f, "Command already exists"),
            Error::QueryExists => write!(f, "Query already exists"),
        }
    }
}

type NodeId = usize;

pub struct Tree {
    pub items: HashMap<NodeId, TreeNode>,
}

#[derive(Default)]
pub struct TreeNode {
    pub children: HashMap<String, NodeId>,
    pub command: Option<Rc<CommandDefinition>>,
    pub query: Option<Rc<CommandDefinition>>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            items: HashMap::from([(0, TreeNode::default())]),
        }
    }

    pub fn insert(&mut self, cmd: Rc<CommandDefinition>) -> Result<(), Error> {
        cmd.command
            .paths()
            .iter()
            .try_for_each(|path| self.insert_at(0, path, cmd.clone()))
    }

    fn insert_at(
        &mut self, id: NodeId, path: &[String], cmd: Rc<CommandDefinition>,
    ) -> Result<(), Error> {
        use std::collections::hash_map::Entry;

        if let Some(part) = path.first() {
            let next_id = self.items.len();

            let entry = self
                .items
                .get_mut(&id)
                .unwrap()
                .children
                .entry(part.clone());

            let node_id = match entry {
                Entry::Occupied(o) => *o.get(),
                Entry::Vacant(v) => {
                    v.insert(next_id);
                    next_id
                }
            };

            // A new node has to be inserted.
            if node_id == next_id {
                self.items.insert(next_id, TreeNode::default());
            }

            self.insert_at(node_id, &path[1..], cmd)?;
        }
        else {
            let node = self.items.get_mut(&id).unwrap();
            if cmd.command.is_query() {
                if let Some(_existing) = &node.query {
                    return Err(Error::QueryExists);
                }
                else {
                    node.query = Some(cmd)
                }
            }
            else if let Some(_existing) = &node.command {
                return Err(Error::CommandExists);
            }
            else {
                node.command = Some(cmd)
            }
        }
        Ok(())
    }
}
