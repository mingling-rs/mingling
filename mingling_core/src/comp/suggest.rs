use std::collections::BTreeSet;

use crate::ShellContext;

/// A completion suggestion that tells the shell how to perform completion.
/// This can be either a set of specific suggestion items or a request for file completion.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "general_renderer", derive(serde::Serialize))]
pub enum Suggest {
    /// A set of specific suggestion items for the shell to display.
    Suggest(BTreeSet<SuggestItem>),

    /// A request for the shell to perform file‑path completion.
    #[default]
    FileCompletion,
}

impl Suggest {
    /// Creates a new Suggest variant containing a `BTreeSet` of suggestions.
    #[must_use]
    pub fn new() -> Self {
        Self::Suggest(BTreeSet::new())
    }

    /// Creates a `FileCompletion` variant.
    #[must_use]
    pub fn file_comp() -> Self {
        Self::FileCompletion
    }

    /// Filters out already typed flag arguments from suggestion results.
    #[must_use]
    pub fn strip_typed_argument(self, ctx: &ShellContext) -> Self {
        ctx.strip_typed_argument(self)
    }
}

impl<T> From<T> for Suggest
where
    T: IntoIterator,
    T::Item: Into<String>,
{
    fn from(items: T) -> Self {
        let suggests = items
            .into_iter()
            .map(|item| SuggestItem::new(item.into()))
            .collect();
        Suggest::Suggest(suggests)
    }
}

impl std::ops::Deref for Suggest {
    type Target = BTreeSet<SuggestItem>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Suggest(suggests) => suggests,
            Self::FileCompletion => panic!("Cannot deref FileCompletion variant"),
        }
    }
}

impl std::ops::DerefMut for Suggest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Suggest(suggests) => suggests,
            Self::FileCompletion => panic!("Cannot deref_mut FileCompletion variant"),
        }
    }
}

/// Represents a single suggestion item for shell completion.
///
/// This enum has two variants:
/// - `Simple(String)`: A suggestion without any description.
/// - `WithDescription(String, String)`: A suggestion with an associated description.
///
/// The first `String` always holds the suggestion text, and the second `String` (if present)
/// holds an optional description providing additional context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "general_renderer", derive(serde::Serialize))]
pub enum SuggestItem {
    /// A simple suggestion with only the suggestion text.
    Simple(String),
    /// A suggestion with both text and a description.
    WithDescription(String, String),
}

impl Default for SuggestItem {
    fn default() -> Self {
        SuggestItem::Simple(String::new())
    }
}

impl PartialOrd for SuggestItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SuggestItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.suggest().cmp(other.suggest())
    }
}

impl SuggestItem {
    /// Creates a new simple suggestion without description.
    #[must_use]
    pub fn new(suggest: String) -> Self {
        Self::Simple(suggest)
    }

    /// Creates a new suggestion with a description.
    #[must_use]
    pub fn new_with_desc(suggest: String, description: String) -> Self {
        Self::WithDescription(suggest, description)
    }

    /// Adds a description to this suggestion, replacing any existing description.
    #[must_use]
    pub fn with_desc(self, description: String) -> Self {
        match self {
            Self::Simple(suggest) | Self::WithDescription(suggest, _) => {
                Self::WithDescription(suggest, description)
            }
        }
    }

    /// Returns the suggestion text.
    #[must_use]
    pub fn suggest(&self) -> &String {
        match self {
            Self::Simple(suggest) | Self::WithDescription(suggest, _) => suggest,
        }
    }

    /// Updates the suggestion text.
    pub fn set_suggest(&mut self, new_suggest: String) {
        match self {
            Self::Simple(suggest) | Self::WithDescription(suggest, _) => *suggest = new_suggest,
        }
    }

    /// Returns the description if present.
    #[must_use]
    pub fn description(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::WithDescription(_, description) => Some(description),
        }
    }

    /// Sets or replaces the description.
    pub fn set_description(&mut self, description: String) {
        match self {
            Self::Simple(suggest) => *self = Self::WithDescription(suggest.clone(), description),
            Self::WithDescription(_, desc) => *desc = description,
        }
    }

    /// Removes and returns the description if present.
    pub fn remove_desc(&mut self) -> Option<String> {
        match self {
            Self::Simple(_) => None,
            Self::WithDescription(suggest, description) => {
                let desc = std::mem::take(description);
                *self = Self::Simple(std::mem::take(suggest));
                Some(desc)
            }
        }
    }
}

impl From<String> for SuggestItem {
    fn from(suggest: String) -> Self {
        Self::new(suggest)
    }
}

impl From<(String, String)> for SuggestItem {
    fn from((suggest, description): (String, String)) -> Self {
        Self::new_with_desc(suggest, description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_new_creates_empty() {
        let s = Suggest::new();
        match s {
            Suggest::Suggest(set) => assert!(set.is_empty(), "expected empty BTreeSet"),
            Suggest::FileCompletion => panic!("expected Suggest variant"),
        }
    }

    #[test]
    fn test_suggest_file_comp() {
        assert_eq!(Suggest::file_comp(), Suggest::FileCompletion);
    }

    #[test]
    fn test_from_vec_string() {
        let items = vec!["foo".to_string(), "bar".to_string()];
        let suggest: Suggest = items.into();
        match suggest {
            Suggest::Suggest(set) => {
                assert_eq!(set.len(), 2);
                assert!(set.contains(&SuggestItem::new("foo".to_string())));
                assert!(set.contains(&SuggestItem::new("bar".to_string())));
            }
            Suggest::FileCompletion => panic!("expected Suggest variant"),
        }
    }

    #[test]
    fn test_from_vec_str_ref() {
        let items = vec!["a", "b", "c"];
        let suggest: Suggest = items.into();
        match suggest {
            Suggest::Suggest(set) => {
                assert_eq!(set.len(), 3);
            }
            Suggest::FileCompletion => panic!("expected Suggest variant"),
        }
    }

    #[test]
    fn test_from_array_str_ref() {
        let items = ["x", "y", "z"];
        let suggest: Suggest = items.into();
        match suggest {
            Suggest::Suggest(set) => {
                assert_eq!(set.len(), 3);
            }
            Suggest::FileCompletion => panic!("expected Suggest variant"),
        }
    }

    #[test]
    fn test_deref_suggest() {
        let s: Suggest = ["hello"].into();
        let set: &BTreeSet<SuggestItem> = &s;
        assert_eq!(set.len(), 1);
    }

    #[test]
    #[should_panic(expected = "Cannot deref FileCompletion variant")]
    fn test_deref_file_completion_panics() {
        let s = Suggest::FileCompletion;
        let _ = &*s;
    }

    #[test]
    fn test_deref_mut_suggest() {
        let mut s = Suggest::Suggest(BTreeSet::new());
        s.insert(SuggestItem::new("inserted".to_string()));
        assert_eq!(s.len(), 1);
    }

    #[test]
    #[should_panic(expected = "Cannot deref_mut FileCompletion variant")]
    fn test_deref_mut_file_completion_panics() {
        let mut s = Suggest::FileCompletion;
        let _ = &mut *s;
    }

    #[test]
    fn test_suggest_item_new() {
        let item = SuggestItem::new("hello".to_string());
        assert!(matches!(item, SuggestItem::Simple(ref s) if s == "hello"));
    }

    #[test]
    fn test_suggest_item_new_with_desc() {
        let item = SuggestItem::new_with_desc("hello".to_string(), "desc".to_string());
        assert!(
            matches!(item, SuggestItem::WithDescription(ref s, ref d) if s == "hello" && d == "desc")
        );
    }

    #[test]
    fn test_with_desc_replaces_existing() {
        let item = SuggestItem::new_with_desc("foo".to_string(), "old".to_string())
            .with_desc("new".to_string());
        assert_eq!(item.description(), Some(&"new".to_string()));
    }

    #[test]
    fn test_with_desc_on_simple() {
        let item = SuggestItem::new("foo".to_string()).with_desc("added".to_string());
        assert_eq!(item.description(), Some(&"added".to_string()));
    }

    #[test]
    fn test_suggest_returns_text() {
        let simple = SuggestItem::new("simple".to_string());
        let desc = SuggestItem::new_with_desc("desc".to_string(), "d".to_string());
        assert_eq!(simple.suggest(), &"simple".to_string());
        assert_eq!(desc.suggest(), &"desc".to_string());
    }

    #[test]
    fn test_description() {
        let simple = SuggestItem::new("x".to_string());
        assert_eq!(simple.description(), None);

        let desc = SuggestItem::new_with_desc("x".to_string(), "y".to_string());
        assert_eq!(desc.description(), Some(&"y".to_string()));
    }

    #[test]
    fn test_set_suggest() {
        let mut item = SuggestItem::new("old".to_string());
        item.set_suggest("new".to_string());
        assert_eq!(item.suggest(), &"new".to_string());

        let mut item = SuggestItem::new_with_desc("old".to_string(), "d".to_string());
        item.set_suggest("newer".to_string());
        assert_eq!(item.suggest(), &"newer".to_string());
    }

    #[test]
    fn test_set_description_on_simple() {
        let mut item = SuggestItem::new("text".to_string());
        item.set_description("added".to_string());
        assert_eq!(item.description(), Some(&"added".to_string()));
    }

    #[test]
    fn test_set_description_replaces_existing() {
        let mut item = SuggestItem::new_with_desc("text".to_string(), "old".to_string());
        item.set_description("new".to_string());
        assert_eq!(item.description(), Some(&"new".to_string()));
    }

    #[test]
    fn test_remove_desc_on_simple() {
        let mut item = SuggestItem::new("text".to_string());
        assert_eq!(item.remove_desc(), None);
        assert!(matches!(item, SuggestItem::Simple(_)));
    }

    #[test]
    fn test_remove_desc_on_with_description() {
        let mut item = SuggestItem::new_with_desc("text".to_string(), "desc".to_string());
        let desc = item.remove_desc();
        assert_eq!(desc, Some("desc".to_string()));
        assert!(matches!(item, SuggestItem::Simple(ref s) if s == "text"));
    }

    #[test]
    fn test_ord_by_suggest_text() {
        let mut items = [
            SuggestItem::new("z".to_string()),
            SuggestItem::new("a".to_string()),
            SuggestItem::new("m".to_string()),
        ];
        items.sort();
        assert_eq!(items[0].suggest(), &"a".to_string());
        assert_eq!(items[1].suggest(), &"m".to_string());
        assert_eq!(items[2].suggest(), &"z".to_string());
    }

    #[test]
    fn test_ord_with_description() {
        let mut items = [
            SuggestItem::new_with_desc("z".to_string(), "zzz".to_string()),
            SuggestItem::new("a".to_string()),
            SuggestItem::new_with_desc("m".to_string(), "mmm".to_string()),
        ];
        items.sort();
        assert_eq!(items[0].suggest(), &"a".to_string());
        assert_eq!(items[1].suggest(), &"m".to_string());
        assert_eq!(items[2].suggest(), &"z".to_string());
    }

    #[test]
    fn test_from_string_for_suggest_item() {
        let item: SuggestItem = "test".to_string().into();
        assert!(matches!(item, SuggestItem::Simple(ref s) if s == "test"));
    }

    #[test]
    fn test_from_tuple_for_suggest_item() {
        let item: SuggestItem = ("key".to_string(), "val".to_string()).into();
        assert!(
            matches!(item, SuggestItem::WithDescription(ref s, ref d) if s == "key" && d == "val")
        );
    }

    #[test]
    fn test_default_suggest_item() {
        let item = SuggestItem::default();
        assert!(matches!(item, SuggestItem::Simple(ref s) if s.is_empty()));
    }

    #[test]
    fn test_strip_typed_argument_removes_typed() {
        let ctx = ShellContext {
            all_words: vec!["--verbose".to_string(), "--help".to_string()],
            ..ShellContext::default()
        };

        let suggest: Suggest = vec!["--verbose", "--output", "--help"].into();
        let stripped = suggest.strip_typed_argument(&ctx);

        match stripped {
            Suggest::Suggest(set) => {
                assert_eq!(set.len(), 1);
                assert!(set.contains(&SuggestItem::new("--output".to_string())));
            }
            Suggest::FileCompletion => panic!("expected Suggest variant"),
        }
    }

    #[test]
    fn test_strip_typed_argument_passes_file_completion() {
        let ctx = ShellContext {
            all_words: vec!["--verbose".to_string()],
            ..ShellContext::default()
        };

        let stripped = Suggest::FileCompletion.strip_typed_argument(&ctx);
        assert_eq!(stripped, Suggest::FileCompletion);
    }

    #[test]
    fn test_strip_typed_argument_keeps_untyped() {
        let ctx = ShellContext {
            all_words: vec!["--verbose".to_string()],
            ..ShellContext::default()
        };

        let suggest: Suggest = vec!["--output", "--help"].into();
        let stripped = suggest.strip_typed_argument(&ctx);

        match stripped {
            Suggest::Suggest(set) => {
                assert_eq!(set.len(), 2);
            }
            Suggest::FileCompletion => panic!("expected Suggest variant"),
        }
    }
}
