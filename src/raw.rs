use crate::is_zst::IsZst as _;
use core::{cmp::max, mem::{replace, MaybeUninit}, num::NonZeroUsize, ops::Range, ptr};

use alloc::{rc::Rc, sync::Arc};
use rc_vec_proc_macro::rc_impl_gen_arc_impl;
use unique_rc::{UniqRc, UniqArc};

#[rc_impl_gen_arc_impl]
pub struct RcRawVec<T> {
    ptr: Option<UniqRc<[MaybeUninit<T>]>>,
}

#[rc_impl_gen_arc_impl]
impl<T> RcRawVec<T> {
    pub(crate) const MIN_NON_ZERO_CAP: usize = match size_of::<T>() {
        1 => 8,
        0..=1024 => 4,
        _ => 1,
    };

    pub fn take(&mut self) -> Self {
        Self { ptr: self.ptr.take() }
    }

    pub const fn new() -> Self {
        Self { ptr: None }
    }

    pub fn from_uniq_slice(raw: UniqRc<[T]>) -> Self {
        let raw_ptr = UniqRc::into_raw(raw);
        let raw_ptr = ptr::slice_from_raw_parts_mut(
            raw_ptr.cast(),
            raw_ptr.len(),
        );
        let raw = unsafe { UniqRc::from_raw_unchecked(raw_ptr) };
        Self { ptr: Some(raw) }
    }

    #[inline]
    pub fn from_raw_uniq_slice(raw: UniqRc<[MaybeUninit<T>]>) -> Self {
        Self { ptr: Some(raw) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        if let Some(ptr) = self.ptr.as_ref() {
            let rc = unsafe { UniqRc::get_rc_unchecked(ptr) };
            Rc::as_ptr(rc).cast()
        } else {
            ptr::dangling()
        }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.as_ptr().cast_mut()
    }

    pub fn slice(&self) -> &[MaybeUninit<T>] {
        self.ptr.as_deref()
            .unwrap_or_default()
    }

    pub fn slice_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self.ptr.as_deref_mut()
            .unwrap_or_default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        alloc_guard(capacity);

        NonZeroUsize::new(capacity).map_or(Self::new(), |cap| {
            let iter = (0..cap.into()).map(|_| {
                MaybeUninit::uninit()
            });
            let ptr = UniqRc::from_iter(iter);
            debug_assert_eq!(ptr.len(), capacity);
            Self { ptr: Some(ptr) }
        })
    }

    pub fn capacity(&self) -> usize {
        if T::ZST {
            usize::MAX
        } else if self.ptr.is_some() {
            self.ptr.as_deref()
                .map_or(0, |ptr| ptr.len())
        } else {
            0
        }
    }

    pub fn into_raw_rc(self) -> Option<UniqRc<[MaybeUninit<T>]>> {
        self.ptr
    }

    /// 对于从未分配或者ZST, 这可能创建新分配
    pub fn into_rc(self) -> UniqRc<[MaybeUninit<T>]> {
        self.into_raw_rc().unwrap_or_else(|| {
            unsafe { UniqRc::new_unchecked(Rc::new([])) }
        })
    }

    #[inline]
    pub fn reserve(&mut self, len: usize, additional: usize) {
        #[cold]
        fn reserve_cold<T>(
            this: &mut RcRawVec<T>,
            len: usize,
            additional: usize,
        ) {
            this.grow_amortized(len, additional);
        }
        if self.needs_to_grow(len, additional) {
            reserve_cold(self, len, additional);
        }
    }

    #[inline(never)]
    pub fn reserve_for_push(&mut self, len: usize) {
        self.grow_amortized(len, 1);
    }

    pub fn reserve_exact(&mut self, len: usize, additional: usize) {
        if self.needs_to_grow(len, additional) {
            self.grow_exact(len, additional);
        }
    }

    pub fn drop_elems(&mut self, len: usize) {
        self.drop_elems_from_range(0..len);
    }

    pub fn drop_elems_from_range(&mut self, range: Range<usize>) {
        if self.ptr.is_none() {
            return;
        }
        let len = range.len();
        let start = range.start;

        unsafe {
            let data = self.as_mut_ptr().add(start).cast::<T>();
            let to_drop = ptr::slice_from_raw_parts_mut(data, len);
            ptr::drop_in_place(to_drop);
        }
    }

    fn needs_to_grow(&self, len: usize, additional: usize) -> bool {
        additional > self.capacity().wrapping_sub(len)
    }


    fn grow_amortized(&mut self, len: usize, additional: usize) {
        debug_assert_ne!(additional, 0);

        if T::ZST { return }

        let required_cap = len.saturating_add(additional);

        let cap = max(self.capacity().saturating_mul(2), required_cap);
        let cap = max(Self::MIN_NON_ZERO_CAP, cap);

        let old = replace(self, Self::with_capacity(cap));
        unsafe {
            let src = old.as_ptr();
            ptr::copy_nonoverlapping(src, self.as_mut_ptr(), len);
        }
    }

    fn grow_exact(&mut self, len: usize, additional: usize) {
        debug_assert_ne!(additional, 0);

        if T::ZST { return }

        let cap = len.saturating_add(additional);

        let old = replace(self, Self::with_capacity(cap));
        unsafe {
            let src = old.as_ptr();
            ptr::copy_nonoverlapping(src, self.as_mut_ptr(), len);
        }
    }

    pub fn shrink_to_fit(&mut self, cap: usize) {
        assert!(cap <= self.capacity());

        if T::ZST { return }

        let old = replace(self, Self::with_capacity(cap));
        unsafe {
            let src = old.as_ptr();
            ptr::copy_nonoverlapping(src, self.as_mut_ptr(), cap);
        }
    }
}

#[rc_impl_gen_arc_impl]
impl<T> Default for RcRawVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cold]
#[inline(never)]
fn alloc_guard(alloc_size: usize) {
    if usize::BITS < 64 && alloc_size > isize::MAX as usize {
        panic!("allocate size {alloc_size} overflow")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::borrow::ToOwned;

    #[test]
    fn it_works() {
        let mut raw = RcRawVec::with_capacity(3);
        assert_eq!(raw.capacity(), 3);
        raw.slice_mut()[0].write("a".to_owned());
        raw.slice_mut()[1].write("b".to_owned());

        assert_eq!(unsafe { raw.slice_mut()[0].assume_init_ref() }, "a");
        assert_eq!(unsafe { raw.slice_mut()[1].assume_init_ref() }, "b");

        raw.reserve_exact(2, 3);

        assert_eq!(raw.capacity(), 5);
        assert_eq!(unsafe { raw.slice_mut()[0].assume_init_ref() }, "a");
        assert_eq!(unsafe { raw.slice_mut()[1].assume_init_ref() }, "b");

        raw.slice_mut()[2].write("c".to_owned());

        assert_eq!(unsafe { raw.slice_mut()[2].assume_init_ref() }, "c");

        raw.drop_elems(3);
    }
}
