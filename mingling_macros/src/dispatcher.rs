#[cfg(feature = "dispatch_tree")]
use just_fmt::snake_case;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Ident, LitStr, Result as SynResult, Token};

#[cfg(feature = "dispatch_tree")]
use crate::COMPILE_TIME_DISPATCHERS;

enum DispatcherChainInput {
    Default {
        cmd_attrs: Vec<Attribute>,
        entry_attrs: Vec<Attribute>,
        command_name: syn::LitStr,
        command_struct: Ident,
        pack: Ident,
    },
    #[cfg(feature = "extra_macros")]
    Auto {
        cmd_attrs: Vec<Attribute>,
        command_name: syn::LitStr,
    },
}

impl Parse for DispatcherChainInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Collect outer attributes for the CMD struct
        let cmd_attrs = input.call(Attribute::parse_outer)?;

        if input.peek(syn::LitStr) {
            // Parse the command name string first
            let command_name: LitStr = input.parse()?;

            // Check if this is the abbreviated form: just "command_name" without ", CMD => Entry"
            if input.is_empty() {
                #[cfg(feature = "extra_macros")]
                {
                    return Ok(DispatcherChainInput::Auto {
                        cmd_attrs,
                        command_name,
                    });
                }
                #[cfg(not(feature = "extra_macros"))]
                {
                    return Err(syn::Error::new(
                        command_name.span(),
                        "expected `, CommandStruct => EntryStruct` after command name",
                    ));
                }
            }

            // Default format: "command_name", CommandStruct => ChainStruct
            input.parse::<Token![,]>()?;
            let command_struct = input.parse()?;
            input.parse::<Token![=>]>()?;
            let entry_attrs = input.call(Attribute::parse_outer)?;
            let pack = input.parse()?;

            Ok(DispatcherChainInput::Default {
                cmd_attrs,
                entry_attrs,
                command_name,
                command_struct,
                pack,
            })
        } else {
            Err(input.lookahead1().error())
        }
    }
}

// NOTICE: The token stream generation patterns in `dispatcher_chain` and `dispatcher_render`
// are nearly identical and could benefit from refactoring into common helper functions.

#[allow(clippy::too_many_lines)]
pub fn dispatcher(input: TokenStream) -> TokenStream {
    // Parse the input
    let dispatcher_input = syn::parse_macro_input!(input as DispatcherChainInput);

    #[cfg(not(feature = "extra_macros"))]
    let (command_name, command_struct, pack, cmd_attrs, entry_attrs) = match dispatcher_input {
        DispatcherChainInput::Default {
            cmd_attrs,
            entry_attrs,
            command_name,
            command_struct,
            pack,
        } => (command_name, command_struct, pack, cmd_attrs, entry_attrs),
    };

    #[cfg(feature = "extra_macros")]
    let (command_name, command_struct, pack, cmd_attrs, entry_attrs) = match dispatcher_input {
        DispatcherChainInput::Default {
            cmd_attrs,
            entry_attrs,
            command_name,
            command_struct,
            pack,
        } => (command_name, command_struct, pack, cmd_attrs, entry_attrs),
        DispatcherChainInput::Auto {
            cmd_attrs,
            command_name,
        } => {
            let command_name_str = command_name.value();
            let pascal = dotted_to_pascal_case(&command_name_str);
            let command_struct = Ident::new(&format!("CMD{pascal}"), command_name.span());
            let pack = Ident::new(&format!("Entry{pascal}"), command_name.span());
            (command_name, command_struct, pack, cmd_attrs, Vec::new())
        }
    };

    let command_name_str = command_name.value();

    let comp_entry = get_comp_entry(&pack);

    let dispatch_tree_entry = get_dispatch_tree_entry(&command_name_str, &command_struct, &pack);

    let program_type = crate::default_program_path();

    let expanded = quote! {
        #(#cmd_attrs)*
        #[derive(Debug, Default)]
        pub struct #command_struct;

        ::mingling::macros::pack!(#(#entry_attrs)* #pack = Vec<String>);

        #comp_entry
        #dispatch_tree_entry

        impl ::mingling::Dispatcher<#program_type> for #command_struct {
            fn node(&self) -> ::mingling::Node {
                ::mingling::macros::node!(#command_name_str)
            }
            fn begin(&self, args: Vec<String>) -> ::mingling::ChainProcess<#program_type> {
                use ::mingling::Groupped;
                #pack::new(args).to_chain()
            }
            fn clone_dispatcher(&self) -> Box<dyn ::mingling::Dispatcher<#program_type>> {
                Box::new(#command_struct)
            }
        }
    };

    expanded.into()
}

#[cfg(feature = "comp")]
fn get_comp_entry(entry_name: &Ident) -> TokenStream2 {
    let comp_entry = quote! {
        impl ::mingling::CompletionEntry for #entry_name {
            fn get_input(self) -> Vec<String> {
                self.inner.clone()
            }
        }
    };
    comp_entry
}

#[cfg(not(feature = "comp"))]
fn get_comp_entry(_entry_name: &Ident) -> TokenStream2 {
    quote! {}
}

#[cfg(feature = "dispatch_tree")]
fn get_dispatch_tree_entry(
    command_name_str: &str,
    command_struct: &Ident,
    entry_name: &Ident,
) -> TokenStream2 {
    let node_name_lit = syn::LitStr::new(command_name_str, proc_macro2::Span::call_site());
    quote! {
        ::mingling::macros::register_dispatcher!(#node_name_lit, #command_struct, #entry_name);
    }
}

#[cfg(not(feature = "dispatch_tree"))]
fn get_dispatch_tree_entry(
    _command_name_str: &str,
    _command_struct: &Ident,
    _entry_name: &Ident,
) -> TokenStream2 {
    quote! {}
}

#[cfg(feature = "dispatch_tree")]
/// Input format: ("node.name", DispatcherType, EntryName)
struct RegisterDispatcherInput {
    node_name: syn::LitStr,
    dispatcher_type: Ident,
    entry_name: Ident,
}

#[cfg(feature = "dispatch_tree")]
impl Parse for RegisterDispatcherInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let node_name = input.parse()?;
        input.parse::<Token![,]>()?;
        let dispatcher_type = input.parse()?;
        input.parse::<Token![,]>()?;
        let entry_name = input.parse()?;
        Ok(RegisterDispatcherInput {
            node_name,
            dispatcher_type,
            entry_name,
        })
    }
}

#[cfg(feature = "dispatch_tree")]
pub fn register_dispatcher(input: TokenStream) -> TokenStream {
    let RegisterDispatcherInput {
        node_name,
        dispatcher_type,
        entry_name,
    } = syn::parse_macro_input!(input as RegisterDispatcherInput);

    let node_name_str = node_name.value();
    let static_name = format!("__internal_dispatcher_{}", snake_case!(node_name_str.clone()));
    let static_ident = Ident::new(&static_name, proc_macro2::Span::call_site());

    // Register node info in the global collection at compile time
    // Format: "node.name:DispatcherType:EntryName"
    crate::get_global_set(&COMPILE_TIME_DISPATCHERS)
        .lock()
        .unwrap()
        .insert(format!(
            "{}:{}:{}",
            node_name_str, dispatcher_type, entry_name
        ));

    let expanded = quote! {
        #[doc(hidden)]
        #[allow(nonstandard_style)]
        pub static #static_ident: #dispatcher_type = #dispatcher_type;
    };

    expanded.into()
}

#[cfg(not(feature = "dispatch_tree"))]
pub fn register_dispatcher(_input: TokenStream) -> TokenStream {
    quote! {}.into()
}

/// Converts a dotted command name (e.g. "remote.add") to `PascalCase` (e.g. "`RemoteAdd`").
///
/// Each segment is split by `.`, the first character of each segment is uppercased,
/// and the segments are joined. This is used by the abbreviated `dispatcher!` syntax
/// (when `Command => Entry` is omitted) to auto-derive struct names.
#[cfg(feature = "extra_macros")]
fn dotted_to_pascal_case(s: &str) -> String {
    s.split('.')
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}
