use core::fmt::{self, Debug};
use core::iter::FusedIterator;
use core::mem;
use core::{ptr, slice};

use rc_vec_proc_macro::rc_impl_gen_arc_impl;

use crate::raw::{ArcRawVec, RcRawVec};

use crate::is_zst::IsZst;

#[rc_impl_gen_arc_impl]
pub struct RcVecIntoIter<T> {
    _raw: Option<RcRawVec<T>>,
    ptr: *const T,
    end: *const T,
}

#[rc_impl_gen_arc_impl]
impl<T> RcVecIntoIter<T> {
    pub fn new(mut raw: RcRawVec<T>, len: usize) -> Self {
        let ptr = raw.as_mut_ptr().cast_const().cast::<T>();
        let end = if T::ZST {
            ptr.wrapping_byte_add(len)
        } else {
            unsafe { ptr.add(len) }
        };

        Self { _raw: Some(raw), ptr, end }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len())
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.ptr.cast_mut(), self.len())
        }
    }

    fn as_raw_mut_slice(&mut self) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(self.ptr.cast_mut(), self.len())
    }
}

#[rc_impl_gen_arc_impl]
unsafe impl<T: Sync> Sync for RcVecIntoIter<T> { }

#[rc_impl_gen_arc_impl]
unsafe impl<T: Send> Send for RcVecIntoIter<T> { }

#[rc_impl_gen_arc_impl]
impl<T> AsRef<[T]> for RcVecIntoIter<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

#[rc_impl_gen_arc_impl]
impl<T> Drop for RcVecIntoIter<T> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.as_raw_mut_slice()) };
    }
}

#[rc_impl_gen_arc_impl]
impl<T> Iterator for RcVecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else if T::ZST {
            self.end = self.end.wrapping_byte_sub(1);

            Some(unsafe { mem::zeroed() })
        } else {
            let old = self.ptr;
            self.ptr = unsafe { self.ptr.add(1) };

            Some(unsafe { old.read() })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = if T::ZST {
            self.end.addr().wrapping_sub(self.ptr.addr())
        } else {
            unsafe { self.end.offset_from(self.ptr) as usize }
        };

        (exact, Some(exact))
    }

    fn count(self) -> usize
    where Self: Sized,
    {
        self.len()
    }
}

#[rc_impl_gen_arc_impl]
impl<T> DoubleEndedIterator for RcVecIntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else if T::ZST {
            self.end = self.end.wrapping_byte_sub(1);

            Some(unsafe { mem::zeroed() })
        } else {
            self.end = unsafe { self.end.sub(1) };

            Some(unsafe { self.end.read() })
        }
    }
}

#[rc_impl_gen_arc_impl]
impl<T> FusedIterator for RcVecIntoIter<T> { }

#[rc_impl_gen_arc_impl]
impl<T> ExactSizeIterator for RcVecIntoIter<T> { }

#[rc_impl_gen_arc_impl]
impl<T> Default for RcVecIntoIter<T> {
    fn default() -> Self {
        let ptr = ptr::dangling();
        Self { _raw: None, ptr, end: ptr }
    }
}

impl<T: Debug> Debug for RcVecIntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RcVecIntoIter")
            .field(&self.as_slice())
            .finish()
    }
}
