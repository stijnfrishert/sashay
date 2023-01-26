use crate::AnyRef;
use core::{any::TypeId, marker::PhantomData};

/// A type-erased mutable reference.
///
/// A mutable borrow of some owned memory, just like regular Rust primitive
/// [references](https://doc.rust-lang.org/std/primitive.reference.html), except that the type of the
/// referee is erased. This allows you to deal with and *store* references of different
/// types within the same collection.
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

    /// Construct an erased reference from its raw parts.
    ///
    /// If you already have a `&mut T`, it is recommended to call [`AnyMut::erase()`].
    ///
    /// This function behaves the same as calling `as *mut T` on a reference, with the addition that
    /// it takes a unique `type_id` representing the type `T`.
    ///
    /// # Safety
    ///
    /// Calling this is only defined behaviour if:
    ///  - The pointer refers to a valid `T`
    ///  - `type_id` is the correct `TypeId` for `T`
    pub unsafe fn from_raw_parts(ptr: *mut (), type_id: TypeId) -> Self {
        Self {
            ptr,
            type_id,
            _phantom: PhantomData,
        }
    }

    /// Unerase back to an _immutable_ reference.
    ///
    /// This behaves essentially the same as [`Any::downcast_ref()`](https://doc.rust-lang.org/core/any/trait.Any.html#method.downcast_ref). If the
    /// original reference's type was `T`, a valid reference is returned. Otherwise, you get `None`.
    ///
    /// Note that while `AnyMut` represents a *mutable* reference, this function unerases it to an *immutable* one.
    /// If you need a mutable reference, use [`AnyMut::unerase_mut()`] or [`AnyMut::unerase_into()`]
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
            unsafe { &*self.ptr.cast_const().cast::<T>() }
        })
    }

    /// Unerase back to a mutable reference.
    ///
    /// This behaves essentially the same as [`Any::downcast_mut()`](https://doc.rust-lang.org/core/any/trait.Any.html#method.downcast_mut). If the
    /// original reference's type was `T`, a valid reference is returned. Otherwise, you get `None`.
    ///
    /// Note that this function unerases to a _mutable_ reference. If you only need an immutable one, you
    /// can use [`AnyMut::unerase()`]
    ///
    /// ```
    /// let mut data : i32 = 7;
    /// let mut any = sashay::AnyMut::erase(&mut data);
    ///
    /// // You can unerase back to a mutable reference
    /// let unerased = any.unerase_mut::<i32>().unwrap();
    /// *unerased = 0;
    /// assert_eq!(data, 0);
    ///
    /// // You can't unerase_mut twice, because this is a _unique_, mutable reference
    /// // any.unerase_mut::<i32>();
    /// ```
    pub fn unerase_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid reference, so it's not null and aligned
            unsafe { &mut *self.ptr.cast::<T>() }
        })
    }

    /// Unerase back into a mutable reference.
    ///
    /// This behaves essentially the same as [`AnyMut::unerase_mut()`],
    /// except that ownership is tranferred into the reference. If the original reference's type was `T`,
    /// a valid reference is returned. Otherwise, you get `None`.
    ///
    /// ```
    /// let mut data : i32 = 7;
    ///
    /// let unerased = {
    ///     // Unerase, transferring ownership into the resulting reference
    ///     let any = sashay::AnyMut::erase(&mut data);
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
    /// *unerased = 11;
    /// assert_eq!(data, 11);
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

    /// Borrow this mutable reference as an immutable one.
    ///
    /// Even though you have mutable and unique access to a reference, this fuction lets you
    /// trade in the mutability for shared access.
    ///
    /// ```
    /// let mut data : i32 = 7;
    /// let any = sashay::AnyMut::erase(&mut data);
    ///
    /// // borrow() can be called multiple times, because immutable references provide shared access
    /// let immutable_a : sashay::AnyRef = any.borrow();
    /// let immutable_b : sashay::AnyRef = any.borrow();
    ///
    /// assert_eq!(immutable_a.unerase::<i32>(), Some(&7));
    /// assert_eq!(immutable_b.unerase::<i32>(), Some(&7));
    /// ```
    pub fn borrow(&self) -> AnyRef {
        // SAFETY:
        // All parts are valid, we just cast to const
        // This is ok, because we have an immutable ref to self
        unsafe { AnyRef::from_raw_parts(self.ptr.cast_const(), self.type_id) }
    }

    /// Retrieve an unsafe immutable pointer to the raw data.
    pub const fn as_ptr(&self) -> *const () {
        self.ptr.cast_const()
    }

    /// Retrieve an unsafe mutable pointer to the raw data.
    pub fn as_mut_ptr(&mut self) -> *mut () {
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

impl<'a, T: 'static> From<&'a mut T> for AnyMut<'a> {
    fn from(reference: &'a mut T) -> Self {
        Self::erase(reference)
    }
}
