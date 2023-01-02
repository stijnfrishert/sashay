use core::{any::TypeId, marker::PhantomData};

/// A type-erased mutable reference
///
/// # Example
///
/// ```
/// let mut data = 'z';
/// let mut any = sashay::AnyMut::erase(&mut data);
/// let reference = any.unerase_mut::<char>().expect("not a reference to `char`");
///
/// *reference = '💤';
///
/// assert_eq!(data, '💤');
/// ```
#[derive(Debug)]
pub struct AnyMut<'a> {
    /// A raw pointer to the referenced data
    ptr: *mut (),

    /// A unique id representing the type of the referenced data
    ///
    /// This is used to ensure we can safely unerase back without accidentally transmuting
    type_id: TypeId,

    /// Phantom data to ensure that we stick to the correct lifetime
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> AnyMut<'a> {
    /// Erase the type of a mutable reference.
    ///
    /// The resulting type retains the lifetime of the original reference, but the
    /// referred to value can only be used after unerasing the type
    ///
    /// ```
    /// let mut data : char = '🦀';
    /// let any = sashay::AnyMut::erase(&mut data);
    ///
    /// assert!(any.contains::<char>());
    /// ```
    pub fn erase<T: 'static>(reference: &'a mut T) -> AnyMut<'a> {
        // Safety:
        //  - The raw parts come from a valid reference
        //  - The TypeId is provided by the compiler
        unsafe { Self::from_raw_parts((reference as *mut T).cast::<()>(), TypeId::of::<T>()) }
    }

    /// Construct an erased reference from its raw parts
    ///
    /// # Safety
    ///
    /// Calling this is only defined behaviour if:
    ///  - The pointer refers to a valid `T`
    ///  - `type_id` is the correct `TypeId` for the element `T`
    pub unsafe fn from_raw_parts(ptr: *mut (), type_id: TypeId) -> Self {
        Self {
            ptr,
            type_id,
            _phantom: PhantomData,
        }
    }

    /// Unerase back to an immutable reference
    ///
    /// This functions essentially the same as [`Any::downcast_ref()`](https://doc.rust-lang.org/core/any/trait.Any.html#method.downcast_ref). If the
    /// original reference's element type was `T`, a valid reference is returned. Otherwise, you get `None`.
    ///
    /// ```
    /// let data : i32 = 7;
    /// let any = sashay::AnyRef::erase(&data);
    ///
    /// assert!(any.unerase::<bool>().is_none());
    /// assert!(any.unerase::<i32>().is_some());
    /// ```
    ///
    /// Note that while this type erased a *mutable* reference, this function unerases it to an *immutable* one.
    /// If you need a *mutable* reference, use [`AnyMut::unerase_mut()`] or [`AnyMut::unerase_into()`]
    pub fn unerase<T: 'static>(&self) -> Option<&T> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid reference, so it's not null and aligned
            unsafe { &*self.ptr.cast_const().cast::<T>() }
        })
    }

    /// Unerase back to a mmutable reference
    ///
    /// This functions essentially the same as [`Any::downcast_mut()`](https://doc.rust-lang.org/core/any/trait.Any.html#method.downcast_mut). If the
    /// original reference's element type was `T`, a valid reference is returned. Otherwise, you get `None`.
    ///
    /// ```
    /// let mut data : i32 = 7;
    /// let mut any = sashay::AnyMut::erase(&mut data);
    ///
    /// assert!(any.unerase_mut::<bool>().is_none());
    /// assert!(any.unerase_mut::<i32>().is_some());
    /// ```
    ///
    /// Note that this function unerases to a mutable reference. If you only need an immutable one, you
    /// can use [`AnyMut::unerase()`]
    pub fn unerase_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid reference, so it's not null and aligned
            unsafe { &mut *self.ptr.cast::<T>() }
        })
    }

    /// Unerase back into a mmutable reference
    ///
    /// This functions essentially the same as [`AnyMut::unerase_mut()`],
    /// except that ownership is tranferred into the reference. If the original reference's element type was `T`,
    /// a valid reference is returned. Otherwise, you get `None`.
    ///
    /// ```
    /// let mut data : i32 = 7;
    /// let any = sashay::AnyMut::erase(&mut data);
    ///
    /// assert!(any.unerase_into::<i32>().is_some());
    /// ```
    pub fn unerase_into<T: 'static>(self) -> Option<&'a mut T> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid reference, so it's not null and aligned
            unsafe { &mut *self.ptr.cast::<T>() }
        })
    }

    // Retrieve an unsafe immutable pointer to the raw data
    pub const fn as_ptr(&self) -> *const () {
        self.ptr.cast_const()
    }

    // Retrieve an unsafe mutable pointer to the raw data
    pub fn as_mut_ptr(&mut self) -> *mut () {
        self.ptr
    }

    /// Does the slice contain elements of type `T`?
    pub fn contains<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }

    /// A unique type id representing the original reference type `T`
    pub const fn type_id(&self) -> &TypeId {
        &self.type_id
    }
}

impl<'a, T: 'static> From<&'a mut T> for AnyMut<'a> {
    fn from(reference: &'a mut T) -> Self {
        Self::erase(reference)
    }
}
