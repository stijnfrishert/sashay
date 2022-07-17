# sashay

 Type-erased slices that mimic `&[T]`, `&mut [T]` and `*const/mut [T]`

 # Example

 ```rust
 let data : [i32; 3] = [0, 1, 2];
 let any = sashay::AnySliceRef::erase(data.as_slice());
 let slice = any.downcast_ref::<i32>().expect("any was not a &[i32]");

 assert_eq!(slice, data.as_slice());
 ```