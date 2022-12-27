use super::{AnySliceMut, AnySliceRef};
use erasable::ErasedPtr;
use std::{any::TypeId, marker::PhantomData};

/// A type-erased pointer to some slice
///
/// Where `AnySliceRef` and `AnySliceMut` mimic `&[T]` and `&mut [T]`, the any-equivalent of
/// `*mut/const [T]` is `AnySlicePtr`.
///
/// This struct behaves like regular pointers, in the sense that copying them is perfectly
/// safe, up to the point where you try to dereference one, and so this function is unsafe.
/// It is up to you to ensure that [`AnySlicePtr`]'s to the same memory location are never
/// accessed immutably and mutably at the same time.
#[derive(Debug, Clone, Copy)]
pub struct AnySlicePtr {
    ptr: ErasedPtr,
    start: usize,
    len: usize,
    type_id: TypeId,
}

impl AnySlicePtr {
    /// Convert to a type-erased, immutable `AnySliceRef`
    ///
    /// # Safety
    ///
    /// Just like regular pointers, they can be copied all over the place, and it is up to
    /// the user to ensure they don't alias when dereferenced, and that they lifetime of the
    /// original reference is respected.
    pub unsafe fn deref<'a>(self) -> AnySliceRef<'a> {
        AnySliceRef {
            ptr: self.ptr,
            start: self.start,
            len: self.len,
            type_id: self.type_id,
            _lifetime: PhantomData,
        }
    }

    /// Convert to a type-erased, mutable `AnySliceMut`
    ///
    /// # Safety
    ///
    /// Just like regular pointers, they can be copied all over the place, and it is up to
    /// the user to ensure they don't alias when dereferenced, and that they lifetime of the
    /// original reference is respected.
    pub unsafe fn deref_mut<'a>(self) -> AnySliceMut<'a> {
        AnySliceMut {
            ptr: self.ptr,
            start: self.start,
            len: self.len,
            type_id: self.type_id,
            _lifetime: PhantomData,
        }
    }

    /// The [`TypeId`] of the elements of the original slice that was passed in
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

impl<'a> From<AnySliceRef<'a>> for AnySlicePtr {
    fn from(slice: AnySliceRef<'a>) -> Self {
        Self {
            ptr: slice.ptr,
            start: slice.start,
            len: slice.len,
            type_id: slice.type_id,
        }
    }
}

impl<'a> From<AnySliceMut<'a>> for AnySlicePtr {
    fn from(slice: AnySliceMut<'a>) -> Self {
        Self {
            ptr: slice.ptr,
            start: slice.start,
            len: slice.len,
            type_id: slice.type_id,
        }
    }
}
