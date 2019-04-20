/*!
provides a Vec-like data structure for dynamically sized types
*/
#![feature(alloc, raw_vec_internals, unsize)]
use std::{
    marker::Unsize,
    ptr,
    mem,
};

extern crate alloc;
use alloc::raw_vec::RawVec;

extern crate referent;
use referent::Referent;

pub struct DSTVec<T: Referent + ?Sized> {
    // these aren't actual pointers, they're offsets + meta, used to
    // construct a pointer on the fly
    pointers: Vec<(usize, T::Meta)>,
    data: RawVec<u8>,
    used_bytes: usize,
}

impl<T: Referent + ?Sized> DSTVec<T> {
    pub fn new() -> DSTVec<T> {
        let data = RawVec::new();
        DSTVec {
            pointers: Vec::new(),
            data: data,
            used_bytes: 0,
        }
    }

    pub fn push<U: Unsize<T>>(&mut self, mut value: U) {
        let align = mem::align_of::<U>();
        let gap = self.used_bytes + align - (self.used_bytes % align);
        let size = mem::size_of::<U>();

        self.data.reserve(self.used_bytes, gap + size);
        self.pointers.reserve(1);

        let offset = self.used_bytes + gap;
        let (_, meta) = T::disassemble_mut(&mut value as &mut T);

        unsafe {
            let back = self.data.ptr().offset(offset as isize);
            ptr::write(back as *mut U, value);
        }

        self.pointers.push((offset, meta));

        self.used_bytes += gap + size;
    }

    pub fn iter<'b, 'a: 'b>(&'a self) -> impl Iterator<Item=&'a T> + 'b {
        self.pointers.iter().map(move |&(offset, meta)| unsafe {
            &*self.assemble(offset, meta)
        })
    }

    fn assemble(&self, offset: usize, meta: T::Meta) -> *mut T {
        unsafe {
            T::assemble_mut(self.data.ptr().offset(offset as isize) as *mut T::Data, meta)
        }
    }
}

impl<T: Referent + ?Sized> Drop for DSTVec<T> {
    fn drop(&mut self) {
        for &(offset, meta) in self.pointers.iter() {
            unsafe {
                ptr::drop_in_place(self.assemble(offset, meta));
            }
        }
    }
}

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
