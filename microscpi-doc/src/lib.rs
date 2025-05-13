use std::{error::Error, path::Path};

use microscpi_common::{Command, CommandPart};
use serde::Serialize;
use syn::{ImplItemFn, Lit, visit::Visit};

/// Represents a serializable SCPI command for documentation export.
#[derive(Debug, Serialize)]
pub struct CommandDocumentation {
    /// The canonical form of the command (long form with all parts).
    pub command: String,
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

    /// Writes the command documentation to a file in the specified format.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let content = self
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        std::fs::write(path, content)
    }
}

impl<'ast> Visit<'ast> for Documentation {
    fn visit_impl_item_fn(&mut self, item_fn: &'ast ImplItemFn) {
        for attr in &item_fn.attrs {
            if attr.path().is_ident("scpi") {
                let mut cmd_name: Option<String> = None;

                let Ok(()) = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("cmd") {
                        if let Lit::Str(name) = meta.value()?.parse()? {
                            cmd_name = Some(name.value());
                            Ok(())
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }) else {
                    continue;
                };

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

                let Some(cmd_name) = cmd_name else {
                    continue;
                };

                let Ok(cmd) = Command::try_from(cmd_name.as_str()) else {
                    continue;
                };

                let (description, attributes) = parse_doc(doc.as_str());

                let doc = CommandDocumentation {
                    command: cmd_name,
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
