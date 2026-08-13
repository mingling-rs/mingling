use just_fmt::kebab_case;

/// Represents a command node used for matching user-input command paths.
///
/// The node consists of multiple segments, separated by dots (`.`), which are automatically
/// converted to kebab-case. For example, the input string `"node.subnode"` would be converted
/// to the node representation `["node", "subnode"]`.
///
/// # Examples
///
/// ```
/// use mingling_core::Node;
///
/// // Create a node and append child segments
/// let node = Node::from("base").join("sub").join("leaf");
/// assert_eq!(node.to_string(), "base.sub.leaf");
/// ```
#[derive(Debug, Default)]
pub struct Node {
    node: Vec<String>,
}

impl Node {
    /// Appends a new segment to the node path.
    ///
    /// This method consumes the current node and returns a new node with the new
    /// segment appended to the end of the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::Node;
    ///
    /// let node = Node::from("base").join("sub");
    /// assert_eq!(node.to_string(), "base.sub");
    /// ```
    #[must_use]
    pub fn join(self, node: impl Into<String>) -> Self {
        let mut new_node = self.node;
        new_node.push(node.into());
        Self { node: new_node }
    }
}

impl From<&str> for Node {
    fn from(s: &str) -> Self {
        let node = s.split('.').map(|part| kebab_case!(part)).collect();
        Self { node }
    }
}

impl From<String> for Node {
    fn from(s: String) -> Self {
        let node = s.split('.').map(|part| kebab_case!(part)).collect();
        Self { node }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.node.cmp(&other.node)
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.node.join("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_from_single_str() {
        let node = Node::from("hello");
        assert_eq!(node.node, vec!["hello"]);
        assert_eq!(node.to_string(), "hello");
    }

    #[test]
    fn test_node_from_dotted_str() {
        let node = Node::from("a.b.c");
        assert_eq!(node.node, vec!["a", "b", "c"]);
        assert_eq!(node.to_string(), "a.b.c");
    }

    #[test]
    fn test_node_kebab_case_conversion() {
        let node = Node::from("HelloWorld.FooBar");
        assert_eq!(node.node, vec!["hello-world", "foo-bar"]);
    }

    #[test]
    fn test_node_from_string() {
        let s = String::from("x.y");
        let node = Node::from(s);
        assert_eq!(node.node, vec!["x", "y"]);
    }

    #[test]
    fn test_node_join() {
        let node = Node::from("base").join("sub");
        assert_eq!(node.node, vec!["base", "sub"]);
    }

    #[test]
    fn test_node_join_multiple() {
        let node = Node::from("a").join("b").join("c");
        assert_eq!(node.to_string(), "a.b.c");
    }

    #[test]
    fn test_node_default_empty() {
        let node = Node::default();
        assert!(node.node.is_empty());
        assert_eq!(node.to_string(), "");
    }

    #[test]
    fn test_node_partial_eq() {
        let a = Node::from("a.b");
        let b = Node::from("a.b");
        let c = Node::from("a.c");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_node_ord() {
        let a = Node::from("a");
        let b = Node::from("b");
        assert!(a < b);
    }

    #[test]
    fn test_node_join_appends_part() {
        let node = Node::from("existing");
        let joined = node.join("new-part");
        assert_eq!(joined.to_string(), "existing.new-part");
    }
}
