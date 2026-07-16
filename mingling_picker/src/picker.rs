use std::{marker::PhantomData, ops::Index};

mod parse;

mod patterns;
pub use patterns::*;

mod result;
pub use result::*;

use crate::{Pickable, PickerArg, PickerArgResult};

/// Picker, used to record all states of a parameter parsing
///
/// Includes the following:
///
/// - Basic arguments
/// - Parsing states
/// - Parsing results
pub struct Picker<'a, Route = ()> {
    route_phantom: PhantomData<Route>,

    /// Internal arguments of Picker
    args: PickerArgs<'a>,
}

impl<'a> Picker<'a> {
    /// Creates a new `Picker` from the command-line arguments (excluding the program name).
    ///
    /// This is equivalent to calling `std::env::args().skip(1)`, which
    /// collects all arguments passed to the program except the first one
    /// (the executable path).
    pub fn from_args() -> Picker<'a, ()> {
        Self::from_args_skip(1)
    }

    /// Creates a new `Picker` from the command-line arguments, skipping the
    /// first `skip` entries.
    ///
    /// This method is useful when you want more control over which arguments
    /// are included. For example, pass `skip = 2` to skip both the program
    /// name and the first argument.
    pub fn from_args_skip(skip: usize) -> Picker<'a, ()> {
        let args = std::env::args().skip(skip).collect::<Vec<String>>();
        Picker {
            route_phantom: PhantomData,
            args: PickerArgs::Owned(args),
        }
    }

    /// Changes the route (phantom type parameter) of the `Picker`.
    ///
    /// This method allows converting a `Picker` from one route type to another,
    /// while preserving the same underlying arguments. The route type is typically
    /// used to distinguish different parsing contexts or to carry compile-time
    /// state information through the picking chain.
    pub fn with_route<NewRoute>(self) -> Picker<'a, NewRoute>
    where
        Self: Sized,
    {
        Picker {
            route_phantom: PhantomData,
            args: self.args,
        }
    }
}

/// Internal arguments of Picker
///
/// - `Slice` - borrowed slice of string slices
/// - `Vec` - owned vector of borrowed string slices
/// - `Owned` - owned vector of owned strings
pub enum PickerArgs<'a> {
    /// Borrowed slice of string slices
    Slice(&'a [&'a str]),
    /// Owned vector of borrowed string slices
    Vec(Vec<&'a str>),
    /// Owned vector of owned strings
    Owned(Vec<String>),
}

impl<'a> Default for PickerArgs<'a> {
    fn default() -> Self {
        Self::Vec(vec![])
    }
}

impl<'a> PickerArgs<'a> {
    /// Returns the number of arguments.
    pub fn len(&self) -> usize {
        match self {
            PickerArgs::Slice(items) => items.len(),
            PickerArgs::Vec(items) => items.len(),
            PickerArgs::Owned(items) => items.len(),
        }
    }

    /// Returns `true` if there are no arguments.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the arguments, yielding `&str` values.
    pub fn iter(&'a self) -> PickerIter<'a> {
        match self {
            PickerArgs::Slice(items) => PickerIter::Slice(items.iter()),
            PickerArgs::Vec(items) => PickerIter::Vec(items.iter()),
            PickerArgs::Owned(items) => PickerIter::Owned(items.iter()),
        }
    }

    /// Returns a reference to the argument at `index`, if it exists.
    pub fn get(&self, index: usize) -> Option<&str> {
        match self {
            PickerArgs::Slice(items) => items.get(index).copied(),
            PickerArgs::Vec(items) => items.get(index).copied(),
            PickerArgs::Owned(items) => items.get(index).map(|s| s.as_str()),
        }
    }
}

impl<'a> Index<usize> for PickerArgs<'a> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            PickerArgs::Slice(items) => items[index],
            PickerArgs::Vec(items) => items[index],
            PickerArgs::Owned(items) => &items[index],
        }
    }
}

impl<'a> IntoIterator for &'a PickerArgs<'a> {
    type Item = &'a str;
    type IntoIter = PickerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            PickerArgs::Slice(items) => PickerIter::Slice(items.iter()),
            PickerArgs::Vec(items) => PickerIter::Vec(items.iter()),
            PickerArgs::Owned(items) => PickerIter::Owned(items.iter()),
        }
    }
}

impl<'a, Route> From<&'a [&'a str]> for Picker<'a, Route> {
    fn from(value: &'a [&'a str]) -> Self {
        Picker {
            route_phantom: PhantomData,
            args: PickerArgs::Slice(value),
        }
    }
}

impl<'a, Route> From<Vec<&'a str>> for Picker<'a, Route> {
    fn from(value: Vec<&'a str>) -> Self {
        Picker {
            route_phantom: PhantomData,
            args: PickerArgs::Vec(value),
        }
    }
}

impl<'a, Route> From<Vec<String>> for Picker<'a, Route> {
    fn from(value: Vec<String>) -> Self {
        Picker {
            route_phantom: PhantomData,
            args: PickerArgs::Owned(value),
        }
    }
}

impl<'a, Route> Picker<'a, Route> {
    /// Returns a reference to the internal `PickerArgs`.
    pub fn args(&self) -> &PickerArgs<'a> {
        &self.args
    }

    /// Returns a mutable reference to the internal `PickerArgs`.
    pub fn args_mut(&mut self) -> &mut PickerArgs<'a> {
        &mut self.args
    }

    /// Consumes `self` and returns the internal `PickerArgs`.
    pub fn into_args(self) -> PickerArgs<'a> {
        self.args
    }

    /// Returns the number of arguments.
    pub fn len(&self) -> usize {
        self.args.len()
    }

    /// Returns `true` if there are no arguments.
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Returns an iterator over the arguments, yielding `&str` values.
    pub fn iter(&'a self) -> PickerIter<'a> {
        self.args.iter()
    }
}

impl<'a, Route> Index<usize> for Picker<'a, Route> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        &self.args[index]
    }
}

impl<'a, Route> Index<usize> for &Picker<'a, Route> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        &self.args[index]
    }
}

impl<'a, Route> IntoIterator for &'a Picker<'a, Route> {
    type Item = &'a str;
    type IntoIter = PickerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter()
    }
}

/// Iterator for `Picker` (and `PickerArgs`), yielding `&'a str` values.
pub enum PickerIter<'a> {
    /// Iterates over a borrowed slice (`&[&str]`)
    Slice(std::slice::Iter<'a, &'a str>),
    /// Iterates over an owned vector of borrowed string slices (`Vec<&str>`)
    Vec(std::slice::Iter<'a, &'a str>),
    /// Iterates over an owned vector of owned strings (`Vec<String>`)
    Owned(std::slice::Iter<'a, String>),
}

impl<'a> Iterator for PickerIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PickerIter::Slice(iter) => iter.next().copied(),
            PickerIter::Vec(iter) => iter.next().copied(),
            PickerIter::Owned(iter) => iter.next().map(|s| s.as_str()),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            PickerIter::Slice(iter) => iter.size_hint(),
            PickerIter::Vec(iter) => iter.size_hint(),
            PickerIter::Owned(iter) => iter.size_hint(),
        }
    }
}

impl<'a> ExactSizeIterator for PickerIter<'a> {}

/// Trait for converting types into a `Picker`
///
/// Implemented for:
/// - `&[&str]` (borrowed slice)
/// - `&[String]` (borrowed slice of owned strings)
/// - `Vec<&str>` (owned vector of borrowed strings)
/// - `Vec<String>` (owned vector of owned strings)
pub trait IntoPicker<'a> {
    /// Converts the value into a `Picker`
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_picker::{IntoPicker, Picker};
    ///
    /// let args: Picker = (&["hello", "world"][..]).to_picker();
    /// assert_eq!(args.len(), 2);
    ///
    /// let args: Picker = vec!["foo", "bar"].to_picker();
    /// assert_eq!(args.len(), 2);
    ///
    /// let args: Picker = vec!["a".to_string(), "b".to_string()].to_picker();
    /// assert_eq!(args.len(), 2);
    /// ```
    fn to_picker(self) -> Picker<'a, ()>;

    /// Creates a `PickerPattern1` from the given arg for the `pick` method.
    ///
    /// This method converts the value into a `Picker` and starts a parameter
    /// picking chain with one arg. The result is initially `Unparsed`.
    fn pick<N>(self, arg: impl Into<&'a PickerArg<'a, N>>) -> PickerPattern1<'a, N, ()>
    where
        Self: Sized,
        N: Pickable<'a> + Default + Sized,
    {
        PickerPattern1 {
            args: self.to_picker().args,
            arg_1: arg.into(),
            result_1: PickerArgResult::Unparsed,
            default_1: None,
            route_1: None,
            post_1: None,
            error_route: None,
        }
    }

    /// Converts the value into a `Picker` with a specified route type.
    ///
    /// This method allows changing the route (phantom type parameter) of the picker.
    /// The route type is typically used to distinguish different parsing contexts or
    /// to carry compile-time state information through the picking chain.
    fn with_route<NewRoute>(self) -> Picker<'a, NewRoute>
    where
        Self: Sized,
    {
        Picker {
            route_phantom: PhantomData,
            args: self.to_picker().args,
        }
    }
}

impl<'a> IntoPicker<'a> for &'a [&'a str] {
    fn to_picker(self) -> Picker<'a, ()> {
        Picker {
            route_phantom: PhantomData,
            args: PickerArgs::Slice(self),
        }
    }
}

impl<'a> IntoPicker<'a> for &'a [String] {
    fn to_picker(self) -> Picker<'a, ()> {
        let vec: Vec<&str> = self.iter().map(|s| s.as_str()).collect();
        Picker {
            route_phantom: PhantomData,
            args: PickerArgs::Vec(vec),
        }
    }
}

impl<'a> IntoPicker<'a> for Vec<&'a str> {
    fn to_picker(self) -> Picker<'a, ()> {
        Picker {
            route_phantom: PhantomData,
            args: PickerArgs::Vec(self),
        }
    }
}

impl<'a> IntoPicker<'a> for Vec<String> {
    fn to_picker(self) -> Picker<'a, ()> {
        Picker {
            route_phantom: PhantomData,
            args: PickerArgs::Owned(self),
        }
    }
}
