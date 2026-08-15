/// Specifies the maximum number of attempts for a confirmation prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmerCount {
    /// Loop indefinitely until the user gives a parseable answer.
    Loop,
    /// Ask at most the specified number of times.
    Max(usize),
}

macro_rules! impl_from_for_confirmer_count {
    ($($t:ty),*) => {
        $(
            impl From<$t> for ConfirmerCount {
                fn from(n: $t) -> Self {
                    if n == 0 {
                        ConfirmerCount::Loop
                    } else {
                        match usize::try_from(n) {
                            Ok(max) => ConfirmerCount::Max(max),
                            Err(_) => ConfirmerCount::Max(usize::MAX),
                        }
                    }
                }
            }
        )*
    };
}

impl_from_for_confirmer_count!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
