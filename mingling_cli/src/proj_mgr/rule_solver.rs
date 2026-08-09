//! Parsing and evaluation of `rule.toml` and `checklist.toml`.
//!
//! `rule.toml` drives template generation:
//! - `[[display]]` entries map a display-block name to a boolean rule; when the
//!   rule evaluates to true, the corresponding `??? >>> NAME` block is enabled.
//! - `[[hide-file]]` entries map a generated file path to a boolean rule; when
//!   the rule evaluates to true, the generated file is removed.
//!
//! `checklist.toml` holds the user's answers as key/value pairs and provides
//! the variables the rules are evaluated against.

use std::collections::HashMap;

use toml_edit::DocumentMut;

/// A `[[display]]` rule: enable display block `name` when `rule` is true.
#[derive(Debug, Clone)]
pub struct DisplayRule {
    /// Display block name, e.g. `ASYNC`, `PARSER_PICKER`.
    pub name: String,
    /// Boolean expression, e.g. `tokio || async_std`.
    pub rule: String,
}

/// A `[[hide-file]]` rule: hide `file` when `rule` is true.
#[derive(Debug, Clone)]
pub struct HideFileRule {
    /// File path relative to the project root, e.g. `./build.rs`.
    pub file: String,
    /// Boolean expression, e.g. `!completion && !pathf`.
    pub rule: String,
}

/// A `[[hide-dir]]` rule: hide the whole directory `dir` when `rule` is true.
#[derive(Debug, Clone)]
pub struct HideDirRule {
    /// Directory path relative to the project root, e.g. `./src/completion/`.
    pub dir: String,
    /// Boolean expression, e.g. `!completion`.
    pub rule: String,
}

/// A `[[user.*]]` entry describing a checklist answer and its default.
///
/// Checklist keys are declared under `[[user.input]]`, `[[user.toggle]]` or
/// `[[user.selection]]`; toggles and selections may carry a `default` value
/// that applies when the user leaves the key unset in `checklist.toml`.
#[derive(Debug, Clone)]
pub struct UserRule {
    /// Checklist key name, e.g. `program_name`, `tokio`, `parser`.
    pub name: String,
    /// Default value (stringified), if declared in `rule.toml`.
    pub default: Option<String>,
}

/// A `[[user.toggle-mutex]]` group: at most one of `mutex` may be enabled.
#[derive(Debug, Clone)]
pub struct UserMutex {
    /// Keys that are mutually exclusive.
    pub mutex: Vec<String>,
    /// Human-readable reason shown when the constraint is violated.
    pub reason: String,
}

/// All rules parsed from `rule.toml`.
#[derive(Debug, Default, Clone)]
pub struct TemplateRules {
    pub users: Vec<UserRule>,
    pub mutexes: Vec<UserMutex>,
    pub display: Vec<DisplayRule>,
    pub hide_files: Vec<HideFileRule>,
    pub hide_dirs: Vec<HideDirRule>,
}

/// Parse `checklist.toml` into a flat map of answers.
///
/// Values are stringified: strings keep their content, booleans become
/// `"true"` / `"false"`, numbers keep their decimal representation.
/// Commented-out (disabled) keys are ignored by the TOML parser.
pub fn parse_checklist(content: &str) -> Result<HashMap<String, String>, String> {
    let doc = content.parse::<DocumentMut>().map_err(|e| e.to_string())?;
    let mut answers = HashMap::new();
    for (key, item) in doc.iter() {
        let toml_edit::Item::Value(value) = item else {
            continue;
        };
        let stringified = match value {
            toml_edit::Value::String(s) => s.value().clone(),
            toml_edit::Value::Boolean(b) => b.value().to_string(),
            toml_edit::Value::Integer(i) => i.value().to_string(),
            toml_edit::Value::Float(f) => f.value().to_string(),
            // Arrays, inline tables and datetimes are not valid checklist answers.
            _ => continue,
        };
        answers.insert(key.to_string(), stringified);
    }
    Ok(answers)
}

/// Parse `rule.toml` into display and hide-file rules.
pub fn parse_rules(content: &str) -> Result<TemplateRules, String> {
    let doc = content.parse::<DocumentMut>().map_err(|e| e.to_string())?;
    let mut rules = TemplateRules::default();

    // `[[user.input]]`, `[[user.toggle]]`, `[[user.selection]]` parse into a
    // `user` table whose keys (`input`/`toggle`/`selection`) hold the arrays.
    if let Some(user_table) = doc.get("user").and_then(|item| item.as_table()) {
        for kind in ["input", "toggle", "selection"] {
            let Some(tables) = user_table
                .get(kind)
                .and_then(|item| item.as_array_of_tables())
            else {
                continue;
            };
            for table in tables {
                let name = table
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or("[[user.*]] entry is missing `name`")?
                    .to_string();
                let default =
                    table
                        .get("default")
                        .and_then(|v| v.as_value())
                        .and_then(|v| match v {
                            toml_edit::Value::Boolean(b) => Some(b.value().to_string()),
                            toml_edit::Value::String(s) => Some(s.value().clone()),
                            _ => None,
                        });
                rules.users.push(UserRule { name, default });
            }
        }

        // `[[user.toggle-mutex]]` declares mutually exclusive toggle groups.
        if let Some(tables) = user_table
            .get("toggle-mutex")
            .and_then(|item| item.as_array_of_tables())
        {
            for table in tables {
                let mutex = table
                    .get("mutex")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default();
                let reason = table
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("these options are mutually exclusive")
                    .to_string();
                rules.mutexes.push(UserMutex { mutex, reason });
            }
        }
    }

    if let Some(tables) = doc
        .get("display")
        .and_then(|item| item.as_array_of_tables())
    {
        for table in tables {
            let name = table
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("[[display]] entry is missing `name`")?
                .to_string();
            let rule = table
                .get("rule")
                .and_then(|v| v.as_str())
                .ok_or("[[display]] entry is missing `rule`")?
                .to_string();
            rules.display.push(DisplayRule { name, rule });
        }
    }

    if let Some(tables) = doc
        .get("hide-file")
        .and_then(|item| item.as_array_of_tables())
    {
        for table in tables {
            let file = table
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or("[[hide-file]] entry is missing `file`")?
                .to_string();
            let rule = table
                .get("rule")
                .and_then(|v| v.as_str())
                .ok_or("[[hide-file]] entry is missing `rule`")?
                .to_string();
            rules.hide_files.push(HideFileRule { file, rule });
        }
    }

    if let Some(tables) = doc
        .get("hide-dir")
        .and_then(|item| item.as_array_of_tables())
    {
        for table in tables {
            let dir = table
                .get("dir")
                .and_then(|v| v.as_str())
                .ok_or("[[hide-dir]] entry is missing `dir`")?
                .to_string();
            let rule = table
                .get("rule")
                .and_then(|v| v.as_str())
                .ok_or("[[hide-dir]] entry is missing `rule`")?
                .to_string();
            rules.hide_dirs.push(HideDirRule { dir, rule });
        }
    }

    Ok(rules)
}

/// Merge the checklist answers with the defaults declared in `rule.toml`.
///
/// A key the user left unset in `checklist.toml` falls back to its declared
/// default (e.g. `pathf` defaults to `true`); keys the user filled in always
/// win. The merged map is what rules are evaluated against.
pub fn resolve_answers(
    answers: &HashMap<String, String>,
    rules: &TemplateRules,
) -> HashMap<String, String> {
    let mut merged = answers.clone();
    for user in &rules.users {
        if let Some(default) = &user.default
            && !merged.contains_key(&user.name)
        {
            merged.insert(user.name.clone(), default.clone());
        }
    }
    merged
}

/// Validate mutually exclusive toggle groups against the resolved answers.
///
/// Returns an error for the first group where more than one key is enabled.
pub fn validate_mutexes(
    answers: &HashMap<String, String>,
    rules: &TemplateRules,
) -> Result<(), String> {
    for group in &rules.mutexes {
        let enabled: Vec<&str> = group
            .mutex
            .iter()
            .filter(|key| is_truthy(key, answers))
            .map(String::as_str)
            .collect();
        if enabled.len() > 1 {
            return Err(format!(
                "{} (enabled: {})",
                group.reason,
                enabled.join(", ")
            ));
        }
    }
    Ok(())
}

/// Evaluate a boolean rule expression against the checklist answers.
///
/// Supported syntax:
/// - bare identifier: `tokio` — true when the key exists with a truthy value
/// - comparison: `parser == clap`
/// - operators: `!`, `&&`, `||`
/// - parentheses for grouping
pub fn eval_rule(rule: &str, answers: &HashMap<String, String>) -> bool {
    let mut parser = RuleParser::new(rule, answers);
    let Some(value) = parser.parse_or() else {
        return false;
    };
    // The whole expression must be consumed; trailing garbage invalidates it.
    parser.skip_ws();
    if parser.pos != parser.chars.len() {
        return false;
    }
    value
}

/// A key is truthy when it is present with a non-empty, non-`"false"` value.
fn is_truthy(key: &str, answers: &HashMap<String, String>) -> bool {
    matches!(answers.get(key), Some(value) if !value.is_empty() && value != "false")
}

/// Recursive-descent parser for `rule.toml` boolean expressions.
struct RuleParser<'a> {
    chars: Vec<char>,
    pos: usize,
    answers: &'a HashMap<String, String>,
}

impl<'a> RuleParser<'a> {
    fn new(rule: &str, answers: &'a HashMap<String, String>) -> Self {
        Self {
            chars: rule.chars().collect(),
            pos: 0,
            answers,
        }
    }

    /// Skips whitespace, then consumes `c` if it matches.
    fn eat(&mut self, c: char) -> bool {
        self.skip_ws();
        if self.chars.get(self.pos) == Some(&c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn skip_ws(&mut self) {
        while self.chars.get(self.pos).is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    /// `or := and ('||' and)*`
    fn parse_or(&mut self) -> Option<bool> {
        let mut value = self.parse_and()?;
        while self.eat('|') && self.eat('|') {
            let rhs = self.parse_and()?;
            value |= rhs;
        }
        Some(value)
    }

    /// `and := unary ('&&' unary)*`
    fn parse_and(&mut self) -> Option<bool> {
        let mut value = self.parse_unary()?;
        while self.eat('&') && self.eat('&') {
            let rhs = self.parse_unary()?;
            value &= rhs;
        }
        Some(value)
    }

    /// `unary := '!' unary | primary`
    fn parse_unary(&mut self) -> Option<bool> {
        if self.eat('!') {
            return Some(!self.parse_unary()?);
        }
        self.parse_primary()
    }

    /// `primary := '(' or ')' | ident (('==' | '!=') ident)?`
    fn parse_primary(&mut self) -> Option<bool> {
        if self.eat('(') {
            let value = self.parse_or()?;
            self.eat(')');
            return Some(value);
        }
        let ident = self.parse_ident()?;
        if self.eat('=') && self.eat('=') {
            let other = self.parse_ident()?;
            return Some(self.answers.get(&ident).map(String::as_str) == Some(other.as_str()));
        }
        if self.eat('!') && self.eat('=') {
            let other = self.parse_ident()?;
            return Some(self.answers.get(&ident).map(String::as_str) != Some(other.as_str()));
        }
        Some(is_truthy(&ident, self.answers))
    }

    fn parse_ident(&mut self) -> Option<String> {
        self.skip_ws();
        let start = self.pos;
        while self
            .chars
            .get(self.pos)
            .is_some_and(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        {
            self.pos += 1;
        }
        if self.pos > start {
            Some(self.chars[start..self.pos].iter().collect())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checklist_parses_flat_values() {
        let content = r#"
program_name = "my-cli"
parser = "picker"
exit_code = true
completion = true
"#;
        let answers = parse_checklist(content).unwrap();
        assert_eq!(answers.get("program_name").unwrap(), "my-cli");
        assert_eq!(answers.get("parser").unwrap(), "picker");
        assert_eq!(answers.get("exit_code").unwrap(), "true");
        assert_eq!(answers.get("completion").unwrap(), "true");
    }

    #[test]
    fn checklist_ignores_commented_keys() {
        let content = r#"
# tokio = true
completion = true
"#;
        let answers = parse_checklist(content).unwrap();
        assert!(!answers.contains_key("tokio"));
        assert!(answers.contains_key("completion"));
    }

    #[test]
    fn rules_parse_display_and_hide_file() {
        let content = r#"
[[display]]
name = "TOKIO"
rule = "tokio"

[[display]]
name = "ASYNC"
rule = "tokio || async_std"

[[display]]
name = "PARSER_PICKER"
rule = "use_parser && parser == picker"

[[hide-file]]
file = "./build.rs"
rule = "!completion && !pathf"
"#;
        let rules = parse_rules(content).unwrap();
        assert_eq!(rules.display.len(), 3);
        assert_eq!(rules.display[0].name, "TOKIO");
        assert_eq!(rules.display[0].rule, "tokio");
        assert_eq!(rules.display[1].rule, "tokio || async_std");
        assert_eq!(rules.display[2].rule, "use_parser && parser == picker");
        assert_eq!(rules.hide_files.len(), 1);
        assert_eq!(rules.hide_files[0].file, "./build.rs");
        assert_eq!(rules.hide_files[0].rule, "!completion && !pathf");
    }

    #[test]
    fn rules_parse_user_defaults() {
        let content = r#"
[[user.toggle]]
name = "pathf"
default = true

[[user.toggle]]
name = "tokio"

[[user.selection]]
name = "parser"
option = [ "clap", "builtin", "picker" ]
"#;
        let rules = parse_rules(content).unwrap();
        assert_eq!(rules.users.len(), 3);
        assert_eq!(rules.users[0].name, "pathf");
        assert_eq!(rules.users[0].default.as_deref(), Some("true"));
        assert_eq!(rules.users[1].name, "tokio");
        assert_eq!(rules.users[1].default, None);
        assert_eq!(rules.users[2].name, "parser");
        assert_eq!(rules.users[2].default, None);
    }

    #[test]
    fn resolve_answers_applies_declared_defaults() {
        let content = r#"
[[user.toggle]]
name = "pathf"
default = true

[[user.toggle]]
name = "completion"
"#;
        let rules = parse_rules(content).unwrap();

        // completion answered, pathf left unset -> pathf falls back to true
        let mut answers = HashMap::new();
        answers.insert("completion".into(), "true".into());
        let merged = resolve_answers(&answers, &rules);
        assert_eq!(merged.get("completion").unwrap(), "true");
        assert_eq!(merged.get("pathf").unwrap(), "true");

        // user-provided value always wins over the default
        let mut answers = HashMap::new();
        answers.insert("pathf".into(), "false".into());
        let merged = resolve_answers(&answers, &rules);
        assert_eq!(merged.get("pathf").unwrap(), "false");
    }

    #[test]
    fn rules_parse_hide_dir() {
        let content = r#"
[[hide-dir]]
dir = "./src/completion/"
rule = "!completion"

[[hide-dir]]
dir = "./src/dispatch/"
rule = "!dispatch_tree"
"#;
        let rules = parse_rules(content).unwrap();
        assert_eq!(rules.hide_dirs.len(), 2);
        assert_eq!(rules.hide_dirs[0].dir, "./src/completion/");
        assert_eq!(rules.hide_dirs[0].rule, "!completion");
        assert_eq!(rules.hide_dirs[1].dir, "./src/dispatch/");
    }

    #[test]
    fn rules_parse_toggle_mutex() {
        let content = r#"
[[user.toggle-mutex]]
mutex = [ "tokio", "async_std", "smol" ]
reason = "You can only select one async runtime"
"#;
        let rules = parse_rules(content).unwrap();
        assert_eq!(rules.mutexes.len(), 1);
        assert_eq!(rules.mutexes[0].mutex, vec!["tokio", "async_std", "smol"]);
        assert_eq!(
            rules.mutexes[0].reason,
            "You can only select one async runtime"
        );
    }

    #[test]
    fn validate_mutexes_allows_zero_or_one() {
        let content = r#"
[[user.toggle-mutex]]
mutex = [ "tokio", "async_std" ]
reason = "one async runtime only"
"#;
        let rules = parse_rules(content).unwrap();

        // None enabled.
        assert!(validate_mutexes(&HashMap::new(), &rules).is_ok());

        // Exactly one enabled.
        let mut answers = HashMap::new();
        answers.insert("tokio".into(), "true".into());
        assert!(validate_mutexes(&answers, &rules).is_ok());
    }

    #[test]
    fn validate_mutexes_rejects_multiple_enabled() {
        let content = r#"
[[user.toggle-mutex]]
mutex = [ "tokio", "async_std" ]
reason = "one async runtime only"
"#;
        let rules = parse_rules(content).unwrap();

        let mut answers = HashMap::new();
        answers.insert("tokio".into(), "true".into());
        answers.insert("async_std".into(), "true".into());
        let err = validate_mutexes(&answers, &rules).unwrap_err();
        assert!(err.contains("one async runtime only"), "unexpected: {err}");
        assert!(err.contains("tokio") && err.contains("async_std"));
    }

    #[test]
    fn toggle_semantics_match_checklist_states() {
        // rule.toml: `pathf` defaults to true, `tokio` has no default.
        let content = r#"
[[user.toggle]]
name = "pathf"
default = true

[[user.toggle]]
name = "tokio"
"#;
        let rules = parse_rules(content).unwrap();

        // 1. Commented out (`# key = true`): key absent -> default applies;
        //    keys without a default stay absent (falsy).
        let merged = resolve_answers(&HashMap::new(), &rules);
        assert_eq!(merged.get("pathf").unwrap(), "true");
        assert!(!merged.contains_key("tokio"));
        assert!(eval_rule("pathf", &merged));
        assert!(!eval_rule("tokio", &merged));

        // 2. Explicit `key = true`.
        let mut answers = HashMap::new();
        answers.insert("pathf".into(), "true".into());
        answers.insert("tokio".into(), "true".into());
        let merged = resolve_answers(&answers, &rules);
        assert!(eval_rule("pathf", &merged));
        assert!(eval_rule("tokio", &merged));

        // 3. Explicit `key = false` overrides the declared default.
        let mut answers = HashMap::new();
        answers.insert("pathf".into(), "false".into());
        let merged = resolve_answers(&answers, &rules);
        assert_eq!(merged.get("pathf").unwrap(), "false");
        assert!(!eval_rule("pathf", &merged));
    }

    #[test]
    fn eval_bare_identifier() {
        let mut answers = HashMap::new();
        answers.insert("tokio".into(), "true".into());
        assert!(eval_rule("tokio", &answers));
        assert!(!eval_rule("async_std", &answers));
    }

    #[test]
    fn eval_false_value_is_falsy() {
        let mut answers = HashMap::new();
        answers.insert("pathf".into(), "false".into());
        assert!(!eval_rule("pathf", &answers));

        // Empty string is also falsy.
        answers.insert("empty".into(), "".into());
        assert!(!eval_rule("empty", &answers));
    }

    #[test]
    fn eval_boolean_operators() {
        let mut answers = HashMap::new();
        answers.insert("tokio".into(), "true".into());
        answers.insert("completion".into(), "true".into());
        answers.insert("pathf".into(), "false".into());

        assert!(eval_rule("tokio || async_std", &answers));
        assert!(!eval_rule("async_std && tokio", &answers));
        assert!(eval_rule("!async_std", &answers));
        // completion is true, so `!completion && !pathf` is false
        assert!(!eval_rule("!completion && !pathf", &answers));
        // with neither key present the same rule becomes true
        assert!(eval_rule("!completion && !pathf", &HashMap::new()));
        assert!(eval_rule("(tokio || async_std) && completion", &answers));
        assert!(!eval_rule("(async_std || tokio) && !completion", &answers));
    }

    #[test]
    fn eval_equality_comparison() {
        let mut answers = HashMap::new();
        answers.insert("parser".into(), "picker".into());
        answers.insert("use_parser".into(), "true".into());

        assert!(eval_rule("parser == picker", &answers));
        assert!(!eval_rule("parser == clap", &answers));
        assert!(eval_rule("use_parser && parser == picker", &answers));
    }

    #[test]
    fn eval_not_equal_comparison() {
        let mut answers = HashMap::new();
        answers.insert("parser".into(), "picker".into());
        answers.insert("use_parser".into(), "true".into());

        assert!(!eval_rule("parser != picker", &answers));
        assert!(eval_rule("parser != clap", &answers));

        // The template's NOT_PARSER_PICKER rule.
        assert!(!eval_rule("!use_parser || parser != picker", &answers));
        answers.remove("use_parser");
        assert!(eval_rule("!use_parser || parser != picker", &answers));
    }

    #[test]
    fn eval_rejects_trailing_garbage() {
        // Unsupported tokens must invalidate the expression instead of being
        // silently ignored.
        let mut answers = HashMap::new();
        answers.insert("parser".into(), "picker".into());
        assert!(!eval_rule("parser >>> picker", &answers));
        assert!(!eval_rule("parser ||", &answers));
    }
}
