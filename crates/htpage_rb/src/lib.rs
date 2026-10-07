use rb_sys::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

unsafe extern "C" fn rb_htpage_compile(_klass: VALUE, rb_source: VALUE) -> VALUE {
    let ptr = rb_string_value_cstr(&rb_source as *const _ as *mut _);
    let c_str = CStr::from_ptr(ptr);
    let source = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            let err_msg = CString::new("Source code is not valid UTF-8").unwrap();
            rb_raise(rb_eStandardError, err_msg.as_ptr());
        }
    };

    match htpage_core::compile(source) {
        Ok(html) => {
            rb_utf8_str_new(html.as_ptr() as *const c_char, html.len() as std::os::raw::c_long)
        }
        Err(e) => {
            let err_msg = CString::new(e).unwrap();
            rb_raise(rb_eStandardError, err_msg.as_ptr());
        }
    }
}

/// Initializes the `Htpage` Ruby module and binds `Htpage.compile`.
///
/// # Safety
///
/// This function is invoked by the Ruby MRI runtime during gem extension initialization.
/// It interacts directly with Ruby C-API functions (`rb_define_module`, `rb_define_singleton_method`).
#[no_mangle]
pub unsafe extern "C" fn Init_htpage_rb() {
    let name = CString::new("Htpage").unwrap();
    let method = CString::new("compile").unwrap();
    let module = rb_define_module(name.as_ptr());
    rb_define_singleton_method(
        module,
        method.as_ptr(),
        Some(std::mem::transmute::<unsafe extern "C" fn(VALUE, VALUE) -> VALUE, unsafe extern "C" fn() -> VALUE>(rb_htpage_compile)),
        1,
    );
}
