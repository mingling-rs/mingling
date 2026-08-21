// Doc Not Optimize
use serde::Serialize;

use crate::ProgramCollect;

/// Marker trait for types that support structured output (JSON / YAML / TOML / RON).
///
/// This trait is a **supertrait** of `serde::Serialize` and is normally
/// implemented through `#[derive(StructuralData)]`, which also registers the
/// type in the global `STRUCTURED_TYPES` registry, required for the
/// `structural_render` match arm to be generated. It may also be implemented
/// manually (e.g. for external types), which only requires `Serialize`.
///
/// The trait is publicly accessible at `mingling::StructuralData`, where it
/// coexists with the same-named derive macro.
pub trait StructuralData<C>: Serialize
where
    C: ProgramCollect<Enum = C>,
{
}
