use std::slice;

unsafe extern "C" {
    safe fn abs(input: i32) -> i32;
}

fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();

    assert!(mid <= len);
    let ptr = values.as_mut_ptr();
    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

fn main() {
    let mut num = 5;

    // creating a pointer is not an unsafe operation
    // note that we created a inmutable and mutable pointer
    // with in the same scope. If we were to try doing the
    // same with reference, the Rust compalier won't let us
    let r1 = &raw const num;
    let r2 = &raw mut num;

    // derefence the pointer is the unsafe operation
    unsafe {
        println!("r1 is: {}", *r1);
        println!("r2 is: {}", *r2);
    }

    let mut v = vec![1, 2, 3, 4, 5, 6];

    let r = &mut v[..];

    let (a, b) = split_at_mut(r, 3);

    assert_eq!(a, &mut [1, 2, 3]);
    assert_eq!(b, &mut [4, 5, 6]);

    println!("Absolute value of -3 according to C: {}", abs(-3));
}
