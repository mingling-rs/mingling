// Doc Not Optimize
use size::Size;

use crate::parser::{Argument, Pickable};

impl Pickable for String {
    type Output = Self;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        args.pick_argument(flag)
    }
}

impl Pickable for Vec<String> {
    type Output = Self;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        Some(args.pick_arguments(flag))
    }
}

macro_rules! impl_pickable_for_number {
    ($($t:ty),*) => {
        $(
            impl Pickable for $t {
                type Output = $t;

                fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
                    let picked = args.pick_argument(flag)?;
                    picked.parse().ok()
                }
            }

            impl Pickable for Vec<$t> {
                type Output = Vec<$t>;

                fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
                    let picked_vec = args.pick_arguments(flag);
                    let mut result = Vec::new();
                    for picked in picked_vec {
                        if let Ok(parsed) = picked.parse() {
                            result.push(parsed);
                        } else {
                            return None;
                        }
                    }
                    Some(result)
                }
            }
        )*
    };
}

impl_pickable_for_number!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);

impl Pickable for bool {
    type Output = Self;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        Some(args.pick_flag(flag))
    }
}

/// Special: parses a size string (e.g. "10MB") into a `usize` representing the number of bytes.
impl Pickable for usize {
    type Output = Self;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        let picked = args.pick_argument(flag)?;
        let size_parse = Size::from_str(picked.as_str());
        size_parse.map_or(None, |size| Self::try_from(size.bytes()).ok())
    }
}

/// Special: parses a comma-separated list of size strings (e.g. "10MB,20KB") into a `Vec<usize>`.
impl Pickable for Vec<usize> {
    type Output = Self;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        let picked_vec = args.pick_arguments(flag);
        let mut result = Self::new();
        for picked in picked_vec {
            let size_parse = Size::from_str(picked.as_str());
            match size_parse {
                Ok(size) => result.push(usize::try_from(size.bytes()).unwrap_or(usize::MAX)),
                Err(_) => return None,
            }
        }
        Some(result)
    }
}

/// Special: dumps the remaining arguments into an `Argument` struct.
impl Pickable for Argument {
    type Output = Self;

    fn pick(
        args: &mut crate::parser::Argument,
        _flag: mingling_core::Flag,
    ) -> Option<Self::Output> {
        Some(args.dump_remains().into())
    }
}

/// Special: parses a single value of type `T` using the `Pickable` implementation for `T`, and wraps it in an `Option`.
impl<T: Pickable<Output = T> + Default> Pickable for Option<T> {
    type Output = Self;

    fn pick(args: &mut Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        let r = T::pick(args, flag);
        Some(r)
    }
}
