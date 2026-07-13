use std::ops::Index;

mod patterns;
pub use patterns::*;

/// Picker, used to record all states of a parameter parsing
///
/// Includes the following:
///
/// - Basic arguments
/// - Parsing states
/// - Parsing results
pub struct Picker<'a> {
    /// Internal arguments of Picker
    args: PickerArguments<'a>,
}

/// Internal arguments of Picker
///
/// - `Slice` - borrowed slice of string slices
/// - `Vec` - owned vector of borrowed string slices
/// - `Owned` - owned vector of owned strings
pub enum PickerArguments<'a> {
    /// Borrowed slice of string slices
    Slice(&'a [&'a str]),
    /// Owned vector of borrowed string slices
    Vec(Vec<&'a str>),
    /// Owned vector of owned strings
    Owned(Vec<String>),
}

impl<'a> From<&'a [&'a str]> for Picker<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        Picker {
            args: PickerArguments::Slice(value),
        }
    }
}

impl<'a> From<Vec<&'a str>> for Picker<'a> {
    fn from(value: Vec<&'a str>) -> Self {
        Picker {
            args: PickerArguments::Vec(value),
        }
    }
}

impl<'a> From<Vec<String>> for Picker<'a> {
    fn from(value: Vec<String>) -> Self {
        Picker {
            args: PickerArguments::Owned(value),
        }
    }
}

impl<'a> Picker<'a> {
    /// Returns the number of arguments in the picker.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::Picker;
    ///
    /// let args = Picker::from(&["hello", "world"][..]);
    /// assert_eq!(args.len(), 2);
    ///
    /// let args = Picker::from(vec!["foo", "bar"]);
    /// assert_eq!(args.len(), 2);
    ///
    /// let args = Picker::from(vec!["a".to_string(), "b".to_string()]);
    /// assert_eq!(args.len(), 2);
    ///
    /// let empty: Picker = Picker::from(&[][..]);
    /// assert_eq!(empty.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        match &self.args {
            PickerArguments::Slice(items) => items.len(),
            PickerArguments::Vec(items) => items.len(),
            PickerArguments::Owned(items) => items.len(),
        }
    }

    /// Returns `true` if the picker contains no arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::Picker;
    ///
    /// let empty: Picker = Picker::from(&[][..]);
    /// assert!(empty.is_empty());
    ///
    /// let args = Picker::from(&["hello"][..]);
    /// assert!(!args.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the arguments, yielding owned `String` values.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::Picker;
    ///
    /// let args = Picker::from(&["hello", "world"][..]);
    /// let collected: Vec<String> = args.iter().collect();
    /// assert_eq!(collected, vec!["hello", "world"]);
    ///
    /// let args = Picker::from(vec!["foo", "bar"]);
    /// let collected: Vec<String> = args.iter().collect();
    /// assert_eq!(collected, vec!["foo", "bar"]);
    ///
    /// let args = Picker::from(vec!["a".to_string(), "b".to_string()]);
    /// let collected: Vec<String> = args.iter().collect();
    /// assert_eq!(collected, vec!["a", "b"]);
    /// ```
    pub fn iter(&'a self) -> PickerIter<'a> {
        match &self.args {
            PickerArguments::Slice(items) => PickerIter::Slice(items.iter()),
            PickerArguments::Vec(items) => PickerIter::Vec(items.iter()),
            PickerArguments::Owned(items) => PickerIter::Owned(items.iter()),
        }
    }
}

impl<'a> Index<usize> for Picker<'a> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.args {
            PickerArguments::Slice(items) => items[index],
            PickerArguments::Vec(items) => items[index],
            PickerArguments::Owned(items) => &items[index],
        }
    }
}

impl<'a> Index<usize> for &Picker<'a> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.args {
            PickerArguments::Slice(items) => items[index],
            PickerArguments::Vec(items) => items[index],
            PickerArguments::Owned(items) => &items[index],
        }
    }
}

impl<'a> IntoIterator for &'a Picker<'a> {
    type Item = String;
    type IntoIter = PickerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match &self.args {
            PickerArguments::Slice(items) => PickerIter::Slice(items.iter()),
            PickerArguments::Vec(items) => PickerIter::Vec(items.iter()),
            PickerArguments::Owned(items) => PickerIter::Owned(items.iter()),
        }
    }
}

/// Iterator for `Picker`, yielding owned `String` values.
///
/// This enum wraps the underlying slice iterators for each variant of
/// `PickerArguments`, allowing uniform iteration over borrowed or owned data.
pub enum PickerIter<'a> {
    /// Iterates over a borrowed slice (`&[&str]`)
    Slice(std::slice::Iter<'a, &'a str>),
    /// Iterates over an owned vector of borrowed string slices (`Vec<&str>`)
    Vec(std::slice::Iter<'a, &'a str>),
    /// Iterates over an owned vector of owned strings (`Vec<String>`)
    Owned(std::slice::Iter<'a, String>),
}

impl<'a> Iterator for PickerIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PickerIter::Slice(iter) => iter.next().map(|s| s.to_string()),
            PickerIter::Vec(iter) => iter.next().map(|s| s.to_string()),
            PickerIter::Owned(iter) => iter.next().cloned(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            PickerIter::Slice(iter) => iter.size_hint(),
            PickerIter::Vec(iter) => iter.size_hint(),
            PickerIter::Owned(iter) => iter.size_hint(),
        }
    }
}

impl<'a> ExactSizeIterator for PickerIter<'a> {}

/// Trait for converting types into a `Picker`
///
/// Implemented for:
/// - `&[&str]` (borrowed slice)
/// - `Vec<&str>` (owned vector of borrowed strings)
/// - `Vec<String>` (owned vector of owned strings)
pub trait IntoPicker<'a> {
    /// Converts the value into a `Picker`
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::{IntoPicker, Picker};
    ///
    /// let args: Picker = (&["hello", "world"][..]).to_picker();
    /// assert_eq!(args.len(), 2);
    ///
    /// let args: Picker = vec!["foo", "bar"].to_picker();
    /// assert_eq!(args.len(), 2);
    ///
    /// let args: Picker = vec!["a".to_string(), "b".to_string()].to_picker();
    /// assert_eq!(args.len(), 2);
    /// ```
    fn to_picker(self) -> Picker<'a>;
}

impl<'a> IntoPicker<'a> for &'a [&'a str] {
    fn to_picker(self) -> Picker<'a> {
        Picker {
            args: PickerArguments::Slice(self),
        }
    }
}

impl<'a> IntoPicker<'a> for Vec<&'a str> {
    fn to_picker(self) -> Picker<'a> {
        Picker {
            args: PickerArguments::Vec(self),
        }
    }
}

impl<'a> IntoPicker<'a> for Vec<String> {
    fn to_picker(self) -> Picker<'a> {
        Picker {
            args: PickerArguments::Owned(self),
        }
    }
}
