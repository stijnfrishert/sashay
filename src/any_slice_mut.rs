use crate::{range::constrain_range, AnyMut, AnyRef, AnySliceRef};
use core::{
    any::TypeId,
    marker::PhantomData,
    mem::size_of,
    ops::RangeBounds,
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// A type-erased mutable slice.
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
/// assert_eq!(any.stride(), std::mem::size_of::<i32>());
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
    ///
    /// The resulting type retains the lifetime and length of the original slice,
    /// and can even be sub-sliced or indexed like regular slices, but the
    /// individual elements can only be used after unerasing the type.
    ///
    /// ```
    /// let mut data : [i32; 3] = [0, 1, 2];
    /// let any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///
    /// assert_eq!(any.len(), data.len());
    /// ```
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
    /// If you already have a `&mut [T]`, it is recommended to call [`AnySliceMut::erase()`].
    ///
    /// This function follows the same API as [`slice::from_raw_parts_mut()`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts_mut.html)
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

    /// Unerase back to an immutable slice.
    ///
    /// This behaves essentially the same as [`Any::downcast_ref()`](https://doc.rust-lang.org/core/any/trait.Any.html#method.downcast_ref). If the
    /// original slice's element type was `T`, a valid slice reference is returned. Otherwise, you get `None`.
    ///
    /// Note that while `AnySliceMut` represents a *mutable* slice, this function unerases it to an *immutable* one.
    /// If you need a mutable slice, use [`AnySliceMut::unerase_mut()`] or [`AnySliceMut::unerase_into()`]
    ///
    /// ```
    /// let mut data : [i32; 3] = [0, 1, 2];
    /// let any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///
    /// // You can unerase multiple times, because this is a shared, immutable reference
    /// let unerased_a = any.unerase::<i32>().unwrap();
    /// let unerased_b = any.unerase::<i32>().unwrap();
    /// assert_eq!(unerased_a, unerased_b);
    ///
    /// // Doesn't compile, because you can't mutate
    /// // unerased_a.fill(0);
    ///
    /// // Unerasing to a different type gives you nothing
    /// assert!(any.unerase::<bool>().is_none());
    /// ```
    pub fn unerase<T: 'static>(&self) -> Option<&[T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts(self.ptr.cast::<T>().cast_const(), self.len) }
        })
    }

    /// Unerase back to a mutable slice.
    ///
    /// This behaves essentially the same as [`Any::downcast_mut()`](https://doc.rust-lang.org/core/any/trait.Any.html#method.downcast_mut). If the
    /// original slice's element type was `T`, a valid slice reference is returned. Otherwise, you get `None`.
    ///
    /// Note that this function unerases to a _mutable_ slice. If you only need an immutable one, you
    /// can use [`AnySliceMut::unerase()`]
    ///
    /// ```
    /// let mut data : [i32; 3] = [0, 1, 2];
    /// let mut any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///
    /// // You can unerase back to a mutable slice
    /// let unerased = any.unerase_mut::<i32>().unwrap();
    /// unerased.fill(0);
    /// assert_eq!(data, [0, 0, 0]);
    ///
    /// // You can't unerase_mut twice, because this is a _unique_, mutable reference
    /// // any.unerase_mut::<i32>();
    /// ```
    pub fn unerase_mut<T: 'static>(&mut self) -> Option<&mut [T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts_mut(self.ptr.cast::<T>(), self.len) }
        })
    }

    /// Unerase back into a mutable slice.
    ///
    /// This behaves essentially the same as [`AnySliceMut::unerase_mut()`],
    /// except that ownership is tranferred into the slice. If the original slice's element type was `T`,
    /// a valid slice reference is returned. Otherwise, you get `None`.
    ///
    /// ```
    /// let mut data : [i32; 3] = [0, 1, 2];
    ///
    /// let unerased = {
    ///     // Unerase, transferring ownership into the resulting slice
    ///     let any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///     let unerased = any.unerase_into::<i32>().unwrap();
    ///
    ///     // Can't unerase anymore after this, ownerhip has been moved out of the any
    ///     // any.unerase_into::<i32>();
    ///
    ///     // Because unerase_into() transfers ownership, the resulting slice's lifetime
    ///     // can escape the any's lifetime scope and just reference the original data
    ///     unerased
    /// };
    ///
    /// unerased.fill(0);
    /// assert_eq!(data, [0, 0, 0]);
    /// ```
    pub fn unerase_into<T: 'static>(self) -> Option<&'a mut [T]> {
        self.contains::<T>().then(|| {
            // SAFETY:
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, so it's not null and aligned
            unsafe { from_raw_parts_mut(self.ptr.cast::<T>(), self.len) }
        })
    }

    /// Borrow this mutable slice as an immutable one.
    ///
    /// Even though you have mutable and unique access to a slice, this fuction lets you
    /// trade in the mutability for shared access.
    ///
    /// ```
    /// let mut data : [i32; 3] = [7, 6, 5];
    /// let any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///
    /// // borrow() can be called multiple times, because immutable references provide shared access
    /// let immutable_a = any.borrow();
    /// let immutable_b = any.borrow();
    ///
    /// assert_eq!(immutable_a.len(), 3);
    /// assert_eq!(immutable_b.len(), 3);
    /// ```
    pub fn borrow(&self) -> AnySliceRef {
        // SAFETY:
        // All parts are valid, we just cast to const
        // This is ok, because we have an immutable ref to self
        unsafe {
            AnySliceRef::from_raw_parts(
                self.ptr.cast_const().cast::<()>(),
                self.len,
                self.stride,
                self.type_id,
            )
        }
    }

    /// Borrow this mutable slice as a mutable slice with lifetime 'self
    ///
    /// This operation might seem redundant, but just like regular slices coming out
    /// of methods have the lifetime or `self`, you can use `borrow_mut()` to do the same.
    /// The resulting `AnySliceMut` has its lifetime tied to `self`.
    ///
    /// ```
    /// struct Container<'a> {
    ///     any: sashay::AnySliceMut<'a>,
    /// }
    ///
    /// impl<'a> Container<'a> {
    ///     fn get_ref<'b>(&'b mut self) -> sashay::AnySliceMut<'b> {
    ///         self.any.borrow_mut()
    ///     }
    /// }
    /// ```
    pub fn borrow_mut(&mut self) -> AnySliceMut {
        unsafe {
            AnySliceMut::from_raw_parts(self.ptr.cast::<()>(), self.len, self.stride, self.type_id)
        }
    }

    /// Retrieve an immutable reference to one of the elements in the slice.
    ///
    /// ```
    /// let mut data : [i32; 3] = [0, 1, 2];
    /// let any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///
    /// assert_eq!(any.get(1).unwrap().unerase_into::<i32>(), Some(&1));
    /// ```
    pub fn get(&self, index: usize) -> Option<AnyRef> {
        if index < self.len {
            // SAFETY:
            // - The index is within the slice length, so we don't go out of bounds
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, and we're jumping from it using a valid stride
            let reference = unsafe {
                AnyRef::from_raw_parts(
                    self.ptr
                        .wrapping_add(index * self.stride)
                        .cast::<()>()
                        .cast_const(),
                    self.type_id,
                )
            };

            Some(reference)
        } else {
            None
        }
    }

    /// Retrieve a mutable reference to one of the elements in the slice.
    ///
    /// ```
    /// let mut data : [i32; 3] = [0, 1, 2];
    /// let mut any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///
    /// let reference = any.get_mut(1).unwrap().unerase_into::<i32>().unwrap();
    /// *reference = 4;
    ///
    /// assert_eq!(data, [0, 4, 2]);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<AnyMut> {
        if index < self.len {
            // SAFETY:
            // - The index is within the slice length, so we don't go out of bounds
            // - We've checked the TypeId of T against the one created at construction, so we're not
            //   accidentally transmuting to a different type
            // - The pointer came directly out of a valid slice, and we're jumping from it using a valid stride
            let reference = unsafe {
                AnyMut::from_raw_parts(
                    self.ptr.wrapping_add(index * self.stride).cast::<()>(),
                    self.type_id,
                )
            };

            Some(reference)
        } else {
            None
        }
    }

    /// Access an immutable subslice within a given range.
    ///
    /// Just like calling slice[0..10] on a regular primitive slice, you can also take a subslice
    /// of type-erased slices. Because of a limitation in the [`Index`](core::ops::Index) trait we can't use
    /// the same syntax, but `subslice()` behaves the same.
    ///
    /// Note that this function subslices _immutably_. If you need a mutable subslice, you
    /// can use [`AnySliceMut::subslice_mut()`]
    ///
    /// ```
    /// let mut data : [i32; 5] = [0, 1, 2, 3, 4];
    /// let any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///
    /// // Take a subslice
    /// let sub = any.subslice(1..4);
    ///
    /// assert_eq!(sub.len(), 3);
    /// assert_eq!(sub.unerase::<i32>().unwrap(), [1, 2, 3].as_slice());
    /// ```
    pub fn subslice<R>(&self, range: R) -> AnySliceRef
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

    /// Access a mutable subslice within a given range.
    ///
    /// Just like calling slice[0..10] on a regular primitive slice, you can also take a subslice
    /// of type-erased slices. Because of a limitation in the [`IndexMut`](core::ops::IndexMut) trait we can't use
    /// the same syntax, but `subslice_mut()` behaves the same.
    ///
    /// Note that this function subslices _mutably_. If you only need an immutable subslice, you
    /// can use [`AnySliceMut::subslice()`]
    ///
    /// ```
    /// let mut data : [i32; 5] = [0, 1, 2, 3, 4];
    /// let mut any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///
    /// // Take a subslice
    /// let mut sub = any.subslice_mut(1..4);
    /// assert_eq!(sub.len(), 3);
    ///
    /// // Mutate it
    /// sub.unerase_mut::<i32>().unwrap().fill(8);
    ///
    /// assert_eq!(data, [0, 8, 8, 8, 4]);
    /// ```
    pub fn subslice_mut<R>(&mut self, range: R) -> AnySliceMut
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

    /// Access a subslice within a given range.
    ///
    /// Just like calling slice[0..10] on a regular primitive slice, you can also take a subslice
    /// of type-erased slices. Note that this function transfers ownership into the newly type-erased slice.
    /// If you do not want that, use [`AnySliceMut::subslice()`] or [`AnySliceMut::subslice_mut()`].
    ///
    /// ```
    /// let mut data : [i32; 5] = [0, 1, 2, 3, 4];
    ///
    /// let sub = {
    ///     // Take a subslice
    ///     let any = sashay::AnySliceMut::erase(data.as_mut_slice());
    ///     let sub = any.subslice_into(1..4);
    ///
    ///     // Because subslice_into() transfers ownership, the resulting subslice's lifetime
    ///     // can escape the original slice's lifetime scope and just reference the original data
    ///     sub
    /// };
    ///
    /// assert_eq!(sub.len(), 3);
    /// assert_eq!(sub.unerase::<i32>().unwrap(), [1, 2, 3].as_slice());
    /// ```
    pub fn subslice_into<R>(self, range: R) -> AnySliceMut<'a>
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

    /// Retrieve an unsafe immutable pointer to the raw slice data.
    pub const fn as_ptr(&self) -> *const () {
        self.ptr.cast::<()>().cast_const()
    }

    /// Retrieve an unsafe mutable pointer to the raw slice data.
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

    /// Was the original slice element of type `T`?
    pub fn contains<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }

    /// The `size_of()` of the original slice elements of type `T`.
    pub const fn stride(&self) -> usize {
        self.stride
    }

    /// A unique type id representing the original slice element `T`.
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
        let unerased_sub = any.subslice_mut(3..).unerase_into::<(u8, u16)>().unwrap();
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
