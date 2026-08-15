/// Specifies the maximum number of attempts for a confirmation prompt.
///
/// # Default Implementations
///
/// `ConfirmCount` implements the following traits by default:
///
/// - [`Debug`] — for formatted output and debugging.
/// - [`Clone`] — to create a copy of the value.
/// - [`Copy`] — since the enum holds no heap-allocated data, it can be trivially copied.
/// - [`PartialEq`] — allows comparing two `ConfirmCount` values for equality.
/// - [`Eq`] — provides full equality semantics (as opposed to just partial).
/// - [`From<T>`] for all primitive integer types (`i8`–`i128`, `isize`, `u8`–`u128`, `usize`),
///   allowing convenient conversion from a raw number.
///
/// # What the Numbers Mean
///
/// The numeric value passed to a `From` conversion represents the **maximum number of times**
/// the confirmation prompt will be shown to the user. For example:
///
/// - `ConfirmCount::from(3)` → asks at most **3** times before giving up.
/// - `ConfirmCount::from(1)` → asks exactly **1** time.
/// - `ConfirmCount::from(0)` → interpreted as [`Loop`], meaning it will keep asking
///   indefinitely until a valid answer is parsed.
///
/// # Examples
///
/// ```
/// use mingling::confirm::ConfirmCount;
///
/// // Convert from a numeric value
/// let count: ConfirmCount = 3.into();
/// assert_eq!(count, ConfirmCount::Max(3));
///
/// // Zero means loop forever
/// let loop_count: ConfirmCount = 0.into();
/// assert_eq!(loop_count, ConfirmCount::Loop);
///
/// // Large values are capped at usize::MAX
/// let big: ConfirmCount = i128::MAX.into();
/// assert_eq!(big, ConfirmCount::Max(usize::MAX));
///
/// // From a usize directly
/// let from_usize = ConfirmCount::from(5usize);
/// assert_eq!(from_usize, ConfirmCount::Max(5));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmCount {
    /// Loop indefinitely until the user gives a parseable answer.
    Loop,
    /// Ask at most the specified number of times.
    Max(usize),
}

macro_rules! impl_from_for_Confirm_count {
    ($($t:ty),*) => {
        $(
            impl From<$t> for ConfirmCount {
                fn from(n: $t) -> Self {
                    if n == 0 {
                        ConfirmCount::Loop
                    } else {
                        match usize::try_from(n) {
                            Ok(max) => ConfirmCount::Max(max),
                            Err(_) => ConfirmCount::Max(usize::MAX),
                        }
                    }
                }
            }
        )*
    };
}

impl_from_for_Confirm_count!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
