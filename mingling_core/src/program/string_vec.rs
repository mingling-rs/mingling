#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct StringVec {
    vec: Vec<String>,
}

impl std::ops::Deref for StringVec {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl From<StringVec> for Vec<String> {
    fn from(val: StringVec) -> Self {
        val.vec
    }
}

impl<const N: usize> From<[&str; N]> for StringVec {
    fn from(slice: [&str; N]) -> Self {
        Self {
            vec: slice.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

impl From<&[&str]> for StringVec {
    fn from(slice: &[&str]) -> Self {
        Self {
            vec: slice.iter().map(|&s| s.to_string()).collect(),
        }
    }
}
impl From<Vec<String>> for StringVec {
    fn from(vec: Vec<String>) -> Self {
        Self { vec }
    }
}

impl From<&[String]> for StringVec {
    fn from(slice: &[String]) -> Self {
        Self {
            vec: slice.to_vec(),
        }
    }
}

impl From<Vec<&str>> for StringVec {
    fn from(vec: Vec<&str>) -> Self {
        Self {
            vec: vec.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_vec_from_array() {
        let sv = StringVec::from(["a", "b", "c"]);
        assert_eq!(sv.vec, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_string_vec_from_slice_ref() {
        let arr = ["x", "y"];
        let sv = StringVec::from(&arr[..]);
        assert_eq!(sv.vec, vec!["x", "y"]);
    }

    #[test]
    fn test_string_vec_from_vec_string() {
        let original = vec!["one".to_string(), "two".to_string()];
        let sv = StringVec::from(original.clone());
        assert_eq!(sv.vec, original);
    }

    #[test]
    fn test_string_vec_from_slice_string() {
        let original = vec!["a".to_string(), "b".to_string()];
        let sv = StringVec::from(&original[..]);
        assert_eq!(sv.vec, original);
    }

    #[test]
    fn test_string_vec_from_vec_str() {
        let sv = StringVec::from(vec!["hello", "world"]);
        assert_eq!(sv.vec, vec!["hello", "world"]);
    }

    #[test]
    fn test_string_vec_deref() {
        let sv = StringVec::from(["alpha", "beta"]);
        let inner: &Vec<String> = &sv;
        assert_eq!(inner.len(), 2);
        assert_eq!(inner[0], "alpha");
    }

    #[test]
    fn test_string_vec_into_vec() {
        let sv = StringVec::from(["foo", "bar"]);
        let v: Vec<String> = sv.into();
        assert_eq!(v, vec!["foo", "bar"]);
    }

    #[test]
    fn test_string_vec_empty_from_empty_array() {
        let sv = StringVec::from([] as [&str; 0]);
        assert!(sv.vec.is_empty());
    }
}
