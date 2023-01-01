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
/// A dynamically sized mutable view into contiguous memory, just like regular Rust primitive
/// [slices](https://doc.rust-lang.org/std/primitive.slice.html), except that the type of the
/// individual elements is erased. This allows you to deal with and *store* slices of different
/// element types within the same collection.
///
/// ```
/// // Slices can be erased...
/// let mut data : [i32; 3] = [0, 1, 2];
/// let mut any = sashay::AnySliceMut::erase(data.as_mut_slice());
///
/// assert_eq!(any.len(), 3);
/// assert_eq!(any.stride(), core::mem::size_of::<i32>());
///
/// // ...and unerased back to their original slice
/// let slice = any.unerase_mut::<i32>().expect("not a reference to `[i32]`");
/// slice.fill(0);
///
/// assert_eq!(data, [0, 0, 0]);
/// ```
#[derive(Debug)]
pub struct AnySliceMut<'a> {
    /// A raw pointer to the referenced slice
    ///
    /// Note: this pointer must be aligned and point to valid values of `T` at
    /// subsequent positions along the stride
    ptr: *mut u8,

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
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> AnySliceMut<'a> {
    /// Erase the type of a mutable slice's elements.
    pub fn erase<T: 'static>(slice: &'a mut [T]) -> AnySliceMut<'a> {
        // Safety:
        //  - The raw parts come from a valid slice
        //  - The TypeId and stride are provided by the compiler
        unsafe {
            Self::from_raw_parts(
                slice.as_mut_ptr().cast::<()>(),
                slice.len(),
                size_of::<T>(),
                TypeId::of::<T>(),
            )
        }
    }

    /// Construct an erased slice from its raw parts.
    ///
    /// If you already have a `&mut [T]`, it is recommended to call [`erase()`](AnySliceRef::erase()).
    ///
    /// This function follows the same API as [`from_raw_parts_mut()`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts_mut.html)
    /// with some additions. The parameters `ptr` and `len` represent the slice memory, though be
    /// aware that `len` is the number of *elements* in the slice, not the byte count. To represent a
    /// pointer of any type, `*mut ()` is used. If you have a `*mut T`, you can cast it using
    /// [`ptr::cast()`](https://doc.rust-lang.org/std/primitive.pointer.html#method.cast).
    ///
    /// Moreover, this function also takes `stride` (the [`size_of()`](https://doc.rust-lang.org/std/mem/fn.size_of.html)
    /// or byte count including padding of the individual elements) and a unique `type_id` representing the type
    /// of the elements.
    ///
    /// # Safety
    ///
    /// Calling this is only defined behaviour if:
    ///  - All safety rules for [`from_raw_parts_mut()`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts_mut.html) hold
    ///  - `stride` is the correct [`size_of()`](https://doc.rust-lang.org/std/mem/fn.size_of.html) for the element type `T` (including padding and such)
    ///  - `type_id` is the correct [`TypeId`](https://doc.rust-lang.org/stable/std/any/struct.TypeId.html) for the element type `T`
    pub unsafe fn from_raw_parts(ptr: *mut (), len: usize, stride: usize, type_id: TypeId) -> Self {
        Self {
            ptr: ptr.cast::<u8>(),
            len,
            stride,
            type_id,
            _phantom: PhantomData,
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
            unsafe { from_raw_parts(self.ptr.cast::<T>().cast_const(), self.len) }
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
            unsafe { from_raw_parts_mut(self.ptr.cast::<T>(), self.len) }
        })
    }

    /// Unerase the type back into a mutable slice
    ///
    /// If the the erased slice ref was created with T, you get the original
    /// slice back. For any other T, this function returns None
    pub fn unerase_into<T: 'static>(self) -> Option<&'a mut [T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts_mut(self.ptr.cast::<T>(), self.len) }
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
            AnySliceRef::from_raw_parts(
                self.ptr
                    .wrapping_add(self.stride * range.start)
                    .cast::<()>()
                    .cast_const(),
                range.len(),
                self.stride,
                self.type_id,
            )
        }
    }

    /// Create a sub-slice of this slice
    pub fn slice_mut<R>(&mut self, range: R) -> AnySliceMut
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
    pub fn slice_into<R>(self, range: R) -> AnySliceMut<'a>
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

    // Retrieve an unsafe immutable pointer to the raw slice data
    pub const fn as_ptr(&self) -> *const () {
        self.ptr.cast::<()>().cast_const()
    }

    // Retrieve an unsafe mutable pointer to the raw slice data
    pub fn as_mut_ptr(&mut self) -> *mut () {
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

impl<'a, T: 'static> From<&'a mut [T]> for AnySliceMut<'a> {
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
        let mut data = [(1u8, 2u16), (3u8, 4u16)];
        let mut any = AnySliceMut::erase(data.as_mut_slice());

        assert_eq!(any.len(), 2);
        assert!(!any.is_empty());
        assert_eq!(any.type_id(), &TypeId::of::<(u8, u16)>());

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
        let unerased_sub = any.slice_mut(3..).unerase_into::<(u8, u16)>().unwrap();
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
