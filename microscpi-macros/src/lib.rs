use std::rc::Rc;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, Attribute, Expr, Ident, ItemImpl, Lit, Type};

mod command;
mod tree;

use command::Command;
use tree::Tree;

struct CommandDefinition {
    pub id: usize,
    pub command: Command,
    pub fn_ident: Ident,
    pub args: Vec<Type>,
}

/// Extracts the `scpi` attribute from a function and returns the command name
/// if present.
///
/// # Arguments
/// * `attr` - The attribute to parse.
///
/// # Errors
/// Returns an error if the attribute contains an invalid SCPI command name.
fn extract_name_attribute(attr: &Attribute) -> Result<Option<String>, syn::Error> {
    let mut cmd: Option<String> = None;

    if attr.path().is_ident("scpi") {
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
        Ok(cmd)
    }
    else {
        Ok(None)
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
fn extract_commands(input: &mut ItemImpl) -> Vec<Rc<CommandDefinition>> {
    let mut commands = Vec::new();
    for item in input.items.iter_mut() {
        if let syn::ImplItem::Fn(ref mut item_fn) = item {
            let mut name_attr_value = None;

            // Retain only non-SCPI attributes.
            item_fn.attrs.retain(|attr| {
                if let Ok(Some(name_value)) = extract_name_attribute(attr) {
                    name_attr_value = Some(name_value);
                    false // Remove this attribute
                }
                else {
                    true // Keep this attribute
                }
            });

            if let Some(name_value) = name_attr_value {
                let cmd = Command::try_from(name_value.as_ref()).unwrap();
                let args = item_fn
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::FnArg::Typed(arg_type) => Some(*arg_type.ty.clone()),
                        syn::FnArg::Receiver(_) => None,
                    })
                    .collect();

                let cmd_def = Rc::new(CommandDefinition {
                    id: commands.len(),
                    command: cmd.clone(),
                    fn_ident: item_fn.sig.ident.to_owned(),
                    args,
                });
                commands.push(cmd_def.clone());
            }
        }
    }
    commands
}

/// Macro attribute to define an SCPI interface.
///
/// This attribute will process an `impl` block and register the SCPI commands
/// defined within it.
#[proc_macro_attribute]
pub fn interface(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    let impl_ty = input.self_ty.clone();

    let commands = extract_commands(&mut input);

    let mut tree = Tree::new();
    commands
        .iter()
        .try_for_each(|cmd| tree.insert(cmd.clone()))
        .unwrap();

    let mut command_items: Vec<proc_macro2::TokenStream> = Vec::new();

    for command in commands {
        let command_id = command.id;
        let func = command.fn_ident.clone();
        let arg_count = command.args.len();

        let args = command
            .args
            .iter()
            .enumerate()
            .map(|(id, _arg)| -> Expr {
                syn::parse_quote! {
                    args.get(#id).unwrap().try_into()?
                }
            })
            .collect::<Punctuated<Expr, Comma>>();

        command_items.push(quote! {
            #command_id => {
                if args.len() != #arg_count {
                    Err(microscpi::Error::UnexpectedNumberOfParameters)
                }
                else {
                    self.#func(#args).await.map(Into::<microscpi::Value<'i>>::into)
                }
            }
        });
    }

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
            static #node_name: microscpi::ScpiTreeNode = microscpi::ScpiTreeNode {
                children: &[
                    #(#entries),*
                ],
                command: #command,
                query: #query
            };
        };

        nodes.push(node_item);
    }

    quote! {
        #(#nodes)*
        #input
        impl<'i> microscpi::Interface<'i> for #impl_ty {
            fn root_node() -> &'static microscpi::ScpiTreeNode {
                &SCPI_NODE_0
            }
            async fn run_command(
                &'i mut self,
                command_id: microscpi::CommandId,
                args: &[microscpi::Value<'i>]
            ) -> Result<microscpi::Value<'i>, microscpi::Error> {
                match command_id {
                    #(#command_items),*,
                    _ => Err(microscpi::Error::UndefinedHeader)
                }
           }
        }
    }
    .into()
}
