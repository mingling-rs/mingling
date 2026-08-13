use crate::ProgramCollect;
use crate::error::ChainProcessError;

mod group;
pub use group::*;

/// A wrapper for any type within a program group.
///
/// `AnyOutput` wraps any concrete type value produced during program execution so it can
/// be uniformly passed between the chain (Chain) and the renderer (Renderer). Its `inner`
/// field stores the original value with type erasure, while the `type_id` and `member_id`
/// fields are used for type checking and dispatch routing, respectively.
///
/// # Field Descriptions
///
/// - `inner`: Stores the concrete type value, erased to `dyn Any + Send + 'static`.
/// - `type_id`: Stores the concrete type's [`TypeId`](std::any::TypeId), used for
///   type checking in methods such as `downcast`, `is`, and `restore`.
/// - `member_id`: Stores the variant identifier returned by [`Grouped::member_id`], used by
///   the dispatcher to determine the corresponding enum variant when routing output.
///
/// # Examples
///
/// ```
/// # use mingling_core::MockProgramCollect as ThisProgram;
/// use mingling_core::AnyOutput;
/// use mingling_core::Grouped;
///
/// // Define a concrete type and implement Grouped for it
/// struct Foo(i32);
///
/// // SAFETY: The member_id corresponds to the correct ThisProgram variant.
/// unsafe impl Grouped<ThisProgram> for Foo {
///     fn member_id() -> ThisProgram {
///         ThisProgram::Foo
///     }
/// }
///
/// // Construct an AnyOutput using AnyOutput::new
/// let output = AnyOutput::new(Foo(42));
///
/// // Check type and downcast back to the concrete type
/// assert!(output.is::<Foo>());
/// let restored: Foo = output.downcast::<Foo>().unwrap();
/// assert_eq!(restored.0, 42);
/// ```
///
/// Alternatively, you can construct `AnyOutput` directly via [`AnyOutput::new_bare`]
/// for types that don't implement [`Grouped`], though this requires manually
/// providing the `member_id`.
#[derive(Debug)]
pub struct AnyOutput<G> {
    /// The concrete type value after type erasure.
    ///
    /// Set during construction (via [`AnyOutput::new`] or [`AnyOutput::new_bare`]),
    /// used for subsequent type checking and downcasting.
    pub(crate) inner: Box<dyn std::any::Any + Send + 'static>,

    /// The [`TypeId`](std::any::TypeId) of the concrete type stored in `inner`.
    ///
    /// Set during construction (via [`AnyOutput::new`] or [`AnyOutput::new_bare`]),
    /// used for type checking in the `downcast`, `restore`, and `is` methods.
    pub(crate) type_id: std::any::TypeId,

    /// The [`Grouped::member_id`] variant identifier corresponding to the concrete type stored in `inner`.
    ///
    /// Set during construction (via [`AnyOutput::new`] or [`AnyOutput::new_bare`]),
    /// used by the dispatcher to determine the corresponding enum variant when routing output.
    pub(crate) member_id: G,
}

impl<G> AnyOutput<G> {
    /// Create an `AnyOutput` from a `Send + Grouped<G>` type.
    ///
    /// # Arguments
    ///
    /// - `value`: A value of a concrete type `T` that implements both `Send` and
    ///   [`Grouped<G>`](`Grouped`), where `G` is the program group enum type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::AnyOutput;
    /// use mingling_core::Grouped;
    ///
    /// // Define a concrete type and implement Grouped for it
    /// struct Foo(i32);
    ///
    /// // SAFETY: The member_id corresponds to the correct ThisProgram variant.
    /// unsafe impl Grouped<ThisProgram> for Foo {
    ///     fn member_id() -> ThisProgram {
    ///         ThisProgram::Foo
    ///     }
    /// }
    ///
    /// // Create an AnyOutput wrapping a concrete value
    /// let output = AnyOutput::new(Foo(42));
    ///
    /// // Verify the stored type id and member_id
    /// assert_eq!(output.type_id(), std::any::TypeId::of::<Foo>());
    /// assert!(output.is::<Foo>());
    ///
    /// // Downcast back to the concrete type
    /// let restored: Foo = output.downcast::<Foo>().unwrap();
    /// assert_eq!(restored.0, 42);
    /// ```
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

    /// Create an `AnyOutput` from a raw value with a manually specified `member_id`.
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
    ///
    /// # Arguments
    ///
    /// - `value`: The raw value to wrap in the `AnyOutput`.
    /// - `member_id`: The variant identifier used by the scheduler for dispatch routing.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::AnyOutput;
    ///
    /// // Create an AnyOutput for a type that doesn't implement Grouped,
    /// // manually specifying the member_id.
    /// let value = String::from("hello");
    ///
    /// // SAFETY: The caller guarantees that ThisProgram::Foo corresponds
    /// // to the String type in the scheduling logic.
    /// let output = unsafe { AnyOutput::new_bare(value, ThisProgram::Foo) };
    ///
    /// // The member_id is stored as provided.
    /// assert_eq!(output.member_id(), ThisProgram::Foo);
    /// assert!(output.is::<String>());
    /// ```
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

    /// Get the [`TypeId`](std::any::TypeId) of the concrete type stored in `inner`.
    ///
    /// The [`TypeId`](std::any::TypeId) is set during construction (via [`AnyOutput::new`] or [`AnyOutput::new_bare`])
    /// and is used for subsequent downcasting and type checking.
    ///
    /// # Returns
    ///
    /// Returns the [`TypeId`](std::any::TypeId) of the concrete type stored in `inner`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::AnyOutput;
    /// use mingling_core::Grouped;
    ///
    /// struct Foo(i32);
    ///
    /// // SAFETY: The member_id corresponds to the correct ThisProgram variant.
    /// unsafe impl Grouped<ThisProgram> for Foo {
    ///     fn member_id() -> ThisProgram {
    ///         ThisProgram::Foo
    ///     }
    /// }
    ///
    /// let output = AnyOutput::new(Foo(42));
    ///
    /// assert_eq!(output.type_id(), std::any::TypeId::of::<Foo>());
    /// ```
    pub const fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }

    /// Get the `member_id` of the concrete type stored in `inner`.
    ///
    /// `member_id` is set during construction (via [`AnyOutput::new`] or [`AnyOutput::new_bare`])
    /// and identifies which variant of the output enum this value corresponds to.
    /// The scheduler uses this value to dispatch the output to the correct next step.
    ///
    /// # Returns
    ///
    /// Returns the `member_id` of the concrete type stored in `inner`, which is
    /// a variant of the program group enum `G`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::AnyOutput;
    /// use mingling_core::Grouped;
    ///
    /// struct Foo(i32);
    ///
    /// // SAFETY: The member_id corresponds to the correct ThisProgram variant.
    /// unsafe impl Grouped<ThisProgram> for Foo {
    ///     fn member_id() -> ThisProgram {
    ///         ThisProgram::Foo
    ///     }
    /// }
    ///
    /// let output = AnyOutput::new(Foo(42));
    ///
    /// assert_eq!(output.member_id(), ThisProgram::Foo);
    /// ```
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::AnyOutput;
    ///
    /// // Using new_bare to construct from a type that doesn't implement Grouped.
    /// let value = String::from("hello");
    ///
    /// // SAFETY: The caller guarantees that ThisProgram::Bar corresponds
    /// // to the String type in the scheduling logic.
    /// let output = unsafe { AnyOutput::new_bare(value, ThisProgram::Bar) };
    ///
    /// assert_eq!(output.member_id(), ThisProgram::Bar);
    /// ```
    pub const fn member_id(&self) -> G
    where
        G: Copy,
    {
        self.member_id
    }

    /// Attempt to downcast the `AnyOutput` to a concrete type.
    ///
    /// This method consumes the `AnyOutput` and attempts to recover the
    /// inner value as the concrete type `T`. The downcast is performed based
    /// on the stored [`TypeId`](std::any::TypeId): if the stored
    /// type matches `T`, the value is extracted and returned; otherwise,
    /// the original `AnyOutput` is returned unchanged inside `Err`.
    ///
    /// # Arguments
    ///
    /// - `self`: The `AnyOutput` value to attempt downcasting from.
    ///
    /// # Returns
    ///
    /// - `Ok(T)`: The inner value successfully downcast to the concrete type `T`.
    /// - `Err(Self)`: The original `AnyOutput` returned when the stored type
    ///   does not match `T`.
    ///
    /// # Errors
    ///
    /// Returns `Err(Self)` with the original `AnyOutput` when the stored
    /// [`TypeId`](std::any::TypeId) does not match
    /// [`TypeId::of::<T>()`](std::any::TypeId).
    ///
    /// # Panics
    ///
    /// Panics if the stored [`TypeId`](std::any::TypeId) is equal to
    /// [`TypeId::of::<T>()`](std::any::TypeId) but the internal downcast
    /// unexpectedly fails. This should never happen in practice since the
    /// type check guarantees type compatibility.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::AnyOutput;
    /// use mingling_core::Grouped;
    ///
    /// struct Foo(i32);
    ///
    /// // SAFETY: The member_id corresponds to the correct ThisProgram variant.
    /// unsafe impl Grouped<ThisProgram> for Foo {
    ///     fn member_id() -> ThisProgram {
    ///         ThisProgram::Foo
    ///     }
    /// }
    ///
    /// // Successful downcast to the matching type.
    /// let output = AnyOutput::new(Foo(42));
    /// let restored: Foo = output.downcast::<Foo>().unwrap();
    /// assert_eq!(restored.0, 42);
    ///
    /// // Failed downcast to a non-matching type returns Err(self).
    /// let output = AnyOutput::new(Foo(7));
    /// let result: Result<String, _> = output.downcast::<String>();
    /// assert!(result.is_err());
    ///
    /// // Recover the original value from the Err result.
    /// let output = AnyOutput::new(Foo(99));
    /// let result: Result<String, _> = output.downcast::<String>();
    /// let original = result.unwrap_err();
    /// let restored: Foo = original.downcast::<Foo>().unwrap();
    /// assert_eq!(restored.0, 99);
    /// ```
    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        if self.type_id == std::any::TypeId::of::<T>() {
            Ok(*self.inner.downcast::<T>().unwrap())
        } else {
            Err(self)
        }
    }

    /// Check if the inner value is of type T
    ///
    /// # Arguments
    ///
    /// - `T`: The type to check against the stored inner value's type.
    ///
    /// # Returns
    ///
    /// Returns `true` if the stored inner value is of type `T`, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::AnyOutput;
    /// use mingling_core::Grouped;
    ///
    /// struct Foo(i32);
    ///
    /// // SAFETY: The member_id corresponds to the correct ThisProgram variant.
    /// unsafe impl Grouped<ThisProgram> for Foo {
    ///     fn member_id() -> ThisProgram {
    ///         ThisProgram::Foo
    ///     }
    /// }
    ///
    /// let output = AnyOutput::new(Foo(42));
    ///
    /// // Check a matching type.
    /// assert!(output.is::<Foo>());
    ///
    /// // Check a non-matching type.
    /// assert!(!output.is::<String>());
    /// ```
    pub fn is<T: 'static>(&self) -> bool {
        self.type_id == std::any::TypeId::of::<T>()
    }

    /// Route the output to the next Chain
    ///
    /// This method consumes the `AnyOutput` and routes it to the next chain
    /// for continued processing.
    ///
    /// # Arguments
    ///
    /// - `self`: The `AnyOutput` value to route to the chain.
    ///
    /// # Returns
    ///
    /// Returns a [`ChainProcess::Ok`] containing the `AnyOutput` paired with
    /// [`NextProcess::Chain`], indicating the scheduler should continue
    /// execution to the next chain step.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::{AnyOutput, ChainProcess, NextProcess};
    /// use mingling_core::Grouped;
    ///
    /// struct Foo(i32);
    ///
    /// // SAFETY: The member_id corresponds to the correct ThisProgram variant.
    /// unsafe impl Grouped<ThisProgram> for Foo {
    ///     fn member_id() -> ThisProgram {
    ///         ThisProgram::Foo
    ///     }
    /// }
    ///
    /// let output = AnyOutput::new(Foo(42));
    ///
    /// let result = output.route_chain();
    /// match result {
    ///     ChainProcess::Ok((any, next)) => {
    ///         assert!(any.is::<Foo>());
    ///         assert_eq!(next, NextProcess::Chain);
    ///     }
    ///     ChainProcess::Err(_) => panic!("Expected ChainProcess::Ok"),
    /// }
    /// ```
    pub const fn route_chain(self) -> ChainProcess<G> {
        ChainProcess::Ok((self, NextProcess::Chain))
    }

    /// Route the output to the Renderer, ending execution
    ///
    /// This method consumes the `AnyOutput` and routes it to the renderer
    /// for final output to the terminal, ending execution.
    ///
    /// # Arguments
    ///
    /// - `self`: The `AnyOutput` value to route to the renderer.
    ///
    /// # Returns
    ///
    /// Returns a [`ChainProcess::Ok`] containing the `AnyOutput` paired with
    /// [`NextProcess::Renderer`], indicating the scheduler should send the
    /// value to the renderer and terminate execution.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::{AnyOutput, ChainProcess, NextProcess};
    /// use mingling_core::Grouped;
    ///
    /// struct Foo(i32);
    ///
    /// // SAFETY: The member_id corresponds to the correct ThisProgram variant.
    /// unsafe impl Grouped<ThisProgram> for Foo {
    ///     fn member_id() -> ThisProgram {
    ///         ThisProgram::Foo
    ///     }
    /// }
    ///
    /// let output = AnyOutput::new(Foo(42));
    ///
    /// let result = output.route_renderer();
    /// match result {
    ///     ChainProcess::Ok((any, next)) => {
    ///         assert!(any.is::<Foo>());
    ///         assert_eq!(next, NextProcess::Renderer);
    ///     }
    ///     ChainProcess::Err(_) => panic!("Expected ChainProcess::Ok"),
    /// }
    /// ```
    pub const fn route_renderer(self) -> ChainProcess<G> {
        ChainProcess::Ok((self, NextProcess::Renderer))
    }

    /// Restore `AnyOutput` back to the original concrete type.
    ///
    /// This method consumes the `AnyOutput` and attempts to recover the
    /// inner value as the concrete type `T`. The type check is performed
    /// based on the stored [`TypeId`](std::any::TypeId): if the stored type
    /// matches `T`, the value is extracted and returned inside `Some`;
    /// otherwise, [`None`] is returned.
    ///
    /// # Safety
    ///
    /// This is only safe when `T` matches the [`TypeId`](std::any::TypeId) stored in the `AnyOutput`.
    /// Generated code (via `gen_program!()`) guarantees this by dispatching on
    /// `member_id` before calling `restore`.
    ///
    /// # Arguments
    ///
    /// - `self`: The `AnyOutput` value to attempt restoration from.
    ///
    /// # Returns
    ///
    /// - `Some(T)`: The inner value successfully restored to the concrete type `T`.
    /// - `None`: The stored [`TypeId`](std::any::TypeId) does not match
    ///   [`TypeId::of::<T>()`](std::any::TypeId).
    ///
    /// # Examples
    ///
    /// ```
    /// # use mingling_core::MockProgramCollect as ThisProgram;
    /// use mingling_core::AnyOutput;
    /// use mingling_core::Grouped;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Foo(i32);
    ///
    /// // SAFETY: The member_id corresponds to the correct ThisProgram variant.
    /// unsafe impl Grouped<ThisProgram> for Foo {
    ///     fn member_id() -> ThisProgram {
    ///         ThisProgram::Foo
    ///     }
    /// }
    ///
    /// // Successful restore to the matching type.
    /// let output = AnyOutput::new(Foo(42));
    /// let restored: Option<Foo> = output.restore::<Foo>();
    /// assert_eq!(restored, Some(Foo(42)));
    ///
    /// // Failed restore to a non-matching type returns None.
    /// let output = AnyOutput::new(Foo(7));
    /// let restored: Option<String> = output.restore::<String>();
    /// assert_eq!(restored, None);
    /// ```
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

/// Chain execution result type
///
/// Stores the `Ok` and `Err` types of an execution result, indicating to the dispatcher
/// what action to perform next:
/// - Returns <code>Ok(([AnyOutput](./struct.AnyOutput.html), [NextProcess::Chain](./enum.NextProcess.html)))</code>, indicating that this type should be used to continue to the next step
/// - Returns <code>Ok(([AnyOutput](./struct.AnyOutput.html), [NextProcess::Renderer](./enum.NextProcess.html)))</code>, indicating that this type should be sent to the renderer and output to the terminal
/// - Returns <code>Err([ChainProcessError](./error/enum.ChainProcessError.html))</code>, indicating that the program should be terminated immediately
///
/// # Type Parameters
///
/// - `G`: The program group enum type, used to identify the concrete type of the output,
///   whose value comes from [`Grouped::member_id`].
///
/// # Examples
///
/// ```
/// # use mingling_core::MockProgramCollect as ThisProgram;
/// # use mingling_core::{ChainProcess, Grouped, NextProcess, AnyOutput, error::ChainProcessError};
/// # struct Foo;
/// # unsafe impl Grouped<ThisProgram> for Foo {
/// #   fn member_id() -> ThisProgram {
/// #       ThisProgram::Foo
/// #   }
/// # }
/// # let output = AnyOutput::new(Foo);
///
/// // Successfully executed in the chain, continue to the next step
/// let result: ChainProcess<ThisProgram> = ChainProcess::Ok((output, NextProcess::Chain));
///
/// // An error occurred during chain processing, terminating the program
/// let error: ChainProcess<ThisProgram> = ChainProcess::Err(ChainProcessError::Other("error".into()));
/// ```
pub enum ChainProcess<G> {
    /// Indicates processing was successful, containing the output value and the next action to perform.
    ///
    /// The first element of the tuple is an `AnyOutput` (the type-erased output value),
    /// and the second element is a `NextProcess`, used to instruct the dispatcher to
    /// route the output to the next step of chain processing (`Chain`) or send it
    /// to the renderer (`Renderer`).
    Ok((AnyOutput<G>, NextProcess)),
    /// Indicates processing failed, containing information about the error that occurred.
    ///
    /// This variant is returned when an error occurs during chain processing,
    /// and the dispatcher will terminate program execution.
    Err(ChainProcessError),
}

/// Indicates the next action to take after processing.
///
/// - `Chain`：Continue to the next chain step
/// - `Renderer`：Send the output to the renderer and end execution
///
/// This enum is used as the second element in the [`ChainProcess::Ok`] variant's tuple,
/// indicating to the dispatcher what action to take after obtaining the output:
/// - When it is [`NextProcess::Chain`], the dispatcher will pass the output to the next
///   chain step to continue processing;
/// - When it is [`NextProcess::Renderer`], the dispatcher will send the output to the
///   renderer for terminal display and terminate program execution.
///
/// # Examples
///
/// ```
/// # use mingling_core::MockProgramCollect as ThisProgram;
/// # use mingling_core::{AnyOutput, ChainProcess, NextProcess};
/// # use mingling_core::Grouped;
/// #
/// # struct Foo(i32);
/// # unsafe impl Grouped<ThisProgram> for Foo {
/// #     fn member_id() -> ThisProgram {
/// #         ThisProgram::Foo
/// #     }
/// # }
/// #
/// let output = AnyOutput::new(Foo(42));
///
/// // Route to the next chain step
/// let cp = output.route_chain();
/// match cp {
///     ChainProcess::Ok((any, next)) => {
///         // Confirm the next action is Chain
///         assert_eq!(next, NextProcess::Chain);
///     }
///     ChainProcess::Err(_) => panic!("Expected Ok"),
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NextProcess {
    /// Continue to the next chain step
    ///
    /// This value indicates that the output should be passed to the next chain step
    /// to continue program processing, rather than being sent directly to the renderer.
    /// This is the most commonly used routing method during chained processing.
    Chain,
    /// Send the output to the renderer and end execution
    ///
    /// This value indicates that the output should be sent directly to the renderer
    /// for terminal display, and program execution will terminate after this output.
    /// This is typically used for the final output of program processing.
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
