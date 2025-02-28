use std::alloc::{alloc, dealloc, Layout};
use std::ffi::{c_char, CString};
use std::os::raw::c_int;
use std::ptr;

#[repr(C)]
pub struct Result {
    message: *mut c_char,
    len: c_int,
}

// Exported function that returns a dynamically allocated struct
#[no_mangle]
pub extern "C" fn Run() -> *mut Result {
    // Create the message as a CString (Rust equivalent of C string)
    let message = CString::new("Hello from Rust shared library!").unwrap();

    // Allocate memory for the Result struct on the heap
    let layout = Layout::new::<Result>();
    let result_ptr = unsafe { alloc(layout) as *mut Result };

    if result_ptr.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        (*result_ptr).message = message.into_raw(); // Transfer ownership to C
        (*result_ptr).len = 31;
    }

    result_ptr
}

// Function to free the memory allocated by `run`
#[no_mangle]
pub extern "C" fn FreeResult(result: *mut Result) {
    if result.is_null() {
        return;
    }

    unsafe {
        // Convert raw C string back to CString and drop it to free memory
        let _ = CString::from_raw((*result).message);

        // Free the struct itself
        let layout = Layout::new::<Result>();
        dealloc(result as *mut u8, layout);
    }
}
