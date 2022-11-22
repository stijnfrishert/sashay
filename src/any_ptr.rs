use super::{AnyMut, AnyRef};
use erasable::ErasedPtr;
use std::{any::TypeId, marker::PhantomData};

/// A type-erased pointer to some reference
///
/// Where `AnyRef` and `AnyMut` mimic `&T` and `&mut T`, the any-equivalent of
/// `*mut/const T` is `AnyPtr`.
///
/// This struct behaves like regular pointers, in the sense that copying them is perfectly
/// safe, up to the point where you try to dereference one, and so this function is unsafe.
/// It is up to you to ensure that [`AnyPtr`]'s to the same memory location are never
/// accessed immutably and mutably at the same time.
#[derive(Debug, Clone, Copy)]
pub struct AnyPtr {
    ptr: ErasedPtr,
    type_id: TypeId,
}

impl AnyPtr {
    /// Convert to a type-erased, immutable `AnyRef`
    ///
    /// # Safety
    ///
    /// Just like regular pointers, they can be copied all over the place, and it is up to
    /// the user to ensure they don't alias when dereferenced, and that they lifetime of the
    /// original reference is respected.
    pub unsafe fn deref<'a>(self) -> AnyRef<'a> {
        AnyRef {
            ptr: self.ptr,
            type_id: self.type_id,
            _lifetime: PhantomData,
        }
    }

    /// Convert to a type-erased, mutable `AnyMut`
    ///
    /// # Safety
    ///
    /// Just like regular pointers, they can be copied all over the place, and it is up to
    /// the user to ensure they don't alias when dereferenced, and that they lifetime of the
    /// original reference is respected.
    pub unsafe fn deref_mut<'a>(self) -> AnyMut<'a> {
        AnyMut {
            ptr: self.ptr,
            type_id: self.type_id,
            _lifetime: PhantomData,
        }
    }

    /// The [`TypeId`] of the elements of the original reference that was passed in
    pub fn type_id(&self) -> &TypeId {
        &self.type_id
    }
}

impl<'a> From<AnyRef<'a>> for AnyPtr {
    fn from(reference: AnyRef<'a>) -> Self {
        Self {
            ptr: reference.ptr,
            type_id: reference.type_id,
        }
    }
}

impl<'a> From<AnyMut<'a>> for AnyPtr {
    fn from(reference: AnyMut<'a>) -> Self {
        Self {
            ptr: reference.ptr,
            type_id: reference.type_id,
        }
    }
}
