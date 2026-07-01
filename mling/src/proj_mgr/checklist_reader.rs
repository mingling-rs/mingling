use std::{collections::HashMap, io, path::Path};

/// Reads and parses a `CHECKLIST.md` file, extracting both *values* (from
/// fenced code blocks) and *toggles* (from checkbox lines).
///
/// # Format
///
/// - **Values**: fenced code blocks where the info string is the key and
///   the first non-empty line is the value.  Example:
///   <code>\`\`\`name<br>my-cli<br>\`\`\`</code>
/// - **Toggles**: checkbox lines like `- [x] \`ns:key\`` (checked) or
///   `- [ ] \`ns:key\`` (unchecked).
///
/// Checked items are stored as `"namespace:key"`.
///
/// # Usage
///
/// ```rust,ignore
/// let reader = CheckListReader::from(&Path::new("CHECKLIST.md")).unwrap();
/// assert_eq!(reader.read_value("name"), Some("my-cli".into()));
/// assert!(reader.read_toggle("ver:0.2"));
/// assert!(!reader.read_toggle("feat:comp"));
/// assert_eq!(reader.read_toggles("feat"), vec!["feat:async"]);
/// ```
pub struct CheckListReader {
    /// Values extracted from fenced code blocks: `key → value`.
    values: HashMap<String, String>,

    /// Checked toggles: `"namespace:key" → true` (only checked items are stored).
    toggles: HashMap<String, bool>,

    /// All toggles (checked or not): `"namespace:key" → checked`.
    all_toggles: HashMap<String, bool>,
}

impl CheckListReader {
    /// Parse a CHECKLIST.md from a file path in a single left-to-right pass
    pub fn from(path: &Path) -> Result<Self, io::Error> {
        let content = std::fs::read_to_string(path)?;

        let mut reader = Self {
            values: HashMap::new(),
            toggles: HashMap::new(),
            all_toggles: HashMap::new(),
        };
        reader.parse(&content);
        Ok(reader)
    }

    /// Parse CHECKLIST.md content in a single pass.
    fn parse(&mut self, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            if let Some(key) = line.strip_prefix("```").map(|s| s.trim())
                && !key.is_empty()
                && !key.starts_with(' ')
            {
                let mut block_lines = Vec::new();
                i += 1;
                while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                    block_lines.push(lines[i]);
                    i += 1;
                }
                let value = block_lines
                    .into_iter()
                    .find(|l| !l.trim().is_empty())
                    .map(|l| l.trim().to_string());
                if let Some(val) = value {
                    self.values.insert(key.to_string(), val);
                }
                continue;
            }

            if let Some(toggle_key) = Self::parse_toggle_line(line) {
                let is_checked = line.contains("[x]") || line.contains("[X]");
                self.all_toggles.insert(toggle_key.clone(), is_checked);
                if is_checked {
                    self.toggles.insert(toggle_key, true);
                }
            }

            i += 1;
        }
    }

    /// Extract the `namespace:key` from a toggle line.
    ///
    /// Matches: `- [x] `namespace:key`` or `- [ ] `namespace:key``
    fn parse_toggle_line(line: &str) -> Option<String> {
        let line = line.trim();
        if !line.starts_with("- [") {
            return None;
        }
        // Find the backtick-enclosed key
        let start = line.find('`')?;
        let rest = &line[start + 1..];
        let end = rest.find('`')?;
        let key = rest[..end].trim();
        if key.is_empty() {
            return None;
        }
        Some(key.to_string())
    }

    /// Read a value by its key.
    ///
    /// Returns `Some(value)` if a fenced code block with that key was found,
    /// or `None` if the key does not exist.
    #[must_use]
    pub fn read_value(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    /// Check if a toggle (`namespace:key`) is checked.
    ///
    /// Returns `true` if the checkbox line was `- [x]`, `false` if it was
    /// `- [ ]` or the key does not exist in the file.
    #[must_use]
    pub fn read_toggle(&self, key: &str) -> bool {
        self.toggles.contains_key(key)
    }

    /// Get all toggles under a given `namespace` that are checked.
    ///
    /// For example, `read_toggles("feat")` returns all checked keys starting
    /// with `"feat:"`, such as `["feat:comp", "feat:parser"]`.
    #[must_use]
    pub fn read_toggles(&self, namespace: &str) -> Vec<String> {
        let prefix = format!("{namespace}:");
        let mut keys: Vec<String> = self
            .toggles
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        keys.sort();
        keys
    }

    /// Get ALL toggles (checked or not) under a namespace.
    ///
    /// Useful for iterating all available options.
    #[must_use]
    pub fn all_toggles(&self, namespace: &str) -> Vec<String> {
        let prefix = format!("{namespace}:");
        let mut keys: Vec<String> = self
            .all_toggles
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        keys.sort();
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_md() -> &'static str {
        r#"> Some intro

## Question 1: What is your project name?

```name
my-cli
```

## Question 2: Which version?

- [x] `ver:0.2`

## Question 3: Features?

- [ ] `feat:structural_renderer`
- [x] `feat:comp`
- [x] `feat:parser`
- [ ] `feat:async`

```other
some value
```
"#
    }

    #[test]
    fn test_read_value() {
        let mut reader = CheckListReader {
            values: HashMap::new(),
            toggles: HashMap::new(),
            all_toggles: HashMap::new(),
        };
        reader.parse(sample_md());
        assert_eq!(reader.read_value("name"), Some("my-cli".into()));
        assert_eq!(reader.read_value("other"), Some("some value".into()));
        assert_eq!(reader.read_value("nonexistent"), None);
    }

    #[test]
    fn test_read_toggle() {
        let mut reader = CheckListReader {
            values: HashMap::new(),
            toggles: HashMap::new(),
            all_toggles: HashMap::new(),
        };
        reader.parse(sample_md());
        assert!(reader.read_toggle("ver:0.2"));
        assert!(reader.read_toggle("feat:comp"));
        assert!(reader.read_toggle("feat:parser"));
        assert!(!reader.read_toggle("feat:structural_renderer"));
        assert!(!reader.read_toggle("feat:async"));
        assert!(!reader.read_toggle("nonexistent:key"));
    }

    #[test]
    fn test_read_toggles() {
        let mut reader = CheckListReader {
            values: HashMap::new(),
            toggles: HashMap::new(),
            all_toggles: HashMap::new(),
        };
        reader.parse(sample_md());
        let feat_toggles = reader.read_toggles("feat");
        assert_eq!(feat_toggles, vec!["feat:comp", "feat:parser"]);
    }

    #[test]
    fn test_all_toggles() {
        let mut reader = CheckListReader {
            values: HashMap::new(),
            toggles: HashMap::new(),
            all_toggles: HashMap::new(),
        };
        reader.parse(sample_md());
        let all = reader.all_toggles("feat");
        assert_eq!(
            all,
            vec![
                "feat:async",
                "feat:comp",
                "feat:parser",
                "feat:structural_renderer",
            ]
        );
    }

    #[test]
    fn test_from_path() {
        // Write a temp CHECKLIST.md, read it, then clean up
        let dir = std::env::temp_dir().join("mling_checklist_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("CHECKLIST.md");
        std::fs::write(&path, sample_md()).unwrap();

        let reader = CheckListReader::from(&path).unwrap();
        assert_eq!(reader.read_value("name"), Some("my-cli".into()));
        assert!(reader.read_toggle("ver:0.2"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_file() {
        let mut reader = CheckListReader {
            values: HashMap::new(),
            toggles: HashMap::new(),
            all_toggles: HashMap::new(),
        };
        reader.parse("");
        assert_eq!(reader.read_value("anything"), None);
        assert!(!reader.read_toggle("ns:key"));
        assert!(reader.read_toggles("ns").is_empty());
    }

    #[test]
    fn test_no_toggle_or_value_lines() {
        let md = "# Just a heading\n\nSome text\n\n```code\nstill not a value\n```\n";
        let mut reader = CheckListReader {
            values: HashMap::new(),
            toggles: HashMap::new(),
            all_toggles: HashMap::new(),
        };
        reader.parse(md);
        // "code" isn't a valid value key (it's used as a language tag, not a name/value key)
        // but our parser treats ANY ```key as a value block. This is correct behavior —
        // malformed CHECKLIST.md may produce unexpected values.
        assert_eq!(reader.read_value("code"), Some("still not a value".into()));
    }
}
