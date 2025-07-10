#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod raw;
mod is_zst;
mod rc_vec;
mod utils;

pub use rc_vec::*;
pub use unique_rc;

#[cfg(test)]
mod tests;

/// Like `vec![]`, create a [`RcVec`]
///
/// # Examples
///
/// ```
/// let vec = rc_vec::rc_vec![1, 2, 3];
/// assert_eq!(vec, [1, 2, 3]);
/// ```
///
/// ```
/// let vec = rc_vec::rc_vec![3; 4];
/// assert_eq!(vec, [3, 3, 3, 3]);
/// ```
///
/// ```
/// let vec = rc_vec::rc_vec![];
/// assert_eq!(vec.len(), 0);
/// # assert_ne!(vec, [1]);
/// ```
#[macro_export]
macro_rules! rc_vec {
    () => {
        $crate::RcVec::new()
    };
    ($elem:expr; $n:expr) => {
        $crate::RcVec::from_elem($elem, $n)
    };
    ($($t:tt)*) => {
        $crate::RcVec::from_array([$($t)*])
    };
}

/// Like `vec![]`, create a [`ArcVec`]
///
/// # Examples
///
/// ```
/// let vec = rc_vec::arc_vec![1, 2, 3];
/// assert_eq!(vec, [1, 2, 3]);
/// ```
///
/// ```
/// let vec = rc_vec::arc_vec![3; 4];
/// assert_eq!(vec, [3, 3, 3, 3]);
/// ```
///
/// ```
/// let vec = rc_vec::arc_vec![];
/// assert_eq!(vec.len(), 0);
/// # assert_ne!(vec, [1]);
/// ```
#[macro_export]
macro_rules! arc_vec {
    () => {
        $crate::ArcVec::new()
    };
    ($elem:expr; $n:expr) => {
        $crate::ArcVec::from_elem($elem, $n)
    };
    ($($t:tt)*) => {
        $crate::ArcVec::from_array([$($t)*])
    };
}
