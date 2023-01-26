//! # Sashay
//!
//! Sashay contains type-erased slices and references that work _kinda_ like `Any`, but not entirely:
//!
//! * `&'a T` -> `AnyRef<'a>`
//! * `&'a mut T` -> `AnyMut<'a>`
//! * `&'a [T]` -> `AnySliceRef<'a>`
//! * `&'a mut [T]` -> `AnySliceMut<'a>`
//!
//! The big advantage of these types if that you can deal with references and slices of any type without having to resort to generic code. Perhaps more importantly, it allows you to store them in homogeneous containers without having to use trait objects (which is what I originally wrote this for).
//!
//! Any of these refs and muts can be constructed by calling `::erase()` on a reference or slice. The erased types are still lifetime-bound, and they also contains a [`TypeId`](https://doc.rust-lang.org/stable/std/any/struct.TypeId.html) to check if any unerasure is valid. Internally the structures hold pointers to the original data.
//!
//! You could `AnyRef/Mut` to erase `[T]` slices, but `AnySliceRef/Mut` retain part of the expected API for primitive slices, such as calling `.len()` or `.is_empty()` and providing access to subslices or individual elements.
//!
//! As far as I know the library is sound and it passes `cargo miri test`, but outside of personal use it is untested in the wild. I have chatted with people in the [Rust Zulip](https://rust-lang.zulipchat.com/#narrow/stream/122651-general/topic/Type-erased.20slices/near/318265693) (big thanks to Lokathor, Ben Kimock, Mario Carneiro and scottmcm) to cover edge cases. Feedback is always appreciated.
//!
//! And last but not least: don't forget to enjoy your day! ;)
//!
//! ## Example
//!
//! ```rust
//! let data : [i32; 3] = [0, 1, 2];
//!
//! // Type-erase a slice
//! let erased = sashay::AnySliceRef::erase(data.as_slice());
//! assert_eq!(erased.len(), 3);
//!
//! // Unerase the whole slice
//! let unerased = erased.unerase::<i32>().expect("any was not a &[i32]");
//! assert_eq!(unerased, data.as_slice());
//!
//! // Unerase just a single element
//! assert_eq!(erased.get(2).unwrap().unerase::<i32>(), Some(&2));
//! ```
//!
//! ## Dependencies
//!
//! `sashay` is `#![no_std]` and has 0 dependencies.

#![no_std]

mod any_mut;
mod any_ref;
mod any_slice_mut;
mod any_slice_ref;
mod range;

pub use any_mut::AnyMut;
pub use any_ref::AnyRef;
pub use any_slice_mut::AnySliceMut;
pub use any_slice_ref::AnySliceRef;
