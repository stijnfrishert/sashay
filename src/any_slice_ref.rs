use core::{
    any::TypeId,
    marker::PhantomData,
    mem::size_of,
    ops::{Bound, Range, RangeBounds},
    slice::from_raw_parts,
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
    /// Erase the type of a slice's elements
    pub fn erase<T: 'static>(slice: &'a [T]) -> AnySliceRef<'a> {
        Self {
            ptr: slice.as_ptr() as *const u8,
            len: slice.len(),
            stride: size_of::<T>(),
            type_id: TypeId::of::<T>(),
            _phantom: PhantomData,
        }
    }

    /// Unerase the type back to a primitive Rust slice
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

    /// Unerase the type back to a primitive Rust slice
    ///
    /// If the the erased slice ref was created with T, you get the original
    /// slice back. For any other T, this function returns None
    pub fn unerase_into<T: 'static>(self) -> Option<&'a [T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts(self.ptr as *const T, self.len) }
        })
    }

    /// Create a sub-slice of this slice
    pub fn sub<R>(&self, range: R) -> AnySliceRef
    where
        R: RangeBounds<usize>,
    {
        let range = Self::range(self.len, range);

        AnySliceRef {
            ptr: self.ptr.wrapping_add(self.stride * range.start),
            len: range.len(),
            stride: self.stride,
            type_id: self.type_id,
            _phantom: PhantomData,
        }
    }

    /// Create a sub-slice of this slice
    pub fn sub_into<R>(self, range: R) -> AnySliceRef<'a>
    where
        R: RangeBounds<usize>,
    {
        let range = Self::range(self.len, range);

        AnySliceRef {
            ptr: self.ptr.wrapping_add(self.stride * range.start),
            len: range.len(),
            stride: self.stride,
            type_id: self.type_id,
            _phantom: PhantomData,
        }
    }

    fn range<R>(len: usize, range: R) -> Range<usize>
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        }
        .min(len);

        let end = match range.end_bound() {
            Bound::Included(end) => *end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => len,
        }
        .min(len);

        start..end
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
        let data = [(1u8, 2u16), (3u8, 4u16)];
        let any = AnySliceRef::erase(data.as_slice());

        assert_eq!(any.len(), 2);
        assert!(!any.is_empty());

        // unerase
        assert_eq!(any.unerase::<(u8, u16)>(), Some(data.as_slice()));
        assert_eq!(any.unerase::<u8>(), None);
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
        assert_eq!(any.sub(0..2).unerase::<(u8, u16)>(), Some(&data[0..2])); // Range
        assert_eq!(any.sub(3..7).unerase::<(u8, u16)>(), Some(&data[3..5]));
        assert_eq!(any.sub(..3).unerase::<(u8, u16)>(), Some(&data[..3]));
        assert_eq!(any.sub(7..).unerase::<(u8, u16)>(), Some(&data[5..])); // RangeFrom
        assert_eq!(any.sub(..).unerase::<(u8, u16)>(), Some(&data[..])); // RangeFull
        assert_eq!(any.sub(1..=2).unerase::<(u8, u16)>(), Some(&data[1..=2])); // RangeInclusive
        assert_eq!(any.sub(..4).unerase::<(u8, u16)>(), Some(&data[..4])); // RangeTo
        assert_eq!(any.sub(..=2).unerase::<(u8, u16)>(), Some(&data[..=2])); // RangeToInclusive

        // sub_into()
        assert_eq!(any.sub_into(0..2).unerase::<(u8, u16)>(), Some(&data[0..2]));
    }
}
