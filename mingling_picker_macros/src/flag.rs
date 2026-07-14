use proc_macro::{TokenStream, TokenTree};
use proc_macro2::TokenStream as TS2;
use quote::quote;

pub(crate) fn flag(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let args = split_at_commas(&tokens);
    if args.is_empty() {
        return quote! { compile_error!("flag! flaguires at least one argument") }.into();
    }

    let first = &args[0];
    let rest = &args[1..];

    // Validate: at most one char literal
    let char_count = rest.iter().filter(|a| is_char_literal(a)).count();
    if char_count > 1 {
        return quote! { compile_error!("flag! only supports at most one short name") }.into();
    }

    // Extract short char and string aliases
    let short_char: Option<char> = rest
        .iter()
        .find(|a| is_char_literal(a))
        .and_then(|a| extract_char(a));
    let aliases: Vec<String> = rest
        .iter()
        .filter(|a| is_string_literal(a))
        .filter_map(|a| extract_string(a))
        .collect();

    // Parse first argument
    //   flag![name: Type]  → name, with explicit type
    //   flag![Type]        → positional type, no name
    let colon_pos = first
        .iter()
        .position(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ':'));
    let has_named_colon = colon_pos.is_some_and(|pos| pos > 0 && !is_double_colon(first, pos));

    let first_slice = first.as_slice();
    let (name, ty, is_named) = if has_named_colon {
        let pos = colon_pos.unwrap();
        // `name : Type`
        (
            Some(&first_slice[..pos]),
            Some(&first_slice[pos + 1..]),
            true,
        )
    } else {
        // No `name:` prefix → the entire first argument is the type
        (None, Some(first_slice), false)
    };

    // Build full names
    let full_names: Vec<String> = match name {
        Some(n) => {
            let mut v = vec![join_idents(n)];
            v.extend(aliases);
            v
        }
        None => aliases,
    };

    let path: TS2 = {
        let ty_ts: TS2 = ty
            .map(|t| {
                let ts: TokenStream = t.iter().cloned().collect();
                TS2::from(ts)
            })
            .unwrap_or(TS2::new());

        #[cfg(feature = "mingling_support")]
        let import = quote! { ::mingling::picker::PickerFlag };

        #[cfg(not(feature = "mingling_support"))]
        let import = quote! { ::mingling_picker::PickerFlag };

        if ty.is_some() {
            quote! { #import::<#ty_ts> }
        } else {
            quote! { #import::<_> }
        }
    };

    // full: &["name", "alias", ...]  or  &[]
    let full_value: TS2 = if !full_names.is_empty() {
        let strs: Vec<proc_macro2::Literal> = full_names
            .iter()
            .map(|s| proc_macro2::Literal::string(s))
            .collect();
        quote! { &[#(#strs),*] }
    } else {
        quote! { &[] }
    };

    // short: Some('c')  or  None
    let short_value: TS2 = match short_char {
        Some(c) => {
            let lit = proc_macro2::Literal::character(c);
            quote! { ::std::option::Option::Some(#lit) }
        }
        None => quote! { ::std::option::Option::None },
    };

    let positional_value: bool = !is_named;

    let result = quote! {
        #path {
            full: #full_value,
            short: #short_value,
            positional: #positional_value,
            internal_type: ::std::marker::PhantomData,
        }
    };

    result.into()
}

fn split_at_commas(tokens: &[TokenTree]) -> Vec<Vec<TokenTree>> {
    let mut result = vec![Vec::new()];
    let mut depth = 0u32;
    for t in tokens {
        match t {
            TokenTree::Group(_g) => {
                depth += 1;
                result.last_mut().unwrap().push(t.clone());
                depth -= 1;
            }
            TokenTree::Punct(p) if p.as_char() == ',' && depth == 0 => {
                result.push(Vec::new());
            }
            _ => result.last_mut().unwrap().push(t.clone()),
        }
    }
    result
}

fn is_double_colon(tokens: &[TokenTree], pos: usize) -> bool {
    pos > 0 && matches!(&tokens[pos - 1], TokenTree::Punct(p) if p.as_char() == ':')
}

fn is_char_literal(tokens: &[TokenTree]) -> bool {
    tokens.len() == 1
        && matches!(&tokens[0], TokenTree::Literal(l) if {
            let s = l.to_string();
            s.starts_with('\'') && s.len() >= 3
        })
}

fn is_string_literal(tokens: &[TokenTree]) -> bool {
    tokens.len() == 1
        && matches!(&tokens[0], TokenTree::Literal(l) if {
            l.to_string().starts_with('"')
        })
}

fn extract_char(tokens: &[TokenTree]) -> Option<char> {
    match &tokens[0] {
        TokenTree::Literal(l) => {
            let s = l.to_string();
            let cs: Vec<char> = s.chars().collect();
            if cs.len() >= 3 && cs[0] == '\'' && cs[cs.len() - 1] == '\'' {
                let inner: String = cs[1..cs.len() - 1].iter().collect();
                match inner.as_str() {
                    "n" => Some('\n'),
                    "t" => Some('\t'),
                    "r" => Some('\r'),
                    "0" => Some('\0'),
                    "\\\\" => Some('\\'),
                    "\\'" => Some('\''),
                    _ => inner.chars().next(),
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_string(tokens: &[TokenTree]) -> Option<String> {
    match &tokens[0] {
        TokenTree::Literal(l) => {
            let s = l.to_string();
            let cs: Vec<char> = s.chars().collect();
            if cs.len() >= 2 && cs[0] == '"' && cs[cs.len() - 1] == '"' {
                Some(cs[1..cs.len() - 1].iter().collect())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn join_idents(tokens: &[TokenTree]) -> String {
    tokens
        .iter()
        .map(|t| match t {
            TokenTree::Ident(id) => id.to_string(),
            _ => String::new(),
        })
        .collect()
}
