`RcVec` based on `Rc` and can be converted from Rc without allocation,
just like `Box` is converted to `Vec`

Due to `Rc`'s API, this implementation cannot use `realloc`, resulting in some performance issues

Similar to `Vec::into_boxed_slice`,
`RcVec::into_uniq_slice` can be converted to `UniqRc`,
which is the packaging of `Rc` and behaves similarly to `Box`

# Examples
```rust
use rc_vec::RcVec;
use std::rc::Rc;

let rc: Rc<[i32]> = Rc::new([1, 2, 3]);
let rcptr = Rc::as_ptr(&rc).cast();

let mut vec = RcVec::from(rc);

assert_eq!(vec.len(), 3);
assert_eq!(vec.capacity(), 3);
assert!(std::ptr::eq(rcptr, vec.as_ptr()));

vec.push(4);

assert_eq!(vec.len(), 4);
assert!(vec.capacity() > 3);
assert!(! std::ptr::eq(rcptr, vec.as_ptr()));

assert_eq!(vec, [1, 2, 3, 4]);
```

# Safety
- miri passed

**I have not checked any synchronization related issues with the Arc variant**
