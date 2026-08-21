// Doc Not Optimize
use serde::Serialize;

use crate::ProgramCollect;

/// Marker trait for types that support structured output (JSON / YAML / TOML / RON).
///
/// This trait is a **supertrait** of `serde::Serialize` and is sealed via
/// `__private::StructuralDataSealed`. It can only be implemented through
/// `#[derive(StructuralData)]`, which registers the type in the global
/// `STRUCTURED_TYPES` registry, required for the `structural_render` match arm
/// to be generated.
///
/// The trait is publicly accessible at `mingling::StructuralData`, where it
/// coexists with the same-named derive macro.
pub trait StructuralData<C>: Serialize + crate::__private::StructuralDataSealed<C>
where
    C: ProgramCollect<Enum = C>,
{
}
