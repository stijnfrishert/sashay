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
    pub fn downcast_ref<T: 'static>(&self) -> Option<&'a T> {
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
        self.downcast_ref()
    }

    /// Try to downcast back to the original reference
    ///
    /// If the type does not match, [`None`] is returned
    pub fn downcast_mut<U: 'static>(&mut self) -> Option<&'a mut U> {
        let expected = TypeId::of::<U>();

        if self.type_id == expected {
            // SAFETY: This is safe, because we've checked that the type ids match
            let mut ptr = unsafe { <NonNull<U>>::unerase(self.ptr) };

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
    pub fn into_mut<T: 'static>(mut self) -> Option<&'a mut T> {
        self.downcast_mut()
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
        let reference;

        // Create any in new scope, to check if the lifetime
        // coming out of downcast can outlive it (but not the data)
        {
            let mut any = AnyMut::erase(&mut data);
            reference = any.downcast_mut::<char>().expect("any was not a &mut char");
        }

        *reference = '💤';

        assert_eq!(data, '💤');
    }
}
