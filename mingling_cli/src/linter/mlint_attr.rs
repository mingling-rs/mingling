/// Result of checking `mlint(allow/warn/deny(...))` attributes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MlintLevelOverride {
    Allow,
    Warn,
    Deny,
}

/// Parse `mlint(allow/warn/deny(lint_name))` from attributes.
/// Returns `None` if the lint is not mentioned in any mlint attribute.
pub fn get_mlint_override(attrs: &[syn::Attribute], lint_name: &str) -> Option<MlintLevelOverride> {
    for attr in attrs {
        if !attr.path().is_ident("mlint") {
            continue;
        }
        let list = match attr.meta.require_list() {
            Ok(l) => l,
            Err(_) => continue,
        };
        // TokenStream::to_string() adds spaces between tokens,
        // e.g. `allow(xxx)` → `allow ( xxx )`. Remove all spaces to compare.
        let flat: String = list
            .tokens
            .to_string()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        for (keyword, variant) in [
            ("allow", MlintLevelOverride::Allow),
            ("warn", MlintLevelOverride::Warn),
            ("deny", MlintLevelOverride::Deny),
        ] {
            if flat.contains(&format!("{keyword}({lint_name})")) {
                return Some(variant);
            }
        }
    }
    None
}
