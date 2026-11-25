//! Inline render streaming for `render!` and related macros.
//!
//! This module provides the runtime side of the `render!` family:
//! - [`RenderEmit`] selects the output stream policy.
//! - [`RenderedRecord`] is one complete record to emit.
//! - [`render_stream`] dispatches a value through the registered renderer (or
//!   the structural stream renderer) and forwards the resulting records to the
//!   active [`Program`](crate::Program) stream context.

use std::io::Write;

use crate::{AnyOutput, Grouped, Program, ProgramCollect, RenderResult, RenderResultMode};

#[cfg(feature = "structural_renderer")]
use crate::config::StructuralRendererSetting;

/// Selects the destination mode for an inline render.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RenderEmit {
    /// Preserve each rendered entry's own [`RenderResultMode`].
    #[default]
    Auto,
    /// Flatten every entry to stdout.
    Stdout,
    /// Flatten every entry to stderr.
    Stderr,
}

/// A single complete record produced by an inline render.
///
/// A record may contain multiple lines (for example pretty JSON or YAML), but
/// it is always emitted as one unit so concurrent records do not interleave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedRecord {
    /// The record's text content.
    pub content: String,
    /// The stream the record should be written to.
    pub mode: RenderResultMode,
}

impl RenderedRecord {
    /// Creates a new record with the given content and output mode.
    #[must_use]
    pub fn new(content: impl Into<String>, mode: RenderResultMode) -> Self {
        Self {
            content: content.into(),
            mode,
        }
    }
}

/// The program's current inline-render state.
///
/// `Direct` writes records immediately (used by `exec`), while `Capture`
/// collects them for `exec_without_render` and tests.
#[derive(Debug)]
pub enum RenderStreamState {
    /// Write records immediately when `enabled`; otherwise drop them.
    Direct {
        /// Whether output is currently enabled.
        enabled: bool,
    },
    /// Collect records for later retrieval.
    Capture(Vec<RenderedRecord>),
}

impl Default for RenderStreamState {
    fn default() -> Self {
        Self::Capture(Vec::new())
    }
}

/// Renders `value` inline and emits the result according to `emit`.
///
/// The dispatch follows the program's structural renderer setting:
/// - When structural rendering is disabled, or the value's type is not
///   registered as structural data, the registered plain renderer is used.
/// - Otherwise the value is rendered as one structural stream record.
///
/// The plain renderer is executed in both cases so its side effects (for
/// example setting `ResExitCode`) are preserved. When structural rendering
/// succeeds, the plain renderer's text output is discarded.
///
/// # Panics
///
/// Panics if the program has not been initialized (i.e. `render_stream` was
/// called outside program execution), or if structural serialization fails.
pub fn render_stream<C, T>(value: &T, emit: RenderEmit)
where
    C: ProgramCollect<Enum = C> + 'static,
    T: Grouped<C> + Clone + Send + 'static,
{
    let program = crate::this::<C>();
    let any = AnyOutput::new(value.clone());

    #[cfg(feature = "structural_renderer")]
    let structural = {
        let setting = &program.structural_renderer_name;
        if matches!(setting, StructuralRendererSetting::Disable) || !C::has_structural(&any) {
            None
        } else {
            Some(C::structural_render_stream(&any, setting))
        }
    };

    // Always run the plain renderer so its side effects are preserved.
    let plain = C::render(any);

    #[cfg(feature = "structural_renderer")]
    {
        match structural {
            Some(Ok(result)) => {
                drop(plain);
                emit_result(result, emit, program);
            }
            Some(Err(err)) => {
                drop(plain);
                panic!(
                    "failed to render `{}` structurally: {}",
                    std::any::type_name::<T>(),
                    String::from(err)
                );
            }
            None => emit_result(plain, emit, program),
        }
    }

    #[cfg(not(feature = "structural_renderer"))]
    {
        emit_result(plain, emit, program);
    }
}

fn emit_result<C>(mut result: RenderResult, emit: RenderEmit, program: &Program<C>)
where
    C: ProgramCollect<Enum = C> + 'static,
{
    match emit {
        RenderEmit::Auto => {}
        RenderEmit::Stdout => result = result.flattened(RenderResultMode::Stdout),
        RenderEmit::Stderr => result = result.flattened(RenderResultMode::Stderr),
    }

    for (content, mode) in result.into_entries() {
        program.emit_rendered_record(RenderedRecord::new(content, mode));
    }
}

/// Writes one record to its target stream and flushes it immediately.
pub fn write_rendered_record(record: &RenderedRecord) {
    let write_result = match record.mode {
        RenderResultMode::Stdout => {
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();
            lock.write_all(record.content.as_bytes())
                .and_then(|()| lock.flush())
        }
        RenderResultMode::Stderr => {
            let stderr = std::io::stderr();
            let mut lock = stderr.lock();
            lock.write_all(record.content.as_bytes())
                .and_then(|()| lock.flush())
        }
    };

    // Broken pipes and other write failures cannot be meaningfully reported
    // from a fire-and-forget stream record.
    let _ = write_result;
}
