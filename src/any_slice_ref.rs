use core::{any::TypeId, marker::PhantomData, slice::from_raw_parts};

pub struct AnySliceRef<'a> {
    /// A pointer to the first element in the slice
    /// Must be aligned
    ptr: *const (),

    /// The number of elements in the slice
    len: usize,

    /// The type if of the elements in the slice
    /// This is used to ensure we can safely cast back to typed slices
    type_id: TypeId,

    /// Phantom data to ensure that we stick to the correct lifetime
    _phantom: PhantomData<&'a ()>,
}

impl<'a> AnySliceRef<'a> {
    /// Erase the type of a slice's elements
    pub fn erase<T: 'static>(slice: &'a [T]) -> AnySliceRef<'a> {
        Self {
            ptr: slice.as_ptr() as *const (),
            len: slice.len(),
            type_id: TypeId::of::<T>(),
            _phantom: PhantomData,
        }
    }

    /// Unerase the type back to a primitive Rust slice
    ///
    /// If the the erased slice ref was created with T, you get the original
    /// slice back. For any other T, this function returns None
    pub fn unerase<T: 'static>(&self) -> Option<&[T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts(self.ptr as *const T, self.len) }
        })
    }

    /// Unerase the type back to a primitive Rust slice
    ///
    /// If the the erased slice ref was created with T, you get the original
    /// slice back. For any other T, this function returns None
    pub fn unerase_into<T: 'static>(self) -> Option<&'a [T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts(self.ptr as *const T, self.len) }
        })
    }

    /// How many elements does the slice contain?
    pub fn len(&self) -> usize {
        self.len
    }

    /// Does the slice contain any elements at all?
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // Does the slice contain elements of type T?
    pub fn contains<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erase_unerase() {
        // Using an (u8, u16) because it has padding
        let data = [(1u8, 2u16), (3u8, 4u16)];
        let any = AnySliceRef::erase(data.as_slice());

        assert_eq!(any.len(), 2);
        assert!(!any.is_empty());

        // unerase
        assert_eq!(any.unerase::<(u8, u16)>(), Some(data.as_slice()));
        assert_eq!(any.unerase::<u8>(), None);
        assert_eq!(any.unerase_into::<(u8, u16)>(), Some(data.as_slice()));
    }
}
