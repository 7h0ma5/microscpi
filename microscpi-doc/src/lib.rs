use std::{error::Error, path::Path};

use microscpi_common::{Command, CommandPart};
use serde::Serialize;
use syn::{ImplItemFn, Lit, visit::Visit};

/// Represents a serializable SCPI command for documentation export.
#[derive(Debug, Serialize)]
pub struct CommandDocumentation {
    /// The full canonical path of the command.
    pub path: String,
    /// The name of the command as defined in the command definition.
    pub name: String,
    /// The parts of the command.
    pub parts: Vec<CommandPart>,
    /// Whether the command is a query (ends with ?).
    pub is_query: bool,
    /// The plain text documentation.
    pub description: Option<String>,
    /// Structured data extracted from YAML blocks in the documentation.
    pub attributes: Option<serde_yaml::Value>,
}

/// Parses a doc comment string, extracting any YAML blocks.
fn parse_doc(doc_str: &str) -> (Option<String>, Option<serde_yaml::Value>) {
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

/// A collection of command documents to be serialized.
#[derive(Debug, Serialize)]
pub struct Documentation {
    /// List of all SCPI commands with their documentation.
    pub commands: Vec<CommandDocumentation>,
}

impl Documentation {
    /// Creates a new empty Documentation.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Parses a rust file containing SCPI command definitions and adds its documentation to the collection.
    pub fn parse_file(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let content = std::fs::read_to_string(path)?;
        let file = syn::parse_file(content.as_str())?;
        self.visit_file(&file);
        Ok(())
    }

    /// Adds a command to the documentation.
    pub fn add_command(&mut self, command: CommandDocumentation) {
        self.commands.push(command);
    }

    /// Serializes the command documentation to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Writes the command documentation to a JSON file.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let content = self
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        std::fs::write(path, content)
    }
}

/// Try to get the SCPI command name from a attribute.
fn get_command_name(attr: &syn::Attribute) -> Option<String> {
    let mut command_name = None;

    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("cmd") {
            if let Lit::Str(name) = meta.value()?.parse()? {
                command_name = Some(name.value());
                Ok(())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    });

    command_name
}

/// Try to get the documentation string from an attribute.
fn get_command_doc(item_fn: &ImplItemFn) -> String {
    let doc: String = item_fn
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| {
            if let Ok(syn::Meta::NameValue(meta)) = attr.meta.clone().try_into() {
                if let syn::Expr::Lit(expr_lit) = meta.value {
                    if let syn::Lit::Str(lit_str) = expr_lit.lit {
                        return Some(lit_str.value());
                    }
                }
            }
            None
        })
        .collect::<Vec<String>>()
        .join("\n");
    doc
}

impl<'ast> Visit<'ast> for Documentation {
    fn visit_impl_item_fn(&mut self, item_fn: &'ast ImplItemFn) {
        for attr in &item_fn.attrs {
            if attr.path().is_ident("scpi") {
                let Some(cmd_name) = get_command_name(attr) else {
                    continue;
                };

                let doc = get_command_doc(item_fn);

                let Ok(cmd) = Command::try_from(cmd_name.as_str()) else {
                    continue;
                };

                let (description, attributes) = parse_doc(doc.as_str());

                let doc = CommandDocumentation {
                    path: cmd.canonical_path(),
                    name: cmd_name,
                    is_query: cmd.is_query(),
                    parts: cmd.parts,
                    description,
                    attributes,
                };

                self.add_command(doc);
            }
        }
    }
}
