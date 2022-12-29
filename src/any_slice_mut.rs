use crate::{range::constrain_range, AnySliceRef};
use core::{
    any::TypeId,
    marker::PhantomData,
    mem::size_of,
    ops::RangeBounds,
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// A type-erased mutable slice
///
/// # Example
///
/// ```
/// let mut data : [i32; 3] = [0, 1, 2];
/// let mut any = sashay::AnySliceMut::erase(data.as_mut_slice());
/// let slice = any.unerase_mut::<i32>().expect("any was not a &mut [i32]");
///
/// slice.fill(0);
///
/// assert_eq!(data, [0, 0, 0]);
/// ```
#[derive(Debug)]
pub struct AnySliceMut<'a> {
    /// A pointer to the first element in the slice
    /// Must be aligned
    ptr: *mut u8,

    /// The number of elements in the slice
    len: usize,

    /// The byte size/stride of the original element type
    stride: usize,

    /// The TypeId of the elements in the slice
    /// This is used to ensure we can safely cast back to typed slices
    type_id: TypeId,

    /// Phantom data to ensure that we stick to the correct lifetime
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> AnySliceMut<'a> {
    pub(crate) fn new(ptr: *mut u8, len: usize, stride: usize, type_id: TypeId) -> Self {
        Self {
            ptr,
            len,
            stride,
            type_id,
            _phantom: PhantomData,
        }
    }

    /// Erase the type of a slice's elements
    pub fn erase<T: 'static>(slice: &'a mut [T]) -> AnySliceMut<'a> {
        Self::new(
            slice.as_mut_ptr() as *mut u8,
            slice.len(),
            size_of::<T>(),
            TypeId::of::<T>(),
        )
    }

    /// Unerase the type back to an immutable slice
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

    /// Unerase the type back to a mutable slice
    ///
    /// If the the erased slice ref was created with T, you get the original
    /// slice back. For any other T, this function returns None
    pub fn unerase_mut<T: 'static>(&mut self) -> Option<&mut [T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts_mut(self.ptr as *mut T, self.len) }
        })
    }

    /// Unerase the type back to a primitive Rust slice
    ///
    /// If the the erased slice ref was created with T, you get the original
    /// slice back. For any other T, this function returns None
    pub fn unerase_into<T: 'static>(self) -> Option<&'a mut [T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts_mut(self.ptr as *mut T, self.len) }
        })
    }

    /// Create a sub-slice of this slice
    pub fn sub<R>(&self, range: R) -> AnySliceRef
    where
        R: RangeBounds<usize>,
    {
        let range = constrain_range(self.len, range);

        AnySliceRef::new(
            self.ptr.wrapping_add(self.stride * range.start),
            range.len(),
            self.stride,
            self.type_id,
        )
    }

    /// Create a sub-slice of this slice
    pub fn sub_mut<R>(&mut self, range: R) -> AnySliceMut
    where
        R: RangeBounds<usize>,
    {
        let range = constrain_range(self.len, range);

        Self::new(
            self.ptr.wrapping_add(self.stride * range.start),
            range.len(),
            self.stride,
            self.type_id,
        )
    }

    /// Create a sub-slice of this slice
    pub fn sub_into<R>(self, range: R) -> AnySliceMut<'a>
    where
        R: RangeBounds<usize>,
    {
        let range = constrain_range(self.len, range);

        Self::new(
            self.ptr.wrapping_add(self.stride * range.start),
            range.len(),
            self.stride,
            self.type_id,
        )
    }

    /// How many elements does the slice contain?
    pub fn len(&self) -> usize {
        self.len
    }

    /// Does the slice contain any elements at all?
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Does the slice contain elements of type `T`?
    pub fn contains<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }

    /// The `size_of()` of the original slice elements of type `T`
    pub fn stride(&self) -> usize {
        self.stride
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // All these tests use an (u8, u16) because it has padding

    #[test]
    fn erase_unerase() {
        let mut data = [(1u8, 2u16), (3u8, 4u16)];
        let mut any = AnySliceMut::erase(data.as_mut_slice());

        assert_eq!(any.len(), 2);
        assert!(!any.is_empty());

        // unerase()
        assert_eq!(any.unerase::<u8>(), None);
        assert_eq!(
            any.unerase::<(u8, u16)>(),
            Some([(1u8, 2u16), (3u8, 4u16)].as_slice())
        );

        // unerase_mut()
        assert_eq!(any.unerase_mut::<u8>(), None);
        let unerased = any.unerase_mut::<(u8, u16)>().unwrap();
        unerased.fill((10u8, 10u16));
        assert_eq!(data, [(10u8, 10u16), (10u8, 10u16)]);
    }

    #[test]
    fn sub() {
        let mut data = [
            (0u8, 1u16),
            (2u8, 3u16),
            (4u8, 5u16),
            (6u8, 7u16),
            (8u8, 9u16),
        ];

        let mut any = AnySliceMut::erase(data.as_mut_slice());

        // sub_mut()
        let unerased_sub = any.sub_mut(3..).unerase_into::<(u8, u16)>().unwrap();
        unerased_sub.fill((10u8, 10u16));

        assert_eq!(
            data,
            [
                (0u8, 1u16),
                (2u8, 3u16),
                (4u8, 5u16),
                (10u8, 10u16),
                (10u8, 10u16),
            ]
        );
    }
}
