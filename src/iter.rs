use super::DSTVec;
use alloc::raw_vec::RawVec;
use std::{ptr, mem};
use super::Referent;

/// An owning iterator over the contents of a DSTVec<T>. Each call to `next` will return a *mut pointer to T, and it is up to the caller to ensure that its value is dropped.
pub struct IntoIter<T: Referent + ?Sized> {
    pointers: ::std::vec::IntoIter<(usize, T::Meta)>,
    data: RawVec<u8>,
}

impl<T: Referent + ?Sized> IntoIter<T> {
    unsafe fn assemble(&self, offset: usize, meta: T::Meta) -> *mut T {
        T::assemble_mut(self.data.ptr().offset(offset as isize) as *mut T::Data, meta)
    }
}

impl<T: Referent + ?Sized> Iterator for IntoIter<T> {
    type Item = *mut T;

    fn next(&mut self) -> Option<*mut T> {
        self.pointers.next().map(|(offset, meta)| {
            unsafe {
                self.assemble(offset, meta)
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pointers.size_hint()
    }
}

impl<T: Referent + ?Sized> ExactSizeIterator for IntoIter<T> {}

impl<T: Referent + ?Sized> Drop for IntoIter<T> {
    fn drop(&mut self) {
        let offsets_with_metas = mem::replace(&mut self.pointers, Vec::new().into_iter());
        for (offset, meta) in offsets_with_metas {
            unsafe {
                ptr::drop_in_place(self.assemble(offset, meta))
            }
        }
    }
}

impl<T: Referent + ?Sized> IntoIterator for DSTVec<T> {
    type Item = *mut T;
    type IntoIter = IntoIter<T>;

    fn into_iter(mut self) -> IntoIter<T> {
        let pointers = mem::replace(&mut self.pointers, Vec::new());
        let data = mem::replace(&mut self.data, RawVec::new());
        IntoIter {
            pointers: pointers.into_iter(),
            data: data,
        }
    }
}
