// Doc Not Optimize
use crate::ProgramCollect;
use crate::error::ChainProcessError;

mod group;
pub use group::*;

/// Any type output
///
/// Accepts any type that implements `Send + Grouped<G>`
/// After being passed into `AnyOutput`, it will be converted to `Box<dyn Any + Send + 'static>`
///
/// Note:
/// - If an enum value that does not belong to this type is incorrectly specified, it will be **unsafely** unwrapped by the scheduler
/// - Structured output via `--json`/`--yaml` is only available for types that implement
///   \[`StructuralData`\], which implies `serde::Serialize`.
/// - It is recommended to use the `pack!` macro from [mingling_macros](https://crates.io/crates/mingling_macros) to create types that can be converted to `AnyOutput`, which guarantees runtime safety
#[derive(Debug)]
pub struct AnyOutput<G> {
    pub(crate) inner: Box<dyn std::any::Any + Send + 'static>,

    /// The [`TypeId`] of the concrete type stored in `inner`.
    ///
    /// This is set during construction and used for type-checking
    /// in downcast, restore, and is methods.
    pub(crate) type_id: std::any::TypeId,

    /// The variant identifier returned by [`Grouped::member_id`] for the
    /// concrete type stored in `inner`.
    ///
    /// This is used by the scheduler to dispatch on the correct enum
    /// variant when routing the output.
    pub(crate) member_id: G,
}

impl<G> AnyOutput<G> {
    /// Create an `AnyOutput` from a `Send + Grouped<G>` type
    pub fn new<T>(value: T) -> Self
    where
        T: Send + Grouped<G> + 'static,
    {
        Self {
            inner: Box::new(value),
            type_id: std::any::TypeId::of::<T>(),
            member_id: T::member_id(),
        }
    }

    /// Create an `AnyOutput` from a raw value with a manually specified [`member_id`].
    ///
    /// This function bypasses the [`Grouped`] trait, meaning the `member_id` you provide
    /// does **not** have to match the actual concrete type `T`. The scheduler uses
    /// `member_id` to determine which enum variant the output belongs to, and later
    /// attempts to restore the value to the concrete type `T` based on that variant.
    ///
    /// # Safety
    ///
    /// - The caller must ensure that `member_id` correctly corresponds to the concrete
    ///   type `T` according to the scheduling logic. If `member_id` does not match,
    ///   calling [`restore`](Self::restore) or [`downcast`](Self::downcast) with the
    ///   type associated with `member_id` will cause **undefined behavior**.
    /// - This safety contract is the caller's responsibility; the compiler cannot
    ///   enforce the correspondence between `member_id` and the stored type.
    pub unsafe fn new_bare<T>(value: T, member_id: G) -> Self
    where
        T: Send + 'static,
    {
        Self {
            inner: Box::new(value),
            type_id: std::any::TypeId::of::<T>(),
            member_id,
        }
    }

    /// Get the [`TypeId`] of the concrete type stored in `inner`.
    ///
    /// The `TypeId` is set during construction (via [`AnyOutput::new`] or [`AnyOutput::new_bare`])
    /// and is used for subsequent downcasting and type checking.
    pub const fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }

    /// Get the `member_id` of the concrete type stored in `inner`.
    ///
    /// `member_id` is set during construction (via [`AnyOutput::new`] or [`AnyOutput::new_bare`])
    /// and identifies which variant of the output enum this value corresponds to.
    /// The scheduler uses this value to dispatch the output to the correct next step.
    pub const fn member_id(&self) -> G
    where
        G: Copy,
    {
        self.member_id
    }

    /// Attempt to downcast the `AnyOutput` to a concrete type.
    ///
    /// # Errors
    ///
    /// Returns `Err(self)` if the downcast fails.
    ///
    /// # Panics
    ///
    /// Panics if the inner value is not of type `T`.
    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        if self.type_id == std::any::TypeId::of::<T>() {
            Ok(*self.inner.downcast::<T>().unwrap())
        } else {
            Err(self)
        }
    }

    /// Check if the inner value is of type T
    pub fn is<T: 'static>(&self) -> bool {
        self.type_id == std::any::TypeId::of::<T>()
    }

    /// Route the output to the next Chain
    pub const fn route_chain(self) -> ChainProcess<G> {
        ChainProcess::Ok((self, NextProcess::Chain))
    }

    /// Route the output to the Renderer, ending execution
    pub const fn route_renderer(self) -> ChainProcess<G> {
        ChainProcess::Ok((self, NextProcess::Renderer))
    }

    /// Restore `AnyOutput` back to the original concrete type.
    ///
    /// # Safety
    ///
    /// This is only safe when `T` matches the `TypeId` stored in the `AnyOutput`.
    /// Generated code (via `gen_program!()`) guarantees this by dispatching on
    /// `member_id` before calling `restore`.
    pub fn restore<T: 'static>(self) -> Option<T> {
        if self.type_id == std::any::TypeId::of::<T>() {
            self.inner
                .downcast::<T>()
                .map_or_else(|_| None, |boxed| Some(*boxed))
        } else {
            None
        }
    }
}

impl<G> std::ops::Deref for AnyOutput<G> {
    type Target = dyn std::any::Any + Send + 'static;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<G> std::ops::DerefMut for AnyOutput<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

/// Chain exec result type
///
/// Stores `Ok` and `Err` types of execution results, used to notify the scheduler what to execute next
/// - Returns <code>Ok(([AnyOutput](./struct.AnyOutput.html), [NextProcess::Chain](./enum.NextProcess.html)))</code> to continue execution with this type next
/// - Returns <code>Ok(([AnyOutput](./struct.AnyOutput.html), [NextProcess::Renderer](./enum.NextProcess.html)))</code> to render this type next and output to the terminal
/// - Returns <code>Err([ChainProcessError](./error/enum.ChainProcessError.html)]</code> to terminate the program directly
pub enum ChainProcess<G> {
    /// Indicates success, containing the output value and the next step to execute.
    Ok((AnyOutput<G>, NextProcess)),
    /// Indicates a processing failure, containing the error that occurred.
    Err(ChainProcessError),
}

/// Indicates the next step after processing
///
/// - `Chain`: Continue execution to the next chain
/// - `Renderer`: Send output to renderer and end execution
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NextProcess {
    /// Continue execution to the next chain
    Chain,
    /// Send output to renderer and end execution
    Renderer,
}

impl std::fmt::Display for NextProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chain => write!(f, "Chain"),
            Self::Renderer => write!(f, "Renderer"),
        }
    }
}

impl<G> From<AnyOutput<G>> for ChainProcess<G> {
    fn from(value: AnyOutput<G>) -> Self {
        Self::Ok((value, NextProcess::Chain))
    }
}

impl<G> From<()> for ChainProcess<G>
where
    G: ProgramCollect<Enum = G>,
{
    fn from(_v: ()) -> Self {
        G::build_empty_result().route_chain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Grouped;

    /// Mock enum for testing `AnyOutput`
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    enum MockGroup {
        Alpha,
        Beta,
        Gamma,
    }

    impl std::fmt::Display for MockGroup {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Alpha => write!(f, "Alpha"),
                Self::Beta => write!(f, "Beta"),
                Self::Gamma => write!(f, "Gamma"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "structural_renderer", derive(serde::Serialize))]
    struct AlphaData {
        value: i32,
    }

    /// # Safety
    ///
    /// This implementation is only for testing purposes to satisfy trait bounds.
    /// Since this code only constructs `AnyOutput` and calls methods like
    /// `downcast`, `is`, `restore`, `route_chain`, and `route_renderer` —
    /// none of which involve `ProgramCollect::do_chain` or
    /// `ProgramCollect::render` — the `type`/`member_id` correspondence is
    /// never exploited in an unsafe way here.
    /// The caller must ensure that the associated `member_id` correctly
    /// corresponds to the type's role in the group.
    unsafe impl Grouped<MockGroup> for AlphaData {
        fn member_id() -> MockGroup {
            MockGroup::Alpha
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "structural_renderer", derive(serde::Serialize))]
    struct BetaData {
        name: String,
    }

    /// # Safety
    ///
    /// This implementation is only for testing purposes to satisfy trait bounds.
    /// Since this code only constructs `AnyOutput` and calls methods like
    /// `downcast`, `is`, `restore`, `route_chain`, and `route_renderer` —
    /// none of which involve `ProgramCollect::do_chain` or
    /// `ProgramCollect::render` — the `type`/`member_id` correspondence is
    /// never exploited in an unsafe way here.
    /// The caller must ensure that the associated `member_id` correctly
    /// corresponds to the type's role in the group.
    unsafe impl Grouped<MockGroup> for BetaData {
        fn member_id() -> MockGroup {
            MockGroup::Beta
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[allow(dead_code)]
    #[cfg_attr(feature = "structural_renderer", derive(serde::Serialize))]
    struct GammaData;

    /// # Safety
    ///
    /// This implementation is only for testing purposes to satisfy trait bounds.
    /// Since this code only constructs `AnyOutput` and calls methods like
    /// `downcast`, `is`, `restore`, `route_chain`, and `route_renderer` —
    /// none of which involve `ProgramCollect::do_chain` or
    /// `ProgramCollect::render` — the `type`/`member_id` correspondence is
    /// never exploited in an unsafe way here.
    /// The caller must ensure that the associated `member_id` correctly
    /// corresponds to the type's role in the group.
    unsafe impl Grouped<MockGroup> for GammaData {
        fn member_id() -> MockGroup {
            MockGroup::Gamma
        }
    }

    // AnyOutput::new

    #[test]
    fn test_any_output_new_stores_type_id_and_member_id() {
        let data = AlphaData { value: 42 };
        let output = AnyOutput::new(data);

        assert_eq!(output.type_id, std::any::TypeId::of::<AlphaData>());
        assert_eq!(output.member_id, MockGroup::Alpha);
    }

    // AnyOutput::downcast

    #[test]
    fn test_any_output_downcast_success() {
        let data = AlphaData { value: 99 };
        let output = AnyOutput::new(data);

        let result: Result<AlphaData, _> = output.downcast::<AlphaData>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 99);
    }

    #[test]
    fn test_any_output_downcast_failure() {
        let data = AlphaData { value: 10 };
        let output = AnyOutput::new(data);

        let result: Result<BetaData, _> = output.downcast::<BetaData>();
        assert!(result.is_err());
    }

    // AnyOutput::is

    #[test]
    fn test_any_output_is_true_for_matching_type() {
        let data = AlphaData { value: 7 };
        let output = AnyOutput::new(data);

        assert!(output.is::<AlphaData>());
    }

    #[test]
    fn test_any_output_is_false_for_non_matching_type() {
        let data = AlphaData { value: 7 };
        let output = AnyOutput::new(data);

        assert!(!output.is::<BetaData>());
    }

    // AnyOutput::route_chain

    #[test]
    fn test_route_chain_returns_ok_with_chain_next() {
        let data = AlphaData { value: 1 };
        let output = AnyOutput::new(data);

        let result = output.route_chain();
        match result {
            ChainProcess::Ok((any, next)) => {
                assert_eq!(any.member_id, MockGroup::Alpha);
                assert_eq!(next, NextProcess::Chain);
            }
            ChainProcess::Err(_) => panic!("Expected ChainProcess::Ok"),
        }
    }

    // AnyOutput::route_renderer

    #[test]
    fn test_route_renderer_returns_ok_with_renderer_next() {
        let data = AlphaData { value: 2 };
        let output = AnyOutput::new(data);

        let result = output.route_renderer();
        match result {
            ChainProcess::Ok((any, next)) => {
                assert_eq!(any.member_id, MockGroup::Alpha);
                assert_eq!(next, NextProcess::Renderer);
            }
            ChainProcess::Err(_) => panic!("Expected ChainProcess::Ok"),
        }
    }

    // AnyOutput: Deref / DerefMut

    #[test]
    fn test_any_output_deref_accesses_inner_any() {
        let data = AlphaData { value: 5 };
        let output = AnyOutput::new(data);

        let inner: &dyn std::any::Any = &*output;
        assert!(inner.downcast_ref::<AlphaData>().is_some());
    }

    #[test]
    fn test_any_output_deref_mut_allows_modification() {
        let data = AlphaData { value: 0 };
        let mut output = AnyOutput::new(data);

        let inner: &mut dyn std::any::Any = &mut *output;
        if let Some(ref mut v) = inner.downcast_mut::<AlphaData>() {
            v.value = 100;
        }

        let result: Result<AlphaData, _> = output.downcast::<AlphaData>();
        assert_eq!(result.unwrap().value, 100);
    }

    // ChainProcess::From<AnyOutput>

    #[test]
    fn test_chain_process_from_any_output() {
        let data = AlphaData { value: 3 };
        let output = AnyOutput::new(data);

        let cp: ChainProcess<MockGroup> = output.into();
        match cp {
            ChainProcess::Ok((any, next)) => {
                assert_eq!(any.member_id, MockGroup::Alpha);
                assert_eq!(next, NextProcess::Chain);
            }
            ChainProcess::Err(_) => panic!("Expected ChainProcess::Ok"),
        }
    }

    // NextProcess::Display

    #[test]
    fn test_next_process_display_chain() {
        assert_eq!(format!("{}", NextProcess::Chain), "Chain");
    }

    #[test]
    fn test_next_process_display_renderer() {
        assert_eq!(format!("{}", NextProcess::Renderer), "Renderer");
    }

    // AnyOutput::restore structural_renderer feature only

    #[cfg(feature = "structural_renderer")]
    #[test]
    fn test_any_output_restore_success() {
        use serde::Serialize;

        #[derive(Debug, Clone, PartialEq, Serialize)]
        struct SerData {
            x: i32,
        }

        /// SAFETY:
        ///
        /// This implementation is only for testing purposes to satisfy trait bounds.
        /// Since this code only constructs `AnyOutput` and calls methods like
        /// `downcast`, `is`, `restore`, `route_chain`, and `route_renderer` —
        /// none of which involve `ProgramCollect::do_chain` or
        /// `ProgramCollect::render` — the `type`/`member_id` correspondence is
        /// never exploited in an unsafe way here.
        /// The caller must ensure that the associated `member_id` correctly
        /// corresponds to the type's role in the group.
        unsafe impl Grouped<MockGroup> for SerData {
            fn member_id() -> MockGroup {
                MockGroup::Gamma
            }
        }

        let data = SerData { x: 42 };
        let output = AnyOutput::new(data);
        let restored: Option<SerData> = output.restore::<SerData>();
        assert_eq!(restored, Some(SerData { x: 42 }));
    }

    #[cfg(feature = "structural_renderer")]
    #[test]
    fn test_any_output_restore_type_mismatch() {
        use serde::Serialize;

        #[derive(Debug, Clone, PartialEq, Serialize)]
        struct SerA {
            a: i32,
        }

        #[derive(Debug, Clone, PartialEq, Serialize)]
        struct SerB {
            b: String,
        }

        /// SAFETY:
        ///
        /// This implementation is only for testing purposes to satisfy trait bounds.
        /// Since this code only constructs `AnyOutput` and calls methods like
        /// `downcast`, `is`, `restore`, `route_chain`, and `route_renderer` —
        /// none of which involve `ProgramCollect::do_chain` or
        /// `ProgramCollect::render` — the `type`/`member_id` correspondence is
        /// never exploited in an unsafe way here.
        /// The caller must ensure that the associated `member_id` correctly
        /// corresponds to the type's role in the group.
        unsafe impl Grouped<MockGroup> for SerA {
            fn member_id() -> MockGroup {
                MockGroup::Alpha
            }
        }

        /// SAFETY:
        ///
        /// This implementation is only for testing purposes to satisfy trait bounds.
        /// Since this code only constructs `AnyOutput` and calls methods like
        /// `downcast`, `is`, `restore`, `route_chain`, and `route_renderer` —
        /// none of which involve `ProgramCollect::do_chain` or
        /// `ProgramCollect::render` — the `type`/`member_id` correspondence is
        /// never exploited in an unsafe way here.
        /// The caller must ensure that the associated `member_id` correctly
        /// corresponds to the type's role in the group.
        unsafe impl Grouped<MockGroup> for SerB {
            fn member_id() -> MockGroup {
                MockGroup::Beta
            }
        }

        let data = SerA { a: 1 };
        let output = AnyOutput::new(data);
        let restored: Option<SerB> = output.restore::<SerB>();
        assert_eq!(restored, None);
    }
}
