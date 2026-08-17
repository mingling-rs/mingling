//! Compile-time build logic for `build_comp!()` and `build_pathf!()`.
//!
//! The build steps run as a side effect of macro expansion (during `gen_program!`),
//! writing artifacts under `{target_directory}/mingling/`.

#[doc(hidden)]
#[cfg(feature = "comp")]
pub(crate) mod comp;

#[doc(hidden)]
#[cfg(feature = "pathf")]
pub(crate) mod pathf;

/// Shared implementation behind `build_comp!()`.
///
/// Accepts an optional string literal (the binary name); defaults to
/// `CARGO_PKG_NAME`. Returns an empty token stream on success, or a
/// `compile_error!` token stream on failure.
#[cfg(feature = "comp")]
pub(crate) fn comp_build_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let bin_name: String = if input.is_empty() {
        std::env::var("CARGO_PKG_NAME").unwrap_or_default()
    } else {
        match syn::parse::<syn::LitStr>(input) {
            Ok(lit) => lit.value(),
            Err(e) => return e.to_compile_error().into(),
        }
    };

    match comp::build_comp_scripts(&bin_name) {
        Ok(()) => proc_macro::TokenStream::new(),
        Err(e) => {
            let msg = format!("build_comp: failed to generate completion scripts: {e}");
            syn::Error::new(proc_macro2::Span::call_site(), msg)
                .to_compile_error()
                .into()
        }
    }
}

/// Shared implementation behind `build_pathf!()`.
///
/// Runs the pathf type-mapping analysis. Returns an empty token stream on
/// success, or a `compile_error!` token stream on failure.
#[cfg(feature = "pathf")]
pub(crate) fn pathf_build_impl(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match pathf::analyze_and_build_type_mapping() {
        Ok(()) => proc_macro::TokenStream::new(),
        Err(e) => {
            let msg = format!("build_pathf: type mapping analysis failed: {e}");
            syn::Error::new(proc_macro2::Span::call_site(), msg)
                .to_compile_error()
                .into()
        }
    }
}
