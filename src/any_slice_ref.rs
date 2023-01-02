use crate::range::constrain_range;
use core::{
    any::TypeId, marker::PhantomData, mem::size_of, ops::RangeBounds, slice::from_raw_parts,
};

/// A type-erased immutable slice.
///
/// A dynamically sized view into contiguous memory, just like regular Rust primitive
/// [slices](https://doc.rust-lang.org/std/primitive.slice.html), except that the type of the
/// individual elements is erased. This allows you to deal with and *store* slices of different
/// element types within the same collection.
///
/// ```
/// // Slices can be erased...
/// let data : [i32; 3] = [0, 1, 2];
/// let any = sashay::AnySliceRef::erase(data.as_slice());
///
/// assert_eq!(any.len(), 3);
/// assert_eq!(any.stride(), std::mem::size_of::<i32>());
///
/// // ...and unerased back to their original slice
/// let slice = any.unerase::<i32>().expect("not a reference to `[i32]`");
///
/// assert_eq!(slice, [0, 1, 2].as_slice());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AnySliceRef<'a> {
    /// A raw pointer to the referenced slice
    ///
    /// Note: this pointer must be aligned and point to valid values of `T` at
    /// subsequent positions along the stride
    ptr: *const u8,

    /// The number of elements in referenced slice
    len: usize,

    /// The stride of the elements in the slice
    ///
    /// This is equal to the `size_of()` of the individual elements in the slice,
    /// such that ptr + N * stride points to subsequent elements
    stride: usize,

    /// A unique id representing the type of the referenced slice elements
    ///
    /// This is used to ensure we can safely unerase back without accidentally transmuting
    type_id: TypeId,

    /// Phantom data to ensure that we stick to the correct lifetime
    _phantom: PhantomData<&'a ()>,
}

impl<'a> AnySliceRef<'a> {
    /// Erase the type of an immutable slice's elements.
    ///
    /// The resulting type retains the lifetime and length of the original slice,
    /// and can even be sub-sliced or indexed like regular slices, but the
    /// individual elements can only be used after unerasing the type.
    ///
    /// ```
    /// let data : [i32; 3] = [0, 1, 2];
    /// let any = sashay::AnySliceRef::erase(data.as_slice());
    ///
    /// assert_eq!(any.len(), data.len());
    /// ```
    pub fn erase<T: 'static>(slice: &'a [T]) -> AnySliceRef<'a> {
        // Safety:
        //  - The raw parts come from a valid slice
        //  - The TypeId and stride are provided by the compiler
        unsafe {
            Self::from_raw_parts(
                slice.as_ptr().cast::<()>(),
                slice.len(),
                size_of::<T>(),
                TypeId::of::<T>(),
            )
        }
    }

    /// Construct an erased slice from its raw parts.
    ///
    /// If you already have a `&[T]`, it is recommended to call [`erase()`](AnySliceRef::erase()).
    ///
    /// This function follows the same API as [`from_raw_parts()`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html)
    /// with some additions. The parameters `ptr` and `len` represent the slice memory, though be
    /// aware that `len` is the number of *elements* in the slice, not the byte count. To represent a
    /// pointer of any type, `*const ()` is used. If you have a `*const T`, you can cast it using
    /// [`ptr::cast()`](https://doc.rust-lang.org/std/primitive.pointer.html#method.cast).
    ///
    /// Moreover, this function also takes `stride` (the [`size_of()`](https://doc.rust-lang.org/std/mem/fn.size_of.html)
    /// or byte count including padding of the individual elements) and a unique `type_id` representing the type
    /// of the elements.
    ///
    /// # Safety
    ///
    /// Calling this is only defined behaviour if:
    ///  - All safety rules for [`from_raw_parts()`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html) hold
    ///  - `stride` is the correct [`size_of()`](https://doc.rust-lang.org/std/mem/fn.size_of.html) for the element type `T` (including padding and such)
    ///  - `type_id` is the correct [`TypeId`](https://doc.rust-lang.org/stable/std/any/struct.TypeId.html) for the element type `T`
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

    /// Unerase back to an immutable slice
    ///
    /// This functions essentially the same as [`Any::downcast_ref()`](https://doc.rust-lang.org/core/any/trait.Any.html#method.downcast_ref). If the
    /// original slice's element type was `T`, a valid slice reference is returned. Otherwise, you get `None`.
    ///
    /// ```
    /// let data : [i32; 3] = [0, 1, 2];
    /// let any = sashay::AnySliceRef::erase(data.as_slice());
    ///
    /// assert!(any.unerase::<bool>().is_none());
    /// assert!(any.unerase::<i32>().is_some());
    /// ```
    pub fn unerase<T: 'static>(&self) -> Option<&[T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts(self.ptr.cast::<T>(), self.len) }
        })
    }

    /// Unerase back into an immutable slice
    ///
    /// This functions essentially the same as [`AnySliceRef::unerase()`],
    /// except that ownership is tranferred into the slice. If the original slice's element type was `T`,
    /// a valid slice reference is returned. Otherwise, you get `None`.
    ///
    /// ```
    /// let data : [i32; 3] = [0, 1, 2];
    /// let any = sashay::AnySliceRef::erase(data.as_slice());
    ///
    /// assert!(any.unerase_into::<i32>().is_some());
    /// ```
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

    /// A unique type id representing the original slice element `T`
    pub const fn type_id(&self) -> &TypeId {
        &self.type_id
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
        assert_eq!(any.type_id(), &TypeId::of::<(u8, u16)>());

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
