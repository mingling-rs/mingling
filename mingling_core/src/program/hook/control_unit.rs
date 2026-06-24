use crate::{AnyOutput, ProgramCollect};

/// Collection variants for program control instructions.
///
/// Defines different forms of program control collections.
pub enum ProgramControls<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Empty collection.
    Empty,

    /// A single control unit.
    Single(ProgramControlUnit<C>),

    /// A collection of multiple control units.
    Multi(Vec<ProgramControlUnit<C>>),
}

impl<C> ProgramControls<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Returns `true` if the collection is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, ProgramControls::Empty)
    }
}

impl<C> From<()> for ProgramControls<C>
where
    C: ProgramCollect<Enum = C>,
{
    fn from(_: ()) -> Self {
        Self::Empty
    }
}

impl<C> From<ProgramControlUnit<C>> for ProgramControls<C>
where
    C: ProgramCollect<Enum = C>,
{
    fn from(unit: ProgramControlUnit<C>) -> Self {
        Self::Single(unit)
    }
}

impl<C> From<Vec<ProgramControlUnit<C>>> for ProgramControls<C>
where
    C: ProgramCollect<Enum = C>,
{
    fn from(units: Vec<ProgramControlUnit<C>>) -> Self {
        Self::Multi(units)
    }
}

impl<C> IntoIterator for ProgramControls<C>
where
    C: ProgramCollect<Enum = C>,
{
    type Item = ProgramControlUnit<C>;
    type IntoIter = ProgramControlsIter<C>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ProgramControls::Empty => ProgramControlsIter {
                inner: vec![].into_iter(),
            },
            ProgramControls::Single(unit) => ProgramControlsIter {
                inner: vec![unit].into_iter(),
            },
            ProgramControls::Multi(units) => ProgramControlsIter {
                inner: units.into_iter(),
            },
        }
    }
}

/// An iterator over [`ProgramControlUnit`] values.
pub struct ProgramControlsIter<C>
where
    C: ProgramCollect<Enum = C>,
{
    inner: std::vec::IntoIter<ProgramControlUnit<C>>,
}

impl<C> Iterator for ProgramControlsIter<C>
where
    C: ProgramCollect<Enum = C>,
{
    type Item = ProgramControlUnit<C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<C> std::iter::FusedIterator for ProgramControlsIter<C>
where
    C: ProgramCollect<Enum = C>,
{
    // Auto impl
}

/// Enumeration of program control units.
///
/// Defines the various control flow instructions that a program may encounter during execution,
/// used to alter the default execution flow (e.g., interruption, jump, or redirection).
pub enum ProgramControlUnit<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Override the program exit code.
    ///
    /// Used when a non-default process exit code needs to be forcibly specified.
    /// The contained `i32` value is the exit code to be set.
    OverrideExitCode(i32),

    /// Route to the render flow.
    ///
    /// Transfers control to the rendering (output) stage,
    /// carrying the `AnyOutput<C>` to be rendered.
    RouteToRender(AnyOutput<C>),

    /// Route to the chain processing flow.
    ///
    /// Transfers control to the next chained processor,
    /// carrying the `AnyOutput<C>` that needs to be passed along.
    RouteToChain(AnyOutput<C>),

    /// Route to the help information flow.
    ///
    /// Transfers control to the help information display module,
    /// carrying the `AnyOutput<C>` containing help-related content.
    RouteToHelp(AnyOutput<C>),
}
