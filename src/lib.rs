/*!
provides a Vec-like data structure for dynamically sized types
*/
#![feature(alloc, raw, unsize)]
extern crate alloc;
pub use alloc::raw_vec::RawVec;
pub use std::marker::Unsize;
pub use std::raw::TraitObject;

#[macro_export]
macro_rules! declare_dstvec {
    ($name:ident, $Trait:ty) => {
        use ::std::{ptr, mem};
        use $crate::Unsize;
        use $crate::RawVec;
        use $crate::TraitObject;
        use ::std::vec;

        pub struct $name {
            pointers: Vec<(usize, *mut ())>,
            data: RawVec<u8>,
            used_bytes: usize,
        }

        impl $name {
            pub fn new() -> $name {
                let data = RawVec::new();
                $name {
                    pointers: Vec::new(),
                    data: data,
                    used_bytes: 0,
                }
            }

            pub fn push<U: Unsize<$Trait>>(&mut self, value: U) {
                let align = mem::align_of::<U>();
                let gap = self.used_bytes + align - (self.used_bytes % align);
                let size = mem::size_of::<U>();

                self.data.reserve(self.used_bytes, gap + size);
                self.pointers.reserve(1);

                unsafe {
                    let back = self.data.ptr().offset((gap + size) as isize);
                    ptr::copy_nonoverlapping(&value, back as *mut U, 1);
                }

                let offset = self.used_bytes + gap;
                let vtable = unsafe {
                    let fat_pointer = &value as &$Trait;
                    let trait_object: TraitObject = mem::transmute(fat_pointer);
                    trait_object.vtable
                };
                self.pointers.push((offset, vtable));

                self.used_bytes += gap + size;

                mem::forget(value);
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                for &(offset, vtable) in self.pointers.iter() {
                    unsafe {
                        let pointer = construct_fat_pointer(self.data.ptr(), offset, vtable);
                        ptr::drop_in_place(pointer);
                    }
                }
            }
        }

        impl IntoIterator for $name {
            type Item = *mut $Trait;
            type IntoIter = IntoIter;

            fn into_iter(mut self) -> IntoIter {
                let pointers = mem::replace(&mut self.pointers, Vec::new());
                let data = mem::replace(&mut self.data, RawVec::new());
                IntoIter {
                    pointers: pointers.into_iter(),
                    data: data,
                }
            }
        }

        pub struct IntoIter {
            pointers: vec::IntoIter<(usize, *mut ())>,
            data: RawVec<u8>,
        }

        impl Iterator for IntoIter {
            type Item = *mut $Trait;

            fn next(&mut self) -> Option<*mut $Trait> {
                self.pointers.next().map(|(offset, vtable)| {
                    unsafe {
                        construct_fat_pointer(self.data.ptr(), offset, vtable)
                    }
                })
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.pointers.size_hint()
            }
        }

        impl ExactSizeIterator for IntoIter {}

        impl Drop for IntoIter {
            fn drop(&mut self) {
                let offsets_with_vtables = mem::replace(&mut self.pointers, Vec::new().into_iter());
                for (offset, vtable) in offsets_with_vtables {
                    unsafe {
                        let pointer = construct_fat_pointer(self.data.ptr(), offset, vtable);
                        ptr::drop_in_place(pointer);
                    }
                }
            }
        }

        unsafe fn construct_fat_pointer(base: *mut u8, offset: usize, vtable: *mut ()) -> *mut $Trait {
            mem::transmute(TraitObject {
                data: base.offset(offset as isize) as *mut (),
                vtable: vtable,
            })
        }
    };
}

#[cfg(test)]
mod tests {
    use std::ptr;

    pub trait FnOnceUnsafe {
        unsafe fn call_once_unsafe(&mut self);
    }

    impl<F: FnOnce()> FnOnceUnsafe for F {
        unsafe fn call_once_unsafe(&mut self) {
            ptr::read(self)()
        }
    }

    mod callbacks {
        use super::FnOnceUnsafe;
        declare_dstvec!(Callbacks, FnOnceUnsafe);
    }

    use self::callbacks::Callbacks;

    #[test]
    fn test_closures() {

        let mut foo = 5;
        let mut bar = 10;


        let mut callbacks = Callbacks::new();
        callbacks.push(|| {
            foo = 20;
        });

        callbacks.push(|| {
            bar = 30;
        });

        for callback in callbacks {
            unsafe {
                (*callback).call_once_unsafe();
            }
        }

        assert_eq!(foo, 20);
        assert_eq!(bar, 30);
    }
}
