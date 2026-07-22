/// Sealed trait for `StructuralData` — only implementable via derive macro.
#[cfg(feature = "structural_renderer")]
pub trait StructuralDataSealed<C>
where
    C: crate::ProgramCollect<Enum = C>,
{
}

/// Re-export so the derive macro can reference the trait without
/// conflicting with the derive macro name at `::mingling::StructuralData`.
#[cfg(feature = "structural_renderer")]
pub use crate::renderer::structural::structural_data::StructuralData;
