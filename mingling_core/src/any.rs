use crate::Groupped;
use crate::error::ChainProcessError;

#[doc(hidden)]
pub mod group;

/// Any type output
///
/// Accepts any type that implements `Send + Groupped<G>`
/// After being passed into `AnyOutput`, it will be converted to `Box<dyn Any + Send + 'static>`
///
/// Note:
/// - If an enum value that does not belong to this type is incorrectly specified, it will be **unsafely** unwrapped by the scheduler
/// - Structured output via `--json`/`--yaml` is only available for types that implement
///   [`StructuralData`], which implies `serde::Serialize`.
/// - It is recommended to use the `pack!` macro from [mingling_macros](https://crates.io/crates/mingling_macros) to create types that can be converted to `AnyOutput`, which guarantees runtime safety
#[derive(Debug)]
pub struct AnyOutput<G> {
    pub(crate) inner: Box<dyn std::any::Any + Send + 'static>,
    pub type_id: std::any::TypeId,
    pub member_id: G,
}

impl<G> AnyOutput<G> {
    /// Create an `AnyOutput` from a `Send + Groupped<G>` type
    pub fn new<T>(value: T) -> Self
    where
        T: Send + Groupped<G> + 'static,
    {
        Self {
            inner: Box::new(value),
            type_id: std::any::TypeId::of::<T>(),
            member_id: T::member_id(),
        }
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
    pub fn route_chain(self) -> ChainProcess<G> {
        ChainProcess::Ok((self, NextProcess::Chain))
    }

    /// Route the output to the Renderer, ending execution
    pub fn route_renderer(self) -> ChainProcess<G> {
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
            match self.inner.downcast::<T>() {
                Ok(boxed) => Some(*boxed),
                Err(_) => None,
            }
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
/// - Returns `Ok((`[`AnyOutput`](./struct.AnyOutput.html)`, `[`NextProcess::Chain`](./enum.NextProcess.html)`))` to continue execution with this type next
/// - Returns `Ok((`[`AnyOutput`](./struct.AnyOutput.html)`, `[`NextProcess::Renderer`](./enum.NextProcess.html)`))` to render this type next and output to the terminal
/// - Returns `Err(`[`ChainProcessError`](./error/enum.ChainProcessError.html)`]` to terminate the program directly
pub enum ChainProcess<G> {
    Ok((AnyOutput<G>, NextProcess)),
    Err(ChainProcessError),
}

/// Indicates the next step after processing
///
/// - `Chain`: Continue execution to the next chain
/// - `Renderer`: Send output to renderer and end execution
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NextProcess {
    Chain,
    Renderer,
}

impl std::fmt::Display for NextProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NextProcess::Chain => write!(f, "Chain"),
            NextProcess::Renderer => write!(f, "Renderer"),
        }
    }
}

impl<G> From<AnyOutput<G>> for ChainProcess<G> {
    fn from(value: AnyOutput<G>) -> Self {
        ChainProcess::Ok((value, NextProcess::Chain))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Groupped;

    /// Mock enum for testing AnyOutput
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
                MockGroup::Alpha => write!(f, "Alpha"),
                MockGroup::Beta => write!(f, "Beta"),
                MockGroup::Gamma => write!(f, "Gamma"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "general_renderer", derive(serde::Serialize))]
    struct AlphaData {
        value: i32,
    }

    impl Groupped<MockGroup> for AlphaData {
        fn member_id() -> MockGroup {
            MockGroup::Alpha
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "general_renderer", derive(serde::Serialize))]
    struct BetaData {
        name: String,
    }

    impl Groupped<MockGroup> for BetaData {
        fn member_id() -> MockGroup {
            MockGroup::Beta
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[allow(dead_code)]
    #[cfg_attr(feature = "general_renderer", derive(serde::Serialize))]
    struct GammaData;

    impl Groupped<MockGroup> for GammaData {
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
            _ => panic!("Expected ChainProcess::Ok"),
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
            _ => panic!("Expected ChainProcess::Ok"),
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
            _ => panic!("Expected ChainProcess::Ok"),
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

    // AnyOutput::restore general_renderer feature only

    #[cfg(feature = "general_renderer")]
    #[test]
    fn test_any_output_restore_success() {
        use serde::Serialize;

        #[derive(Debug, Clone, PartialEq, Serialize)]
        struct SerData {
            x: i32,
        }

        impl Groupped<MockGroup> for SerData {
            fn member_id() -> MockGroup {
                MockGroup::Gamma
            }
        }

        let data = SerData { x: 42 };
        let output = AnyOutput::new(data);
        let restored: Option<SerData> = output.restore::<SerData>();
        assert_eq!(restored, Some(SerData { x: 42 }));
    }

    #[cfg(feature = "general_renderer")]
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

        impl Groupped<MockGroup> for SerA {
            fn member_id() -> MockGroup {
                MockGroup::Alpha
            }
        }

        impl Groupped<MockGroup> for SerB {
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
