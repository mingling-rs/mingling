use arg_picker::{Pickable, PickerArg, PickerArgInfo, parselib::ParserStyle};
use mingling_core::{Suggest, SuggestItem};

/// Trait for converting picker arguments into `Suggest` items for autocompletion.
///
/// This trait is implemented for `PickerArg` references and provides methods to
/// generate completion suggestions with optional descriptions.
pub trait PickerArgSuggest {
    #[doc(hidden)]
    fn into_suggest_raw(self, description: Option<&String>) -> Suggest;

    /// Converts the picker argument into a `Suggest` with the given description.
    ///
    /// The description is used when the suggestion item supports additional text,
    /// such as `SuggestItem::WithDescription`.
    fn into_suggest_with_desc(self, description: impl Into<String>) -> Suggest
    where
        Self: Sized,
    {
        let desc = description.into();
        self.into_suggest_raw(Some(&desc))
    }

    /// Converts the picker argument into a `Suggest` without a description.
    ///
    /// The resulting suggestion will use `SuggestItem::Simple` for each possible flag.
    fn into_suggest(self) -> Suggest
    where
        Self: Sized,
    {
        self.into_suggest_raw(None)
    }
}

impl<'a, T> PickerArgSuggest for &'a PickerArg<'a, T>
where
    T: Pickable<'a>,
{
    fn into_suggest_raw(self, description: Option<&String>) -> Suggest {
        let mut suggest = Suggest::new();
        let info = PickerArgInfo::from(self);
        let possible_flags =
            arg_picker::parselib::build_possible_flags(ParserStyle::global_style(), &info);
        for flag in possible_flags {
            match description {
                Some(desc) => suggest.insert(SuggestItem::WithDescription(flag, desc.clone())),
                None => suggest.insert(SuggestItem::Simple(flag)),
            };
        }
        suggest
    }
}
