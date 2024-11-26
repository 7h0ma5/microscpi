use std::rc::Rc;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{parse_macro_input, Attribute, Expr, Ident, ImplItemFn, ItemImpl, Lit, Path, Type};

mod command;
mod tree;

use command::Command;
use tree::Tree;

enum CommandHandler {
    UserFunction(Ident),
    StandardFunction(&'static str),
}

#[derive(Default)]
struct Config {
    pub error_commands: bool,
    pub standard_commands: bool,
}

struct CommandDefinition {
    pub id: usize,
    pub command: Command,
    pub handler: CommandHandler,
    pub args: Vec<Type>,
    pub future: bool,
}

impl CommandDefinition {
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
        }
        else {
            quote! { #fn_call? }
        };

        quote! {
            #command_id => {
                if args.len() != #arg_count {
                    Err(::microscpi::Error::UnexpectedNumberOfParameters)
                }
                else {
                    let result = #fn_call;
                    result.write_response(response).await.unwrap();
                    Ok(())
                }
            }
        }
    }
}

impl CommandDefinition {
    /// Extracts the `scpi` attribute from a function and returns the command
    /// name if present.
    ///
    /// # Arguments
    /// * `attr` - The attribute to parse.
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
                }
                else {
                    Err(meta.error("Invalid SCPI command name"))
                }
            }
            else {
                Ok(())
            }
        })?;

        let args = func
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(arg_type) => Some(*arg_type.ty.clone()),
                syn::FnArg::Receiver(_) => None,
            })
            .collect();

        if let Some(cmd) = &cmd {
            Ok(CommandDefinition {
                id: 0,
                command: Command::try_from(cmd.as_str()).map_err(|_| {
                    syn::Error::new(attr.span(), "Invalid SCPI command syntax")
                })?,
                handler: CommandHandler::UserFunction(func.sig.ident.to_owned()),
                args,
                future: func.sig.asyncness.is_some(),
            })
        }
        else {
            Err(syn::Error::new(attr.span(), "Missing SCPI command path"))
        }
    }
}

/// Extracts all SCPI command functions from an `impl` block.
///
/// # Arguments
/// * `input` - The implementation item of the struct where to extract the SCPI
///   commands from.
///
/// # Returns
/// A vector containing all command definitions.
fn extract_commands(input: &mut ItemImpl) -> Result<Vec<Rc<CommandDefinition>>, syn::Error> {
    let mut commands = Vec::new();
    for item in input.items.iter_mut() {
        if let syn::ImplItem::Fn(ref mut item_fn) = item {
            // Find the first SCPI attribute for this function, parse it and then remove
            // it from the list attributes, so the compiler does not complain about an
            // unknown attribute.
            if let Some(idx) = item_fn
                .attrs
                .iter()
                .position(|attr| attr.path().is_ident("scpi"))
            {
                let attr = item_fn.attrs.remove(idx);
                let mut cmd = CommandDefinition::parse(item_fn, &attr)?;
                cmd.id = commands.len();
                commands.push(Rc::new(cmd).clone());
            }
        }
    }
    Ok(commands)
}

/// Macro attribute to define an SCPI interface.
///
/// This attribute will process an `impl` block and register the SCPI commands
/// defined within it.
#[proc_macro_attribute]
pub fn interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs: Punctuated<Path, Comma> = parse_macro_input!(attr with Punctuated::parse_terminated);
    let mut input_impl = parse_macro_input!(item as ItemImpl);

    let mut config = Config::default();

    for path in attrs {
        if path.is_ident("ErrorCommands") {
            config.error_commands = true;
        }
        else if path.is_ident("StandardCommands") {
            config.standard_commands = true;
        }
    }

    let impl_ty = input_impl.self_ty.clone();

    let mut commands = match extract_commands(&mut input_impl) {
        Ok(commands) => commands,
        Err(err) => { return err.to_compile_error().into(); }
    };

    if config.standard_commands {
        commands.push(Rc::new(CommandDefinition {
            id: commands.len(),
            args: Vec::new(),
            command: Command::try_from("SYSTem:VERSion?").unwrap(),
            handler: CommandHandler::StandardFunction("StandardCommands::system_version"),
            future: false,
        }));
    }

    if config.error_commands {
        commands.push(Rc::new(CommandDefinition {
            id: commands.len(),
            args: Vec::new(),
            command: Command::try_from("SYSTem:ERRor:[NEXT]?").unwrap(),
            handler: CommandHandler::StandardFunction("ErrorCommands::system_error_next"),
            future: false,
        }));

        commands.push(Rc::new(CommandDefinition {
            id: commands.len(),
            args: Vec::new(),
            command: Command::try_from("SYSTem:ERRor:COUNt?").unwrap(),
            handler: CommandHandler::StandardFunction("ErrorCommands::system_error_count"),
            future: false,
        }));
    }

    let mut tree = Tree::new();
    commands
        .iter()
        .try_for_each(|cmd| tree.insert(cmd.clone()))
        .unwrap();

    let command_items: Vec<proc_macro2::TokenStream> =
        commands.iter().map(|cmd| cmd.call()).collect();

    let mut nodes: Vec<proc_macro2::TokenStream> = Vec::new();

    for (node_id, cmd_node) in tree.items {
        let node_name = format_ident!("SCPI_NODE_{}", node_id);

        let entries = cmd_node.children.iter().map(|(name, node_id)| {
            let reference = format_ident!("SCPI_NODE_{}", node_id);
            quote!((#name, &#reference))
        });

        let command = if let Some(command_id) = cmd_node.command.map(|cmd_def| cmd_def.id) {
            quote! { Some(#command_id) }
        }
        else {
            quote! { None }
        };
        let query = if let Some(command_id) = cmd_node.query.map(|cmd_def| cmd_def.id) {
            quote! { Some(#command_id) }
        }
        else {
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
