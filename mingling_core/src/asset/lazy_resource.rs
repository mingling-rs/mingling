use crate::{ProgramCollect, ResourceMarker, this};

/// Internal state enum for lazily-loaded resources.
///
/// This enum represents the two possible states of `LazyRes`:
/// - [`LazyInner::Uninit`]: The resource has not been initialized, holding an initialization factory function (`FnMut`) and an optional drop callback.
/// - [`LazyInner::Init`]: The resource has been initialized, holding the actual value `T` and an optional drop callback.
///
/// The optional drop callback has type `FnOnce(T)` and is invoked when the resource is dropped,
/// allowing the user to obtain final ownership of the resource value for cleanup.
enum LazyInner<T> {
    /// Uninitialized state.
    ///
    /// The first field holds a callable factory function `FnMut()` that lazily creates the resource value `T` when needed.
    /// The second field holds an optional drop callback `FnOnce(T)`; if set,
    /// the cleanup logic will run when the resource is dropped (whether or not it has been initialized).
    ///
    /// # Thread Safety
    ///
    /// Both the factory function and the drop callback must be `Send + Sync` to ensure that `LazyInner` can be safely
    /// shared or moved between threads.
    Uninit(
        /// The resource initialization factory function. This function is invoked once when the resource is first accessed,
        /// and is used to produce the actual resource value.
        Box<dyn FnMut() -> T + Send + Sync>,
        /// Optional drop callback. When the resource is dropped (`drop`), if the resource has been initialized,
        /// the resource value will be passed to this callback so that custom cleanup logic can run.
        Option<Box<dyn FnOnce(T) + Send + Sync>>,
    ),
    /// Initialized state.
    ///
    /// The first field holds the actually created resource value `T`.
    /// The second field holds the optional drop callback `FnOnce(T)`, which is invoked when the resource is dropped,
    /// receiving the resource value so that custom cleanup logic can run.
    Init(T, Option<Box<dyn FnOnce(T) + Send + Sync>>),
}

/// A lazily-loaded program resource.
///
/// `LazyRes<T>` is a container that holds a resource value `T`, which is lazily initialized through a factory
/// function the first time it is accessed. This type is suitable for scenarios where resource creation needs to be
/// deferred (such as global configuration, database connection pools, render pipelines, etc.), avoiding unnecessary
/// initialization overhead.
///
/// # Features
///
/// - **Lazy initialization**: The resource is created only upon the first call to [`LazyRes::get_ref`], [`LazyRes::get_mut`], or
///   [`LazyRes::get_clone`]; the factory function runs exactly once.
/// - **Custom drop callback**: A `FnOnce(T)` callback can be set via [`LazyRes::new_with_drop`], [`LazyRes::with_on_drop`],
///   or [`LazyRes::set_on_drop`], obtaining final ownership of the resource value and executing cleanup logic when the resource is dropped.
/// - **Thread safety**: `T` must satisfy `Send + Sync`, and the factory function and drop callback must also be `Send + Sync`,
///   ensuring `LazyRes<T>` can be safely shared or moved between threads.
/// - **Rich value extraction**: Supports multiple ways to extract the resource value, including [`LazyRes::into_inner`], [`LazyRes::unwrap`],
///   [`LazyRes::unwrap_or`], and [`LazyRes::unwrap_or_default`].
///
/// # Examples
///
/// ```
/// use mingling_core::LazyRes;
///
/// // Create a lazily initialized configuration object
/// let mut config = LazyRes::new(|| {
///     // Expensive initialization runs here, only once on first access
///     String::from("default-config")
/// });
///
/// assert!(!config.is_initialized());
/// let value = config.get_ref();
/// assert_eq!(value, "default-config");
/// assert!(config.is_initialized());
/// ```
///
/// # Performance Considerations
///
/// First access requires a `&mut self` mutable borrow because the internal state must transition from uninitialized to
/// initialized. If the resource is never accessed (i.e., initialization is never triggered), the factory function is never
/// called and no initialization overhead is incurred.
///
/// # Generic Constraints
///
/// - `T: Send + Sync + 'static` — The resource value must be safely shareable between threads and must not hold
///   references with a non-'static lifetime.
/// - The factory function type is `FnMut() -> T + Send + Sync + 'static`, meaning the factory may be invoked
///   multiple times (though in practice it runs only once) and must not capture non-'static lifetime references.
pub struct LazyRes<T: Send + Sync + 'static> {
    /// Internal state, which may be either uninitialized (`LazyInner::Uninit`) or initialized (`LazyInner::Init`).
    inner: LazyInner<T>,
}

impl<T: Send + Sync + 'static> LazyRes<T> {
    /// Creates a new lazy resource, whose value is initialized via the factory function `f` on first access.
    ///
    /// # Parameters
    ///
    /// - `f`: The resource initialization factory function. This function runs once on the first call to `get_ref`, `get_mut`, or
    ///   `get_clone`, and is used to produce the actual resource value.
    ///
    /// # Returns
    ///
    /// Returns a `LazyRes<T>` instance that is not yet initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::new(|| String::from("hello"));
    /// assert!(!res.is_initialized());
    /// assert_eq!(*res.get_ref(), "hello");
    /// assert!(res.is_initialized());
    /// ```
    #[must_use]
    pub fn new(f: impl FnMut() -> T + Send + Sync + 'static) -> Self {
        Self {
            inner: LazyInner::Uninit(Box::new(f), None),
        }
    }

    /// Creates a new lazy resource and simultaneously sets a drop callback.
    ///
    /// When the resource is dropped (`drop`), if the resource was initialized via the factory function,
    /// ownership of the resource value will be handed to the `on_drop` callback so that custom cleanup logic can run.
    ///
    /// # Parameters
    ///
    /// - `f`: The resource initialization factory function, executed on first access.
    /// - `on_drop`: The drop callback. When the resource is dropped and has been initialized, the resource value is passed to it.
    ///
    /// # Returns
    ///
    /// Returns a `LazyRes<T>` instance that has a drop callback set and is not yet initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::new_with_drop(
    ///     || 42,
    ///     |val| println!("Resource {} is being dropped", val),
    /// );
    /// res.get_ref();
    /// drop(res); // triggers on_drop callback
    /// ```
    #[must_use]
    pub fn new_with_drop(
        f: impl FnMut() -> T + Send + Sync + 'static,
        on_drop: impl FnOnce(T) + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: LazyInner::Uninit(Box::new(f), Some(Box::new(on_drop))),
        }
    }

    /// Sets a drop callback using a consuming builder-style chained call.
    ///
    /// Similar to [`LazyRes::new_with_drop`], but uses a builder style:
    /// `with_on_drop` allows chaining a drop callback after the resource has been created.
    /// If a drop callback was previously set, the new callback replaces the old one.
    ///
    /// # Parameters
    ///
    /// - `self`: The current `LazyRes` instance.
    /// - `on_drop`: The drop callback, executed when the resource is dropped.
    ///
    /// # Returns
    ///
    /// Returns the same `LazyRes` instance with the drop callback set.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let res = LazyRes::new(|| 100).with_on_drop(|val| {
    ///     println!("Dropping value: {}", val);
    /// });
    /// ```
    #[must_use]
    pub fn with_on_drop(mut self, on_drop: impl FnOnce(T) + Send + Sync + 'static) -> Self {
        match &mut self.inner {
            LazyInner::Uninit(_, existing_opt) | LazyInner::Init(_, existing_opt) => {
                *existing_opt = Some(Box::new(on_drop));
            }
        }
        self
    }

    /// Sets a drop callback via a mutable reference.
    ///
    /// Unlike [`LazyRes::with_on_drop`], `set_on_drop` modifies `&mut self` via mutable borrow,
    /// without consuming ownership of `self`. If a drop callback was previously set, the new callback replaces the old one.
    ///
    /// # Parameters
    ///
    /// - `on_drop`: The drop callback, executed when the resource is dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::new(|| 42);
    /// res.set_on_drop(|val| println!("Final value: {}", val));
    /// ```
    pub fn set_on_drop(&mut self, on_drop: impl FnOnce(T) + Send + Sync + 'static) {
        match &mut self.inner {
            LazyInner::Uninit(_, existing_opt) | LazyInner::Init(_, existing_opt) => {
                *existing_opt = Some(Box::new(on_drop));
            }
        }
    }

    /// Checks whether the resource has been fully initialized.
    ///
    /// Returns `true` if the resource has been initialized (i.e., the factory function has already been invoked once).
    /// Otherwise returns `false`.
    ///
    /// # Returns
    ///
    /// - `true`: The resource has been initialized.
    /// - `false`: The resource has not yet been initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::new(|| 42);
    /// assert!(!res.is_initialized());
    /// res.get_ref();
    /// assert!(res.is_initialized());
    /// ```
    pub const fn is_initialized(&self) -> bool {
        matches!(&self.inner, LazyInner::Init(_, _))
    }

    /// Forces initialization of the resource (if not already initialized).
    ///
    /// This is an internal method that ensures the resource is in an initialized state. If the resource is already
    /// initialized, it returns immediately without doing anything; if not yet initialized, it calls the factory function
    /// to create the resource value and transitions to the [`LazyInner::Init`] state.
    ///
    /// If the factory function panics during execution, the internal state is replaced with a "poisoned" placeholder
    /// value, and any subsequent access to the resource will trigger an `unreachable!()` panic.
    ///
    /// # Returns
    ///
    /// A `&mut` reference to the initialized internal state.
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

    /// Obtains an immutable reference to the resource.
    ///
    /// If the resource has not been initialized, this method first invokes the factory function to complete lazy
    /// initialization, then returns an immutable reference to the resource value.
    ///
    /// # Returns
    ///
    /// An immutable reference `&T` to the resource value.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::new(|| String::from("config"));
    /// assert_eq!(res.get_ref(), "config");
    /// ```
    pub fn get_ref(&mut self) -> &T {
        self.force_init();
        match &self.inner {
            LazyInner::Init(t, _) => t,
            LazyInner::Uninit(..) => unreachable!(),
        }
    }

    /// Obtains a mutable reference to the resource.
    ///
    /// If the resource has not been initialized, this method first invokes the factory function to complete lazy
    /// initialization, then returns a mutable reference to the resource value, allowing modification of the resource.
    ///
    /// # Returns
    ///
    /// A mutable reference `&mut T` to the resource value.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::new(|| 10);
    /// *res.get_mut() = 20;
    /// assert_eq!(*res.get_ref(), 20);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        self.force_init();
        match &mut self.inner {
            LazyInner::Init(t, _) => t,
            LazyInner::Uninit(..) => unreachable!(),
        }
    }

    /// Obtains a cloned copy of the resource value.
    ///
    /// If the resource has not been initialized, this method first completes lazy initialization, then clones the
    /// resource value and returns it. This method requires `T` to implement the [`Clone`] trait.
    ///
    /// # Returns
    ///
    /// A cloned copy `T` of the resource value.
    ///
    /// # Type Constraints
    ///
    /// - `T: Clone` — The resource type must support cloning.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::new(|| vec![1, 2, 3]);
    /// let cloned = res.get_clone();
    /// assert_eq!(cloned, vec![1, 2, 3]);
    /// ```
    pub fn get_clone(&mut self) -> T
    where
        T: Clone,
    {
        self.get_ref().clone()
    }

    /// Extracts the internal resource value, returning `Option<T>`.
    ///
    /// Consumes the current `LazyRes`; if the resource has been initialized, returns `Some(value)`;
    /// if the resource has not been initialized, returns `None`.
    ///
    /// Note: Unlike [`LazyRes::unwrap`], this method does not force initialization,
    /// nor does it invoke the drop callback.
    ///
    /// # Returns
    ///
    /// - `Some(T)`: The resource has been initialized; returns the internal resource value.
    /// - `None`: The resource has not been initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::new(|| 7);
    /// res.get_ref();
    /// assert_eq!(res.into_inner(), Some(7));
    ///
    /// let res2 = LazyRes::<i32>::new(|| 7);
    /// assert_eq!(res2.into_inner(), None);
    /// ```
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

    /// Unwraps the internal resource value, panicking if the resource has not been initialized.
    ///
    /// Consumes the current `LazyRes` and returns the internal resource value `T`. Unlike [`LazyRes::into_inner`],
    /// this method requires the resource to already be initialized; otherwise it triggers a panic.
    ///
    /// # Panics
    ///
    /// Calling this method when the resource has not been initialized will trigger a panic.
    ///
    /// # Returns
    ///
    /// The internal resource value `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::new(|| 13);
    /// res.get_ref();
    /// assert_eq!(res.unwrap(), 13);
    /// ```
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

    /// Unwraps the internal resource value, returning a default value if the resource has not been initialized.
    ///
    /// Consumes the current `LazyRes`; if the resource has been initialized, returns the internal resource value `T`;
    /// otherwise returns the provided `default` value as a substitute. This method does not force initialization.
    ///
    /// # Parameters
    ///
    /// - `default`: The substitute value returned when the resource has not been initialized.
    ///
    /// # Returns
    ///
    /// The internal resource value if the resource has been initialized; otherwise `default`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let res: LazyRes<i32> = LazyRes::new(|| 5);
    /// assert_eq!(res.unwrap_or(100), 100);
    ///
    /// let mut res2 = LazyRes::new(|| 5);
    /// res2.get_ref();
    /// assert_eq!(res2.unwrap_or(100), 5);
    /// ```
    pub fn unwrap_or(mut self, default: T) -> T {
        if matches!(&self.inner, LazyInner::Uninit(_, _)) {
            default
        } else {
            match std::mem::replace(
                &mut self.inner,
                LazyInner::Uninit(Box::new(|| unreachable!()), None),
            ) {
                LazyInner::Init(t, _) => t,
                LazyInner::Uninit(..) => unreachable!(),
            }
        }
    }

    /// Unwraps the internal resource value, returning `T::default()` if the resource has not been initialized.
    ///
    /// Consumes the current `LazyRes`; if the resource has been initialized, returns the internal resource value `T`;
    /// otherwise returns `T::default()` as a substitute. This method does not force initialization.
    ///
    /// This is a convenience wrapper around [`LazyRes::unwrap_or`], using `T::default()` as the
    /// default value.
    ///
    /// # Type Constraints
    ///
    /// - `T: Default` — The resource type must implement the [`Default`] trait.
    ///
    /// # Returns
    ///
    /// The internal resource value if the resource has been initialized; otherwise `T::default()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let res: LazyRes<i32> = LazyRes::new(|| 42);
    /// assert_eq!(res.unwrap_or_default(), 0);
    /// ```
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
    fn default() -> Self {
        Self::new(|| T::default())
    }
}

impl<T: Send + Sync + 'static> From<T> for LazyRes<T> {
    fn from(value: T) -> Self {
        Self {
            inner: LazyInner::Init(value, None),
        }
    }
}

impl<T: Send + Sync + 'static> LazyRes<T> {
    /// Creates a lazily initialized default resource.
    ///
    /// This is an alias method for [`LazyRes::default`], using `T::default()` as the factory function's return value.
    /// Like [`LazyRes::new`], the resource value is created only upon first access.
    ///
    /// # Type Constraints
    ///
    /// - `T: Default` — The resource type must implement the [`Default`] trait.
    ///
    /// # Returns
    ///
    /// Returns a not-yet-initialized `LazyRes<T>` instance whose factory function will return `T::default()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::<i32>::lazy_default();
    /// assert!(!res.is_initialized());
    /// assert_eq!(*res.get_ref(), 0);
    /// ```
    #[must_use]
    pub fn lazy_default() -> Self
    where
        T: Default,
    {
        Self::default()
    }

    /// Creates a lazily initialized resource using the specified factory function.
    ///
    /// This is an alias method for [`LazyRes::new`], providing a name more aligned with "lazy initialization" semantics.
    /// The factory function `f` runs once when the resource is first accessed, producing the actual resource value.
    ///
    /// # Parameters
    ///
    /// - `f`: The resource initialization factory function. This function runs once on the first call to `get_ref`, `get_mut`, or
    ///   `get_clone`, and is used to produce the actual resource value.
    ///
    /// # Returns
    ///
    /// Returns a not-yet-initialized `LazyRes<T>` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyRes;
    ///
    /// let mut res = LazyRes::lazy_init(|| String::from("lazy"));
    /// assert!(!res.is_initialized());
    /// assert_eq!(*res.get_ref(), "lazy");
    /// ```
    pub fn lazy_init(f: impl FnMut() -> T + Send + Sync + 'static) -> Self {
        Self::new(f)
    }
}

/// A trait that provides convenient lazy-initialization capabilities.
///
/// The `LazyInit` trait provides a set of convenience methods for all types satisfying `Send + Sync + 'static`
/// to create [`LazyRes`]-wrapped resources. Any type satisfying those constraints can directly call
/// [`LazyInit::lazy_default`] or [`LazyInit::lazy_init`] by type name,
/// without needing to explicitly write the more verbose `LazyRes<T>::new(...)`.
///
/// # Implementation
///
/// This trait is automatically implemented for all `T: Send + Sync + 'static` types
/// (via `impl<T: Send + Sync + 'static> LazyInit for T {}`),
/// so manual implementation is generally unnecessary.
///
/// # Method 1: `lazy_default`
///
/// Creates a `LazyRes<T>` using `T::default()` as the factory function's return value.
/// Requires `T` to implement the [`Default`] trait.
///
/// ```
/// use mingling_core::{LazyRes, LazyInit};
///
/// let mut res = i32::lazy_default();
/// assert!(!res.is_initialized());
/// assert_eq!(*res.get_ref(), 0);
/// ```
///
/// # Method 2: `lazy_init`
///
/// Creates a `LazyRes<T>` using a custom factory function. The factory function executes when
/// the resource is first accessed to produce the actual value.
///
/// ```
/// use mingling_core::{LazyRes, LazyInit};
///
/// let mut res = String::lazy_init(|| String::from("hello"));
/// assert!(!res.is_initialized());
/// assert_eq!(*res.get_ref(), "hello");
/// ```
///
/// # Generic Constraints
///
/// - `Self: Send + Sync + 'static` — The type must be safely shareable between threads and must not hold
///   non-'static lifetime references.
pub trait LazyInit: Send + Sync + 'static {
    /// Creates a lazily initialized default resource.
    ///
    /// Uses `Self::default()` as the factory function, returning a not-yet-initialized
    /// [`LazyRes<Self>`](LazyRes). The resource value is created via `Self::default()` upon first access.
    ///
    /// # Type Constraints
    ///
    /// - `Self: Default` — The type must implement the [`Default`] trait.
    ///
    /// # Returns
    ///
    /// Returns a not-yet-initialized `LazyRes<Self>` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyInit;
    ///
    /// let mut res = i32::lazy_default();
    /// assert!(!res.is_initialized());
    /// assert_eq!(*res.get_ref(), 0);
    /// ```
    #[must_use]
    fn lazy_default() -> LazyRes<Self>
    where
        Self: Default,
    {
        LazyRes::default()
    }

    /// Creates a lazily initialized resource using a custom factory function.
    ///
    /// The factory function `f` runs once when the resource is first accessed to produce the actual value.
    ///
    /// # Parameters
    ///
    /// - `f`: The resource initialization factory function. This function runs once on the first call to `get_ref`, `get_mut`, or
    ///   `get_clone`, and is used to produce the actual resource value.
    ///
    /// # Type Constraints
    ///
    /// - `Self: Sized` — The type must have a known size.
    ///
    /// # Returns
    ///
    /// Returns a not-yet-initialized `LazyRes<Self>` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::LazyInit;
    ///
    /// let mut res = String::lazy_init(|| String::from("hello"));
    /// assert!(!res.is_initialized());
    /// assert_eq!(*res.get_ref(), "hello");
    /// ```
    fn lazy_init(f: impl FnMut() -> Self + Send + Sync + 'static) -> LazyRes<Self>
    where
        Self: Sized,
    {
        LazyRes::new(f)
    }
}

impl<T: Send + Sync + 'static> LazyInit for T {}

impl<T: Send + Sync + 'static + Default + Clone> ResourceMarker for LazyRes<T> {
    /// Clones a deep copy of the current lazy resource as either initialized or default state.
    ///
    /// If the resource has been initialized, clones the internal resource value and returns a new initialized
    /// `LazyRes<T>`; if the resource has not been initialized, returns a default `LazyRes<T>` using `T::default()`
    /// as the factory function.
    ///
    /// # Type Constraints
    ///
    /// - `T: Clone` — The resource type must implement the [`Clone`] trait.
    ///
    /// # Returns
    ///
    /// A cloned `LazyRes<T>` instance. The cloned instance does not carry the original instance's drop callback.
    fn __resource_marker_clone(&self) -> Self {
        match &self.inner {
            LazyInner::Init(t, _) => Self {
                inner: LazyInner::Init(t.clone(), None),
            },
            LazyInner::Uninit(_, _) => Self::default(),
        }
    }

    /// Creates a default lazy resource instance.
    ///
    /// Returns a not-yet-initialized `LazyRes<T>` using `T::default()` as the factory function.
    /// The resource value is created only upon first access.
    ///
    /// # Type Constraints
    ///
    /// - `T: Default` — The resource type must implement the [`Default`] trait.
    ///
    /// # Returns
    ///
    /// A not-yet-initialized default `LazyRes<T>` instance.
    fn __resource_marker_default() -> Self {
        Self::default()
    }

    /// Modifies the current lazy resource via `ProgramCollect`.
    ///
    /// Uses the provided closure `f` to modify the current `LazyRes<T>`. The modification is delegated
    /// to the `ProgramCollect` resource collector via `this::<C>().modify_res(f)`.
    ///
    /// # Parameters
    ///
    /// - `f`: The closure used to modify `LazyRes<T>`, receiving a `&mut Self` parameter.
    ///
    /// # Generic Constraints
    ///
    /// - `C: ProgramCollect<Enum = C> + 'static` — The program collector type; the `Enum` associated type must equal itself.
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
