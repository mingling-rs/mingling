/// Provides metadata for an Entry.
///
/// Any type can be attached to an Entry as metadata, allowing the program to
/// carry compile-time-typed, arbitrary description data alongside each
/// registered entry. The [`Metadata`] trait bridges an Entry type (`Self`) to
/// an arbitrary metadata type `B`.
///
/// It is recommended to use the `#[metadata(Entry)]` attribute macro from
/// [mingling_macros](https://crates.io/crates/mingling_macros) to implement this
/// trait and register the entry via `register_metadata!`.
pub trait Metadata<B> {
    /// Initializes and returns the metadata value of type `B` for this entry.
    fn init_metadata() -> B;
}
