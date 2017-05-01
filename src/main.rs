/*!
provides a Vec-like data structure for dynamically sized types
*/
#![feature(alloc, raw, unsize, conservative_impl_trait)]
#![allow(unused)]
extern crate alloc;
use alloc::raw_vec::RawVec;
use std::{ptr, mem};
use std::marker::Unsize;
use std::raw::TraitObject;

macro_rules! declare_dstvec {
    ($name:ident, $Trait:ty) => {
        pub struct $name {
            pointers: Vec<*mut $Trait>,
            data: RawVec<u8>,
            back: *mut u8,
        }

        impl_dstvec!($name, $Trait);
    };
}

macro_rules! impl_dstvec {
    ($name:ident, $Trait:ty) => {
        impl $name {
            pub fn new() -> $name {
                let data = RawVec::new();
                let data_ptr = data.ptr();
                $name {
                    pointers: Vec::new(),
                    data: data,
                    back: data_ptr,
                }
            }

            pub unsafe fn push<U: Unsize<$Trait> + ?Sized>(&mut self, value: *const U) {
                let value = value as *const $Trait;
                let used = self.used_data();
                let extra = mem::size_of_val(&*value);
                let old_location = self.data.ptr();

                self.data.reserve(used, extra);
                self.pointers.reserve(1);

                if self.data.ptr() != old_location {
                    let mut location = self.data.ptr();

                    for i in 0..self.pointers.len() {
                        let size = mem::size_of_val(&*self.pointers[i] as &$Trait);
                        self.pointers[i] = $name::repoint(self.pointers[i], location);
                        location = location.offset(size as isize);
                    }

                    self.back = location;
                }

                ptr::copy(value as *const u8, self.back, 1);
                self.pointers.push($name::repoint(value as *mut $Trait, self.back));
                self.back = self.back.offset(extra as isize);
            }

            fn used_data(&self) -> usize {
                self.pointers.iter()
                .map(|ptr| unsafe {
                    mem::size_of_val(&*ptr)
                }).sum()
            }

            unsafe fn repoint(ptr: *mut $Trait, new_location: *mut u8) -> *mut $Trait {
                use std::raw::TraitObject;

                let raw_object: TraitObject = mem::transmute(ptr);

                mem::transmute(TraitObject {
                    data: new_location as *mut (),
                    vtable: raw_object.vtable,
                })
            }

            fn into_raw_iter(mut self) -> impl ExactSizeIterator<Item=*mut $Trait> {
                let pointers = mem::replace(&mut self.pointers, Vec::new());
                let data = mem::replace(&mut self.data, RawVec::new());
                IntoIter {
                    pointers: pointers.into_iter(),
                    data: data,
                }
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                for &pointer in self.pointers.iter() {
                    unsafe {
                        ptr::drop_in_place(pointer)
                    }
                }
            }
        }

        pub struct IntoIter {
            pointers: std::vec::IntoIter<*mut $Trait>,
            data: RawVec<u8>,
        }

        impl Iterator for IntoIter {
            type Item = *mut $Trait;

            fn next(&mut self) -> Option<*mut $Trait> {
                self.pointers.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.pointers.size_hint()
            }
        }

        impl ExactSizeIterator for IntoIter {}

        impl Drop for IntoIter {
            fn drop(&mut self) {
                for pointer in mem::replace(&mut self.pointers, Vec::new().into_iter()) {
                    unsafe {
                        ptr::drop_in_place(pointer)
                    }
                }
            }
        }
    };
}


macro_rules! push_dstvec {
    ($dstv:expr, $value:expr) => {
        unsafe {
            $dstv.push(&$value);
            ::std::mem::forget($value);
        }
    }
}

pub trait FnOnceUnsafe {
    unsafe fn call_once_unsafe(&mut self);
}

impl<F: FnOnce()> FnOnceUnsafe for F {
    unsafe fn call_once_unsafe(&mut self) {
        ptr::read(self)()
    }
}

declare_dstvec!(Callbacks, FnOnceUnsafe);

fn main() {
    let mut callbacks = Callbacks::new();
    push_dstvec!(callbacks, ||{
        println!("YOLO!");
    });

    push_dstvec!(callbacks, ||{
        println!("What's up?");
    });

    for callback in callbacks.into_raw_iter() {
        unsafe {
            (&mut *callback).call_once_unsafe();
        }
    }
}
