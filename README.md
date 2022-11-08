# Sashay

Sashay contains type-erased and life-time erased types that mimic both regular Rust references and slices.

* `&T` -> `AnyRef`
* `&mut T` -> `AnyMut`
* `*const/mut T` -> `AnyPtr`
* `&[T]` -> `AnySliceRef`
* `&mut [T]` -> `AnySliceMut`
* `*const/mut [T]` -> `AnySlicePtr`

Any of these refs and muts be constructed by calling `::erase()` on a reference/slice. The lifetime is still stored on the object, as well as the [`TypeId`](https://doc.rust-lang.org/stable/std/any/struct.TypeId.html), which is used to check if any downcast is valid.

The advantage of the slice types over using regular references to `[T]` is that the `AnySlice*` types retain the slice length without having to downcast.

As far as I know the library is sound and passed `cargo miri test`, but outside of personal use it is untested, not used in production code (yet?) and has not been audited. If you have constructive feedback, that is much appreciated.

## Example

```rust
let data : [i32; 3] = [0, 1, 2];
let any = sashay::AnySliceRef::erase(data.as_slice());
let slice = any.downcast_ref::<i32>().expect("any was not a &[i32]");

assert_eq!(slice, data.as_slice());
```