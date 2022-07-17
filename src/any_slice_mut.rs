use erasable::{erase, ErasablePtr, ErasedPtr};
use std::{any::TypeId, marker::PhantomData, ptr::NonNull, slice::from_raw_parts_mut};

/// A type-erased mutable slice
///
/// # Example
///
/// ```
/// let mut data : [i32; 3] = [0, 1, 2];
/// let any = sashay::AnySliceMut::erase(data.as_mut_slice());
/// let slice = any.downcast::<i32>().expect("any was not a &mut [i32]");
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
    pub fn erase<U: 'static>(slice: &'a mut [U]) -> Self {
        Self {
            ptr: erase(slice.into()),
            len: slice.len(),
            type_id: TypeId::of::<U>(),
            _lifetime: PhantomData,
        }
    }

    /// Try to downcast back to the original slice
    ///
    /// If the type does not match, [`None`] is returned
    pub fn downcast<U: 'static>(&self) -> Option<&'a mut [U]> {
        let expected = TypeId::of::<U>();

        if self.type_id == expected {
            // SAFETY: This is safe, because we've checked that the type ids match
            let ptr = unsafe { <NonNull<U>>::unerase(self.ptr) };

            // SAFETY: The length is valid, we got it from the original slice at erasure and the ptr can't be null.
            let slice = unsafe { from_raw_parts_mut(ptr.as_ptr(), self.len) };

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