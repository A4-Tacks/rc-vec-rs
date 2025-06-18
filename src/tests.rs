#![allow(unused_imports)]

use alloc::{borrow::ToOwned, boxed::Box, rc::Rc, string::String};

use super::rc_vec::*;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Zst;
impl Drop for Zst {
    fn drop(&mut self) {
        ()
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
