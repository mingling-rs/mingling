/// Marks an enum so that its variants can be recognized by Mingling
///
/// By implementing [`EnumTag`], Mingling can obtain the enum's name, information, etc.,
/// which helps with argument parsing, completion, etc.
///
/// # Manual impl
///
/// In general, [`EnumTag`] is recommended to be derived using [`#[derive(EnumTag)]`](https://docs.rs/mingling/latest/mingling/derive.EnumTag.html),
/// but if you need to implement it manually, please refer to the following:
///
/// ```
/// # use mingling_core::EnumTag;
/// enum Choice {
///     Foo, Bar
/// }
///
/// impl EnumTag for Choice {
///     fn enum_info(&self) -> (&'static str, &'static str) {
///         ("Choice", "Choice enum")
///     }
///     fn enums() -> &'static [(&'static str, &'static str)] {
///         &[("Foo", "Foo variant"), ("Bar", "Bar variant")]
///     }
///     fn build_enum(name: String) -> Option<Self> {
///         match name.as_str() {
///             "Foo" => Some(Choice::Foo),
///             "Bar" => Some(Choice::Bar),
///             _ => None,
///         }
///     }
/// }
/// ```
pub trait EnumTag {
    /// Returns the name and description of the enum
    ///
    /// Returns a tuple `(enum_name, enum_description)`, where `enum_name` is the name of the enum,
    /// and `enum_description` is a brief description of the enum, used in scenarios such as
    /// argument parsing error messages and completion.
    fn enum_info(&self) -> (&'static str, &'static str);

    /// Returns the names and descriptions of all variants of this enum
    ///
    /// Returns a slice where each element is a `(variant_name, variant_description)` tuple,
    /// describing all variants of the enum and their meanings, used for argument completion and parsing.
    fn enums() -> &'static [(&'static str, &'static str)];

    /// Builds the corresponding enum value from a string name
    ///
    /// The input `name` is the string argument to be parsed. If it matches a variant name,
    /// the corresponding `Some(enum_value)` is returned; otherwise `None` is returned.
    fn build_enum(name: String) -> Option<Self>
    where
        Self: Sized;
}
