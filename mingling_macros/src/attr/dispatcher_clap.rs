use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemStruct, LitBool, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Parsed key-value options after the first positional arguments
struct ClapOptions {
    /// `error = ErrorStruct`
    error_struct: Option<Ident>,
    /// `help = true` (bool only)
    help_enabled: bool,
}

impl Parse for ClapOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut error_struct = None;
        let mut help_enabled = false;

        while !input.is_empty() {
            // Parse leading comma
            input.parse::<Token![,]>()?;

            // Allow trailing comma
            if input.is_empty() {
                break;
            }

            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "error" {
                let value: Ident = input.parse()?;
                if error_struct.is_some() {
                    return Err(syn::Error::new(key.span(), "duplicate `error` key"));
                }
                error_struct = Some(value);
            } else if key == "help" {
                let value: LitBool = input.parse()?;
                if value.value() == false {
                    // help = false is allowed but does nothing
                    help_enabled = false;
                } else {
                    help_enabled = true;
                }
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "unknown key, expected `error` or `help`",
                ));
            }
        }

        Ok(ClapOptions {
            error_struct,
            help_enabled,
        })
    }
}

/// Input for the dispatcher_clap attribute
struct DispatcherClapInput {
    /// `("cmd", Disp, ...)`
    command_name: LitStr,
    dispatcher_struct: Ident,
    options: ClapOptions,
}

impl Parse for DispatcherClapInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Format: "cmd", Disp, ...
        let command_name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let dispatcher_struct: Ident = input.parse()?;

        let options = if input.is_empty() {
            ClapOptions {
                error_struct: None,
                help_enabled: false,
            }
        } else {
            input.parse::<ClapOptions>()?
        };

        Ok(DispatcherClapInput {
            command_name,
            dispatcher_struct,
            options,
        })
    }
}

#[cfg(feature = "clap")]
pub(crate) fn dispatcher_clap_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_input = parse_macro_input!(attr as DispatcherClapInput);
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = &input_struct.ident;

    let program_path = crate::default_program_path();

    let command_name_str = attr_input.command_name.value();
    let dispatcher_struct = &attr_input.dispatcher_struct;
    let options = &attr_input.options;

    // Generate the `begin` method body
    let begin_body = if let Some(ref error_struct) = options.error_struct {
        quote! {
            if ::mingling::this::<#program_path>().user_context.help {
                return ::mingling::Routable::<#program_path>::to_chain(#struct_name::default());
            }
            match <#struct_name as ::clap::Parser>::try_parse_from(clap_args) {
                Ok(parsed) => ::mingling::Routable::<#program_path>::to_chain(parsed),
                Err(e) => {
                    return ::mingling::Routable::<#program_path>::to_render(#error_struct::new(format!("{}", e.render().ansi())))
                },
            }
        }
    } else {
        quote! {
            if ::mingling::this::<#program_path>().user_context.help {
                return ::mingling::Routable::<#program_path>::to_chain(#struct_name::default());
            }
            let parsed = <#struct_name as ::clap::Parser>::try_parse_from(clap_args)
                .unwrap_or_else(|e| e.exit());
            ::mingling::Routable::<#program_path>::to_chain(parsed)
        }
    };

    // Generate the error pack type
    let error_pack = options.error_struct.as_ref().map(|error_struct| {
        quote! {
            ::mingling::macros::pack!(#error_struct = String);
        }
    });

    // Generate the #[help] block if help = true
    let help_gen = if options.help_enabled {
        let dispatcher_name_str = dispatcher_struct.to_string();
        let help_fn_name_str = format!("__{}_help", just_fmt::snake_case!(&dispatcher_name_str));
        let help_fn_name = Ident::new(&help_fn_name_str, proc_macro2::Span::call_site());

        Some(quote! {
            #[allow(non_snake_case)]
            #[::mingling::macros::help]
            pub(crate) fn #help_fn_name(_prev: #struct_name) -> ::mingling::RenderResult {
                use std::io::Write;
                use clap::ColorChoice;

                let this = ::mingling::this::<#program_path>();
                match this.stdout_setting.clap_help_print_behaviour {
                    ::mingling::ClapHelpPrintBehaviour::WriteToRenderResult => {
                        let mut cmd = <#struct_name as ::clap::CommandFactory>::command()
                            .color(ColorChoice::Always);
                        let styled = cmd.render_help();
                        let mut result = ::mingling::RenderResult::new();
                        let _ = write!(result, "{}", styled.ansi());
                        result
                    }
                    ::mingling::ClapHelpPrintBehaviour::PrintDirectly => {
                        let mut command = <#struct_name as ::clap::CommandFactory>::command();
                        command.print_help().unwrap();
                        ::mingling::RenderResult::new()
                    }
                }
            }
        })
    } else {
        None
    };

    let dispatch_tree_entry =
        get_dispatch_tree_entry(&command_name_str, dispatcher_struct, &struct_name);

    let expanded = quote! {
        // Keep the original struct definition
        #input_struct

        // Generate the error wrapper type via pack!
        #error_pack

        // Generate the help block if enabled
        #help_gen

        // Dispatch tree registration (if feature enabled)
        #dispatch_tree_entry

        // Generate the dispatcher struct
        #[doc(hidden)]
        #[derive(Default)]
        pub(crate) struct #dispatcher_struct;

        impl ::mingling::Dispatcher<#program_path> for #dispatcher_struct {
            fn node(&self) -> ::mingling::Node {
                ::mingling::macros::node!(#command_name_str)
            }

            fn begin(
                &self,
                args: Vec<String>,
            ) -> ::mingling::ChainProcess<#program_path> {
                // Prepend a dummy program name for clap's parse_from
                let clap_args = std::iter::once(String::new())
                    .chain(args)
                    .collect::<Vec<_>>();

                #begin_body
            }

            fn clone_dispatcher(
                &self,
            ) -> Box<dyn ::mingling::Dispatcher<#program_path>> {
                Box::new(#dispatcher_struct)
            }
        }
    };

    expanded.into()
}

#[cfg(feature = "dispatch_tree")]
fn get_dispatch_tree_entry(
    command_name_str: &str,
    dispatcher_struct: &Ident,
    entry_name: &Ident,
) -> proc_macro2::TokenStream {
    let node_name_lit = syn::LitStr::new(command_name_str, proc_macro2::Span::call_site());
    quote! {
        ::mingling::macros::register_dispatcher!(#node_name_lit, #dispatcher_struct, #entry_name);
    }
}

#[cfg(not(feature = "dispatch_tree"))]
fn get_dispatch_tree_entry(
    _command_name_str: &str,
    _dispatcher_struct: &Ident,
    _entry_name: &Ident,
) -> proc_macro2::TokenStream {
    quote! {}
}
