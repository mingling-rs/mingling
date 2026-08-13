// Doc Not Optimize
/// Represents an error that occurs during serialization of a structural renderer.
///
/// This error stores a human-readable message describing what went wrong
/// during the serialization process.
#[derive(Debug)]
pub struct StructuralRendererSerializeError {
    /// The underlying error message.
    error: String,
}

impl StructuralRendererSerializeError {
    /// Creates a new `StructuralRendererSerializeError` with the given error message.
    #[must_use]
    pub const fn new(error: String) -> Self {
        Self { error }
    }
}

impl From<&str> for StructuralRendererSerializeError {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl std::ops::Deref for StructuralRendererSerializeError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.error
    }
}

impl From<StructuralRendererSerializeError> for String {
    fn from(val: StructuralRendererSerializeError) -> Self {
        val.error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_error_with_message() {
        let msg = "serialization failed".to_string();
        let err = StructuralRendererSerializeError::new(msg.clone());
        assert_eq!(err.error, msg);
    }

    #[test]
    fn from_str_creates_error_from_string_slice() {
        let err: StructuralRendererSerializeError = "oops".into();
        assert_eq!(err.error, "oops");
    }

    #[test]
    fn deref_accesses_inner_error_string() {
        let err = StructuralRendererSerializeError::new("inner message".to_string());
        let derefed: &String = &err;
        assert_eq!(derefed, "inner message");
    }

    #[test]
    fn into_string_extracts_message() {
        let err = StructuralRendererSerializeError::new("extract me".to_string());
        let s: String = err.into();
        assert_eq!(s, "extract me");
    }
}
