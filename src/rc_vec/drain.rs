use core::{
    fmt::Debug,
    iter::FusedIterator,
    mem::take,
    ptr::{self, NonNull},
    slice,
};
use crate::is_zst::IsZst;
use rc_vec_proc_macro::rc_impl_gen_arc_impl;

use super::{ArcVec, RcVec};

#[rc_impl_gen_arc_impl]
pub struct RcVecDrain<'a, T: 'a> {
    pub(super) tail_start: usize,
    pub(super) tail_len: usize,
    pub(super) iter: slice::Iter<'a, T>,
    pub(super) vec: NonNull<RcVec<T>>,
}

impl<'a, T: Debug + 'a> Debug for RcVecDrain<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("RcVecDrain")
            .field(&self.iter.as_slice())
            .finish()
    }
}

impl<'a, T: Debug + 'a> Debug for ArcVecDrain<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ArcVecDrain")
            .field(&self.iter.as_slice())
            .finish()
    }
}

#[rc_impl_gen_arc_impl]
impl<'a, T: 'a> RcVecDrain<'a, T> {
    pub fn as_slice(&self) -> &[T] {
        self.iter.as_slice()
    }
}

#[rc_impl_gen_arc_impl]
impl<'a, T: 'a> Iterator for RcVecDrain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ele| unsafe { ptr::read(ele) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[rc_impl_gen_arc_impl]
impl<'a, T: 'a> DoubleEndedIterator for RcVecDrain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|ele| unsafe { ptr::read(ele) })
    }
}

#[rc_impl_gen_arc_impl]
struct RcVecMoveGuard<'r, 'a, T>(&'r mut RcVecDrain<'a, T>);

#[rc_impl_gen_arc_impl]
impl<'r, 'a, T> Drop for RcVecMoveGuard<'r, 'a, T> {
    fn drop(&mut self) {
        unsafe {
            let src_vec = self.0.vec.as_mut();
            let start = src_vec.len();
            let tail = self.0.tail_start;

            if tail != start {
                let dest = src_vec.as_mut_ptr().add(start);
                src_vec.as_ptr().add(tail)
                    .copy_to(dest, self.0.tail_len);
            }

            src_vec.set_len(start+self.0.tail_len);
        }
    }
}

#[rc_impl_gen_arc_impl]
impl<'a, T: 'a> Drop for RcVecDrain<'a, T> {
    fn drop(&mut self) {
        let iter = take(&mut self.iter);
        let drop_len = iter.len();

        let mut vec = self.vec;

        if T::ZST {
            unsafe {
                let vec = vec.as_mut();
                let old_len = vec.len();

                vec.set_len(old_len + drop_len + self.tail_len);
                vec.truncate(old_len + self.tail_len);
            }

            return;
        }

        let _guard = RcVecMoveGuard(self);

        if drop_len == 0 {
            return;
        }

        let drop_ptr = iter.as_slice().as_ptr();

        unsafe {
            let vec_ptr = vec.as_mut().as_ptr();
            let drop_offset = drop_ptr.offset_from(vec_ptr).try_into().unwrap();
            let range = drop_offset..drop_offset+drop_len;
            vec.as_mut().raw.drop_elems_from_range(range);
        }
    }
}

#[rc_impl_gen_arc_impl]
impl<T> ExactSizeIterator for RcVecDrain<'_, T> { }

#[rc_impl_gen_arc_impl]
impl<T> FusedIterator for RcVecDrain<'_, T> { }
