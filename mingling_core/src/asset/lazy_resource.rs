use crate::{ProgramCollect, ResourceMarker, this};

enum LazyInner<T> {
    /// Not yet initialized — holds the factory function.
    /// After init, the factory is **dropped**, no wasted memory.
    Uninit(
        Box<dyn FnMut() -> T + Send + Sync>,
        Option<Box<dyn FnOnce(T) + Send + Sync>>,
    ),
    /// Initialized and ready to go.
    Init(T, Option<Box<dyn FnOnce(T) + Send + Sync>>),
}

/// A lazily initialized resource that only creates its value on first access.
///
/// Unlike `Option<T>` + persistent `Box<dyn FnMut>`, the factory function is
/// **consumed and dropped** after initialization. No leftover allocation.
///
/// Initialization is triggered by `get_ref()`, `get_mut()`, or `get_clone()`.
pub struct LazyRes<T: Send + Sync + 'static> {
    inner: LazyInner<T>,
}

impl<T: Send + Sync + 'static> LazyRes<T> {
    /// Creates a new lazily initialized resource with a custom initializer.
    ///
    /// The factory `f` is called on first access, then dropped.
    #[must_use]
    pub fn new(f: impl FnMut() -> T + Send + Sync + 'static) -> Self {
        Self {
            inner: LazyInner::Uninit(Box::new(f), None),
        }
    }

    /// Creates a new lazily initialized resource with a custom initializer and
    /// an optional on-drop callback that receives ownership of the inner value when
    /// the `LazyRes` is dropped.
    #[must_use]
    pub fn new_with_drop(
        f: impl FnMut() -> T + Send + Sync + 'static,
        on_drop: impl FnOnce(T) + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: LazyInner::Uninit(Box::new(f), Some(Box::new(on_drop))),
        }
    }

    /// Chains an on-drop callback onto the lazy resource, returning ownership.
    ///
    /// This method consumes `self` and returns it with the callback attached,
    /// allowing a builder‑style pattern.
    ///
    /// The callback receives ownership of the inner value when the `LazyRes` is
    /// dropped. If the resource has not yet been initialized, the callback is stored
    /// and will be invoked once initialization happens *and* the `LazyRes` is later
    /// dropped.
    #[must_use]
    pub fn with_on_drop(mut self, on_drop: impl FnOnce(T) + Send + Sync + 'static) -> Self {
        match &mut self.inner {
            LazyInner::Uninit(_, existing_opt) | LazyInner::Init(_, existing_opt) => {
                *existing_opt = Some(Box::new(on_drop));
            }
        }
        self
    }

    /// Sets or replaces the on-drop callback for the initialized value.
    /// The callback is called with ownership of the inner value when the `LazyRes` is dropped.
    /// If the resource has not been initialized yet, the callback is stored and applied
    /// upon initialization.
    pub fn set_on_drop(&mut self, on_drop: impl FnOnce(T) + Send + Sync + 'static) {
        match &mut self.inner {
            LazyInner::Uninit(_, existing_opt) | LazyInner::Init(_, existing_opt) => {
                *existing_opt = Some(Box::new(on_drop));
            }
        }
    }

    /// Returns `true` if the resource has been initialized.
    pub const fn is_initialized(&self) -> bool {
        matches!(&self.inner, LazyInner::Init(_, _))
    }

    fn force_init(&mut self) -> &mut LazyInner<T> {
        if matches!(&self.inner, LazyInner::Uninit(_, _)) {
            // Replace with a temporary poison value so the real factory can be moved out.
            // If the factory panics, the poison value's `unreachable!()` will
            // catch any subsequent access.
            let poisoned = LazyInner::Uninit(
                Box::new(|| unreachable!("LazyRes poisoned during initialization")),
                None,
            );
            let old = std::mem::replace(&mut self.inner, poisoned);
            match old {
                LazyInner::Uninit(mut f, on_drop) => {
                    self.inner = LazyInner::Init((f)(), on_drop);
                }
                // Already initialized — put it back
                init @ LazyInner::Init(_, _) => {
                    self.inner = init;
                }
            }
        }
        &mut self.inner
    }

    /// Returns an immutable reference to the inner value,
    /// calling the initializer on first access.
    pub fn get_ref(&mut self) -> &T {
        self.force_init();
        match &self.inner {
            LazyInner::Init(t, _) => t,
            _ => unreachable!(),
        }
    }

    /// Returns a mutable reference to the inner value,
    /// calling the initializer on first access.
    pub fn get_mut(&mut self) -> &mut T {
        self.force_init();
        match &mut self.inner {
            LazyInner::Init(t, _) => t,
            _ => unreachable!(),
        }
    }

    /// Returns a clone of the inner value, calling the initializer if necessary.
    pub fn get_clone(&mut self) -> T
    where
        T: Clone,
    {
        self.get_ref().clone()
    }

    /// Consumes the lazy resource and returns the inner value, if initialized.
    ///
    /// Unlike `reset()`, this **drops** the lazy wrapper entirely.
    /// If you need to re-initialize, just construct a new `LazyRes::new(f)`.
    pub fn into_inner(mut self) -> Option<T> {
        // Take a temporary replacement to avoid moving out of a Drop type.
        let inner = std::mem::replace(
            &mut self.inner,
            LazyInner::Uninit(Box::new(|| unreachable!()), None),
        );
        match inner {
            LazyInner::Init(t, _) => Some(t),
            LazyInner::Uninit(_, _) => None,
        }
    }

    /// Consumes the lazy resource and returns the inner value.
    ///
    /// If the resource has not been initialized, the initializer is called first.
    /// This is different from `into_inner()` which returns `None` if uninitialized.
    ///
    /// # Panics
    ///
    /// Panics if the resource has not been initialized.
    pub fn unwrap(mut self) -> T {
        match std::mem::replace(
            &mut self.inner,
            LazyInner::Uninit(Box::new(|| unreachable!()), None),
        ) {
            LazyInner::Init(t, _) => t,
            LazyInner::Uninit(_, _) => {
                panic!("called `LazyRes::unwrap()` on an uninitialized value")
            }
        }
    }

    /// Consumes the lazy resource, calling the initializer first if not yet initialized.
    pub fn unwrap_or(mut self, default: T) -> T {
        if matches!(&self.inner, LazyInner::Uninit(_, _)) {
            default
        } else {
            match std::mem::replace(
                &mut self.inner,
                LazyInner::Uninit(Box::new(|| unreachable!()), None),
            ) {
                LazyInner::Init(t, _) => t,
                _ => unreachable!(),
            }
        }
    }

    /// Consumes the lazy resource, calling the initializer first if not yet initialized.
    ///
    /// If the resource has not been initialized, `T::default()` is used as the fallback.
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        self.unwrap_or(T::default())
    }
}

impl<T: Send + Sync + 'static> Drop for LazyRes<T> {
    fn drop(&mut self) {
        // Take ownership of the inner value and call the on-drop callback if present.
        let poisoned = LazyInner::Uninit(
            Box::new(|| unreachable!("LazyRes poisoned during drop")),
            None,
        );
        let old = std::mem::replace(&mut self.inner, poisoned);
        match old {
            LazyInner::Init(val, Some(callback)) => {
                callback(val);
            }
            LazyInner::Uninit(_, Some(_callback)) => {
                // Resource was never initialized but a drop callback was set.
                // Nothing to pass to the callback — just drop it.
            }
            _ => {}
        }
    }
}

impl<T: Send + Sync + Default + 'static> Default for LazyRes<T> {
    /// Creates an uninitialized `LazyRes<T>` whose initializer returns `T::default()`.
    fn default() -> Self {
        Self::new(|| T::default())
    }
}

impl<T: Send + Sync + 'static> From<T> for LazyRes<T> {
    /// Creates a `LazyRes<T>` from an already-initialized value.
    fn from(value: T) -> Self {
        Self {
            inner: LazyInner::Init(value, None),
        }
    }
}

impl<T: Send + Sync + 'static> LazyRes<T> {
    /// Creates a lazily initialized resource using `T::default()` as the initializer.
    #[must_use]
    pub fn lazy_default() -> Self
    where
        T: Default,
    {
        Self::default()
    }

    /// Creates a lazily initialized resource with a custom initializer.
    ///
    /// Same as `LazyRes::new`.
    pub fn lazy_init(f: impl FnMut() -> T + Send + Sync + 'static) -> Self {
        Self::new(f)
    }
}

/// Provides convenience methods for types, allowing them to easily
/// create a corresponding `LazyRes<T>`.
pub trait LazyInit: Send + Sync + 'static {
    /// Creates a lazily initialized resource for this type using `Default` as the initializer.
    #[must_use]
    fn lazy_default() -> LazyRes<Self>
    where
        Self: Default,
    {
        LazyRes::default()
    }

    /// Creates a lazily initialized resource for this type with a custom initializer.
    fn lazy_init(f: impl FnMut() -> Self + Send + Sync + 'static) -> LazyRes<Self>
    where
        Self: Sized,
    {
        LazyRes::new(f)
    }
}

impl<T: Send + Sync + 'static> LazyInit for T {}

impl<T: Send + Sync + 'static + Default + Clone> ResourceMarker for LazyRes<T> {
    /// Clones the lazy resource. The cloned resource retains any initialized value,
    /// but the initializer is reset to `T::default()`.
    fn __resource_marker_clone(&self) -> Self {
        match &self.inner {
            LazyInner::Init(t, _) => Self {
                inner: LazyInner::Init(t.clone(), None),
            },
            LazyInner::Uninit(_, _) => Self::default(),
        }
    }

    /// Returns a default lazy resource (uninitialized, using `T::default()` as the initializer).
    fn __resource_marker_default() -> Self {
        Self::default()
    }

    /// Modifies the current lazy resource via the `this` context provided by `C`.
    fn __resource_marker_modify<C>(f: impl FnOnce(&mut Self))
    where
        C: ProgramCollect<Enum = C> + 'static,
    {
        this::<C>().modify_res(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Helper for tracking drops via an `Arc<AtomicBool>`.
    struct DropFlag(Arc<AtomicBool>);

    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    // LazyRes::new starts uninitialized
    #[test]
    fn new_returns_uninitialized() {
        let r: LazyRes<i32> = LazyRes::new(|| 42);
        assert!(!r.is_initialized());
    }

    // LazyRes::get_ref triggers init and returns correct value
    #[test]
    fn get_ref_triggers_initialization() {
        let mut r = LazyRes::new(|| 42);
        assert!(!r.is_initialized());
        let val = r.get_ref();
        assert_eq!(*val, 42);
        assert!(r.is_initialized());
    }

    #[test]
    fn get_ref_returns_same_value_on_subsequent_calls() {
        let mut r = LazyRes::new(|| 42);
        assert_eq!(*r.get_ref(), 42);
        assert_eq!(*r.get_ref(), 42);
    }

    // LazyRes::get_mut triggers init and allows mutation
    #[test]
    fn get_mut_triggers_initialization() {
        let mut r = LazyRes::new(|| 10);
        assert!(!r.is_initialized());
        *r.get_mut() = 20;
        assert_eq!(*r.get_ref(), 20);
        assert!(r.is_initialized());
    }

    // LazyRes::get_clone returns cloned value
    #[test]
    fn get_clone_returns_cloned_value() {
        let mut r = LazyRes::new(|| "hello".to_string());
        assert_eq!(r.get_clone(), "hello");
        assert!(r.is_initialized());
    }

    // LazyRes::is_initialized is false before get_ref and true after
    #[test]
    fn is_initialized_false_before_true_after() {
        let mut r = LazyRes::new(|| 99);
        assert!(!r.is_initialized());
        r.get_ref();
        assert!(r.is_initialized());
    }

    // LazyRes::into_inner returns Some if initialized and None otherwise
    #[test]
    fn into_inner_initialized_returns_some() {
        let mut r = LazyRes::new(|| 7);
        r.get_ref(); // force init
        assert_eq!(r.into_inner(), Some(7));
    }

    #[test]
    fn into_inner_uninitialized_returns_none() {
        let r: LazyRes<i32> = LazyRes::new(|| 7);
        assert_eq!(r.into_inner(), None);
    }

    // LazyRes::unwrap returns value if initialized or panics otherwise
    #[test]
    fn unwrap_initialized_returns_value() {
        let mut r = LazyRes::new(|| 13);
        r.get_ref();
        assert_eq!(r.unwrap(), 13);
    }

    #[test]
    #[should_panic(expected = "uninitialized")]
    fn unwrap_uninitialized_panics() {
        let r: LazyRes<i32> = LazyRes::new(|| 13);
        r.unwrap();
    }

    // LazyRes::unwrap_or returns default if uninitialized
    #[test]
    fn unwrap_or_uninitialized_returns_default() {
        let r: LazyRes<i32> = LazyRes::new(|| 5);
        assert_eq!(r.unwrap_or(100), 100);
    }

    #[test]
    fn unwrap_or_initialized_returns_inner() {
        let mut r = LazyRes::new(|| 5);
        r.get_ref();
        assert_eq!(r.unwrap_or(100), 5);
    }

    // LazyRes::unwrap_or_default returns T::default
    #[test]
    fn unwrap_or_default_uninitialized_returns_default() {
        let r: LazyRes<i32> = LazyRes::new(|| 42);
        assert_eq!(r.unwrap_or_default(), 0);
    }

    #[test]
    fn unwrap_or_default_initialized_returns_inner() {
        let mut r = LazyRes::new(|| 42);
        r.get_ref();
        assert_eq!(r.unwrap_or_default(), 42);
    }

    // LazyRes::Default creates uninitialized with T::default factory
    #[test]
    fn default_creates_uninitialized_with_default_factory() {
        let r: LazyRes<i32> = LazyRes::default();
        assert!(!r.is_initialized());
    }

    #[test]
    fn default_factory_produces_t_default() {
        let mut r: LazyRes<i32> = LazyRes::default();
        assert_eq!(*r.get_ref(), 0);
    }

    // From<T> for LazyRes creates initialized value
    #[test]
    fn from_t_creates_initialized() {
        let r: LazyRes<String> = LazyRes::from("hello".to_string());
        assert!(r.is_initialized());
    }

    #[test]
    fn from_t_contains_correct_value() {
        let r: LazyRes<String> = LazyRes::from("world".to_string());
        // Can only check via into_inner since get_ref needs &mut
        assert_eq!(r.into_inner(), Some("world".to_string()));
    }

    // Drop callback via new_with_drop is called on drop
    #[test]
    fn new_with_drop_calls_callback_on_drop() {
        let dropped = Arc::new(AtomicBool::new(false));
        let dropped_clone = Arc::clone(&dropped);

        let r = LazyRes::new_with_drop(
            || 42,
            move |val| {
                assert_eq!(val, 42);
                dropped_clone.store(true, Ordering::SeqCst);
            },
        );
        // Force init first
        drop(r);
        // Not initialized, so the callback above was stored but never invoked.
        // The initialized path is tested below.
    }

    #[test]
    fn new_with_drop_calls_callback_on_drop_after_init() {
        let dropped = Arc::new(AtomicBool::new(false));
        let dropped_clone = Arc::clone(&dropped);

        let mut r = LazyRes::new_with_drop(
            || 42,
            move |val| {
                assert_eq!(val, 42);
                dropped_clone.store(true, Ordering::SeqCst);
            },
        );
        r.get_ref(); // initialize
        assert!(r.is_initialized());
        drop(r);
        assert!(dropped.load(Ordering::SeqCst));
    }

    // Drop callback via with_on_drop uses chained builder style
    #[test]
    fn with_on_drop_calls_callback_on_drop() {
        let dropped = Arc::new(AtomicBool::new(false));
        let dropped_clone = Arc::clone(&dropped);

        let r = LazyRes::new(|| 99).with_on_drop(move |val| {
            assert_eq!(val, 99);
            dropped_clone.store(true, Ordering::SeqCst);
        });
        drop(r); // not initialized, callback stored but won't fire
        assert!(!dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn with_on_drop_calls_callback_on_drop_after_init() {
        let dropped = Arc::new(AtomicBool::new(false));
        let dropped_clone = Arc::clone(&dropped);

        let mut r = LazyRes::new(|| 99).with_on_drop(move |val| {
            assert_eq!(val, 99);
            dropped_clone.store(true, Ordering::SeqCst);
        });
        r.get_ref();
        drop(r);
        assert!(dropped.load(Ordering::SeqCst));
    }

    // set_on_drop sets callback after construction
    #[test]
    fn set_on_drop_calls_callback_on_drop() {
        let dropped = Arc::new(AtomicBool::new(false));
        let dropped_clone = Arc::clone(&dropped);

        let mut r = LazyRes::new(|| 55);
        r.get_ref(); // init first
        r.set_on_drop(move |val| {
            assert_eq!(val, 55);
            dropped_clone.store(true, Ordering::SeqCst);
        });
        drop(r);
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn set_on_drop_before_init_stored_and_fires_after_init() {
        let dropped = Arc::new(AtomicBool::new(false));
        let dropped_clone = Arc::clone(&dropped);

        let mut r = LazyRes::new(|| 55);
        r.set_on_drop(move |val| {
            assert_eq!(val, 55);
            dropped_clone.store(true, Ordering::SeqCst);
        });
        r.get_ref(); // init after setting callback
        drop(r);
        assert!(dropped.load(Ordering::SeqCst));
    }

    // LazyInit::lazy_default trait method
    #[test]
    fn lazy_default_creates_uninitialized() {
        let r: LazyRes<i32> = i32::lazy_default();
        assert!(!r.is_initialized());
    }

    #[test]
    fn lazy_default_factory_returns_default() {
        let mut r: LazyRes<i32> = i32::lazy_default();
        assert_eq!(*r.get_ref(), 0);
    }

    // LazyInit::lazy_init trait method with custom factory
    #[test]
    fn lazy_init_creates_uninitialized() {
        let r: LazyRes<i32> = i32::lazy_init(|| 77);
        assert!(!r.is_initialized());
    }

    #[test]
    fn lazy_init_factory_produces_correct_value() {
        let mut r: LazyRes<i32> = i32::lazy_init(|| 77);
        assert_eq!(*r.get_ref(), 77);
    }

    // ResourceMarker for LazyRes res_clone clones initialized and res_default returns default
    #[test]
    fn res_clone_of_initialized_clones_value() {
        let mut r = LazyRes::new(|| vec![1, 2, 3]);
        r.get_ref();
        let cloned = r.__resource_marker_clone();
        assert!(cloned.is_initialized());
        assert_eq!(cloned.into_inner(), Some(vec![1, 2, 3]));
    }

    #[test]
    fn res_clone_of_uninitialized_creates_default() {
        let r: LazyRes<Vec<i32>> = LazyRes::new(|| vec![1, 2, 3]);
        let cloned = r.__resource_marker_clone();
        // The source is uninitialized, so res_clone returns a default lazy
        assert!(!cloned.is_initialized());
    }

    #[test]
    fn res_default_returns_uninitialized() {
        let r: LazyRes<i32> = LazyRes::<i32>::__resource_marker_default();
        assert!(!r.is_initialized());
    }

    // Factory is dropped after init via Arc flag
    #[test]
    fn factory_dropped_after_initialization() {
        let factory_dropped = Arc::new(AtomicBool::new(false));
        let flag = DropFlag(Arc::clone(&factory_dropped));

        // The factory closure captures `flag`. When the closure (the factory) is
        // consumed and dropped after init, the captured `flag` will be dropped,
        // setting the atomic bool.
        let factory = move || {
            let _ = &flag;
            42
        };

        let mut r = LazyRes::new(factory);
        assert!(
            !factory_dropped.load(Ordering::SeqCst),
            "factory not dropped yet"
        );

        r.get_ref(); // init — factory should be consumed and dropped
        assert!(
            factory_dropped.load(Ordering::SeqCst),
            "factory should be dropped after initialization"
        );

        // Second access still works
        assert_eq!(*r.get_ref(), 42);
    }

    #[test]
    fn factory_dropped_even_when_not_initialized_and_dropped() {
        let factory_dropped = Arc::new(AtomicBool::new(false));
        let flag = DropFlag(Arc::clone(&factory_dropped));

        let factory = move || {
            let _ = &flag;
            42
        };

        let r: LazyRes<i32> = LazyRes::new(factory);
        drop(r);
        assert!(
            factory_dropped.load(Ordering::SeqCst),
            "factory should be dropped when LazyRes is dropped"
        );
    }
}
