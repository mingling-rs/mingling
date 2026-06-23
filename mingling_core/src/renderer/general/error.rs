/// Represents an error that occurs during serialization of a general renderer.
///
/// This error stores a human-readable message describing what went wrong
/// during the serialization process.
#[derive(Debug)]
pub struct GeneralRendererSerializeError {
    /// The underlying error message.
    error: String,
}

impl GeneralRendererSerializeError {
    #[must_use]
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

impl From<&str> for GeneralRendererSerializeError {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl std::ops::Deref for GeneralRendererSerializeError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.error
    }
}

impl From<GeneralRendererSerializeError> for String {
    fn from(val: GeneralRendererSerializeError) -> Self {
        val.error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_error_with_message() {
        let msg = "serialization failed".to_string();
        let err = GeneralRendererSerializeError::new(msg.clone());
        assert_eq!(err.error, msg);
    }

    #[test]
    fn from_str_creates_error_from_string_slice() {
        let err: GeneralRendererSerializeError = "oops".into();
        assert_eq!(err.error, "oops");
    }

    #[test]
    fn deref_accesses_inner_error_string() {
        let err = GeneralRendererSerializeError::new("inner message".to_string());
        let derefed: &String = &err;
        assert_eq!(derefed, "inner message");
    }

    #[test]
    fn into_string_extracts_message() {
        let err = GeneralRendererSerializeError::new("extract me".to_string());
        let s: String = err.into();
        assert_eq!(s, "extract me");
    }
}
