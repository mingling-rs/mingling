// Doc Not Optimize
use serde::Serialize;

use crate::ProgramCollect;

/// Marker trait for types that support structured output (JSON / YAML / TOML / RON).
///
/// This trait is a **supertrait** of `serde::Serialize` and is normally
/// implemented through `#[derive(StructuralData)]`, which registers the type
/// for the `structural_render` match arm so it can be rendered structurally
/// without a separate `#[renderer]` function. It may also be implemented
/// manually (e.g. for external types), which only requires `Serialize`.
///
/// The trait is publicly accessible at `mingling::StructuralData`, where it
/// coexists with the same-named derive macro.
pub trait StructuralData<C>: Serialize
where
    C: ProgramCollect<Enum = C>,
{
}
