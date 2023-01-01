use crate::range::constrain_range;
use core::{
    any::TypeId, marker::PhantomData, mem::size_of, ops::RangeBounds, slice::from_raw_parts,
};

/// A type-erased immutable slice
///
/// # Example
///
/// ```
/// let data : [i32; 3] = [0, 1, 2];
/// let any = sashay::AnySliceRef::erase(data.as_slice());
/// let slice = any.unerase::<i32>().expect("any was not a &[i32]");
///
/// assert_eq!(slice, data.as_slice());
#[derive(Debug, Clone, Copy)]
pub struct AnySliceRef<'a> {
    /// A pointer to the first element in the slice
    /// Must be aligned
    ptr: *const u8,

    /// The number of elements in the slice
    len: usize,

    /// The byte size/stride of the original element type
    stride: usize,

    /// The TypeId of the elements in the slice
    /// This is used to ensure we can safely cast back to typed slices
    type_id: TypeId,

    /// Phantom data to ensure that we stick to the correct lifetime
    _phantom: PhantomData<&'a ()>,
}

impl<'a> AnySliceRef<'a> {
    /// Construct an erased slice from its raw parts
    ///
    /// # Safety
    ///
    /// Calling this is only defined behaviour if:
    ///  - All safety rules for `core::slice::from_raw_parts()` hold
    ///  - `stride` is the correct `size_of()` for the element `T`
    ///  - `type_id` is the correct `TypeId` for the element `T`
    pub const unsafe fn from_raw_parts(
        ptr: *const (),
        len: usize,
        stride: usize,
        type_id: TypeId,
    ) -> Self {
        Self {
            ptr: ptr.cast::<u8>(),
            len,
            stride,
            type_id,
            _phantom: PhantomData,
        }
    }

    /// Erase the type of a slice's elements
    pub fn erase<T: 'static>(slice: &'a [T]) -> AnySliceRef<'a> {
        // Safety:
        //  - The raw parts come from a valid slice
        //  - The TypeId and stride come directly from the slice element `T`
        unsafe {
            Self::from_raw_parts(
                slice.as_ptr().cast::<()>(),
                slice.len(),
                size_of::<T>(),
                TypeId::of::<T>(),
            )
        }
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
            unsafe { from_raw_parts(self.ptr.cast::<T>(), self.len) }
        })
    }

    /// Unerase the type back into an immutable slice
    ///
    /// If the the erased slice ref was created with T, you get the original
    /// slice back. For any other T, this function returns None
    pub fn unerase_into<T: 'static>(self) -> Option<&'a [T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts(self.ptr.cast::<T>(), self.len) }
        })
    }

    /// Create a sub-slice of this slice
    pub fn slice<R>(&self, range: R) -> AnySliceRef
    where
        R: RangeBounds<usize>,
    {
        let range = constrain_range(self.len, range);

        // Safety:
        // - The `ptr` is increased in steps of `stride`, so points to a valid and aligned `T`
        // - `constrain_range()` ensures that the ptr offset and len fall within the original slice range
        // - `type_id` and `stride` were already valid, and they haven't changed
        unsafe {
            Self::from_raw_parts(
                self.ptr
                    .wrapping_add(self.stride * range.start)
                    .cast::<()>(),
                range.len(),
                self.stride,
                self.type_id,
            )
        }
    }

    /// Create a sub-slice of this slice
    pub fn slice_into<R>(self, range: R) -> AnySliceRef<'a>
    where
        R: RangeBounds<usize>,
    {
        let range = constrain_range(self.len, range);

        // Safety:
        // - The `ptr` is increased in steps of `stride`, so points to a valid and aligned `T`
        // - `constrain_range()` ensures that the ptr offset and len fall within the original slice range
        // - `type_id` and `stride` were already valid, and they haven't changed
        unsafe {
            Self::from_raw_parts(
                self.ptr
                    .wrapping_add(self.stride * range.start)
                    .cast::<()>(),
                range.len(),
                self.stride,
                self.type_id,
            )
        }
    }

    // Retrieve an unsafe pointer to the raw slice data
    pub const fn as_ptr(&self) -> *const () {
        self.ptr.cast::<()>()
    }

    /// How many elements does the slice contain?
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Does the slice contain any elements at all?
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Does the slice contain elements of type `T`?
    pub fn contains<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }

    /// The `size_of()` of the original slice elements of type `T`
    pub const fn stride(&self) -> usize {
        self.stride
    }
}

impl<'a, T: 'static> From<&'a [T]> for AnySliceRef<'a> {
    fn from(slice: &'a [T]) -> Self {
        Self::erase(slice)
    }
}

impl<'a, T: 'static> From<&'a mut [T]> for AnySliceRef<'a> {
    fn from(slice: &'a mut [T]) -> Self {
        Self::erase(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // All these tests use an (u8, u16) because it has padding

    #[test]
    fn erase_unerase() {
        let data = [(1u8, 2u16), (3u8, 4u16)];
        let any = AnySliceRef::erase(data.as_slice());

        assert_eq!(any.len(), 2);
        assert!(!any.is_empty());

        // unerase()
        assert_eq!(any.unerase::<u8>(), None);
        assert_eq!(any.unerase::<(u8, u16)>(), Some(data.as_slice()));
        assert_eq!(any.unerase_into::<(u8, u16)>(), Some(data.as_slice()));
    }

    #[test]
    fn sub() {
        let data = [
            (0u8, 1u16),
            (2u8, 3u16),
            (4u8, 5u16),
            (6u8, 7u16),
            (8u8, 9u16),
        ];

        let any = AnySliceRef::erase(data.as_slice());

        // sub()
        assert_eq!(any.slice(0..2).unerase::<(u8, u16)>(), Some(&data[0..2])); // Range
        assert_eq!(any.slice(3..7).unerase::<(u8, u16)>(), Some(&data[3..5]));
        assert_eq!(any.slice(..3).unerase::<(u8, u16)>(), Some(&data[..3]));
        assert_eq!(any.slice(7..).unerase::<(u8, u16)>(), Some(&data[5..])); // RangeFrom
        assert_eq!(any.slice(..).unerase::<(u8, u16)>(), Some(&data[..])); // RangeFull
        assert_eq!(any.slice(1..=2).unerase::<(u8, u16)>(), Some(&data[1..=2])); // RangeInclusive
        assert_eq!(any.slice(..4).unerase::<(u8, u16)>(), Some(&data[..4])); // RangeTo
        assert_eq!(any.slice(..=2).unerase::<(u8, u16)>(), Some(&data[..=2])); // RangeToInclusive

        // sub_into()
        assert_eq!(
            any.slice_into(0..2).unerase::<(u8, u16)>(),
            Some(&data[0..2])
        );
    }
}
