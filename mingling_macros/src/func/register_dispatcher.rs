// Doc Not Optimize
#[cfg(feature = "dispatch_tree")]
use just_fmt::snake_case;
use proc_macro::TokenStream;
use quote::quote;
#[cfg(feature = "dispatch_tree")]
use syn::parse::{Parse, ParseStream};
#[cfg(feature = "dispatch_tree")]
use syn::{Ident, LitStr, Result as SynResult, Token};

#[cfg(feature = "dispatch_tree")]
use crate::COMPILE_TIME_DISPATCHERS;
#[cfg(feature = "dispatch_tree")]
use crate::get_global_set;

#[cfg(feature = "dispatch_tree")]
struct RegisterDispatcherInput {
    node_name: LitStr,
    dispatcher_type: Ident,
    entry_name: Ident,
}

#[cfg(feature = "dispatch_tree")]
impl Parse for RegisterDispatcherInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let node_name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let dispatcher_type: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let entry_name: Ident = input.parse()?;
        Ok(RegisterDispatcherInput {
            node_name,
            dispatcher_type,
            entry_name,
        })
    }
}

#[cfg(feature = "dispatch_tree")]
pub(crate) fn register_dispatcher(input: TokenStream) -> TokenStream {
    let RegisterDispatcherInput {
        node_name,
        dispatcher_type,
        entry_name,
    } = syn::parse_macro_input!(input as RegisterDispatcherInput);

    let node_name_str = node_name.value();
    let static_name = format!(
        "__internal_dispatcher_{}",
        snake_case!(node_name_str.clone())
    );
    let static_ident = Ident::new(&static_name, proc_macro2::Span::call_site());

    // Register node info in the global collection at compile time
    // Format: "node.name:DispatcherType:EntryName"
    get_global_set(&COMPILE_TIME_DISPATCHERS)
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
pub(crate) fn register_dispatcher(_input: TokenStream) -> TokenStream {
    quote! {}.into()
}
