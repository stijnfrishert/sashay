use crate::AnyRef;
use erasable::{erase, ErasablePtr, ErasedPtr};
use std::{any::TypeId, marker::PhantomData, ptr::NonNull};

/// A type-erased mutable reference
///
/// # Example
///
/// ```
/// let mut data = 'z';
/// let mut any = sashay::AnyMut::erase(&mut data);
/// let reference = any.downcast_mut::<char>().expect("any was not a &mut char");
///
/// *reference = '💤';
///
/// assert_eq!(data, '💤');
/// ```
#[derive(Debug)]
pub struct AnyMut<'a> {
    pub(super) ptr: ErasedPtr,
    pub(super) type_id: TypeId,
    pub(super) _lifetime: PhantomData<&'a mut ()>,
}

impl<'a> AnyMut<'a> {
    /// Erase the element type of a reference
    pub fn erase<T: 'static>(reference: &'a mut T) -> Self {
        Self {
            ptr: erase(reference.into()),
            type_id: TypeId::of::<T>(),
            _lifetime: PhantomData,
        }
    }

    /// Try to downcast back to the original reference
    ///
    /// If the type does not match, [`None`] is returned
    pub fn downcast_ref<'b, T: 'static>(&'b self) -> Option<&'b T>
    where
        'a: 'b,
    {
        let expected = TypeId::of::<T>();

        if self.type_id == expected {
            // SAFETY: This is safe, because we've checked that the type ids match
            let ptr = unsafe { <NonNull<T>>::unerase(self.ptr) };

            // SAFETY: The ptr can't be null and was initialized
            let reference = unsafe { ptr.as_ref() };

            Some(reference)
        } else {
            None
        }
    }

    /// Try to downcast back to the original reference
    ///
    /// If the type does not match, [`None`] is returned
    pub fn into_ref<T: 'static>(self) -> Option<&'a T> {
        let expected = TypeId::of::<T>();

        if self.type_id == expected {
            // SAFETY: This is safe, because we've checked that the type ids match
            let ptr = unsafe { <NonNull<T>>::unerase(self.ptr) };

            // SAFETY: The ptr can't be null and was initialized
            let reference = unsafe { ptr.as_ref() };

            Some(reference)
        } else {
            None
        }
    }

    /// Try to downcast back to the original reference
    ///
    /// If the type does not match, [`None`] is returned
    pub fn downcast_mut<'b, T: 'static>(&'b mut self) -> Option<&'b mut T>
    where
        'a: 'b,
    {
        let expected = TypeId::of::<T>();

        if self.type_id == expected {
            // SAFETY: This is safe, because we've checked that the type ids match
            let mut ptr = unsafe { <NonNull<T>>::unerase(self.ptr) };

            // SAFETY: The ptr can't be null and was initialized
            let reference = unsafe { ptr.as_mut() };

            Some(reference)
        } else {
            None
        }
    }

    /// Try to downcast back to the original reference
    ///
    /// If the type does not match, [`None`] is returned
    pub fn into_mut<T: 'static>(self) -> Option<&'a mut T> {
        let expected = TypeId::of::<T>();

        if self.type_id == expected {
            // SAFETY: This is safe, because we've checked that the type ids match
            let mut ptr = unsafe { <NonNull<T>>::unerase(self.ptr) };

            // SAFETY: The ptr can't be null and was initialized
            let reference = unsafe { ptr.as_mut() };

            Some(reference)
        } else {
            None
        }
    }

    /// Convert the mutable reference to an immutable one
    pub fn as_immutable<'b>(&'b self) -> AnyRef<'b>
    where
        'a: 'b,
    {
        AnyRef {
            ptr: self.ptr,
            type_id: self.type_id,
            _lifetime: PhantomData,
        }
    }

    /// Convert the mutable reference to an immutable one
    pub fn into_immutable(self) -> AnyRef<'a> {
        AnyRef {
            ptr: self.ptr,
            type_id: self.type_id,
            _lifetime: PhantomData,
        }
    }

    /// The [`TypeId`] of the elements of the original reference that was erased
    pub fn type_id(&self) -> &TypeId {
        &self.type_id
    }
}

impl<'a, T> From<&'a mut T> for AnyMut<'a>
where
    T: 'static,
{
    fn from(reference: &'a mut T) -> Self {
        AnyMut::erase(reference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downcast_ref() {
        let mut data = 'z';
        let any = AnyMut::erase(&mut data);
        let copy = *any.downcast_ref::<char>().expect("any was not a &char");

        assert_eq!(copy, data);
    }

    #[test]
    fn downcast_mut() {
        let mut data = 'z';
        let mut any = AnyMut::erase(&mut data);
        let reference = any.downcast_mut::<char>().expect("any was not a &mut char");

        *reference = '💤';

        assert_eq!(data, '💤');
    }

    #[test]
    fn as_immutable() {
        let mut data = 'z';
        let any = AnyMut::erase(&mut data);
        let im1 = any.as_immutable();
        let im2 = any.as_immutable();
        assert_eq!(im1.downcast_ref::<char>().unwrap(), &'z');
        assert_eq!(im2.downcast_ref::<char>().unwrap(), &'z');
    }
}
