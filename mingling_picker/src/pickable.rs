use crate::{PickerArg, PickerArgAttr, PickerArgInfo, PickerArgResult, PickerArgs};

/// `Pickable` trait defines how to parse a type instance from command-line arguments.
///
/// This trait is the core abstraction of the `Picker` argument parsing system, dividing the
/// parsing process into two phases:
///
/// 1. **Tag phase ([`Pickable::tag`])**: Determines which argument positions the `Pickable` needs to handle.
/// 2. **Pick phase ([`Pickable::pick`])**: Converts the raw strings at the tagged positions into the actual type.
///
/// Types implementing this trait must also implement [`Default`], so that a default value
/// can be used as a fallback when parsing fails.
///
/// # Type Parameters
///
/// * `'a` - Lifetime parameter, used to associate references in [`PickerArg`].
pub trait Pickable<'a>
where
    Self: Sized,
{
    /// Returns the parse-order attribute of this flag.
    ///
    /// This attribute is used to inform the parser about the parse order
    /// between different `Pickable` types.
    /// See [`PickerArgAttr`] for specific ordering definitions.
    ///
    /// # Parameters
    ///
    /// * `flag` - The current flag instance, which contains a reference to `Self`.
    ///
    /// # Returns
    ///
    /// Returns a [`PickerArgAttr`] describing the parse-order attribute of this flag.
    fn get_attr(flag: &'a PickerArg<'a, Self>) -> PickerArgAttr;

    /// Tag phase: Determines which argument positions the `Pickable` needs to handle.
    ///
    /// This function receives a [`TagPhaseContext`] containing argument context information.
    /// During this phase, the parser invokes each `Pickable` and collects the position indices
    /// they return, in order to determine which arguments to parse later.
    ///
    /// # Parameters
    ///
    /// * `ctx` - The tag phase context, containing argument information, all parameters of the
    ///   current Picker, and an availability mask.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<usize>` representing the indices of the arguments in the argument list
    /// that this `Pickable` needs to handle.
    fn tag(ctx: TagPhaseContext) -> Vec<usize>;

    /// Pick phase: Converts the raw string arguments tagged during the `tag` phase into
    /// the actual expected type.
    ///
    /// This function receives a slice of the raw strings that were tagged in the `tag` step
    /// and converts them into an instance of `Self`.
    ///
    /// # Parameters
    ///
    /// * `raw_strs` - A slice of strings containing the raw argument values to parse.
    ///
    /// # Returns
    ///
    /// Returns [`PickerArgResult<Self>`], i.e., the `Self` instance on success, or an appropriate
    /// error message on failure.
    fn pick(raw_strs: &[&str]) -> PickerArgResult<Self>;
}

/// Tag phase context, providing the necessary argument and state information for
/// [`Pickable::tag`].
pub struct TagPhaseContext<'a> {
    /// Argument information describing the structure and metadata of the argument
    /// to be parsed.
    pub arg_info: &'a PickerArgInfo<'a>,

    /// A read-only list of all arguments in the current [`Picker`].
    pub args: &'a PickerArgs<'a>,

    /// Mask indicating which argument positions have already been claimed.
    ///
    /// For example, if the mask is `[0, 0, 1, 0]`, then the argument at index `2`
    /// has already been tagged by another `Pickable`.
    pub mask: &'a [u8],
}
