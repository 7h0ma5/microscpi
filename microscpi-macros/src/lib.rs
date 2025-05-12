use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use proc_macro::TokenStream;
use proc_macro_error2::{abort, emit_warning, proc_macro_error};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Attribute, Expr, Ident, ImplItemFn, ItemImpl, Lit, Path, Type, parse_macro_input};
use syn::{ExprAssign, ExprLit, ExprPath};

mod command;
#[cfg(feature = "doc")]
mod doc;
mod tree;

use command::Command;
#[cfg(feature = "doc")]
use doc::{CommandDocumentation, Documentation};
use tree::Tree;

/// This crate provides procedural macros for the microscpi library.
///
/// The main macro is `interface`, which processes an implementation block
/// to generate the code needed for an SCPI command interpreter.
///
/// # Command Documentation Export
///
/// This crate also provides functionality to export command documentation during the build phase.
/// To enable this, add the `export` attribute to the `interface` macro:
///
/// ```ignore
/// #[microscpi::interface(StandardCommands, export = "commands.json")]
/// ```
///
/// The exported documentation includes:
/// - Command name in canonical form
/// - Whether the command is a query
/// - Function documentation, including any YAML structured data
/// - Argument types

/// Represents a handler for an SCPI command.
///
/// This can be either:
/// - A user-defined function within the impl block
/// - A standard function from the microscpi library
#[derive(Clone)]
enum CommandHandler {
    /// A user-defined function identified by its identifier
    UserFunction(Ident),
    /// A standard function from the microscpi library, identified by its path
    StandardFunction(&'static str),
}

impl CommandHandler {
    /// Returns the span of the command handler for error reporting.
    fn span(&self) -> proc_macro2::Span {
        match self {
            CommandHandler::UserFunction(ident) => ident.span(),
            CommandHandler::StandardFunction(_) => proc_macro2::Span::call_site(),
        }
    }
}

/// Configuration options for the generated SCPI interface.
#[derive(Default)]
struct Config {
    /// Whether to include standard error commands
    pub error_commands: bool,
    /// Whether to include standard SCPI commands
    pub standard_commands: bool,
    /// Whether to include status commands
    pub status_commands: bool,
    /// Export documentation to this file path (optional)
    pub export_path: Option<String>,
}

/// Defines a complete SCPI command with its handler function and arguments.
#[derive(Clone)]
struct CommandDefinition {
    /// Unique identifier for this command in the command tree
    pub id: Option<usize>,
    /// The parsed SCPI command
    pub command: Command,
    /// The function that handles this command
    pub handler: CommandHandler,
    /// Types of the expected arguments
    pub args: Vec<Type>,
    /// Whether the handler is an async function
    pub future: bool,
    /// Documentation for this command
    pub doc: Option<String>,
}

impl CommandDefinition {
    /// Generates the argument expressions for calling the command handler.
    ///
    /// This creates code to extract and convert each argument from the input
    /// argument list using the appropriate conversion.
    fn args(&self) -> Punctuated<Expr, Comma> {
        self.args
            .iter()
            .enumerate()
            .map(|(id, _arg)| -> Expr {
                syn::parse_quote! {
                    args.get(#id).unwrap().try_into()?
                }
            })
            .collect()
    }

    fn call(&self) -> proc_macro2::TokenStream {
        let command_id = self.id;
        let arg_count = self.args.len();
        let args = self.args();

        let fn_call = match &self.handler {
            CommandHandler::UserFunction(ident) => {
                let func = ident.clone();
                quote! { self.#func(#args) }
            }
            CommandHandler::StandardFunction(path) => {
                let path: Path = syn::parse(path.parse().unwrap()).unwrap();
                quote! { ::microscpi::#path(self, #args) }
            }
        };

        let fn_call = if self.future {
            quote! { #fn_call.await? }
        } else {
            quote! { #fn_call? }
        };

        quote! {
            #command_id => {
                if args.len() != #arg_count {
                    Err(::microscpi::Error::UnexpectedNumberOfParameters)
                }
                else {
                    let result = #fn_call;
                    result.write_response(response)
                }
            }
        }
    }
}

impl CommandDefinition {
    /// Parses a function and its `scpi` attribute to create a CommandDefinition.
    ///
    /// # Arguments
    /// * `func` - The function to parse
    /// * `attr` - The `scpi` attribute to parse
    ///
    /// # Returns
    /// A CommandDefinition if the attribute contains a valid SCPI command.
    ///
    /// # Errors
    /// Returns an error if the attribute contains an invalid SCPI command name.
    fn parse(func: &ImplItemFn, attr: &Attribute) -> syn::Result<CommandDefinition> {
        let mut cmd: Option<String> = None;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("cmd") {
                if let Lit::Str(name) = meta.value()?.parse()? {
                    cmd = Some(name.value());
                    Ok(())
                } else {
                    abort!(
                        meta.path.span(),
                        "SCPI command name must be a string literal"
                    )
                }
            } else {
                Ok(())
            }
        })?;

        // Analyze the function signature to collect argument types
        let args = func
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(arg_type) => Some(*arg_type.ty.clone()),
                syn::FnArg::Receiver(_) => None,
            })
            .collect();

        // Extract documentation comments from the function
        let doc = extract_doc_comments(&func.attrs);

        if let Some(cmd) = cmd {
            Ok(CommandDefinition {
                id: None,
                command: Command::try_from(cmd.as_str())
                    .map_err(|_| syn::Error::new(attr.span(), "Invalid SCPI command syntax"))?,
                handler: CommandHandler::UserFunction(func.sig.ident.to_owned()),
                args,
                future: func.sig.asyncness.is_some(),
                doc,
            })
        } else {
            abort!(
                attr.span(),
                "Missing `cmd` attribute in SCPI command. Expected: #[scpi(cmd = \"COMMAND:NAME\")]"
            )
        }
    }
}

struct CommandSet {
    commands: Vec<Rc<CommandDefinition>>,
}

impl CommandSet {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, mut command: CommandDefinition) {
        command.id = Some(self.commands.len());
        self.commands.push(Rc::new(command));
    }

    /// Exports all commands in this set to a documentation file.
    ///
    /// # Arguments
    /// * `format` - The format to export in (JSON or YAML)
    /// * `output_path` - The file path to write the documentation to
    #[cfg(feature = "doc")]
    pub fn export_documentation(&self, output_path: &str) -> std::io::Result<()> {
        let mut docs = Documentation::new();

        for cmd in self.commands.iter() {
            // Convert argument types to strings
            docs.add_command(CommandDocumentation::from(&**cmd));
        }

        // Write the documentation to the specified file
        docs.write_to_file(output_path)
    }

    /// Extracts all SCPI command functions from an `impl` block.
    ///
    /// This function processes all methods in the provided implementation block,
    /// looking for those with the `#[scpi]` attribute, and converts them to
    /// CommandDefinition objects.
    ///
    /// # Arguments
    /// * `input` - The implementation item of the struct from which to extract the SCPI
    ///   commands.
    ///
    /// # Returns
    /// A vector containing all found command definitions.
    ///
    /// # Errors
    /// Returns an error if any SCPI attribute fails to parse.
    fn extract_commands(&mut self, input: &mut ItemImpl) -> Result<(), syn::Error> {
        for item in input.items.iter_mut() {
            if let syn::ImplItem::Fn(item_fn) = item {
                // Find all SCPI attributes for this function, parse them and then remove
                // them from the list of attributes, so the compiler does not complain about
                // unknown attributes.
                let mut idx = 0;
                while idx < item_fn.attrs.len() {
                    if item_fn.attrs[idx].path().is_ident("scpi") {
                        let attr = item_fn.attrs.remove(idx);
                        self.push(CommandDefinition::parse(item_fn, &attr)?);
                    } else {
                        idx += 1;
                    }
                }
            }
        }
        Ok(())
    }
}

impl AsRef<[Rc<CommandDefinition>]> for CommandSet {
    fn as_ref(&self) -> &[Rc<CommandDefinition>] {
        self.commands.as_ref()
    }
}

/// Macro attribute to define an SCPI interface.
///
/// This attribute processes an `impl` block and registers the SCPI commands
/// defined within it. It generates the code needed to implement the
/// `microscpi::Interface` trait, including the command tree and command handler
/// dispatch logic.
///
/// # Options
///
/// The interface can be configured with additional options:
///
/// ```ignore
/// use microscpi::Interface;
///
/// #[microscpi::interface(StandardCommands, ErrorCommands, export = "commands.json")]
/// impl ExampleInterface {
///     // ...
/// }
/// ```
///
/// Available options:
/// - `StandardCommands`: Add standard SCPI commands (e.g., `SYSTem:VERSion?`)
/// - `ErrorCommands`: Add error-related commands (e.g., `SYSTem:ERRor:[NEXT]?`)
/// - `StatusCommands`: Add status-related commands (e.g., `*OPC`, `*CLS`)
/// - `export = "path/to/file"`: Export command documentation to the specified JSON file
///
/// # Documentation
///
/// Documentation for commands is extracted from the doc comments on the handler functions.
/// You can include structured data in YAML format within your documentation:
///
/// ```ignore
/// /// Returns the device identifier.
/// ///
/// /// ```yaml
/// /// unit: string
/// /// example: "ACME,Widget3000,1234,v1.02"
/// /// ```
/// #[scpi(cmd = "*IDN?")]
/// fn identify(&mut self) -> Result<String, Error> {
///     // ...
/// }
/// ```
///
/// This documentation and structured data will be included in exported command documentation.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs: Punctuated<Expr, Comma> = parse_macro_input!(attr with Punctuated::parse_terminated);
    let mut input_impl = parse_macro_input!(item as ItemImpl);

    let mut config = Config::default();

    // Process configuration options from the attribute parameters
    for attr in attrs {
        match &attr {
            Expr::Path(ExprPath { path, .. }) => {
                if path.is_ident("ErrorCommands") {
                    config.error_commands = true;
                } else if path.is_ident("StandardCommands") {
                    config.standard_commands = true;
                } else if path.is_ident("StatusCommands") {
                    config.status_commands = true;
                } else {
                    abort!(attr.span(), "Unknown SCPI interface option.");
                }
            }
            Expr::Assign(ExprAssign { left, right, .. }) => match &**left {
                Expr::Path(ExprPath { path, .. }) => {
                    if let Ok("export") = path.require_ident().map(Ident::to_string).as_deref() {
                        if cfg!(not(feature = "doc")) {
                            abort!(
                                attr.span(),
                                "Documentation export not available, microscpi-macros compiled without 'doc' feature."
                            )
                        } else if let Expr::Lit(ExprLit {
                            lit: Lit::Str(value),
                            ..
                        }) = &**right
                        {
                            config.export_path = Some(value.value());
                        } else {
                            abort!(right.span(), "Export path must be a string.");
                        }
                    } else {
                        abort!(left.span(), "Unknown SCPI interface option.");
                    }
                }
                _ => {
                    abort!(attr.span(), "Unknown SCPI interface option.");
                }
            },
            _ => {
                abort!(attr.span(), "Unknown SCPI interface option.");
            }
        }
    }

    let mut command_set = CommandSet::new();

    if config.standard_commands {
        command_set.push(CommandDefinition {
            id: None,
            args: Vec::new(),
            command: Command::try_from("SYSTem:VERSion?").unwrap(),
            handler: CommandHandler::StandardFunction("StandardCommands::system_version"),
            future: false,
            doc: Some("Returns the SCPI version supported by the instrument.".to_string()),
        });
    }

    if config.error_commands {
        command_set.push(CommandDefinition {
            id: None,
            args: Vec::new(),
            command: Command::try_from("SYSTem:ERRor:[NEXT]?").unwrap(),
            handler: CommandHandler::StandardFunction("ErrorCommands::system_error_next"),
            future: false,
            doc: Some("Returns the next error from the error queue.".to_string()),
        });

        command_set.push(CommandDefinition {
            id: None,
            args: Vec::new(),
            command: Command::try_from("SYSTem:ERRor:COUNt?").unwrap(),
            handler: CommandHandler::StandardFunction("ErrorCommands::system_error_count"),
            future: false,
            doc: Some("Returns the number of errors in the error queue.".to_string()),
        });
    }

    if config.status_commands {
        command_set.push(CommandDefinition {
            id: None,
            args: Vec::new(),
            command: Command::try_from("*OPC").unwrap(),
            handler: CommandHandler::StandardFunction("StatusCommands::set_operation_complete"),
            future: false,
            doc: Some(
                "Sets the Operation Complete bit in the Standard Event Status Register."
                    .to_string(),
            ),
        });

        command_set.push(CommandDefinition {
            id: None,
            args: Vec::new(),
            command: Command::try_from("*OPC?").unwrap(),
            handler: CommandHandler::StandardFunction("StatusCommands::operation_complete"),
            future: false,
            doc: Some("Returns 1 when all pending operations are complete.".to_string()),
        });

        command_set.push(CommandDefinition {
            id: None,
            args: Vec::new(),
            command: Command::try_from("*CLS").unwrap(),
            handler: CommandHandler::StandardFunction("StatusCommands::clear_event_status"),
            future: false,
            doc: Some("Clears all event registers and error queue.".to_string()),
        });

        command_set.push(CommandDefinition {
            id: None,
            args: Vec::new(),
            command: Command::try_from("*ESE?").unwrap(),
            handler: CommandHandler::StandardFunction("StatusCommands::event_status_enable"),
            future: false,
            doc: Some("Returns the Standard Event Status Enable Register value.".to_string()),
        });

        command_set.push(CommandDefinition {
            id: None,
            args: vec![Type::Verbatim(quote! { u8 })],
            command: Command::try_from("*ESE").unwrap(),
            handler: CommandHandler::StandardFunction("StatusCommands::set_event_status_enable"),
            future: false,
            doc: Some("Sets the Standard Event Status Enable Register.".to_string()),
        });

        command_set.push(CommandDefinition {
            id: None,
            args: Vec::new(),
            command: Command::try_from("*ESR?").unwrap(),
            handler: CommandHandler::StandardFunction("StatusCommands::event_status_register"),
            future: false,
            doc: Some("Returns the Standard Event Status Register value.".to_string()),
        });
    }

    // Extract user-defined SCPI commands from the implementation block
    if let Err(error) = command_set.extract_commands(&mut input_impl) {
        return error.into_compile_error().into();
    }

    // Export command documentation if requested
    #[cfg(feature = "doc")]
    if let Some(export_path) = &config.export_path {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));

        // Determine full path (if relative, use OUT_DIR as base)
        let file_path = if PathBuf::from(export_path).is_relative() {
            out_dir.join(export_path)
        } else {
            PathBuf::from(export_path)
        };

        let file_name = file_path.to_string_lossy().to_string();

        // Create directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Export the documentation
        if let Err(e) = command_set.export_documentation(&file_name) {
            emit_warning!(
                proc_macro2::Span::call_site(),
                "Failed to export SCPI command documentation: {}",
                e
            );
        }
    }

    let mut tree = Tree::new();

    // Insert all commands into the command tree
    for cmd in command_set.as_ref().iter() {
        if let Err(error) = tree.insert(cmd.clone()) {
            abort!(
                cmd.handler.span(),
                "Failed to register SCPI command '{}': {}",
                cmd.command.canonical_path(),
                error
            )
        }
    }

    let command_items: Vec<proc_macro2::TokenStream> =
        command_set.as_ref().iter().map(|cmd| cmd.call()).collect();

    let mut nodes: Vec<proc_macro2::TokenStream> = Vec::new();

    for (node_id, cmd_node) in tree.items {
        let node_name = format_ident!("SCPI_NODE_{}", node_id);

        let entries = cmd_node.children.iter().map(|(name, node_id)| {
            let reference = format_ident!("SCPI_NODE_{}", node_id);
            quote!((#name, &#reference))
        });

        let command = if let Some(command_id) = cmd_node.command.map(|cmd_def| cmd_def.id) {
            quote! { Some(#command_id) }
        } else {
            quote! { None }
        };
        let query = if let Some(command_id) = cmd_node.query.map(|cmd_def| cmd_def.id) {
            quote! { Some(#command_id) }
        } else {
            quote! { None }
        };

        let node_item = quote! {
            static #node_name: ::microscpi::Node = ::microscpi::Node {
                children: &[
                    #(#entries),*
                ],
                command: #command,
                query: #query
            };
        };

        nodes.push(node_item);
    }

    let impl_ty = input_impl.self_ty.clone();

    let mut interface_impl: ItemImpl = syn::parse_quote! {
        impl ::microscpi::Interface for #impl_ty {
            fn root_node(&self) -> &'static ::microscpi::Node {
                &SCPI_NODE_0
            }
            async fn execute_command<'a>(
                &'a mut self,
                command_id: ::microscpi::CommandId,
                args: &[::microscpi::Value<'a>],
                response: &mut impl ::microscpi::Write
            ) -> Result<(), ::microscpi::Error> {
                use ::microscpi::Response;
                match command_id {
                    #(#command_items),*,
                    _ => Err(::microscpi::Error::UndefinedHeader)
                }
           }
        }
    };

    // Copy the generics from the main implementation
    interface_impl.generics = input_impl.generics.clone();

    quote! {
        #(#nodes)*
        #input_impl
        #interface_impl
    }
    .into()
}

/// Helper function to extract doc comments from attributes.
fn extract_doc_comments(attrs: &[syn::Attribute]) -> Option<String> {
    let doc_comments: Vec<String> = attrs
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
        .collect();

    if doc_comments.is_empty() {
        None
    } else {
        Some(doc_comments.join("\n"))
    }
}
