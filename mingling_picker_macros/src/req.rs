use proc_macro::{TokenStream, TokenTree};
use proc_macro2::TokenStream as TS2;
use quote::quote;

pub(crate) fn req(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let args = split_at_commas(&tokens);
    if args.is_empty() {
        return quote! { compile_error!("req! requires at least one argument") }.into();
    }

    let first = &args[0];
    let rest = &args[1..];

    // Validate: at most one char literal
    let char_count = rest.iter().filter(|a| is_char_literal(a)).count();
    if char_count > 1 {
        return quote! { compile_error!("req! only supports at most one short name") }.into();
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
    let colon_pos = first
        .iter()
        .position(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ':'));
    let has_generics = first
        .iter()
        .any(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == '<'));

    let first_slice = first.as_slice();
    let (name, ty, is_named) = match colon_pos {
        Some(pos) if pos > 0 && !is_double_colon(first, pos) => {
            // `name : Type`
            (
                Some(&first_slice[..pos]),
                Some(&first_slice[pos + 1..]),
                true,
            )
        }
        _ => {
            if has_generics || !is_single_ident(first) || !has_char_or_string(rest) {
                // Type path, generic type, or bare type (no more args)
                (None, Some(first_slice), false)
            } else {
                // Single ident followed by char/string → bare name
                (Some(first_slice), None, true)
            }
        }
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

    // Generate code
    let import = quote! { ::mingling::picker::PickerRequirement };

    // Type parameter
    let ty_ts: TS2 = ty
        .map(|t| {
            let ts: TokenStream = t.iter().cloned().collect();
            ts.to_string().parse().unwrap()
        })
        .unwrap_or(TS2::new());

    let with_type = if ty.is_some() {
        quote! { #import::<#ty_ts> }
    } else {
        quote! { #import::<_> }
    };

    // .with_full(...)
    let with_full = if !full_names.is_empty() {
        let strs: Vec<proc_macro2::Literal> = full_names
            .iter()
            .map(|s| proc_macro2::Literal::string(s))
            .collect();
        quote! { .with_full(&[#(#strs),*]) }
    } else {
        TS2::new()
    };

    // .with_short(...)
    let with_short = short_char
        .map(|c| {
            let lit = proc_macro2::Literal::character(c);
            quote! { .with_short(#lit) }
        })
        .unwrap_or_default();

    // .with_positional(...)
    let pos = !is_named;

    let result = quote! {
        #with_type::default()
            #with_full
            #with_short
            .with_positional(#pos)
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

fn is_single_ident(tokens: &[TokenTree]) -> bool {
    tokens.len() == 1 && matches!(&tokens[0], TokenTree::Ident(_))
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

fn has_char_or_string(args: &[Vec<TokenTree>]) -> bool {
    args.iter()
        .any(|a| is_char_literal(a) || is_string_literal(a))
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
