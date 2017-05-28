extern crate dst_vec;
use dst_vec::DSTVec;

#[test]
fn test_it_works() {
    let mut slices = DSTVec::<[i32]>::new();
    slices.push([1,2,3]);
    slices.push([4,5,6,7]);
    slices.push([]);
    slices.push([-1]);

    assert!(slices.iter().collect::<Vec<_>>() == &[
        &[1,2,3] as &[_],
        &[4,5,6,7],
        &[],
        &[-1],
    ]);
}
