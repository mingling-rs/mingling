// Doc Not Optimize
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
            if input.parse::<Token![=]>().is_err() {
                return Err(syn::Error::new(
                    key.span(),
                    "expected `key = value`; note: the explicit CMD struct argument \
                     was removed in 0.5.0, use `dispatcher_clap!(\"name\", help = ..., error = ...)`",
                ));
            }

            if key == "error" {
                let value: Ident = input.parse()?;
                if error_struct.is_some() {
                    return Err(syn::Error::new(key.span(), "duplicate `error` key"));
                }
                error_struct = Some(value);
            } else if key == "help" {
                let value: LitBool = input.parse()?;
                if value.value() {
                    help_enabled = true;
                } else {
                    // help = false is allowed but does nothing
                    help_enabled = false;
                }
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "unknown key, expected `error` or `help`",
                ));
            }
        }

        Ok(Self {
            error_struct,
            help_enabled,
        })
    }
}

/// Input for the `dispatcher_clap` attribute
struct DispatcherClapInput {
    /// `("cmd", options...)`
    command_name: LitStr,
    options: ClapOptions,
}

impl Parse for DispatcherClapInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Format: "cmd", options...
        let command_name: LitStr = input.parse()?;

        let options = if input.is_empty() {
            ClapOptions {
                error_struct: None,
                help_enabled: false,
            }
        } else {
            input.parse::<ClapOptions>()?
        };

        Ok(Self {
            command_name,
            options,
        })
    }
}

#[cfg(feature = "clap")]
#[allow(clippy::too_many_lines)]
pub(crate) fn dispatcher_clap_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_input = parse_macro_input!(attr as DispatcherClapInput);
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = &input_struct.ident;

    let program_path = crate::default_program_path();

    let command_name_str = attr_input.command_name.value();
    let command_name_lit = syn::LitStr::new(&command_name_str, attr_input.command_name.span());

    // The dispatcher struct is now generated internally.
    let dispatcher_struct = Ident::new(
        &format!("__Dispatcher{}", just_fmt::pascal_case!(&command_name_str)),
        attr_input.command_name.span(),
    );
    let command_snake = just_fmt::snake_case!(command_name_str.clone());
    let mod_name = Ident::new(
        &format!("__mingling_dispatcher_{command_snake}"),
        attr_input.command_name.span(),
    );
    let static_ident = Ident::new(
        &format!("__internal_dispatcher_{command_snake}"),
        attr_input.command_name.span(),
    );
    let options = &attr_input.options;

    // Generate the `begin` method body
    let begin_body = options.error_struct.as_ref().map_or_else(
        || {
            quote! {
                if ::mingling::this::<#program_path>().user_context.help {
                    return ::mingling::Routable::<#program_path>::to_chain(#struct_name::default());
                }
                let parsed = <#struct_name as ::clap::Parser>::try_parse_from(clap_args)
                    .unwrap_or_else(|e| e.exit());
                ::mingling::Routable::<#program_path>::to_chain(parsed)
            }
        },
        |error_struct| {
            quote! {
                if ::mingling::this::<#program_path>().user_context.help {
                    return ::mingling::Routable::<#program_path>::to_chain(#struct_name::default());
                }
                match <#struct_name as ::clap::Parser>::try_parse_from(clap_args) {
                    Ok(parsed) => ::mingling::Routable::<#program_path>::to_chain(parsed),
                    Err(e) => {
                        return ::mingling::Routable::<#program_path>::to_render(#error_struct(format!("{}", e.render().ansi())))
                    },
                }
            }
        },
    );

    // Generate the error pack type
    let error_pack = options.error_struct.as_ref().map(|error_struct| {
        quote! {
            #[derive(::mingling::Grouped, ::mingling::Wrap, Default)]
            pub struct #error_struct(pub ::std::string::String);
        }
    });

    // Generate the #[help] block if help = true
    let help_gen = if options.help_enabled {
        let help_fn_name_str = format!("__{}_help", just_fmt::snake_case!(&command_name_str));
        let help_fn_name = Ident::new(&help_fn_name_str, proc_macro2::Span::call_site());

        Some(quote! {
            #[allow(non_snake_case)]
            #[::mingling::macros::help]
            pub(crate) fn #help_fn_name(_prev: #struct_name) -> ::mingling::RenderResult {
                use std::io::Write;
                use clap::ColorChoice;

                let this = ::mingling::this::<#program_path>();
                match this.stdout_setting.clap_help_print_behaviour {
                    ::mingling::config::ClapHelpPrintBehaviour::WriteToRenderResult => {
                        let mut cmd = <#struct_name as ::clap::CommandFactory>::command()
                            .color(ColorChoice::Always);
                        let styled = cmd.render_help();
                        let mut result = ::mingling::RenderResult::new();
                        let _ = write!(result, "{}", styled.ansi());
                        result
                    }
                    ::mingling::config::ClapHelpPrintBehaviour::PrintDirectly => {
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

    let expanded = quote! {
        // Keep the original struct definition
        #input_struct

        // Generate the error wrapper type
        #error_pack

        // Generate the help block if enabled
        #help_gen

        // Generate the dispatcher module, registration and impl
        #[doc(hidden)]
        #[allow(non_snake_case)]
        pub(crate) mod #mod_name {
            use super::*;

            #[doc(hidden)]
            #[derive(Default)]
            pub struct #dispatcher_struct;

            ::mingling::macros::register_dispatcher!(
                #command_name_lit,
                #dispatcher_struct,
                #struct_name
            );

            impl ::mingling::Dispatcher<#program_path> for #dispatcher_struct {
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
            }
        }

        #[allow(unused_imports)]
        pub(crate) use #mod_name::#dispatcher_struct;
        #[allow(unused_imports)]
        pub(crate) use #mod_name::#static_ident;
    };

    expanded.into()
}
