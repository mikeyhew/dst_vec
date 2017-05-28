use std::ptr;

extern crate dst_vec;
use dst_vec::DSTVec;

#[macro_use] extern crate referent;

trait FnOnceUnsafe<'a> {
    unsafe fn call_once_unsafe(&'a mut self);
}

derive_referent!(FnOnceUnsafe<'a>, 'a);

impl<'a, F: FnOnce()> FnOnceUnsafe<'a> for F {
    unsafe fn call_once_unsafe(&'a mut self) {
        ptr::read(self)()
    }
}

type Callbacks<'a> = DSTVec<FnOnceUnsafe<'a>>;

#[test]
fn test_closures() {

    let mut foo = 5;
    let mut bar = 10;

    let p_foo = &mut foo as *mut _;
    let p_bar = &mut bar as *mut _;

    let mut callbacks = Callbacks::new();

    /// We need to use raw pointers because the borrow checker thinks these
    /// closures will outlive the current function.
    callbacks.push(move || {
        unsafe {
            *p_foo = 20;
        }
    });

    callbacks.push(move || {
        unsafe {
            *p_bar = 30;
        }
    });

    /// the code is safe because the closures are dropped here.
    for callback in callbacks {
        unsafe {
            (*callback).call_once_unsafe();
        }
    }

    assert_eq!(foo, 20);
    assert_eq!(bar, 30);
}

// fn main() {
//     let mut callbacks = Callbacks::new();
//
//     callbacks.push(|| {
//         println!("Hello");
//     });
//
//     callbacks.push(|| {
//         println!("World!");
//     });
//
//     for callback in callbacks {
//         unsafe {
//             (*callback).call_once_unsafe();
//         }
//     }
// }
