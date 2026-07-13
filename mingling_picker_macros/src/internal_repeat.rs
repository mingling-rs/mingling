use proc_macro::{Delimiter, Group, Ident, Literal, TokenStream, TokenTree};

pub(crate) fn internal_repeat(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let (range_start, range_end, body_start) = parse_range(&tokens);

    let mut body: Vec<TokenTree> = tokens[body_start..].to_vec();
    if body.len() == 1
        && let TokenTree::Group(g) = &body[0]
        && g.delimiter() == Delimiter::Brace
    {
        body = g.stream().into_iter().collect();
    }

    let mut result = Vec::new();
    for i in range_start..=range_end {
        result.extend(expand_body(&body, i));
    }
    result.into_iter().collect()
}

/// Parse `start .. end =>` or `start ..= end =>` or `count =>` (backward compat).
/// Returns `(start, end_inclusive, body_start_index)`.
fn parse_range(tokens: &[TokenTree]) -> (usize, usize, usize) {
    // Find => separator
    let arrow_pos = tokens.windows(2).position(|w| {
        matches!(&w[0], TokenTree::Punct(p) if p.as_char() == '=')
            && matches!(&w[1], TokenTree::Punct(p) if p.as_char() == '>')
    });

    let (arrow_pos, body_start) = match arrow_pos {
        Some(p) => (p, p + 2),
        None => return (1, 12, 0), // fallback
    };

    let before: Vec<&TokenTree> = tokens[..arrow_pos].iter().collect();

    // Try to find `..` or `..=` pattern
    // `..` is two Punct('.') tokens
    let dotdot = before.windows(2).position(|w| {
        matches!(w[0], TokenTree::Punct(p) if p.as_char() == '.')
            && matches!(w[1], TokenTree::Punct(p) if p.as_char() == '.')
    });

    if let Some(dd) = dotdot {
        // Start value: tokens before `..`
        let start = parse_usize_tokens(&before[..dd]);
        let after_dd = &before[dd + 2..];

        // Check for `..=` (inclusive range)
        let (inclusive, end_tokens) = if after_dd
            .first()
            .is_some_and(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == '='))
        {
            (true, &after_dd[1..])
        } else {
            (false, after_dd)
        };

        let end = parse_usize_tokens(end_tokens);

        if inclusive {
            (start, end, body_start)
        } else {
            // Exclusive end: if end >= start, iterate start..end, so end_inclusive = end - 1
            if end > start {
                (start, end - 1, body_start)
            } else {
                (1, 12, body_start) // fallback
            }
        }
    } else {
        // No `..` found — fallback to simple count
        let count = parse_usize_tokens(&before);
        (1, count, body_start)
    }
}

/// Parse a sequence of tokens as a single usize value.
fn parse_usize_tokens(tokens: &[&TokenTree]) -> usize {
    let s: String = tokens
        .iter()
        .map(|t| match t {
            TokenTree::Literal(l) => l.to_string(),
            TokenTree::Ident(id) => id.to_string(),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join("")
        .replace(' ', "");

    s.parse().unwrap_or(12)
}

/// Walk tokens, replacing `$` in identifier tails, and expanding
/// `( … )+` / `( … ,)+` / `( … ;)+` groups.
fn expand_body(tokens: &[TokenTree], outer: usize) -> Vec<TokenTree> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        // Check for a parenthesized repetition group: ( ... ) sep? +
        if let Some(exp) = try_expand_paren_group(tokens, i, outer) {
            let (items, consumed) = exp;
            out.extend(items);
            i += consumed;
            continue;
        }

        // Identifier followed by `$` → combined ident + number
        if let TokenTree::Ident(id) = &tokens[i] {
            if i + 1 < tokens.len()
                && let TokenTree::Punct(p) = &tokens[i + 1]
                && p.as_char() == '$'
            {
                let name = format!("{}{outer}", id);
                out.push(TokenTree::Ident(Ident::new(&name, id.span())));
                i += 2;
                continue;
            }
            out.push(tokens[i].clone());
            i += 1;
            continue;
        }

        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == '$' => {
                out.push(TokenTree::Literal(Literal::usize_suffixed(outer)));
            }
            TokenTree::Group(g) => {
                let inner = expand_body_vec(&g.stream(), outer);
                out.push(TokenTree::Group(Group::new(
                    g.delimiter(),
                    inner.into_iter().collect(),
                )));
            }
            other => out.push(other.clone()),
        }
        i += 1;
    }
    out
}

fn expand_body_vec(stream: &TokenStream, outer: usize) -> Vec<TokenTree> {
    let v: Vec<TokenTree> = stream.clone().into_iter().collect();
    expand_body(&v, outer)
}

/// Try to expand `( … )+` / `( … ,)+` / `( … ;)+` at position `i`.
///
/// The `+` is AFTER the closing paren. An optional separator (`,` or `;`)
/// may appear between `)` and `+`.
/// Returns `(expanded_tokens, consumed_count)` or `None`.
fn try_expand_paren_group(
    tokens: &[TokenTree],
    i: usize,
    outer: usize,
) -> Option<(Vec<TokenTree>, usize)> {
    let group = match tokens.get(i)? {
        TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => g,
        _ => return None,
    };

    // Check tokens AFTER the group for `+` (optionally preceded by `,` or `;`)
    let rest = &tokens[i + 1..];
    let sep: Option<&str> = match rest.first() {
        Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
            if matches!(rest.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '+') {
                Some(",")
            } else {
                return None;
            }
        }
        Some(TokenTree::Punct(p)) if p.as_char() == ';' => {
            if matches!(rest.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '+') {
                Some(";")
            } else {
                return None;
            }
        }
        Some(TokenTree::Punct(p)) if p.as_char() == '+' => None,
        _ => return None,
    };

    let consumed = if sep.is_some() { 3 } else { 2 }; // group + sep? + +

    let inner: Vec<TokenTree> = group.stream().into_iter().collect();

    let mut out = Vec::new();
    for n in 1..=outer {
        if n > 1
            && let Some(s) = sep
        {
            out.push(TokenTree::Punct(proc_macro::Punct::new(
                s.chars().next().unwrap(),
                proc_macro::Spacing::Alone,
            )));
        }
        out.extend(expand_body(&inner, n));
    }

    Some((out, consumed))
}
