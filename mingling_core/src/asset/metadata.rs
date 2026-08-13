/// Provides metadata for an Entry.
///
/// Any type can be attached to an Entry as metadata, allowing the program to
/// carry compile-time-typed, arbitrary description data alongside each
/// registered entry. The [`Metadata`] trait bridges an Entry type (`Self`) to
/// an arbitrary metadata type `DataType`.
///
/// # Manual impl
///
/// It is recommended to use the `#[metadata(Entry)]` attribute macro from
/// [mingling_macros](https://crates.io/crates/mingling_macros) to implement this
/// trait and register the entry via `register_metadata!`.
///
/// If you need to implement it manually, you can refer to the following
/// example:
///
/// ```
/// # use mingling_core::MockProgramCollect as ThisProgram;
/// # use mingling_core::Metadata;
/// struct EntryGreet;
/// struct MyInformation {
///     name: String
/// }
///
/// impl Metadata<MyInformation> for EntryGreet {
///     fn init_metadata() -> MyInformation {
///         MyInformation { name: "Greeting".into() }
///     }
/// }
///
/// // Register the MyInformation metadata for EntryGreet using `register_metadata!`.
/// // mingling::macros::register_metadata!(EntryGreet, MyInformation);
/// ```
pub trait Metadata<DataType> {
    /// Initializes and returns the metadata value of type `DataType` for this entry.
    fn init_metadata() -> DataType;
}
