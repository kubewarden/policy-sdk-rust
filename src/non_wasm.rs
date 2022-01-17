// This is a stub module for building targets other than wasm. This
// ensures that the functions used to implement the waPC protocol can
// be linked. This allows to run targets such as `cargo test` on host
// targets.

#[no_mangle]
pub fn __console_log(_ptr: *const u8, _len: usize) {}

#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub fn __host_call(
    _bd_ptr: *const u8,
    _bd_len: usize,
    _ns_ptr: *const u8,
    _ns_len: usize,
    _op_ptr: *const u8,
    _op_len: usize,
    _ptr: *const u8,
    _len: usize,
) -> usize {
    0
}

#[no_mangle]
pub fn __host_response(_ptr: *mut u8) {}

#[no_mangle]
pub fn __host_response_len() -> usize {
    0
}

#[no_mangle]
pub fn __host_error_len() -> usize {
    0
}

#[no_mangle]
pub fn __host_error(_ptr: *mut u8) {}

#[no_mangle]
pub fn __guest_response(_ptr: *const u8, _len: usize) {}

#[no_mangle]
pub fn __guest_error(_ptr: *const u8, _len: usize) {}

#[no_mangle]
pub fn __guest_request(_op_ptr: *mut u8, _ptr: *mut u8) {}
