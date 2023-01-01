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

    /// Unerase the type back to an immutable reference
    ///
    /// If the the erased reference was created with T, you get the original
    /// reference back. For any other T, this function returns None
    pub fn unerase<T: 'static>(&self) -> Option<&T> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid reference, so it's not null and aligned
            unsafe { &*self.ptr.cast_const().cast::<T>() }
        })
    }

    /// Unerase the type back to a mutable reference
    ///
    /// If the the erased reference was created with T, you get the original
    /// reference back. For any other T, this function returns None
    pub fn unerase_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid reference, so it's not null and aligned
            unsafe { &mut *self.ptr.cast::<T>() }
        })
    }

    /// Unerase the type back into a mutable reference
    ///
    /// If the the erased slice ref was created with T, you get the original
    /// slice back. For any other T, this function returns None
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
