/// A type that provides descriptive information for any [`Grouped`](https://docs.rs/mingling/latest/mingling/trait.Grouped.html) type.
///
/// `mingling_core::metadata::Description` is a conventional type provided by the Metadata system,
/// used to attach readable descriptive metadata to each Entry in a program. It can be attached to
/// any type that implements the `Metadata` trait, serving as the human-readable description text
/// for that Entry.
///
/// # Examples
///
/// Create using the `new` method:
///
/// ```
/// # use mingling_core::metadata::Description;
/// let desc = Description::new("This is an example description");
/// ```
///
/// Create using a `From` conversion:
///
/// ```
/// # use mingling_core::metadata::Description;
/// let desc: Description = "This is an example description".into();
/// ```
///
/// Create using a `From<String>` conversion:
///
/// ```
/// # use mingling_core::metadata::Description;
/// let desc: Description = String::from("This is an example description").into();
/// ```
pub struct Description {
    desc: String,
}

impl Description {
    /// Creates a new `Description` instance.
    ///
    /// # Arguments
    ///
    /// * `desc` - The description text, which can be any type that implements `Into<String>`
    ///   (such as `&str`, `String`, `&String`, etc.).
    ///
    /// # Returns
    ///
    /// Returns a `Description` instance containing the specified description text.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::metadata::Description;
    /// let desc = Description::new("This is an example description");
    /// ```
    pub fn new<S: Into<String>>(desc: S) -> Self {
        Self { desc: desc.into() }
    }
}

impl From<String> for Description {
    fn from(desc: String) -> Self {
        Self { desc }
    }
}

impl From<&str> for Description {
    fn from(desc: &str) -> Self {
        Self {
            desc: desc.to_string(),
        }
    }
}

impl From<Description> for String {
    fn from(desc: Description) -> Self {
        desc.desc
    }
}

impl From<&Description> for String {
    fn from(desc: &Description) -> Self {
        desc.desc.clone()
    }
}

impl std::ops::Deref for Description {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.desc
    }
}

impl std::ops::DerefMut for Description {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.desc
    }
}

impl std::fmt::Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.desc)
    }
}
