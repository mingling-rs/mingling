// Doc Not Optimize
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{Program, ProgramCollect};

/// A single-slot container that can be initialized once, read many times,
/// and taken out once (for cleanup before process exit).
///
/// # Safety
///
/// - `set()` is called once during `exec_wrapper`, before any other access.
/// - `get_raw()` is called during execution (concurrent reads are safe because
///   the inner value is immutable once set until `take()`).
/// - `take()` is called only after execution completes, when no code still
///   holds a reference from `get_raw()`.
pub struct ProgramCell {
    initialized: AtomicBool,
    inner: UnsafeCell<Option<Box<dyn std::any::Any + Send + Sync>>>,
}

// Safety: Sync is safe because:
// - `initialized` is AtomicBool (Sync)
// - `inner` is only read-after-write in sequence: set() → repeated get(), then
//   optionally take() after all get() callers are done.
// - No concurrent write+read or write+write exists.
unsafe impl Sync for ProgramCell {}

impl ProgramCell {
    pub(crate) const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            inner: UnsafeCell::new(None),
        }
    }

    /// Initialize the cell with a value. Panics if already initialized.
    pub(crate) fn set(&self, val: Box<dyn std::any::Any + Send + Sync>) {
        assert!(
            !self.initialized.swap(true, Ordering::AcqRel),
            "ProgramCell already initialized"
        );
        // SAFETY: `set()` is the sole writer — the `swap(true, AcqRel)` above
        // guarantees exclusive access before the write becomes visible.
        unsafe {
            *self.inner.get() = Some(val);
        }
    }

    /// Returns a reference to the stored value, or `None` if not yet
    /// initialized or already taken.
    pub(crate) fn get_raw(&self) -> Option<&Box<dyn std::any::Any + Send + Sync>> {
        if self.initialized.load(Ordering::Acquire) {
            // SAFETY: after the Acquire load sees `true`, the matching
            // Release-store in `set()` has happened, so the write to `inner`
            // is visible. Only shared references (no mutation) are handed
            // out, so this is safe. If `take()` has already been called
            // (initialized → false), `get_raw()` returns `None` because the
            // Acquire load won't see `true`.
            unsafe { (*self.inner.get()).as_ref() }
        } else {
            None
        }
    }

    /// Take ownership of the stored value and reset the cell.
    /// After this, `get_raw()` returns `None`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that **no references returned by `get_raw()`**
    /// are still alive when this method is called — otherwise a dangling
    /// pointer would be exposed.
    ///
    /// This is intended to be called once, in `exec_and_exit()`, **after**
    /// execution has finished and no code still holds references from
    /// `get_raw()`.
    pub(crate) unsafe fn take(&self) -> Option<Box<dyn std::any::Any + Send + Sync>> {
        // Swap the flag to false so that future `get_raw()` calls return None.
        if self.initialized.swap(false, Ordering::AcqRel) {
            // SAFETY: `take()` is the sole mutator, called after all
            // `get_raw()` callers have finished. No other thread reads
            // `inner` at this point.
            unsafe { (*self.inner.get()).take() }
        } else {
            None
        }
    }
}

/// Global static reference to the current program instance
#[allow(clippy::redundant_pub_crate)]
pub(crate) static THIS_PROGRAM: ProgramCell = ProgramCell::new();

/// Returns a reference to the current program instance, panics if not set.
///
/// # Panics
///
/// Panics if the program has not been initialized yet.
#[must_use]
pub fn this<C>() -> &'static Program<C>
where
    C: ProgramCollect<Enum = C> + 'static,
{
    try_get_this_program().expect("Program not initialized")
}

/// Returns a reference to the current program instance, if set.
fn try_get_this_program<C>() -> Option<&'static Program<C>>
where
    C: ProgramCollect<Enum = C> + 'static,
{
    THIS_PROGRAM.get_raw()?.downcast_ref::<Program<C>>()
}

#[cfg(test)]
mod tests {
    use super::ProgramCell;

    #[test]
    fn test_program_cell_set_and_get_raw() {
        let cell = ProgramCell::new();
        cell.set(Box::new(42_i32));
        let val = cell.get_raw();
        assert!(val.is_some());
        assert_eq!(*val.unwrap().downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_program_cell_get_raw_uninitialized() {
        let cell = ProgramCell::new();
        assert!(cell.get_raw().is_none());
    }

    #[test]
    #[should_panic(expected = "ProgramCell already initialized")]
    fn test_program_cell_set_twice_panics() {
        let cell = ProgramCell::new();
        cell.set(Box::new(1_i32));
        cell.set(Box::new(2_i32));
    }

    #[test]
    fn test_program_cell_take() {
        let cell = ProgramCell::new();
        cell.set(Box::new(99_i32));

        // SAFETY: test-local cell, no outstanding references.
        let taken = unsafe { cell.take() };
        assert!(taken.is_some());
        assert_eq!(*taken.unwrap().downcast_ref::<i32>().unwrap(), 99);

        // After take, get_raw returns None.
        assert!(cell.get_raw().is_none());

        // Calling take again returns None.
        let taken_again = unsafe { cell.take() };
        assert!(taken_again.is_none());
    }
}
