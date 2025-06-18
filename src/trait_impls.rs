use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use core::{
    borrow::{Borrow, BorrowMut},
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
    ptr,
    slice::SliceIndex,
};
use rc_vec_proc_macro::rc_impl_gen_arc_impl;
use unique_rc::{UniqArc, UniqRc};

use crate::{ArcVec, RcVec};

mod into_iter;

#[rc_impl_gen_arc_impl]
impl<T> AsRef<Self> for RcVec<T> {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[rc_impl_gen_arc_impl]
impl<T> AsMut<Self> for RcVec<T> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[rc_impl_gen_arc_impl]
impl<T> AsRef<[T]> for RcVec<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

#[rc_impl_gen_arc_impl]
impl<T> AsMut<[T]> for RcVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

#[rc_impl_gen_arc_impl]
impl<T> Borrow<[T]> for RcVec<T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

#[rc_impl_gen_arc_impl]
impl<T> BorrowMut<[T]> for RcVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

#[rc_impl_gen_arc_impl]
impl<T: Clone> From<&[T]> for RcVec<T> {
    fn from(value: &[T]) -> Self {
        let uniq_rc: UniqRc<[T]> = UniqRc::from(value);
        Self::from(uniq_rc)
    }
}

#[rc_impl_gen_arc_impl]
impl<T: Clone> From<&mut [T]> for RcVec<T> {
    fn from(value: &mut [T]) -> Self {
        let uniq_rc: UniqRc<[T]> = UniqRc::from(value);
        Self::from(uniq_rc)
    }
}

#[rc_impl_gen_arc_impl]
impl<T: Clone> From<Rc<[T]>> for RcVec<T> {
    fn from(value: Rc<[T]>) -> Self {
        let uniq_rc = UniqRc::from(value);
        Self::from(uniq_rc)
    }
}

#[rc_impl_gen_arc_impl]
impl From<UniqRc<str>> for RcVec<u8> {
    fn from(value: UniqRc<str>) -> Self {
        let len = value.len();
        let rc = UniqRc::into_raw(value).cast::<u8>();
        let slice = ptr::slice_from_raw_parts_mut(rc, len);
        let value = unsafe { UniqRc::from_raw_unchecked(slice) };
        value.into()
    }
}

#[rc_impl_gen_arc_impl]
impl<T, const N: usize> From<[T; N]> for RcVec<T> {
    fn from(value: [T; N]) -> Self {
        Self::from_iter(value)
    }
}

#[rc_impl_gen_arc_impl]
impl<T> FromIterator<T> for RcVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut buf = Self::with_capacity(iter.size_hint().0);
        buf.extend(iter);
        buf
    }
}

#[rc_impl_gen_arc_impl]
impl<T> Extend<T> for RcVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        iter.for_each(|value| self.push(value));
    }
}

#[rc_impl_gen_arc_impl]
impl<'a, T: Copy> Extend<&'a T> for RcVec<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied());
    }
}

#[rc_impl_gen_arc_impl]
impl<T> From<RcVec<T>> for Rc<[T]> {
    fn from(value: RcVec<T>) -> Self {
        value.into_rc_slice()
    }
}

// 这将产生分配和拷贝
#[rc_impl_gen_arc_impl]
impl<T> From<RcVec<T>> for Box<[T]> {
    fn from(value: RcVec<T>) -> Self {
        let len = value.len();
        let raw = value.into_raw_vec();
        Box::from_iter(raw.slice()[..len].iter().map(|value| {
            unsafe { value.assume_init_read() }
        }))
    }
}

#[rc_impl_gen_arc_impl]
impl From<&str> for RcVec<u8> {
    fn from(value: &str) -> Self {
        let uniq_rc: UniqRc<str> = UniqRc::from(value);
        let len = uniq_rc.len();
        let str_ptr = UniqRc::into_raw(uniq_rc);
        let uniq_rc = unsafe {
            let slice_ptr = ptr::slice_from_raw_parts_mut(
                str_ptr.cast::<u8>(),
                len,
            );
            UniqRc::from_raw_unchecked(slice_ptr)
        };
        Self::from(uniq_rc)
    }
}

#[rc_impl_gen_arc_impl]
impl<T: Eq> Eq for RcVec<T> {}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq for RcVec<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq<[T]> for RcVec<T> {
    fn eq(&self, other: &[T]) -> bool {
        **self == *other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq<RcVec<T>> for Vec<T> {
    fn eq(&self, other: &RcVec<T>) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq<Vec<T>> for RcVec<T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq, const N: usize> PartialEq<[T; N]> for RcVec<T> {
    fn eq(&self, other: &[T; N]) -> bool {
        **self == *other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq, const N: usize> PartialEq<&[T; N]> for RcVec<T> {
    fn eq(&self, other: &&[T; N]) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq, const N: usize> PartialEq<&mut [T; N]> for RcVec<T> {
    fn eq(&self, other: &&mut [T; N]) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq<&[T]> for RcVec<T> {
    fn eq(&self, other: &&[T]) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq<&mut [T]> for RcVec<T> {
    fn eq(&self, other: &&mut [T]) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq<RcVec<T>> for [T] {
    fn eq(&self, other: &RcVec<T>) -> bool {
        *self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq<RcVec<T>> for &[T] {
    fn eq(&self, other: &RcVec<T>) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq<RcVec<T>> for &mut [T] {
    fn eq(&self, other: &RcVec<T>) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialEq> PartialEq<UniqRc<[T]>> for RcVec<T> {
    fn eq(&self, other: &UniqRc<[T]>) -> bool {
        **self == **other
    }
}

#[rc_impl_gen_arc_impl]
impl<T: Debug> Debug for RcVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[rc_impl_gen_arc_impl]
impl<T: Hash> Hash for RcVec<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

#[rc_impl_gen_arc_impl]
impl<T: PartialOrd> PartialOrd for RcVec<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (**self).partial_cmp(other)
    }
}

#[rc_impl_gen_arc_impl]
impl<T: Ord> Ord for RcVec<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (**self).cmp(other)
    }
}

#[rc_impl_gen_arc_impl]
impl<T, I: SliceIndex<[T]>> Index<I> for RcVec<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &(**self)[index]
    }
}

#[rc_impl_gen_arc_impl]
impl<T, I: SliceIndex<[T]>> IndexMut<I> for RcVec<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut (**self)[index]
    }
}

#[rc_impl_gen_arc_impl]
impl<'a, T> IntoIterator for &'a RcVec<T> {
    type Item = &'a T;
    type IntoIter = <&'a [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (**self).iter()
    }
}

#[rc_impl_gen_arc_impl]
impl<'a, T> IntoIterator for &'a mut RcVec<T> {
    type Item = &'a mut T;
    type IntoIter = <&'a mut [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (**self).iter_mut()
    }
}

#[rc_impl_gen_arc_impl]
impl<T> IntoIterator for RcVec<T> {
    type Item = T;
    type IntoIter = into_iter::RcVecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        into_iter::RcVecIntoIter::new(self.into_raw_vec(), len)
    }
}

#[rc_impl_gen_arc_impl]
impl<T, const N: usize> TryFrom<RcVec<T>> for [T; N] {
    type Error = RcVec<T>;

    fn try_from(mut vec: RcVec<T>) -> Result<Self, Self::Error> {
        if vec.len() != N {
            return Err(vec);
        }

        unsafe { vec.set_len(0); }

        let ptr = vec.as_ptr().cast::<[T; N]>();
        let arr = unsafe { ptr.read() };
        Ok(arr)
    }
}

#[rc_impl_gen_arc_impl]
impl<T, const N: usize> TryFrom<RcVec<T>> for Rc<[T; N]> {
    type Error = RcVec<T>;

    fn try_from(vec: RcVec<T>) -> Result<Self, Self::Error> {
        if vec.len() != N {
            return Err(vec);
        }

        let slice = Rc::<[T]>::from(vec);
        Ok(slice.try_into().ok().unwrap())
    }
}

#[cfg(feature = "std")]
#[doc(cfg(feature = "std"))]
impl std::io::Write for RcVec<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        let len = bufs.iter().map(|b| b.len()).sum();
        self.reserve(len);
        for buf in bufs {
            self.extend_from_slice(buf);
        }
        Ok(len)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
