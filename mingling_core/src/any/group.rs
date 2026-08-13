/// Member ID for types within a program
///
/// This trait provides a member ID for program-internal types, used to determine
/// the downcast type during dispatch, routing, rendering, and other stages.
///
/// # Safety
///
/// This trait is typically provided by the corresponding [`Grouped Derive`](https://docs.rs/mingling/latest/mingling/derive.Grouped.html).
/// If implemented manually, **make sure** the ID is **exactly identical** to
/// the name registered by the `register_type!` macro; otherwise, undefined
/// behavior will inevitably occur when the program routes to that type!
///
/// # Manual impl
///
/// In general, we recommend using [`#[derive(Grouped)]`](https://docs.rs/mingling/latest/mingling/derive.Grouped.html) to implement it.
/// However, if you must implement it manually, please follow exactly this pattern:
///
/// ```
/// # use mingling_core::Grouped;
/// enum ThisProgram {
///     // Global ID registered by `register_type!`
///     StateMyType,
/// }
///
/// struct StateMyType;
///
/// // SAFETY: This ensures the StateMyType variant during ThisProgram dispatch always corresponds to this type
/// unsafe impl Grouped<ThisProgram> for StateMyType {
///     fn member_id() -> ThisProgram {
///         // must semantically correspond to the type itself!
///         ThisProgram::StateMyType
///     }
/// }
/// ```
pub unsafe trait Grouped<Group>
where
    Self: Sized + 'static,
{
    /// Get the member ID for this type
    ///
    /// # Safety
    ///
    /// The returned enum variant must exactly correspond to this type itself,
    /// i.e., the returned `Group` enum variant must semantically represent this
    /// type itself. If an incorrect variant is returned, it will cause a type
    /// casting error and lead to undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// # use mingling_core::Grouped;
    /// # enum ThisProgram {
    /// #     StateMyType,
    /// # }
    /// # struct StateMyType;
    /// # unsafe impl Grouped<ThisProgram> for StateMyType {
    /// // The following macro registers the type ID
    /// // mingling::macros::register_type!(StateMyType);
    ///
    /// fn member_id() -> ThisProgram {
    ///     // must semantically correspond to the type itself!
    ///     ThisProgram::StateMyType
    /// }
    /// # }
    /// ```
    fn member_id() -> Group;
}
