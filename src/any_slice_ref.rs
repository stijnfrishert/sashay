use erasable::{erase, ErasablePtr, ErasedPtr};
use std::{any::TypeId, marker::PhantomData, ptr::NonNull, slice::from_raw_parts};

/// A type-erased immutable slice
///
/// # Example
///
/// ```
/// let data : [i32; 3] = [0, 1, 2];
/// let any = sashay::AnySliceRef::erase(data.as_slice());
/// let slice = any.downcast_ref::<i32>().expect("any was not a &[i32]");
///
/// assert_eq!(slice, data.as_slice());
/// ```
#[derive(Clone, Copy)]
pub struct AnySliceRef<'a> {
    pub(super) ptr: ErasedPtr,
    pub(super) len: usize,
    pub(super) type_id: TypeId,
    pub(super) _lifetime: PhantomData<&'a ()>,
}

impl<'a> AnySliceRef<'a> {
    /// Erase the element type of a slice
    pub fn erase<T: 'static>(slice: &'a [T]) -> Self {
        Self {
            ptr: erase(slice.into()),
            len: slice.len(),
            type_id: TypeId::of::<T>(),
            _lifetime: PhantomData,
        }
    }

    /// Try to downcast back to the original slice
    ///
    /// If the type does not match, [`None`] is returned
    pub fn downcast_ref<T: 'static>(&self) -> Option<&'a [T]> {
        let expected = TypeId::of::<T>();

        if self.type_id == expected {
            // SAFETY: This is safe, because we've checked that the type ids match
            let ptr = unsafe { <NonNull<T>>::unerase(self.ptr) };

            // SAFETY: The length is valid, we got it from the original slice at erasure and the ptr can't be null.
            let slice = unsafe { from_raw_parts(ptr.as_ptr(), self.len) };

            Some(slice)
        } else {
            None
        }
    }

    /// The [`TypeId`] of the elements of the original slice that was erased
    pub fn type_id(&self) -> &TypeId {
        &self.type_id
    }

    /// The length of the original slice that was erased
    pub fn len(&self) -> usize {
        self.len
    }

    /// Does the slice contain any elements?
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<'a, T> From<&'a [T]> for AnySliceRef<'a>
where
    T: 'static,
{
    fn from(slice: &'a [T]) -> Self {
        AnySliceRef::erase(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downcast_ref() {
        let data: [i32; 3] = [0, 1, 2];
        let slice;

        // Create any in new scope, to check if the lifetime
        // coming out of downcast can outlive it (but not the data)
        {
            let any = AnySliceRef::erase(data.as_slice());
            slice = any.downcast_ref::<i32>().expect("any was not a &[i32]");
        }

        assert_eq!(slice, data.as_slice());
    }

    #[test]
    fn getters() {
        let data: [i32; 3] = [0, 1, 2];
        let any = AnySliceRef::erase(data.as_slice());

        assert_eq!(any.type_id(), &TypeId::of::<i32>());
        assert_eq!(any.len(), 3);
        assert!(!any.is_empty());

        let data: [i32; 0] = [];
        let any = AnySliceRef::erase(data.as_slice());

        assert_eq!(any.len(), 0);
        assert!(any.is_empty());
    }
}
