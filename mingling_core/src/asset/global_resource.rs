use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{ChainProcess, Program, ProgramCollect, this};

pub(crate) type GlobalResources = Arc<Mutex<HashMap<TypeId, Box<dyn Any + Sync + Send>>>>;

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Insert a resource of the given type, cloning the provided value into the store
    pub fn with_resource<Res: 'static + Send + Sync + ResourceMarker>(&mut self, res: Res) {
        if let Ok(mut guard) = self.resources.lock() {
            guard.insert(TypeId::of::<Res>(), Box::new(Arc::new(res)));
        }
    }

    /// Modify a resource by type, applying a closure to the resource if present
    pub fn modify_res<Res, Return>(&self, f: impl FnOnce(&mut Res) -> Return) -> Return
    where
        Res: 'static + Default + ResourceMarker + Send + Sync,
        Return: Default,
    {
        let Ok(mut guard) = self.resources.lock() else {
            return Return::default();
        };
        if let Some(arc_res) = guard
            .get_mut(&TypeId::of::<Res>())
            .and_then(|a| a.downcast_mut::<Arc<Res>>())
        {
            let mut new_res = match Arc::try_unwrap(std::mem::take(arc_res)) {
                Ok(val) => val,
                Err(arc) => (*arc).res_clone(),
            };
            let r = f(&mut new_res);
            *arc_res = Arc::new(new_res);
            return r;
        }
        Return::default()
    }

    /// Internal syntax for the `&mut MyResource` syntax of #[chain], do not use directly
    #[doc(hidden)]
    pub fn __modify_res_and_return_route<Res>(
        &self,
        f: impl FnOnce(&mut Res) -> ChainProcess<C>,
    ) -> ChainProcess<C>
    where
        Res: 'static + Default + ResourceMarker + Send + Sync,
    {
        let Ok(mut guard) = self.resources.lock() else {
            let mut default_res = Res::res_default();
            return f(&mut default_res);
        };
        if let Some(arc_res) = guard
            .get_mut(&TypeId::of::<Res>())
            .and_then(|a| a.downcast_mut::<Arc<Res>>())
        {
            let mut new_res = match Arc::try_unwrap(std::mem::take(arc_res)) {
                Ok(val) => val,
                Err(arc) => (*arc).res_clone(),
            };
            let r = f(&mut new_res);
            *arc_res = Arc::new(new_res);
            r
        } else {
            let mut default_res = Res::res_default();
            f(&mut default_res)
        }
    }

    /// Internal syntax for the `&mut MyResource` syntax of async #[chain], do not use directly.
    ///
    /// Extracts a mutable resource from the global store (clone-out), returning an
    /// owned value. The caller must call [`__store_res`] to write back modifications.
    #[doc(hidden)]
    #[must_use]
    pub fn __extract_res_mut<Res: 'static + Default + ResourceMarker + Send + Sync>(&self) -> Res {
        let Ok(mut guard) = self.resources.lock() else {
            return Res::res_default();
        };
        if let Some(arc_res) = guard
            .get_mut(&TypeId::of::<Res>())
            .and_then(|a| a.downcast_mut::<Arc<Res>>())
        {
            match Arc::try_unwrap(std::mem::take(arc_res)) {
                Ok(val) => val,
                Err(arc) => (*arc).res_clone(),
            }
        } else {
            Res::res_default()
        }
    }

    /// Internal syntax for the `&mut MyResource` syntax of async #[chain], do not use directly.
    ///
    /// Stores a modified resource value back into the global store.
    #[doc(hidden)]
    pub fn __store_res<Res: 'static + Send + Sync + ResourceMarker>(&self, val: Res) {
        if let Ok(mut guard) = self.resources.lock() {
            guard.insert(TypeId::of::<Res>(), Box::new(Arc::new(val)));
        }
    }

    /// Get an resources by type, returning `Res` if present
    #[must_use]
    pub fn res<Res: 'static + Send + Sync>(&self) -> Option<GlobalResource<Res>> {
        let guard = self.resources.lock().ok()?;
        let boxed_any = guard.get(&TypeId::of::<Res>())?;
        let arc_res = boxed_any.as_ref().downcast_ref::<Arc<Res>>()?;
        Some(GlobalResource::from(Arc::clone(arc_res)))
    }

    /// Get a resource by type, returning `GlobalResource<Res>` if present
    pub fn res_or_route<Res: 'static + Send + Sync>(
        &self,
        route: ChainProcess<C>,
    ) -> Result<GlobalResource<Res>, ChainProcess<C>> {
        match self.res() {
            Some(r) => Ok(r),
            None => Err(route),
        }
    }

    /// Get a resource by type, returning `GlobalResource<Res>` or inserting a default
    #[must_use]
    pub fn res_or_default<Res: 'static + Send + Sync + ResourceMarker>(
        &self,
    ) -> GlobalResource<Res> {
        self.res()
            .unwrap_or_else(|| GlobalResource::from(Arc::new(Res::res_default())))
    }
}

/// Global assets for storing Program global state information
pub struct GlobalResource<ResType: 'static + Send + Sync> {
    res_arc: Arc<ResType>,
}

impl<ResType: 'static + Send + Sync> GlobalResource<ResType> {
    /// Create a new `GlobalAsset` from an `AssetType` value.
    pub fn new(res: ResType) -> Self {
        Self {
            res_arc: Arc::new(res),
        }
    }
}

impl<ResType: 'static + Send + Sync> From<Arc<ResType>> for GlobalResource<ResType> {
    fn from(arc: Arc<ResType>) -> Self {
        Self { res_arc: arc }
    }
}

impl<ResType: 'static + Send + Sync> std::ops::Deref for GlobalResource<ResType> {
    type Target = ResType;

    fn deref(&self) -> &Self::Target {
        &self.res_arc
    }
}

impl<ResType: 'static + Send + Sync> AsRef<ResType> for GlobalResource<ResType> {
    fn as_ref(&self) -> &ResType {
        &self.res_arc
    }
}

/// Resource marker trait, types that implement the Clone and Default traits can be considered as resources
pub trait ResourceMarker {
    #[must_use]
    fn res_clone(&self) -> Self;
    fn res_default() -> Self;
    fn modify<C>(f: impl FnOnce(&mut Self))
    where
        C: ProgramCollect<Enum = C> + 'static;
}

impl<T: Default + Clone + Send + Sync + 'static> ResourceMarker for T {
    fn res_clone(&self) -> Self {
        Clone::clone(self)
    }

    fn res_default() -> Self {
        Default::default()
    }

    fn modify<C>(f: impl FnOnce(&mut Self))
    where
        C: ProgramCollect<Enum = C> + 'static,
    {
        this::<C>().modify_res(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_resource_new_and_deref() {
        let res = GlobalResource::new(42i32);
        assert_eq!(*res, 42);
    }

    #[test]
    fn global_resource_from_arc() {
        let arc = Arc::new(42i32);
        let res = GlobalResource::from(arc);
        assert_eq!(*res, 42);
    }

    #[test]
    fn global_resource_as_ref() {
        let res = GlobalResource::new(42i32);
        assert_eq!(res.as_ref(), &42);
    }

    #[test]
    fn resource_marker_i32_res_clone() {
        let val = 42i32;
        let cloned = val.res_clone();
        assert_eq!(cloned, 42);
    }

    #[test]
    fn resource_marker_i32_res_default() {
        assert_eq!(<i32 as ResourceMarker>::res_default(), 0i32);
    }

    #[test]
    fn resource_marker_string_res_clone() {
        let val = "hello".to_string();
        let cloned = val.res_clone();
        assert_eq!(cloned, "hello");
    }

    #[test]
    fn resource_marker_string_res_default() {
        assert_eq!(<String as ResourceMarker>::res_default(), "");
    }

    #[test]
    fn resource_marker_vec_res_clone() {
        let val = vec![1, 2, 3];
        let cloned = val.res_clone();
        assert_eq!(cloned, vec![1, 2, 3]);
    }

    #[test]
    fn resource_marker_vec_res_default() {
        let empty: Vec<i32> = vec![];
        assert_eq!(<Vec<i32> as ResourceMarker>::res_default(), empty);
    }

    // Note: Tests for Program::with_resource, res(), res_or_route(), res_or_default(),
    // and modify_res() require a concrete ProgramCollect implementation, which is
    // complex and outside the scope of these unit tests.
    // Those are better covered by integration tests.
}
