use erasable::{erase, ErasablePtr, ErasedPtr};
use std::{any::TypeId, marker::PhantomData, ptr::NonNull};

/// A type-erased immutable reference
///
/// # Example
///
/// ```
/// let data : char = '🦀';
/// let any = sashay::AnyRef::erase(&data);
/// let reference = any.downcast_ref::<char>().expect("any was not a &char");
///
/// assert_eq!(reference, &data);
/// ```
#[derive(Clone, Copy)]
pub struct AnyRef<'a> {
    pub(super) ptr: ErasedPtr,
    pub(super) type_id: TypeId,
    pub(super) _lifetime: PhantomData<&'a ()>,
}

impl<'a> AnyRef<'a> {
    /// Erase the element type of a reference
    pub fn erase<T: 'static>(reference: &'a T) -> Self {
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

    /// The [`TypeId`] of the elements of the original reference that was erased
    pub fn type_id(&self) -> &TypeId {
        &self.type_id
    }
}

impl<'a, T> From<&'a T> for AnyRef<'a>
where
    T: 'static,
{
    fn from(reference: &'a T) -> Self {
        AnyRef::erase(reference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downcast_ref() {
        let data = 8;
        let any = AnyRef::erase(&data);
        let reference = any.downcast_ref::<i32>().expect("any was not a &i32");

        assert_eq!(reference, &data);
    }
}
