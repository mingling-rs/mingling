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
enum DispatcherClapInput {
    /// `("cmd", Disp, ...)`
    Default {
        command_name: LitStr,
        dispatcher_struct: Ident,
        options: ClapOptions,
    },
    /// `(Program, "cmd", Disp, ...)`
    Explicit {
        group_name: syn::Path,
        command_name: LitStr,
        dispatcher_struct: Ident,
        options: ClapOptions,
    },
}

impl Parse for DispatcherClapInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if (input.peek(Ident) || input.peek(Token![crate]))
            && (input.peek2(Token![::]) || input.peek2(Token![,]))
        {
            // Explicit format: Program, "cmd", Disp, ...
            let group_name: syn::Path = input.parse()?;
            input.parse::<Token![,]>()?;
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

            Ok(DispatcherClapInput::Explicit {
                group_name,
                command_name,
                dispatcher_struct,
                options,
            })
        } else if lookahead.peek(syn::LitStr) {
            // Default format: "cmd", Disp, ...
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

            Ok(DispatcherClapInput::Default {
                command_name,
                dispatcher_struct,
                options,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[cfg(feature = "clap")]
pub fn dispatcher_clap_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_input = parse_macro_input!(attr as DispatcherClapInput);
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = &input_struct.ident;

    // Determine the program name and other fields
    let (command_name_str, dispatcher_struct, options, program_path) = match &attr_input {
        DispatcherClapInput::Default {
            command_name,
            dispatcher_struct,
            options,
        } => (
            command_name.value(),
            dispatcher_struct.clone(),
            ClapOptions {
                error_struct: options.error_struct.clone(),
                help_enabled: options.help_enabled,
            },
            crate::default_program_path(),
        ),
        DispatcherClapInput::Explicit {
            group_name,
            command_name,
            dispatcher_struct,
            options,
        } => (
            command_name.value(),
            dispatcher_struct.clone(),
            ClapOptions {
                error_struct: options.error_struct.clone(),
                help_enabled: options.help_enabled,
            },
            quote! { #group_name },
        ),
    };

    // Generate the `begin` method body
    let begin_body = if let Some(ref error_struct) = options.error_struct {
        quote! {
            if ::mingling::this::<#program_path>().user_context.help {
                return #struct_name::default().to_chain();
            }
            match <#struct_name as ::clap::Parser>::try_parse_from(clap_args) {
                Ok(parsed) => parsed.to_chain(),
                Err(e) => {
                    return #error_struct::new(format!("{}", e.render().ansi())).to_render()
                },
            }
        }
    } else {
        quote! {
            if ::mingling::this::<#program_path>().user_context.help {
                return #struct_name::default().to_chain();
            }
            let parsed = <#struct_name as ::clap::Parser>::try_parse_from(clap_args)
                .unwrap_or_else(|e| e.exit());
            parsed.to_chain()
        }
    };

    // Generate the error pack type
    let error_pack = options.error_struct.as_ref().map(|error_struct| {
        quote! {
            ::mingling::macros::pack!(#program_path, #error_struct = String);
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
            fn #help_fn_name(_prev: #struct_name) {
                use std::io::Write;
                use clap::ColorChoice;

                let this = ::mingling::this::<#program_path>();
                match this.stdout_setting.clap_help_print_behaviour {
                    ::mingling::ClapHelpPrintBehaviour::WriteToRenderResult => {
                        let mut cmd = <#struct_name as ::clap::CommandFactory>::command()
                            .color(ColorChoice::Always);
                        let styled = cmd.render_help();
                        write!(__renderer_inner_result, "{}", styled.ansi()).unwrap();
                    }
                    ::mingling::ClapHelpPrintBehaviour::PrintDirectly => {
                        let mut command = <#struct_name as ::clap::CommandFactory>::command();
                        command.print_help().unwrap();
                    }
                }
            }
        })
    } else {
        None
    };

    let dispatch_tree_entry =
        get_dispatch_tree_entry(&command_name_str, &dispatcher_struct, &struct_name);

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
        pub struct #dispatcher_struct;

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
