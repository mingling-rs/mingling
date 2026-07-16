use crate::TagPhaseContext;
use crate::parselib::{ArgMatcher, Matcher, ParserStyle, PositionalMatcher};

/// `SingleMatcher` is a composite matcher for single-value parameters.
///
/// It delegates to [`PositionalMatcher`] for positional args and
/// [`ArgMatcher`] for named args, adding a guard: if a named flag
/// captures only itself with no inline value (eq mode), the result
/// is cleared so that [`Pickable::pick`] receives `[]` → `NotFound`.
///
/// This is the standard tag implementation for all `Single`-type
/// `Pickable` implementations (e.g., `String`, `i32`, `u64`).
pub struct SingleMatcher;

impl SingleMatcher {
    /// Match a single positional value or a named flag+value pair.
    ///
    /// For named args, returns `[]` when the flag has no following
    /// value and no inline separator — indicating a missing value.
    #[inline(always)]
    pub fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        if ctx.arg_info.positional {
            PositionalMatcher::match_one(ctx.into())
                .map(|i| vec![i])
                .unwrap_or_default()
        } else {
            let args = ctx.args;
            let positions = ArgMatcher::match_all(ctx.into());
            if positions.len() == 1 {
                let sep = ParserStyle::global_style().value_separator;
                if let Some(raw) = args.get(positions[0])
                    && !raw.contains(sep) {
                        return vec![];
                    }
            }
            positions
        }
    }
}
