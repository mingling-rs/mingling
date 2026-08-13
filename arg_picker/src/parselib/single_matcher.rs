// Doc Not Optimize
use crate::TagPhaseContext;
use crate::parselib::{ArgMatcher, Matcher, ParserStyle, PositionalMatcher};

/// `SingleMatcher` is a composite matcher for single-value parameters.
///
/// It delegates to [`PositionalMatcher`] for positional args and
/// [`ArgMatcher`] for named args, adding a guard: if a named flag
/// captures only itself with no inline value (eq mode), the result
/// is cleared so that [`Pickable::pick`](crate::Pickable::pick) receives `[]` → `NotFound`.
///
/// This is the standard tag implementation for all `Single`-type
/// `Pickable` implementations (e.g., `String`, `i32`, `u64`).
pub struct SingleMatcher;

impl SingleMatcher {
    /// Match a single positional value or a named flag+value pair.
    ///
    /// For named args, only complete pairs (flag + value) are kept.
    /// Flag occurrences without a following value or inline separator
    /// are dropped so they remain available for other matchers.
    #[inline]
    #[must_use]
    pub fn tag(ctx: TagPhaseContext) -> Vec<usize> {
        if ctx.arg_info.positional {
            PositionalMatcher::match_one(ctx.into()).map_or_else(Vec::new, |i| vec![i])
        } else {
            let args = ctx.args;
            let positions = ArgMatcher::match_all(ctx.into());
            let sep = ParserStyle::global_style().value_separator;

            // Walk pairs: [flag, value, flag, value, ...]
            // Drop any flag that has no following value and no inline separator.
            let mut i = 0;
            let mut result = Vec::with_capacity(positions.len());
            while i < positions.len() {
                let flag_idx = positions[i];
                if let Some(raw) = args.get(flag_idx)
                    && raw.contains(sep)
                {
                    // Eq mode: value is inline, keep just the flag.
                    result.push(flag_idx);
                    i += 1;
                    continue;
                }
                if i + 1 < positions.len() {
                    // Pair: flag + value.
                    result.push(flag_idx);
                    result.push(positions[i + 1]);
                    i += 2;
                } else {
                    // Flag without value — drop it.
                    i += 1;
                }
            }
            result
        }
    }
}
