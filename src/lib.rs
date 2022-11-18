#![no_std]

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(improper_ctypes)]
#[allow(clippy::missing_safety_doc)]
pub mod ffi {
	include!("../gen/bindings.rs");
}

pub mod inline;
