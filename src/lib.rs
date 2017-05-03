/*!
provides a Vec-like data structure for dynamically sized types
*/
#![feature(alloc, unsize)]
extern crate alloc;

mod iter;
pub use iter::IntoIter;

mod traits;
pub use traits::Referent;

use alloc::raw_vec::RawVec;
use std::marker::Unsize;
use std::{ptr, mem};
use std::vec;

pub struct DSTVec<T: Referent + ?Sized> {
    // these aren't actual pointers, they're offets + meta, used to construct a
    // pointer on the fly
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
            ptr::copy_nonoverlapping(&value, back as *mut U, 1);
        }

        self.pointers.push((offset, meta));

        self.used_bytes += gap + size;

        mem::forget(value);
    }

    unsafe fn assemble(&self, offset: usize, meta: T::Meta) -> *mut T {
        T::assemble_mut(self.data.ptr().offset(offset as isize) as *mut T::Data, meta)
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
