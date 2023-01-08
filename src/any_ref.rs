use core::{any::TypeId, marker::PhantomData};

/// A type-erased immutable reference.
///
/// An immutable borrow of some owned memory, just like regular Rust primitive
/// [references](https://doc.rust-lang.org/std/primitive.reference.html), except that the type of the
/// referee is erased. This allows you to deal with and *store* references of different
/// types within the same collection.
///
/// # Example
///
/// ```
/// let data : char = '🦀';
/// let any = sashay::AnyRef::erase(&data);
/// let reference = any.unerase::<char>().expect("not a reference to `char`");
///
/// assert_eq!(reference, &data);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AnyRef<'a> {
    /// A raw pointer to the referenced data
    ptr: *const (),

    /// A unique id representing the type of the referenced data
    ///
    /// This is used to ensure we can safely unerase back without accidentally transmuting
    type_id: TypeId,

    /// Phantom data to ensure that we stick to the correct lifetime
    _phantom: PhantomData<&'a ()>,
}

impl<'a> AnyRef<'a> {
    /// Erase the type of an immutable reference.
    ///
    /// The resulting type retains the lifetime of the original reference, but the
    /// referred to value can only be used after unerasing the type
    ///
    /// ```
    /// let data : char = '🦀';
    /// let any = sashay::AnyRef::erase(&data);
    ///
    /// assert!(any.contains::<char>());
    /// ```
    pub fn erase<T: 'static>(reference: &'a T) -> AnyRef<'a> {
        // Safety:
        //  - The raw parts come from a valid reference
        //  - The TypeId is provided by the compiler
        unsafe { Self::from_raw_parts((reference as *const T).cast::<()>(), TypeId::of::<T>()) }
    }

    /// Construct an erased reference from its raw parts.
    ///
    /// If you already have a `&T`, it is recommended to call [`AnyRef::erase()`].
    ///
    /// This function behaves the same as calling `as *const T` on a reference, with the addition that
    /// it takes a unique `type_id` representing the type `T`.
    ///
    /// # Safety
    ///
    /// Calling this is only defined behaviour if:
    ///  - The pointer refers to a valid `T`
    ///  - `type_id` is the correct `TypeId` for `T`
    pub unsafe fn from_raw_parts(ptr: *const (), type_id: TypeId) -> Self {
        Self {
            ptr,
            type_id,
            _phantom: PhantomData,
        }
    }

    /// Unerase back to an immutable reference.
    ///
    /// This behaves essentially the same as [`Any::downcast_ref()`](https://doc.rust-lang.org/core/any/trait.Any.html#method.downcast_ref). If the
    /// original reference's type was `T`, a valid reference is returned. Otherwise, you get `None`.
    ///
    /// ```
    /// let data : i32 = 7;
    /// let any = sashay::AnyRef::erase(&data);
    ///
    /// // You can unerase multiple times, because this is a shared, immutable reference
    /// let unerased_a = any.unerase::<i32>().unwrap();
    /// let unerased_b = any.unerase::<i32>().unwrap();
    /// assert_eq!(unerased_a, unerased_b);
    ///
    /// // Doesn't compile, because you can't mutate
    /// // *unerased_a = 0;
    ///
    /// // Unerasing to a different type gives you nothing
    /// assert!(any.unerase::<bool>().is_none());
    /// ```
    pub fn unerase<T: 'static>(&self) -> Option<&T> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid reference, so it's not null and aligned
            unsafe { &*self.ptr.cast::<T>() }
        })
    }

    /// Unerase back into an immutable reference.
    ///
    /// This behaves essentially the same as [`AnyRef::unerase()`],
    /// except that ownership is tranferred into the reference. If the original reference's element type was `T`,
    /// a valid reference is returned. Otherwise, you get `None`.
    ///
    /// ```
    /// let data : i32 = 7;
    ///
    /// let unerased = {
    ///     // Unerase, transferring ownership into the resulting reference
    ///     let any = sashay::AnyRef::erase(&data);
    ///     let unerased = any.unerase_into::<i32>().unwrap();
    ///
    ///     // Can't unerase anymore after this, ownerhip has been moved out of the any
    ///     // any.unerase_into::<i32>();
    ///
    ///     // Because unerase_into() transfers ownership, the resulting reference's lifetime
    ///     // can escape the any's lifetime scope and just reference the original data
    ///     unerased
    /// };
    ///
    /// assert_eq!(unerased, &7);
    /// // Can't unerase anymore after this, ownerhip has been moved out of the any
    /// ```
    pub fn unerase_into<T: 'static>(self) -> Option<&'a T> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid reference, so it's not null and aligned
            unsafe { &*self.ptr.cast::<T>() }
        })
    }

    /// Retrieve an unsafe immutable pointer to the raw data.
    pub const fn as_ptr(&self) -> *const () {
        self.ptr
    }

    /// Was the original referee of type `T`?
    pub fn contains<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }

    /// A unique type id representing the original reference type `T`.
    pub const fn type_id(&self) -> &TypeId {
        &self.type_id
    }
}

impl<'a, T: 'static> From<&'a mut T> for AnyRef<'a> {
    fn from(reference: &'a mut T) -> Self {
        Self::erase(reference)
    }
}
