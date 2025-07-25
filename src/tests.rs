#![allow(unused_imports)]

extern crate std;

use core::panic::AssertUnwindSafe;
use std::panic::catch_unwind;

use alloc::{borrow::ToOwned, boxed::Box, rc::Rc, string::String};

use crate::rc_vec;

use super::rc_vec::*;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Zst;
impl Drop for Zst {
    fn drop(&mut self) {
        ()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Boom(i8);

impl Drop for Boom {
    #[track_caller]
    fn drop(&mut self) {
        if self.0 == 0 {
            panic!("Boom by zero")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Atom(i8);

impl Clone for Atom {
    #[track_caller]
    fn clone(&self) -> Self {
        if self.0 == 0 {
            panic!("Atom by zero")
        }
        Atom(self.0)
    }
}

#[test]
fn push() {
    let mut arr = RcVec::new();

    assert_eq!(arr.len(), 0);
    assert_eq!(arr.capacity(), 0);

    arr.push("a".to_owned());

    assert_eq!(arr.len(), 1);
    assert!(arr.capacity() != 0);

    arr.push("b".to_owned());

    assert_eq!(arr.len(), 2);
    assert!(arr.capacity() != 0);

    assert_eq!(*arr, ["a".to_owned(), "b".to_owned()]);
}

#[test]
fn clone() {
    let vec = rc_vec!["a".to_owned(), "b".to_owned()];
    let cloned = vec.clone();
    assert_eq!(vec, cloned);
}

#[test]
#[should_panic = "Atom by zero"]
fn clone_unwind() {
    let vec = rc_vec![Atom(3), Atom(2), Atom(1), Atom(0), Atom(-1)];
    let _cloned = vec.clone();
    unreachable!();
}

#[test]
fn push_zst() {
    let mut arr = RcVec::new();

    assert_eq!(arr.len(), 0);
    assert_eq!(arr.capacity(), usize::MAX);

    arr.push(Zst);

    assert_eq!(arr.len(), 1);
    assert_eq!(arr.capacity(), usize::MAX);

    arr.push(Zst);

    assert_eq!(arr.len(), 2);
    assert_eq!(arr.capacity(), usize::MAX);

    assert_eq!(*arr, [Zst, Zst]);

    assert!(arr.into_raw_uniq_slice_optional().is_none());
}

#[test]
fn pop() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());

    assert_eq!(*arr, ["a".to_owned(), "b".to_owned()]);

    assert_eq!(arr.pop(), Some("b".to_owned()));
    assert_eq!(arr.pop(), Some("a".to_owned()));
    assert_eq!(arr.pop(), None);

    assert_eq!(arr.len(), 0);
    assert_ne!(arr.capacity(), 0);
}

#[test]
fn remove() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());
    arr.push("c".to_owned());

    assert_eq!(*arr, ["a".to_owned(), "b".to_owned(), "c".to_owned()]);
    assert_eq!(arr.remove(1), "b".to_owned());
    assert_eq!(*arr, ["a".to_owned(), "c".to_owned()]);
    assert_eq!(arr.remove(1), "c".to_owned());
    assert_eq!(*arr, ["a".to_owned()]);
    assert_eq!(arr.remove(0), "a".to_owned());
}

#[test]
fn insert() {
    let mut arr = RcVec::new();

    arr.insert(0, "a".to_owned());
    arr.insert(0, "b".to_owned());
    arr.insert(0, "c".to_owned());

    assert_eq!(*arr, ["c".to_owned(), "b".to_owned(), "a".to_owned()]);

    arr.insert(1, "d".to_owned());
    assert_eq!(*arr, ["c".to_owned(), "d".to_owned(), "b".to_owned(), "a".to_owned()]);

    arr.insert(4, "e".to_owned());
    assert_eq!(*arr, ["c".to_owned(), "d".to_owned(), "b".to_owned(), "a".to_owned(), "e".to_owned()]);
}

#[test]
fn remove1() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());
    arr.push("c".to_owned());

    assert_eq!(*arr, ["a".to_owned(), "b".to_owned(), "c".to_owned()]);
    assert_eq!(arr.remove(0), "a".to_owned());
    assert_eq!(*arr, ["b".to_owned(), "c".to_owned()]);
    assert_eq!(arr.remove(0), "b".to_owned());
    assert_eq!(*arr, ["c".to_owned()]);
    assert_eq!(arr.remove(0), "c".to_owned());
}

#[test]
fn remove_zst() {
    let mut arr = RcVec::new();

    arr.push(Zst);
    arr.push(Zst);
    arr.push(Zst);

    assert_eq!(*arr, [Zst, Zst, Zst]);
    assert_eq!(arr.remove(0), Zst);
    assert_eq!(*arr, [Zst, Zst]);
    assert_eq!(arr.remove(0), Zst);
    assert_eq!(*arr, [Zst]);
    assert_eq!(arr.remove(0), Zst);
}

#[test]
fn swap_remove() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());
    arr.push("c".to_owned());

    assert_eq!(*arr, ["a".to_owned(), "b".to_owned(), "c".to_owned()]);
    assert_eq!(arr.swap_remove(0), "a".to_owned());
    assert_eq!(*arr, ["c".to_owned(), "b".to_owned()]);
    assert_eq!(arr.swap_remove(1), "b".to_owned());
    assert_eq!(*arr, ["c".to_owned()]);
    assert_eq!(arr.swap_remove(0), "c".to_owned());
    assert_eq!(arr.len(), 0);
}

#[test]
fn clear() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());
    arr.push("c".to_owned());

    assert_eq!(*arr, ["a".to_owned(), "b".to_owned(), "c".to_owned()]);
    arr.clear();
    assert_eq!(arr.len(), 0);
    assert_ne!(arr.capacity(), 0);
}

#[test]
fn truncate() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());
    arr.push("c".to_owned());

    assert_eq!(*arr, ["a".to_owned(), "b".to_owned(), "c".to_owned()]);
    arr.truncate(4);
    assert_eq!(*arr, ["a".to_owned(), "b".to_owned(), "c".to_owned()]);
    arr.truncate(3);
    assert_eq!(*arr, ["a".to_owned(), "b".to_owned(), "c".to_owned()]);
    arr.truncate(2);
    assert_eq!(*arr, ["a".to_owned(), "b".to_owned()]);
}

#[ignore = "miri leak"]
#[test]
fn leak() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());
    arr.push("c".to_owned());

    let leaked = arr.leak();
    assert_eq!(*leaked, ["a".to_owned(), "b".to_owned(), "c".to_owned()]);
}

#[test]
fn swap_remove_zst() {
    let mut arr = RcVec::new();

    arr.push(Zst);
    arr.push(Zst);
    arr.push(Zst);

    assert_eq!(*arr, [Zst, Zst, Zst]);
    assert_eq!(arr.swap_remove(1), Zst);
    assert_eq!(*arr, [Zst, Zst]);
}

#[test]
fn from_rcvec_to_rc() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());

    let rc = arr.into_rc_slice();
    assert_eq!(rc.len(), 2);
    assert_eq!(rc[0], "a");
    assert_eq!(rc[1], "b");
}

#[test]
fn from_rcvec_to_box() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());

    let boxed: Box<[String]> = Box::from(arr);
    assert_eq!(boxed.len(), 2);
    assert_eq!(boxed[0], "a");
    assert_eq!(boxed[1], "b");
}

#[test]
fn from_str_to_byte_rcvec() {
    let s = "foobar";
    let rcvec = RcVec::from(s);
    assert_eq!(rcvec[..], *"foobar".as_bytes());
}

#[test]
fn into_iter_ref() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());

    let mut iter = arr.iter();
    assert_eq!(iter.next(), Some(&"a".to_owned()));
    assert_eq!(iter.next(), Some(&"b".to_owned()));
    assert_eq!(iter.next(), None);
}

#[test]
fn into_iter_ref_mut() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());

    let mut iter = arr.iter_mut();
    assert_eq!(iter.next(), Some(&mut "a".to_owned()));
    assert_eq!(iter.next(), Some(&mut "b".to_owned()));
    assert_eq!(iter.next(), None);
}

#[test]
fn into_iter_ref_mut_and_assign() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());

    let mut iter = arr.iter_mut();
    assert_eq!(iter.next(), Some(&mut "a".to_owned()));
    assert_eq!(iter.next(), Some(&mut "b".to_owned()));
    assert_eq!(iter.next(), None);

    assert_eq!(arr, ["a".to_owned(), "b".to_owned()]);

    let mut iter = arr.iter_mut();
    *iter.next().unwrap() = "c".to_owned();

    assert_eq!(arr, ["c".to_owned(), "b".to_owned()]);
}

#[test]
fn into_iter_owned() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());

    let mut iter = arr.into_iter();
    assert_eq!(iter.next(), Some("a".to_owned()));
    assert_eq!(iter.next(), Some("b".to_owned()));
    assert_eq!(iter.next(), None);
}

#[test]
fn into_iter_owned_no_consume() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());

    let _iter = arr.into_iter();
}

#[test]
fn into_iter_owned_no_consume_zst() {
    let mut arr = RcVec::new();

    arr.push(Zst);
    arr.push(Zst);

    let _iter = arr.into_iter();
}

#[test]
fn into_iter_owned_no_consume_all() {
    let mut arr = RcVec::new();

    arr.push("a".to_owned());
    arr.push("b".to_owned());

    let mut iter = arr.into_iter();
    assert_eq!(iter.next(), Some("a".to_owned()));
}

#[test]
fn into_iter_owned_zst() {
    let mut arr = RcVec::new();

    arr.push(Zst);
    arr.push(Zst);

    let mut iter = arr.into_iter();
    assert_eq!(iter.next(), Some(Zst));
}

#[test]
fn try_into_array() {
    let mut vec = RcVec::new();

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    let vec = <[_; 3]>::try_from(vec).unwrap_err();
    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);
    let arr = <[_; 2]>::try_from(vec).unwrap();
    assert_eq!(arr, ["a".to_owned(), "b".to_owned()]);
}

#[test]
fn try_into_rc_array() {
    let mut vec = RcVec::new();

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    let vec = Rc::<[_; 3]>::try_from(vec).unwrap_err();
    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);
    let arr = Rc::<[_; 2]>::try_from(vec).unwrap();
    assert_eq!(arr, Rc::new(["a".to_owned(), "b".to_owned()]));
}

#[cfg(feature = "std")]
#[test]
fn write() {
    extern crate std;
    use std::io::Write;
    let mut vec = RcVec::new();

    write!(vec, "hello").unwrap();

    assert_eq!(vec, b"hello");
}

#[test]
fn shrink_to_fit() {
    let mut vec = RcVec::with_capacity(68);
    assert_eq!(vec.capacity(), 68);

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec.capacity(), 68);
    vec.shrink_to_fit();
    assert_eq!(vec.capacity(), 2);
}

#[test]
fn shrink_to() {
    let mut vec = RcVec::with_capacity(68);
    assert_eq!(vec.capacity(), 68);

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec.capacity(), 68);
    vec.shrink_to(24);
    assert_eq!(vec.capacity(), 24);
    vec.shrink_to(0);
    assert_eq!(vec.capacity(), 2);
}

#[test]
fn extend_from_slice() {
    let mut vec = RcVec::new();

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);

    vec.extend_from_slice(&["c".to_owned(), "d".to_owned()]);

    assert_eq!(vec, ["a".to_owned(), "b".to_owned(), "c".to_owned(), "d".to_owned()]);
}

#[test]
fn extend_from_zst_slice() {
    let mut vec = RcVec::new();

    vec.push(Zst);
    vec.push(Zst);

    assert_eq!(vec, [Zst, Zst]);

    vec.extend_from_slice(&[Zst, Zst]);

    assert_eq!(vec, [Zst, Zst, Zst, Zst]);
    assert_ne!(vec, [Zst, Zst, Zst]);
}

#[test]
fn extend_from_within() {
    let mut vec = RcVec::new();

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);

    vec.extend_from_within(..1);

    assert_eq!(vec, ["a".to_owned(), "b".to_owned(), "a".to_owned()]);
}

#[test]
fn extend_from_within1() {
    let mut vec = RcVec::new();

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);

    vec.extend_from_within(..2);

    assert_eq!(vec, ["a".to_owned(), "b".to_owned(), "a".to_owned(), "b".to_owned()]);
}

#[test]
fn extend_from_within2() {
    let mut vec = RcVec::new();

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);

    vec.extend_from_within(..0);

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);
}

#[test]
fn into_rc_slice() {
    let mut vec = RcVec::with_capacity(2);

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);
    assert_eq!(vec.capacity(), 2);

    let rc = vec.into_rc_slice();
    assert_eq!(*rc, ["a".to_owned(), "b".to_owned()]);
}

#[test]
fn into_rc_slice1() {
    let mut vec = RcVec::with_capacity(4);

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);
    assert_eq!(vec.capacity(), 4);

    let rc = vec.into_rc_slice();
    assert_eq!(*rc, ["a".to_owned(), "b".to_owned()]);
}

#[test]
fn into_raw_uniq_slice() {
    let mut vec = RcVec::with_capacity(2);

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);
    assert_eq!(vec.capacity(), 2);

    let raw = vec.into_raw_uniq_slice();
    assert_eq!(raw.len(), 2);
    assert_eq!(unsafe { raw[0].assume_init_ref() }, &"a".to_owned());
    assert_eq!(unsafe { raw[1].assume_init_ref() }, &"b".to_owned());

    unsafe {
        raw[0].assume_init_read();
        raw[1].assume_init_read();
    }
}

#[test]
fn into_raw_uniq_slice_spare() {
    let mut vec = RcVec::with_capacity(3);

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.capacity(), 3);

    let raw = vec.into_raw_uniq_slice();
    assert_eq!(raw.len(), 3);
    assert_eq!(unsafe { raw[0].assume_init_ref() }, &"a".to_owned());
    assert_eq!(unsafe { raw[1].assume_init_ref() }, &"b".to_owned());

    unsafe {
        raw[0].assume_init_read();
        raw[1].assume_init_read();
    }
}

#[test]
fn deref() {
    let mut vec = RcVec::new();

    assert_eq!(*vec, [] as [String; 0]);

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);
    assert_eq!(*vec, ["a".to_owned(), "b".to_owned()]);
    assert_eq!(vec.len(), 2);

    assert_eq!(vec[0], "a".to_owned());
}

#[test]
fn deref_mut() {
    let mut vec = RcVec::new();

    assert_eq!(*vec, [] as [String; 0]);

    vec.push("a".to_owned());
    vec.push("b".to_owned());

    assert_eq!(vec, ["a".to_owned(), "b".to_owned()]);
    assert_eq!(*vec, ["a".to_owned(), "b".to_owned()]);
    assert_eq!(vec.len(), 2);

    assert_eq!(vec[0], "a".to_owned());
    vec[0] = "c".to_owned();
    assert_eq!(vec[0], "c".to_owned());
    assert_eq!(vec, ["c".to_owned(), "b".to_owned()]);
}

#[test]
fn drain() {
    let mut rcvec = rc_vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
    let mut iter = rcvec.drain(1..);
    assert_eq!(iter.next(), Some("b".to_owned()));
    assert_eq!(iter.next(), Some("c".to_owned()));
    assert_eq!(iter.next(), None);
    drop(iter);
    assert_eq!(rcvec, rc_vec!["a".to_owned()]);
}

#[test]
fn drain1() {
    let mut rcvec = rc_vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
    let mut iter = rcvec.drain(1..2);
    assert_eq!(iter.next(), Some("b".to_owned()));
    assert_eq!(iter.next(), None);
    drop(iter);
    assert_eq!(rcvec, rc_vec!["a".to_owned(), "c".to_owned()]);
}

#[test]
fn drain2() {
    let mut rcvec = rc_vec!["a".to_owned(), "b".to_owned(), "c".to_owned(), "d".to_owned()];
    let mut iter = rcvec.drain(1..2);
    assert_eq!(iter.next(), Some("b".to_owned()));
    assert_eq!(iter.next(), None);
    drop(iter);
    assert_eq!(rcvec, rc_vec!["a".to_owned(), "c".to_owned(), "d".to_owned()]);
}

#[test]
fn drain_zst() {
    let mut rcvec = rc_vec![Zst, Zst, Zst];
    let mut iter = rcvec.drain(1..);
    assert_eq!(iter.next(), Some(Zst));
    assert_eq!(iter.next(), Some(Zst));
    assert_eq!(iter.next(), None);
    drop(iter);
    assert_eq!(rcvec, rc_vec![Zst]);
}

#[test]
#[should_panic = "Boom by zero"]
fn drain_unwind() {
    let mut rcvec = rc_vec![
        Boom(3),
        Boom(2),
        Boom(1),
        Boom(0),
        Boom(-1),
        Boom(-2),
        Boom(-3)
    ];
    let mut iter = rcvec.drain(2..5);
    assert_eq!(iter.next(), Some(Boom(1)));
}

#[test]
fn append() {
    let mut vec = rc_vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
    let mut vec2 = rc_vec!["d".to_owned(), "e".to_owned(), "f".to_owned()];
    vec.append(&mut vec2);
    assert_eq!(vec, [
        "a".to_owned(),
        "b".to_owned(),
        "c".to_owned(),
        "d".to_owned(),
        "e".to_owned(),
        "f".to_owned(),
    ]);
    assert_eq!(vec2, []);
}

#[test]
fn split_off() {
    let mut vec = rc_vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
    let vec2 = vec.split_off(1);
    assert_eq!(vec, ["a".to_owned()]);
    assert_eq!(vec2, ["b".to_owned(), "c".to_owned()]);
}

#[test]
fn retain() {
    let mut rcvec = rc_vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
    rcvec.retain(|s| s != "b");
    assert_eq!(rcvec, rc_vec!["a".to_owned(), "c".to_owned()]);
}

#[test]
fn retain_empty() {
    let mut rcvec: RcVec<String> = rc_vec![];
    rcvec.retain(|s| s != "b");
    assert_eq!(rcvec, rc_vec![]);
}

#[test]
fn retain_zst() {
    let mut rcvec = rc_vec![Zst, Zst, Zst, Zst, Zst, Zst, Zst];
    let mut i = -1;
    rcvec.retain(|_| {
        i += 1;
        if (2..=4).contains(&i) {
            return false;
        }
        true
    });
    assert_eq!(rcvec, rc_vec![Zst, Zst, Zst, Zst]);
}

#[test]
#[should_panic = "Boom by zero"]
fn retain_unwind() {
    let mut rcvec = rc_vec![2, 1, 0, -1, -2];
    rcvec.retain(|b| {
        assert_ne!(*b, 0, "Boom by zero");
        *b != 1
    });
    unreachable!()
}

#[test]
fn retain_unwind_catch() {
    let mut rcvec = rc_vec![2, 1, 0, -1, -2];
    let ret = catch_unwind(AssertUnwindSafe(|| {
        rcvec.retain(|b| {
            assert_ne!(*b, 0, "Boom by zero");
            *b != 1
        });
    }));
    assert_eq!(ret.is_err(), true);
    assert_eq!(rcvec, [2, 0, -1, -2]);
}
