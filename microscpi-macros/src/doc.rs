use quote::quote;
use serde::Serialize;

use crate::{CommandDefinition, command::CommandPart};

/// Represents a serializable SCPI command for documentation export.
#[derive(Debug, Serialize)]
pub struct CommandDocumentation {
    /// The canonical form of the command (long form with all parts).
    pub command: String,
    /// The parts of the command.
    pub parts: Vec<CommandPart>,
    /// Whether the command is a query (ends with ?).
    pub is_query: bool,
    /// The types of arguments the command takes.
    pub args: Vec<String>,
    /// The plain text documentation.
    pub description: Option<String>,
    /// Structured data extracted from YAML blocks in the documentation.
    pub attributes: Option<serde_yaml::Value>,
}

/// Parses a doc comment string, extracting any YAML blocks.
pub fn parse_doc(doc_str: &str) -> (Option<String>, Option<serde_yaml::Value>) {
    // Regular expression to remove a single space in front of each line.
    let clean_re = regex::Regex::new(r"(?m)^ ?").unwrap();
    let doc_str = clean_re.replace_all(doc_str, "");

    // Look for YAML code blocks
    let re = regex::Regex::new(r"```yaml\s+([\s\S]+?)\s+```").unwrap();

    let mut attributes = None;
    let description = if let Some(captures) = re.captures(&doc_str) {
        if let Some(yaml) = captures.get(1) {
            match serde_yaml::from_str(yaml.as_str()) {
                Ok(yaml) => {
                    attributes = Some(yaml);
                    // Remove the YAML block from the text
                    re.replace(&doc_str, "").trim().to_string()
                }
                Err(_) => doc_str.to_string(),
            }
        } else {
            doc_str.to_string()
        }
    } else {
        doc_str.to_string()
    };

    (
        if !description.is_empty() {
            Some(description)
        } else {
            None
        },
        attributes,
    )
}

impl From<&CommandDefinition> for CommandDocumentation {
    fn from(cmd: &CommandDefinition) -> Self {
        let args: Vec<String> = cmd.args.iter().map(|ty| quote!(#ty).to_string()).collect();
        let (description, attributes) = cmd
            .doc
            .as_ref()
            .map(String::as_str)
            .map_or((None, None), parse_doc);

        Self {
            command: cmd.command.canonical_path(),
            parts: cmd.command.parts.clone(),
            is_query: cmd.command.is_query(),
            args,
            attributes,
            description,
        }
    }
}

/// A collection of command documents to be serialized.
#[derive(Debug, Serialize)]
pub struct Documentation {
    /// List of all SCPI commands with their documentation.
    pub commands: Vec<CommandDocumentation>,
}

impl Documentation {
    /// Creates a new empty CommandDocumentation.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Adds a command to the documentation.
    pub fn add_command(&mut self, command: CommandDocumentation) {
        self.commands.push(command);
    }

    /// Serializes the command documentation to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Writes the command documentation to a file in the specified format.
    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let content = self
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        std::fs::write(path, content)
    }
}
