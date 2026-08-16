// Doc Not Optimize
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Ident, LitStr, Token};

enum DispatcherChainInput {
    /// `dispatcher!("name", EntryType)` — explicit entry type
    Default {
        cmd_attrs: Vec<Attribute>,
        entry_attrs: Vec<Attribute>,
        command_name: syn::LitStr,
        pack: Ident,
    },
    /// `dispatcher!("name")` — entry type derived from the command name
    #[cfg(feature = "extras")]
    Auto {
        cmd_attrs: Vec<Attribute>,
        command_name: syn::LitStr,
    },
}

impl Parse for DispatcherChainInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Collect outer attributes for the hidden dispatcher struct
        let cmd_attrs = input.call(Attribute::parse_outer)?;

        let command_name: LitStr = input.parse()?;

        if input.is_empty() {
            // Abbreviated form: just "command_name"
            #[cfg(feature = "extras")]
            {
                return Ok(Self::Auto {
                    cmd_attrs,
                    command_name,
                });
            }
            #[cfg(not(feature = "extras"))]
            {
                return Err(syn::Error::new(
                    command_name.span(),
                    "expected `, EntryType` after command name",
                ));
            }
        }

        // Explicit form: "command_name", EntryType
        input.parse::<Token![,]>()?;
        let entry_attrs = input.call(Attribute::parse_outer)?;
        let pack: Ident = input.parse()?;

        // The old `"name", CMD => Entry` form was removed in 0.5.0.
        if input.peek(Token![=>]) {
            return Err(syn::Error::new(
                pack.span(),
                "the `dispatcher!(\"name\", CMD => Entry)` form was removed in 0.5.0; \
                 use `dispatcher!(\"name\", Entry)` — the dispatcher struct is generated internally",
            ));
        }

        Ok(Self::Default {
            cmd_attrs,
            entry_attrs,
            command_name,
            pack,
        })
    }
}

pub(crate) fn dispatcher(input: TokenStream) -> TokenStream {
    let dispatcher_input = syn::parse_macro_input!(input as DispatcherChainInput);

    #[cfg(not(feature = "extras"))]
    let (command_name, pack, cmd_attrs, entry_attrs) = match dispatcher_input {
        DispatcherChainInput::Default {
            cmd_attrs,
            entry_attrs,
            command_name,
            pack,
        } => (command_name, pack, cmd_attrs, entry_attrs),
    };

    #[cfg(feature = "extras")]
    let (command_name, pack, cmd_attrs, entry_attrs) = match dispatcher_input {
        DispatcherChainInput::Default {
            cmd_attrs,
            entry_attrs,
            command_name,
            pack,
        } => (command_name, pack, cmd_attrs, entry_attrs),
        DispatcherChainInput::Auto {
            cmd_attrs,
            command_name,
        } => {
            let command_name_str = command_name.value();
            let pascal = just_fmt::pascal_case!(&command_name_str);
            let pack = Ident::new(&format!("Entry{pascal}"), command_name.span());
            (command_name, pack, cmd_attrs, Vec::new())
        }
    };

    let command_name_str = command_name.value();
    let hidden_dispatcher = Ident::new(
        &format!("__Dispatcher{}", just_fmt::pascal_case!(&command_name_str)),
        command_name.span(),
    );

    let comp_entry = get_comp_entry(&pack);

    let compile_time_registration =
        get_compile_time_registration(&command_name_str, &hidden_dispatcher, &pack);

    let program_type = crate::default_program_path();

    let expanded = quote! {
        #[derive(::mingling::Grouped, ::mingling::Wrap, Default)]
        #(#entry_attrs)*
        pub struct #pack(pub ::std::vec::Vec<::std::string::String>);

        #(#cmd_attrs)*
        #[doc(hidden)]
        #[derive(Debug, Default)]
        #[allow(nonstandard_style)]
        pub struct #hidden_dispatcher;

        #compile_time_registration

        impl From<#pack> for crate::Entry {
            fn from(value: #pack) -> Self {
                crate::Entry(value.0)
            }
        }

        #comp_entry

        impl ::mingling::Dispatcher<#program_type> for #hidden_dispatcher {
            fn begin(&self, args: Vec<String>) -> ::mingling::ChainProcess<#program_type> {
                use ::mingling::Grouped;
                ::mingling::Routable::to_chain(#pack(args))
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
                self.0.clone()
            }
        }
    };
    comp_entry
}

#[cfg(not(feature = "comp"))]
fn get_comp_entry(_entry_name: &Ident) -> TokenStream2 {
    quote! {}
}

/// Registers the dispatcher at compile time (collects its node into the
/// global `COMPILE_TIME_DISPATCHERS` registry and emits the
/// `__internal_dispatcher_*` static), regardless of the `dispatch_tree`
/// feature. The feature only selects which matching strategy
/// (trie vs. linear list) is generated later by `gen_program!`.
fn get_compile_time_registration(
    command_name_str: &str,
    dispatcher_struct: &Ident,
    entry_name: &Ident,
) -> TokenStream2 {
    let node_name_lit = syn::LitStr::new(command_name_str, proc_macro2::Span::call_site());
    quote! {
        ::mingling::macros::register_dispatcher!(#node_name_lit, #dispatcher_struct, #entry_name);
    }
}
