use serde::Serialize;

use crate::ProgramCollect;

/// Marker trait for types that support structured output (JSON / YAML / TOML / RON).
///
/// This trait is a **supertrait** of `serde::Serialize` and is sealed via
/// `__private::StructuralDataSealed`. It can only be implemented through:
///
/// - `#[derive(StructuralData)]`
/// - `pack_structural!`
/// - `group_structural!`
///
/// These entry points also register the type in the global `STRUCTURED_TYPES`
/// registry, which is required for the `structural_render` match arm to be generated.
#[doc(hidden)]
pub trait StructuralData<C>: Serialize + crate::__private::StructuralDataSealed<C>
where
    C: ProgramCollect<Enum = C>,
{
}
