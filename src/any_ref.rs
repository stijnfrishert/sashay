use core::{any::TypeId, marker::PhantomData};

/// A type-erased immutable reference
///
/// # Example
///
/// ```
/// let data : char = '🦀';
/// let any = sashay::AnyRef::erase(&data);
/// let reference = any.unerase::<char>().expect("any was not a &char");
///
/// assert_eq!(reference, &data);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AnyRef<'a> {
    /// A pointer to the data referred to
    ptr: *const (),

    /// The TypeId of the referred to data
    /// This is used to ensure we can safely cast back to typed references
    type_id: TypeId,

    /// Phantom data to ensure that we stick to the correct lifetime
    _phantom: PhantomData<&'a ()>,
}

impl<'a> AnyRef<'a> {
    /// Construct an erased reference from its raw parts
    ///
    /// # Safety
    ///
    /// Calling this is only defined behaviour if:
    ///  - The pointer refers to a valid `T`
    ///  - `type_id` is the correct `TypeId` for the element `T`
    pub unsafe fn from_raw_parts(ptr: *const (), type_id: TypeId) -> Self {
        Self {
            ptr,
            type_id,
            _phantom: PhantomData,
        }
    }

    /// Erase the type of a reference
    pub fn erase<T: 'static>(reference: &'a T) -> AnyRef<'a> {
        // Safety:
        //  - The raw parts come from a valid reference
        //  - The TypeId comes directly from the reference `T`
        unsafe { Self::from_raw_parts((reference as *const T).cast::<()>(), TypeId::of::<T>()) }
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
            unsafe { &*self.ptr.cast::<T>() }
        })
    }

    /// Unerase the type back into an immutable reference
    ///
    /// If the the erased slice ref was created with T, you get the original
    /// slice back. For any other T, this function returns None
    pub fn unerase_into<T: 'static>(self) -> Option<&'a T> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid reference, so it's not null and aligned
            unsafe { &*self.ptr.cast::<T>() }
        })
    }

    // Retrieve an unsafe immutable pointer to the raw data
    pub const fn as_ptr(&self) -> *const () {
        self.ptr
    }

    /// Does the slice contain elements of type `T`?
    pub fn contains<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }
}

impl<'a, T: 'static> From<&'a mut T> for AnyRef<'a> {
    fn from(reference: &'a mut T) -> Self {
        Self::erase(reference)
    }
}
