// Doc Not Optimize
use just_fmt::snake_case;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Result as SynResult, Token};

use crate::COMPILE_TIME_DISPATCHERS;
use crate::get_global_set;

struct RegisterDispatcherInput {
    node_name: LitStr,
    dispatcher_type: Ident,
    entry_name: Ident,
}

impl Parse for RegisterDispatcherInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let node_name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let dispatcher_type: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let entry_name: Ident = input.parse()?;
        Ok(Self {
            node_name,
            dispatcher_type,
            entry_name,
        })
    }
}

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
        .insert(format!("{node_name_str}:{dispatcher_type}:{entry_name}"));

    let expanded = quote! {
        #[doc(hidden)]
        #[allow(nonstandard_style)]
        pub static #static_ident: #dispatcher_type = #dispatcher_type;
    };

    expanded.into()
}
