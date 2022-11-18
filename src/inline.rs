// C/C++ inline functions mostly dont work when called from Rust
// the Rust equivalent would be something like macros or rust inline functions
// this module is dedicated to rewriting RIOT_WOT inline functions

use crate::ffi::*;

#[inline]
pub fn gcoap_response(pdu: &mut coap_pkt_t, buf: &mut u8, len: u64, code: u32) -> i64 {
	#[cfg(target_pointer_width = "32")]
	let len = len as u32;
	unsafe {
		if gcoap_resp_init(pdu, buf, len, code) == 0 {
			#[cfg(target_pointer_width = "32")]
			return coap_opt_finish(pdu, COAP_OPT_FINISH_NONE as u16).into();

			#[cfg(target_pointer_width = "64")]
			return coap_opt_finish(pdu, COAP_OPT_FINISH_NONE as u16);
		}
	}
	-1
}

#[inline]
pub fn coap_get_uri_path(pkt: &mut coap_pkt_t, target: &mut str) -> i64 {
	let res = unsafe {
		coap_opt_get_string(
			pkt,
			COAP_OPT_URI_PATH as u16,
			target.as_bytes_mut().as_mut_ptr(),
			CONFIG_NANOCOAP_URI_MAX.into(),
			'/' as core::ffi::c_char,
		)
	};

	#[cfg(target_pointer_width = "32")]
	return res.into();

	#[cfg(target_pointer_width = "64")]
	res
}

#[inline]
pub fn coap_method2flag(code: i64) -> coap_method_flags_t {
	1 << (code - 1)
}

// this function must be unsafe because of dereferenced raw pointer
/// # Safety
/// the raw pointer is not being passed to any other function
#[inline]
pub unsafe fn coap_get_code_detail(pkt: *const coap_pkt_t) -> i64 {
	let code = (*(*pkt).hdr).code;

	#[cfg(target_pointer_width = "32")]
	return (code & 0x1f).into();

	// using return because attributes on expressions are experimental
	#[cfg(target_pointer_width = "64")]
	return (code & 0x1f) as i64;
}
