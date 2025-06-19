#[cfg(doc)]
use alloc::{boxed::Box, vec::Vec};

use alloc::{rc::Rc, sync::Arc};
use core::{
    cmp::max,
    iter,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, RangeBounds},
    ptr, slice,
};
use rc_vec_proc_macro::rc_impl_gen_arc_impl;
use unique_rc::{UniqArc, UniqRc};

use crate::{
    raw::{ArcRawVec, RcRawVec},
    utils,
};

/// [`RcVec`] based on [`Rc`] and can be converted from Rc without allocation,
/// just like [`Box`] is converted to [`Vec`]
///
/// # Examples
///
/// ```
/// # use std::rc::Rc;
/// use rc_vec::RcVec;
///
/// let rc: Rc<[i32]> = Rc::new([1, 2, 3]);
/// let mut rc_vec = RcVec::from(rc);
///
/// assert_eq!(rc_vec, [1, 2, 3]);
/// rc_vec.push(4);
/// assert_eq!(rc_vec, [1, 2, 3, 4]);
/// ```
#[rc_impl_gen_arc_impl]
pub struct RcVec<T> {
    raw: RcRawVec<T>,
    len: usize,
}

#[rc_impl_gen_arc_impl]
impl<T> Deref for RcVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        let ptr = self.raw.as_ptr();
        unsafe { slice::from_raw_parts(ptr, self.len) }
    }
}

#[rc_impl_gen_arc_impl]
impl<T> DerefMut for RcVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = self.raw.as_mut_ptr();
        unsafe { slice::from_raw_parts_mut(ptr, self.len) }
    }
}

#[rc_impl_gen_arc_impl]
impl<T> Drop for RcVec<T> {
    fn drop(&mut self) {
        self.raw.drop_elems(self.len);
    }
}

#[rc_impl_gen_arc_impl]
impl<T> From<UniqRc<[T]>> for RcVec<T> {
    fn from(value: UniqRc<[T]>) -> Self {
        let len = value.len();
        let raw = RcRawVec::from_uniq_slice(value);
        Self { raw, len }
    }
}

#[rc_impl_gen_arc_impl]
impl<T> Default for RcVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[rc_impl_gen_arc_impl]
impl<T> RcVec<T> {
    /// Create a new [`RcVec`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use rc_vec::RcVec;
    /// let mut vec = RcVec::new();
    /// vec.push(3);
    /// assert_eq!(vec, [3]);
    /// ```
    pub fn new() -> Self {
        Self { raw: RcRawVec::new(), len: 0 }
    }

    /// Create a new [`RcVec`] Initial capacity of `capacity`
    ///
    /// # Examples
    ///
    /// ```
    /// # use rc_vec::RcVec;
    /// let vec = RcVec::with_capacity(176);
    /// assert_eq!(vec.capacity(), 176);
    /// assert_eq!(vec.len(), 0);
    /// # assert_ne!(vec, [1]);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self { raw: RcRawVec::with_capacity(capacity), len: 0 }
    }

    /// Readonly permission pointer
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }

    /// Read and Write permission pointer
    ///
    /// # Examples
    ///
    /// ```
    /// # use rc_vec::rc_vec;
    /// let mut vec = rc_vec![1, 2, 3];
    /// assert_eq!(vec, [1, 2, 3]);
    /// unsafe { *vec.as_mut_ptr() += 1 }
    /// assert_eq!(vec, [2, 2, 3]);
    /// ```
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.as_mut_ptr()
    }

    /// Get initialized datas count
    ///
    /// # Examples
    ///
    /// ```
    /// # use rc_vec::RcVec;
    /// let mut vec = RcVec::with_capacity(4);
    /// assert_eq!(vec.capacity(), 4);
    ///
    /// vec.push(2333);
    ///
    /// assert_eq!(vec.len(), 1);
    /// assert_eq!(vec.capacity(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Like `.len() == 0`
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get allocated capacity
    ///
    /// # Examples
    ///
    /// ```
    /// # use rc_vec::RcVec;
    /// let vec = RcVec::with_capacity(176);
    /// assert_eq!(vec.capacity(), 176);
    /// assert_eq!(vec.len(), 0);
    /// # assert_ne!(vec, [1]);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity() {
            self.raw.reserve_for_push(self.len);
        }

        unsafe {
            let end = self.as_mut_ptr().add(self.len);
            end.write(value);
            self.len += 1;
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(self.len, additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.raw.reserve_exact(self.len, additional);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                Some(self.as_ptr().add(self.len).read())
            }
        }
    }

    #[inline]
    #[track_caller]
    pub fn remove(&mut self, index: usize) -> T {
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("remove index (is {index}) should be < len (is {len})")
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        unsafe {
            let ret;
            {
                let ptr = self.as_mut_ptr().add(index);
                ret = ptr.read();
                ptr::copy(ptr.add(1), ptr, len - index - 1);
            }
            self.set_len(len - 1);
            ret
        }
    }

    #[inline]
    #[track_caller]
    pub fn swap_remove(&mut self, index: usize) -> T {
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("swap_remove index (is {index}) should be < len (is {len})")
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        unsafe {
            let value = self.as_ptr().add(index).read();
            let ptr = self.as_mut_ptr();
            ptr.add(index).copy_from(ptr.add(len-1), 1);
            self.set_len(len-1);
            value
        }
    }

    /// Reallocate to remove excess capacity
    ///
    /// # Examples
    ///
    /// ```
    /// # use rc_vec::RcVec;
    /// let mut vec = RcVec::with_capacity(16);
    /// vec.push(233);
    /// assert_eq!(vec.len(), 1);
    /// assert_eq!(vec.capacity(), 16);
    ///
    /// vec.shrink_to_fit();
    /// assert_eq!(vec.len(), 1);
    /// assert_eq!(vec.capacity(), 1);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        if self.capacity() > self.len() {
            self.raw.shrink_to_fit(self.len());
        }
    }

    /// Reallocate to max(`min_capacity`, `.len()`)
    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.capacity() > min_capacity {
            self.raw.shrink_to_fit(max(self.len(), min_capacity));
        }
    }

    /// Like [`Vec::set_len`]
    ///
    /// # Safety
    /// See [`Vec::set_len`] for safety concerns and examples.
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }

    /// Like [`Vec::spare_capacity_mut`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use rc_vec::RcVec;
    /// # use std::mem::MaybeUninit;
    /// let mut vec = RcVec::with_capacity(4);
    ///
    /// vec.push(0);
    /// assert_eq!(*vec, [0]);
    ///
    /// let spare = vec.spare_capacity_mut();
    /// assert_eq!(spare.len(), 3);
    /// spare[0] = MaybeUninit::new(1);
    /// assert_eq!(*vec, [0]);
    ///
    /// unsafe {
    ///     // SAFETY: The initialized data has been written to spare
    ///     vec.set_len(vec.len() + 1);
    /// }
    /// assert_eq!(*vec, [0, 1]);
    /// ```
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        &mut self.raw.slice_mut()[self.len..]
    }

    unsafe fn split_at_spare_mut_with_len(
        &mut self,
    ) -> (&mut [T], &mut [MaybeUninit<T>], &mut usize) {
        let (initialized, spare)
            = self.raw.slice_mut().split_at_mut(self.len);
        let initialized_ptr = initialized.as_mut_ptr().cast::<T>();
        let initialized = slice::from_raw_parts_mut(initialized_ptr, self.len);

        (initialized, spare, &mut self.len)
    }

    /// # Safety
    /// `slice` `0..len` must be initialized
    #[inline]
    pub unsafe fn from_raw_uniq_slice(
        slice: UniqRc<[MaybeUninit<T>]>,
        len: usize,
    ) -> Self {
        Self { raw: RcRawVec::from_raw_uniq_slice(slice), len }
    }

    pub fn from_uniq_slice(slice: UniqRc<[T]>) -> Self {
        slice.into()
    }

    #[inline]
    pub fn into_raw_uniq_slice(mut self) -> UniqRc<[MaybeUninit<T>]> {
        self.raw.take().into_rc()
    }

    #[inline]
    pub fn into_raw_uniq_slice_optional(mut self) -> Option<UniqRc<[MaybeUninit<T>]>> {
        self.raw.take().into_raw_rc()
    }

    #[inline]
    pub fn into_uniq_slice(mut self) -> UniqRc<[T]> {
        self.shrink_to_fit();
        let len = self.len();
        debug_assert_eq!(len, self.capacity());

        let raw = UniqRc::into_raw(self.raw.take().into_rc());
        let slice = ptr::slice_from_raw_parts_mut(raw.cast::<T>(), len);
        unsafe { UniqRc::from_raw_unchecked(slice) }
    }

    #[inline]
    pub fn into_rc_slice(self) -> Rc<[T]> {
        self.into_uniq_slice().into()
    }

    #[inline]
    pub(crate) fn into_raw_vec(mut self) -> RcRawVec<T> {
        self.raw.take()
    }

    pub fn truncate(&mut self, len: usize) {
        let old_len = self.len();

        // NOTE: use `>`, reference from Vec::truncate
        if len > old_len {
            return;
        }

        unsafe {
            self.set_len(len);
            self.raw.drop_elems_from_range(len..old_len);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        let old_len = self.len();
        unsafe {
            self.set_len(0);
            self.raw.drop_elems(old_len);
        }
    }

    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where F: FnMut() -> T,
    {
        let len = self.len();

        if new_len > len {
            self.extend(iter::repeat_with(f).take(new_len-len));
        } else {
            self.truncate(len);
        }
    }

    pub fn leak(mut self) -> &'static mut [T] {
        let len = self.len();

        if len == 0 {
            return Default::default();
        }

        self.raw.take()
            .into_raw_rc()
            .map(UniqRc::into_raw)
            .map(|raw| {
                unsafe { slice::from_raw_parts_mut(raw.cast::<T>(), len) }
            })
            .unwrap_or_default()
    }

    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.into_iter()
    }
}

#[rc_impl_gen_arc_impl]
impl<T: Clone> RcVec<T> {
    pub fn resize(&mut self, new_len: usize, value: T) {
        let len = self.len();

        if new_len > len {
            self.extend(iter::repeat_n(value, new_len-len));
        } else {
            self.truncate(len);
        }
    }

    pub fn extend_from_slice(&mut self, buf: &[T]) {
        self.reserve(buf.len());

        for ele in buf {
            let len = self.len();
            let ele = ele.clone();

            unsafe {
                self.as_mut_ptr().add(len).write(ele);
                self.set_len(len + 1);
            }
        }
    }

    pub fn extend_from_within<R>(&mut self, src: R)
    where R: RangeBounds<usize>,
    {
        let range = utils::range(src, ..self.len());
        self.reserve(range.len());

        let (this, spare, len) = unsafe {
            self.split_at_spare_mut_with_len()
        };

        let to_clone = unsafe { this.get_unchecked(range) };

        to_clone.iter().zip(spare)
            .map(|(src, dst)| dst.write(src.clone()))
            .for_each(|_| *len += 1);
    }
}

#[rc_impl_gen_arc_impl]
impl<T> RcVec<T> {
    /// Macro support
    #[doc(hidden)]
    #[allow(unused)]
    pub fn from_elem(elem: T, len: usize) -> Self
    where T: Clone,
    {
        Self::from_iter(iter::repeat_n(elem, len))
    }

    /// Macro support
    #[doc(hidden)]
    #[allow(unused)]
    pub fn from_array<const N: usize>(arr: [T; N]) -> Self {
        arr.into()
    }
}
