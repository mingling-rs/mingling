/// Provides a description for any Grouped type.
pub struct Description {
    desc: String,
}

impl Description {
    /// Creates a new `Description` instance.
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
