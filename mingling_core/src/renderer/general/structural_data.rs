use serde::Serialize;

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
/// registry, which is required for the `general_render` match arm to be generated.
#[doc(hidden)]
pub trait StructuralData: Serialize + crate::__private::StructuralDataSealed {}
