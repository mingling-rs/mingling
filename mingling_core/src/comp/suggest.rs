#![allow(deprecated)]

use std::collections::BTreeSet;

use crate::ShellContext;

/// A completion suggestion that tells the shell how to perform command completion.
/// It can be a set of concrete suggestion items, or a file completion request.
///
/// This enum has two variants:
/// - `Suggest(BTreeSet<SuggestItem>)`: Contains a set of concrete completion suggestion items that the shell displays for the user to choose from.
/// - `FileCompletion`: Requests the shell to perform file path completion (e.g., automatically completing filenames while typing a path).
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "structural_renderer", derive(serde::Serialize))]
pub enum Suggest {
    /// A set of concrete completion suggestion items for the shell to display to the user.
    /// Each suggestion item can be a simple string or include a description.
    /// Uses a `BTreeSet` to ensure suggestions are sorted by text order and contain no duplicates.
    Suggest(BTreeSet<SuggestItem>),

    /// Requests the shell to perform file path completion.
    /// This is the default completion method, used when a command has no explicit completion rules.
    #[default]
    FileCompletion,
}

impl Suggest {
    /// Creates a new `Suggest` variant containing an empty `BTreeSet` of suggestions.
    ///
    /// # Returns
    ///
    /// Returns `Suggest::Suggest(BTreeSet::new())`, i.e., an empty suggestion set.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "comp")] {
    /// # use mingling_core::Suggest;
    /// let suggest = Suggest::new();
    /// assert_eq!(suggest, Suggest::Suggest(std::collections::BTreeSet::new()));
    /// # }
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self::Suggest(BTreeSet::new())
    }

    /// Creates a `FileCompletion` variant.
    ///
    /// # Returns
    ///
    /// Returns `Suggest::FileCompletion`, requesting the shell to perform file path completion.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "comp")] {
    /// # use mingling_core::Suggest;
    /// let suggest = Suggest::file_comp();
    /// assert_eq!(suggest, Suggest::FileCompletion);
    /// # }
    /// ```
    #[must_use]
    pub const fn file_comp() -> Self {
        Self::FileCompletion
    }

    /// Filters out already typed flag arguments from suggestion results.
    ///
    /// # Deprecation
    ///
    /// When using the `picker` feature, this method does not work under all
    /// `ParserStyle` settings and should be avoided in favor of alternative
    /// completion filtering approaches.
    #[must_use]
    #[cfg_attr(
        feature = "picker",
        deprecated(
            note = "When using the `picker` feature, this method does not work under all ParserStyle settings"
        )
    )]
    pub fn strip_typed_argument(self, ctx: &ShellContext) -> Self {
        ctx.strip_typed_argument(self)
    }

    /// Combines two `Suggest` values.
    ///
    /// If both values are `Suggest::Suggest`, their `BTreeSet`s are merged
    /// (all items from `other` are added into `self`). Otherwise, the first
    /// `Suggest::Suggest` (or `FileCompletion`) is returned unchanged.
    ///
    /// # Returns
    ///
    /// Returns a new `Suggest` value. If both `self` and `other` are
    /// `Suggest::Suggest`, the resulting `Suggest::Suggest` contains the union
    /// of both suggestion sets. If `self` is `Suggest::FileCompletion`, it is
    /// returned unchanged, regardless of `other`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "comp")] {
    /// # use mingling_core::Suggest;
    /// let a: Suggest = ["foo", "bar"].into();
    /// let b: Suggest = ["baz"].into();
    /// let combined = a.clone().combine(b);
    /// match combined {
    ///     Suggest::Suggest(set) => {
    ///         assert_eq!(set.len(), 3);
    ///         assert!(set.contains(&"foo".to_string().into()));
    ///         assert!(set.contains(&"bar".to_string().into()));
    ///         assert!(set.contains(&"baz".to_string().into()));
    ///     }
    ///     Suggest::FileCompletion => panic!("expected Suggest variant"),
    /// }
    ///
    /// // FileCompletion is returned unchanged.
    /// let combined = Suggest::FileCompletion.combine(a);
    /// assert_eq!(combined, Suggest::FileCompletion);
    /// # }
    /// ```
    #[must_use]
    pub fn combine(self, other: impl Into<Self>) -> Self {
        let other = other.into();
        match (self, other) {
            (Self::Suggest(suggest), Self::Suggest(other)) => {
                Self::Suggest(suggest.into_iter().chain(other).collect())
            }
            (suggest, _) => suggest,
        }
    }

    /// Adds multiple simple suggestions (without descriptions) to the `Suggest` set.
    ///
    /// Each item produced by the iterator is wrapped in a [`SuggestItem::Simple`]
    /// variant and inserted into the underlying `BTreeSet`.
    ///
    /// # Arguments
    ///
    /// * `items` — A collection of suggestion strings to add.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "comp")] {
    /// # use mingling_core::Suggest;
    /// let mut suggest = Suggest::new();
    /// suggest.add_suggest(vec!["foo".to_string(), "bar".to_string()]);
    /// match suggest {
    ///     Suggest::Suggest(set) => {
    ///         assert_eq!(set.len(), 2);
    ///         assert!(set.contains(&"foo".to_string().into()));
    ///         assert!(set.contains(&"bar".to_string().into()));
    ///     }
    ///     Suggest::FileCompletion => panic!("expected Suggest variant"),
    /// }
    /// # }
    /// ```
    pub fn add_suggest(&mut self, items: impl Into<Vec<String>>) {
        for item in items.into() {
            self.insert(SuggestItem::Simple(item));
        }
    }

    /// Adds multiple suggestions with a shared description to the `Suggest` set.
    ///
    /// Each item produced by the iterator is wrapped in a
    /// [`SuggestItem::WithDescription`] variant using the provided description,
    /// and inserted into the underlying `BTreeSet`.
    ///
    /// # Arguments
    ///
    /// * `items` — A collection of suggestion strings to add.
    /// * `desc` — The description to attach to each suggestion. Must implement
    ///   `Into<String>`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "comp")] {
    /// # use mingling_core::{Suggest, SuggestItem};
    /// let mut suggest = Suggest::new();
    /// suggest.add_suggest_with_description(
    ///     vec!["--foo".to_string(), "--bar".to_string()],
    ///     "Sets the option",
    /// );
    /// match suggest {
    ///     Suggest::Suggest(set) => {
    ///         assert_eq!(set.len(), 2);
    ///         assert!(set.contains(&SuggestItem::new_with_desc("--foo".to_string(), "Sets the option".to_string())));
    ///         assert!(set.contains(&SuggestItem::new_with_desc("--bar".to_string(), "Sets the option".to_string())));
    ///     }
    ///     Suggest::FileCompletion => panic!("expected Suggest variant"),
    /// }
    /// # }
    /// ```
    pub fn add_suggest_with_description(
        &mut self,
        items: impl Into<Vec<String>>,
        desc: impl Into<String>,
    ) {
        let desc_str = desc.into();
        for item in items.into() {
            self.insert(SuggestItem::WithDescription(item, desc_str.clone()));
        }
    }

    /// Adds a prefix to every suggestion in the `Suggest` set.
    ///
    /// This method takes the current `Suggest` value and prepends the given
    /// prefix to the suggestion text of each item. If the `Suggest` value is
    /// [`Suggest::FileCompletion`], it is returned unchanged.
    ///
    /// # Arguments
    ///
    /// * `prefix` — The string to prepend to each suggestion. Must implement
    ///   `Into<String>`.
    ///
    /// # Returns
    ///
    /// A new `Suggest` value where each item's suggestion text is prefixed
    /// with the given string. For example, `["foo", "bar"]` with prefix `"--"`
    /// becomes `["--foo", "--bar"]`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "comp")] {
    /// # use mingling_core::Suggest;
    /// let suggest: Suggest = ["foo", "bar"].into();
    /// let prefixed = suggest.add_prefix("--");
    /// match prefixed {
    ///     Suggest::Suggest(set) => {
    ///         assert_eq!(set.len(), 2);
    ///         assert!(set.contains(&"--foo".to_string().into()));
    ///         assert!(set.contains(&"--bar".to_string().into()));
    ///     }
    ///     Suggest::FileCompletion => panic!("expected Suggest variant"),
    /// }
    ///
    /// // FileCompletion is returned unchanged.
    /// let unchanged = Suggest::FileCompletion.add_prefix("--");
    /// assert_eq!(unchanged, Suggest::FileCompletion);
    /// # }
    /// ```
    #[must_use]
    pub fn add_prefix(self, prefix: impl Into<String>) -> Self {
        let suggest = match self {
            Self::Suggest(s) => s,
            Self::FileCompletion => return Self::FileCompletion,
        };
        let prefix = prefix.into();
        let prefixed = suggest
            .into_iter()
            .map(|item| {
                let mut new_item = item;
                new_item.set_suggest(format!("{}{}", prefix, new_item.suggest()));
                new_item
            })
            .collect();
        Self::Suggest(prefixed)
    }

    /// Appends a suffix to every suggestion in the `Suggest` set.
    ///
    /// This method takes the current `Suggest` value and appends the given
    /// suffix to the suggestion text of each item. If the `Suggest` value is
    /// [`Suggest::FileCompletion`], it is returned unchanged.
    ///
    /// # Arguments
    ///
    /// * `suffix` — The string to append to each suggestion. Must implement
    ///   `Into<String>`.
    ///
    /// # Returns
    ///
    /// A new `Suggest` value where each item's suggestion text is suffixed
    /// with the given string. For example, `["foo", "bar"]` with suffix `"="`
    /// becomes `["foo=", "bar="]`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "comp")] {
    /// # use mingling_core::Suggest;
    /// let suggest: Suggest = ["foo", "bar"].into();
    /// let suffixed = suggest.add_suffix("=");
    /// match suffixed {
    ///     Suggest::Suggest(set) => {
    ///         assert_eq!(set.len(), 2);
    ///         assert!(set.contains(&"foo=".to_string().into()));
    ///         assert!(set.contains(&"bar=".to_string().into()));
    ///     }
    ///     Suggest::FileCompletion => panic!("expected Suggest variant"),
    /// }
    ///
    /// // FileCompletion is returned unchanged.
    /// let unchanged = Suggest::FileCompletion.add_suffix("=");
    /// assert_eq!(unchanged, Suggest::FileCompletion);
    /// # }
    /// ```
    #[must_use]
    pub fn add_suffix(self, suffix: impl Into<String>) -> Self {
        let suggest = match self {
            Self::Suggest(s) => s,
            Self::FileCompletion => return Self::FileCompletion,
        };
        let suffix = suffix.into();
        let suffixed = suggest
            .into_iter()
            .map(|item| {
                let mut new_item = item;
                new_item.set_suggest(format!("{}{}", new_item.suggest(), suffix));
                new_item
            })
            .collect();
        Self::Suggest(suffixed)
    }
}

impl<T> From<T> for Suggest
where
    T: IntoIterator,
    T::Item: Into<SuggestItem>,
{
    fn from(items: T) -> Self {
        let suggests = items.into_iter().map(Into::into).collect();
        Self::Suggest(suggests)
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

/// Represents a single shell completion suggestion item.
///
/// This enum contains two variants:
/// - `Simple(String)`: Contains only the suggestion text, with no accompanying description.
/// - `WithDescription(String, String)`: Contains the suggestion text and a corresponding description.
///
/// The meaning of the parameters in both variants is as follows:
/// - The first `String` (the only parameter in `Simple`, and the first parameter in
///   `WithDescription`) always represents the suggestion text — the string that will be
///   inserted into the command line when the user selects it.
/// - The second `String` in `WithDescription` represents the optional description for the
///   suggestion, used to show the user the purpose or meaning of the option, helping them
///   make a choice from the completion list.
///
/// ## Ordering behavior
///
/// `SuggestItem` implements `Ord` and `PartialOrd`, ordering solely by the suggestion
/// text (`suggest()`) in lexicographic order; the `description` does not participate in
/// the ordering comparison. This allows `BTreeSet<SuggestItem>` to ensure suggestions are
/// de-duplicated and sorted by text order.
///
/// ## Behavior under the `structural_renderer` feature
///
/// When the `structural_renderer` feature is enabled, `SuggestItem` derives
/// `serde::Serialize`, allowing completion suggestion items to be serialized into JSON or
/// other supported structured formats. The serialized structure depends on the variant:
///
/// - `Simple(text)` serializes as a string containing the `text` field (or an object,
///   depending on the serialization context).
/// - `WithDescription(text, desc)` serializes as an object containing both `text` and
///   `desc` fields, allowing front-end renderers to display both the text and description
///   when presenting the completion list.
///
/// This feature is primarily used for graphical or rich-text shell interfaces (such as
/// web-based terminal emulators), in order to transmit completion suggestions as
/// structured data to the rendering layer.
///
/// # Examples
///
/// ```
/// # use mingling_core::SuggestItem;
/// // Simple suggestion item, no description
/// let simple = SuggestItem::new("--help".to_string());
///
/// // Suggestion with a description
/// let with_desc = SuggestItem::new_with_desc(
///     "--verbose".to_string(),
///     "Output detailed log information".to_string(),
/// );
///
/// assert_eq!(simple.suggest(), &"--help".to_string());
/// assert_eq!(with_desc.suggest(), &"--verbose".to_string());
/// assert_eq!(with_desc.description(), Some(&"Output detailed log information".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "structural_renderer", derive(serde::Serialize))]
pub enum SuggestItem {
    /// A simple suggestion item containing only the suggestion text.
    ///
    /// The `String` parameter represents the suggestion text — the string that will be
    /// inserted into the command line when the user selects it.
    /// This variant has no description, and is suitable for completion options that do
    /// not require additional explanation (such as file names or simple commands).
    Simple(String),

    /// A suggestion item containing both suggestion text and a description.
    ///
    /// - The first `String`: the suggestion text — the string that will be inserted into
    ///   the command line when the user selects it.
    /// - The second `String`: the description for this suggestion, used to show the user
    ///   the purpose or meaning of the option.
    ///
    /// This variant is suitable for completion options that need to provide additional
    /// context to the user (such as long options with explanations like `--flag`).
    WithDescription(String, String),
}

impl Default for SuggestItem {
    fn default() -> Self {
        Self::Simple(String::new())
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
    ///
    /// # Arguments
    ///
    /// * `suggest` — The suggestion text to store in this `SuggestItem`.
    ///
    /// # Returns
    ///
    /// Returns a [`SuggestItem::Simple`] containing the given suggestion text.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::SuggestItem;
    /// let item = SuggestItem::new("--help".to_string());
    /// assert_eq!(item.suggest(), &"--help".to_string());
    /// ```
    #[must_use]
    pub const fn new(suggest: String) -> Self {
        Self::Simple(suggest)
    }

    /// Creates a new suggestion with a description.
    ///
    /// # Arguments
    ///
    /// * `suggest` — The suggestion text to store in this `SuggestItem`.
    /// * `description` — The description for this suggestion, used to show the user the
    ///   purpose or meaning of the option.
    ///
    /// # Returns
    ///
    /// Returns a [`SuggestItem::WithDescription`] containing the given suggestion text
    /// and description.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::SuggestItem;
    /// let item = SuggestItem::new_with_desc(
    ///     "--verbose".to_string(),
    ///     "Output detailed log information".to_string(),
    /// );
    /// assert_eq!(item.suggest(), &"--verbose".to_string());
    /// assert_eq!(item.description(), Some(&"Output detailed log information".to_string()));
    /// ```
    #[must_use]
    pub const fn new_with_desc(suggest: String, description: String) -> Self {
        Self::WithDescription(suggest, description)
    }

    /// Adds a description to this suggestion, replacing any existing description.
    ///
    /// # Arguments
    ///
    /// * `description` — The new description to attach to this suggestion. Must implement
    ///   `Into<String>`.
    ///
    /// # Returns
    ///
    /// Returns a new `SuggestItem` with the given description. If this was previously a
    /// [`SuggestItem::Simple`] variant, it is converted to
    /// [`SuggestItem::WithDescription`] with the original suggestion text preserved.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::SuggestItem;
    /// let item = SuggestItem::new("--help".to_string()).with_desc("Show help message".to_string());
    /// assert_eq!(item.suggest(), &"--help".to_string());
    /// assert_eq!(item.description(), Some(&"Show help message".to_string()));
    /// ```
    #[must_use]
    pub fn with_desc(self, description: String) -> Self {
        match self {
            Self::Simple(suggest) | Self::WithDescription(suggest, _) => {
                Self::WithDescription(suggest, description)
            }
        }
    }

    /// Returns the suggestion text.
    ///
    /// The suggestion text is the string that will be inserted into the command line when
    /// the user selects this completion item. Both the [`SuggestItem::Simple`] and
    /// [`SuggestItem::WithDescription`] variants contain a suggestion text, so this method
    /// works uniformly on both variants.
    ///
    /// # Returns
    ///
    /// Returns `&String` referencing the suggestion text contained in this `SuggestItem`.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::SuggestItem;
    /// // Simple item
    /// let simple = SuggestItem::new("--help".to_string());
    /// assert_eq!(simple.suggest(), &"--help".to_string());
    ///
    /// // Item with description
    /// let with_desc = SuggestItem::new_with_desc(
    ///     "--verbose".to_string(),
    ///     "Output detailed log information".to_string(),
    /// );
    /// assert_eq!(with_desc.suggest(), &"--verbose".to_string());
    /// ```
    #[must_use]
    pub const fn suggest(&self) -> &String {
        match self {
            Self::Simple(suggest) | Self::WithDescription(suggest, _) => suggest,
        }
    }

    /// Updates the suggestion text.
    ///
    /// This method replaces the suggestion text of the [`SuggestItem`] with the provided
    /// string. It works uniformly on both the [`SuggestItem::Simple`] and
    /// [`SuggestItem::WithDescription`] variants, updating only the suggestion text and
    /// leaving any existing description unchanged.
    ///
    /// # Arguments
    ///
    /// * `new_suggest` — The new suggestion text to set. Must implement `Into<String>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::SuggestItem;
    /// let mut item = SuggestItem::new("--help".to_string());
    /// item.set_suggest("--verbose".to_string());
    /// assert_eq!(item.suggest(), &"--verbose".to_string());
    /// ```
    pub fn set_suggest(&mut self, new_suggest: String) {
        match self {
            Self::Simple(suggest) | Self::WithDescription(suggest, _) => *suggest = new_suggest,
        }
    }

    /// Returns the description if present.
    ///
    /// # Returns
    ///
    /// Returns `Some(&String)` containing the description if this item is a
    /// [`SuggestItem::WithDescription`] variant; returns `None` if this item
    /// is a [`SuggestItem::Simple`] variant (i.e. no description is attached).
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::SuggestItem;
    /// // Simple item has no description.
    /// let simple = SuggestItem::new("--help".to_string());
    /// assert_eq!(simple.description(), None);
    ///
    /// // Item with description returns it.
    /// let with_desc = SuggestItem::new_with_desc(
    ///     "--verbose".to_string(),
    ///     "Output detailed log information".to_string(),
    /// );
    /// assert_eq!(with_desc.description(), Some(&"Output detailed log information".to_string()));
    /// ```
    #[must_use]
    pub const fn description(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::WithDescription(_, description) => Some(description),
        }
    }

    /// Sets or replaces the description.
    ///
    /// This method sets the description of the [`SuggestItem`]. If this item is a
    /// [`SuggestItem::Simple`] variant, it is converted to
    /// [`SuggestItem::WithDescription`] with the original suggestion text preserved.
    /// If this item is already a [`SuggestItem::WithDescription`] variant, the
    /// existing description is replaced.
    ///
    /// # Arguments
    ///
    /// * `description` — The new description to set. Must implement `Into<String>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::SuggestItem;
    /// // On a simple item
    /// let mut item = SuggestItem::new("--help".to_string());
    /// item.set_description("Show help message".to_string());
    /// assert_eq!(item.description(), Some(&"Show help message".to_string()));
    ///
    /// // Replacing an existing description
    /// let mut item = SuggestItem::new_with_desc(
    ///     "--verbose".to_string(),
    ///     "Old description".to_string(),
    /// );
    /// item.set_description("New description".to_string());
    /// assert_eq!(item.description(), Some(&"New description".to_string()));
    /// ```
    pub fn set_description(&mut self, description: String) {
        match self {
            Self::Simple(suggest) => *self = Self::WithDescription(suggest.clone(), description),
            Self::WithDescription(_, desc) => *desc = description,
        }
    }

    /// Removes and returns the description if present.
    ///
    /// If this item is a [`SuggestItem::WithDescription`] variant, the description
    /// is removed and returned, and the item is converted to a
    /// [`SuggestItem::Simple`] variant containing the same suggestion text. If
    /// this item is already a [`SuggestItem::Simple`] variant, `None` is returned
    /// and the item is left unchanged.
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` containing the removed description if this item
    /// had a description; returns `None` if this item had no description.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::SuggestItem;
    /// // Item with a description
    /// let mut item = SuggestItem::new_with_desc(
    ///     "--verbose".to_string(),
    ///     "Output detailed log information".to_string(),
    /// );
    /// assert_eq!(item.remove_desc(), Some("Output detailed log information".to_string()));
    /// assert!(matches!(item, SuggestItem::Simple(ref s) if s == "--verbose"));
    ///
    /// // Item without a description
    /// let mut item = SuggestItem::new("--help".to_string());
    /// assert_eq!(item.remove_desc(), None);
    /// ```
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

impl From<&str> for SuggestItem {
    fn from(suggest: &str) -> Self {
        Self::new(suggest.to_string())
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
