use erasable::{erase, ErasablePtr, ErasedPtr};
use std::{
    any::TypeId,
    marker::PhantomData,
    ptr::NonNull,
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// A type-erased mutable slice
///
/// # Example
///
/// ```
/// let mut data : [i32; 3] = [0, 1, 2];
/// let mut any = sashay::AnySliceMut::erase(data.as_mut_slice());
/// let slice = any.downcast_mut::<i32>().expect("any was not a &mut [i32]");
///
/// slice.fill(0);
///
/// assert_eq!(data, [0, 0, 0]);
/// ```
pub struct AnySliceMut<'a> {
    pub(super) ptr: ErasedPtr,
    pub(super) len: usize,
    pub(super) type_id: TypeId,
    pub(super) _lifetime: PhantomData<&'a mut ()>,
}

impl<'a> AnySliceMut<'a> {
    /// Erase the element type of a slice
    pub fn erase<T: 'static>(slice: &'a mut [T]) -> Self {
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

    /// Try to downcast back to the original slice
    ///
    /// If the type does not match, [`None`] is returned
    pub fn into_ref<T: 'static>(self) -> Option<&'a [T]> {
        self.downcast_ref()
    }

    /// Try to downcast back to the original slice
    ///
    /// If the type does not match, [`None`] is returned
    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&'a mut [T]> {
        let expected = TypeId::of::<T>();

        if self.type_id == expected {
            // SAFETY: This is safe, because we've checked that the type ids match
            let ptr = unsafe { <NonNull<T>>::unerase(self.ptr) };

            // SAFETY: The length is valid, we got it from the original slice at erasure and the ptr can't be null.
            let slice = unsafe { from_raw_parts_mut(ptr.as_ptr(), self.len) };

            Some(slice)
        } else {
            None
        }
    }

    /// Try to downcast back to the original slice
    ///
    /// If the type does not match, [`None`] is returned
    pub fn into_mut<T: 'static>(mut self) -> Option<&'a mut [T]> {
        self.downcast_mut()
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

impl<'a, T> From<&'a mut [T]> for AnySliceMut<'a>
where
    T: 'static,
{
    fn from(slice: &'a mut [T]) -> Self {
        AnySliceMut::erase(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downcast_ref() {
        let mut data: [i32; 3] = [0, 1, 2];
        let any = AnySliceMut::erase(data.as_mut_slice());
        let copy = any
            .downcast_ref::<i32>()
            .expect("any was not a &[i32]")
            .to_vec();

        assert_eq!(copy.as_slice(), data.as_slice());
    }

    #[test]
    fn downcast_mut() {
        let mut data: [i32; 3] = [0, 1, 2];
        let slice;

        // Create any in new scope, to check if the lifetime
        // coming out of downcast can outlive it (but not the data)
        {
            let mut any = AnySliceMut::erase(data.as_mut_slice());
            slice = any.downcast_mut::<i32>().expect("any was not a &mut [i32]");
        }

        slice.fill(0);

        assert_eq!(data, [0, 0, 0]);
    }

    #[test]
    fn getters() {
        let mut data: [i32; 3] = [0, 1, 2];
        let any = AnySliceMut::erase(data.as_mut_slice());

        assert_eq!(any.type_id(), &TypeId::of::<i32>());
        assert_eq!(any.len(), 3);
        assert!(!any.is_empty());

        let mut data: [i32; 0] = [];
        let any = AnySliceMut::erase(data.as_mut_slice());

        assert_eq!(any.len(), 0);
        assert!(any.is_empty());
    }
}
