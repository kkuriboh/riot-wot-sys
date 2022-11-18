// rewrite of RIOT/tests/wot_coap_generation
#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::panic::PanicInfo;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use lazy_static::lazy_static;
use riot_wot_sys::{
	ffi::*,
	inline::{coap_get_code_detail, coap_get_uri_path, coap_method2flag, gcoap_response},
};
use riot_wrappers::{cstr::cstr, led::LED, mutex::Mutex, println};

/*#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	loop {}
}*/

// const NULL: *const impl Any = core::ptr::null();
const MAIN_QUEUE_SIZE: u8 = 4;

struct LedControls<const GPIO: u8> {
	state: bool,
	led: LED<GPIO>,
}

impl<const GPIO: u8> LedControls<GPIO> {
	const fn new() -> Self {
		assert!(GPIO <= 7);
		Self {
			led: LED::<GPIO>::new(),
			state: false,
		}
	}

	fn is_led_on(&self) -> bool {
		println!("the led is on");
		self.state
	}

	fn get_led_status(&self) -> &'static str {
		if self.is_led_on() {
			return "On";
		}
		"Off"
	}

	fn turn_on_led(&mut self) {
		println!("led turned on");
		self.led.set_high();
		self.state = true;
	}

	fn turn_off_led(&mut self) {
		println!("led turned off");
		self.led.set_low();
		self.state = false;
	}

	fn toggle_led(&mut self) {
		println!("led switched to: {}", !self.state);
		self.led.toggle();
		self.state = !self.state;
	}

	fn led_cmd(&mut self, argv: &str) -> Result<&str, &str> {
		// bad usage of rust pattern matching, this should be an enum,
		// using &str because of the api. if you are using this piece of code as an "follow through",
		// try converting it to an enum or using wrappers
		match argv {
			"on" => self.turn_on_led(),
			"off" => self.turn_off_led(),
			"toggle" => self.toggle_led(),
			"status" => return Ok(self.get_led_status()),
			_ => {
				if argv.len() <= 0 {
					let message = "Usage: led [on|off|toggle|status]";
					println!("{}", message);
					return Err(message);
				} else {
					let message = "led: Invalid command";
					println!("{}", message);
					return Err(message);
				}
			}
		}
		let message = "Command ran successfully";
		println!("{}", message);
		Ok(message)
	}

	fn echo_handler(&self, pkt: &mut coap_pkt_t, buf: &mut u8, len: u32) -> usize {
		let mut uri = String::new();

		if coap_get_uri_path(pkt, &mut uri) <= 0 {
			return unsafe {
				coap_reply_simple(
					pkt,
					COAP_CODE_INTERNAL_SERVER_ERROR,
					buf,
					len,
					COAP_FORMAT_JSON,
					core::ptr::null(),
					0,
				)
				.try_into()
				.unwrap()
			};
		}

		// let binding = [uri.as_bytes(), "/echo/".as_bytes()].concat();
		// let sub_uri = core::str::from_utf8(&binding).unwrap();
		let sub_uri = uri + "/echo/";
		let sub_uri_len = sub_uri.len();

		unsafe {
			coap_reply_simple(
				pkt,
				COAP_CODE_CONTENT,
				buf,
				len,
				COAP_FORMAT_JSON,
				sub_uri.as_bytes().as_ptr() as *const u8,
				sub_uri_len.try_into().unwrap(),
			)
			.try_into()
			.unwrap()
		}
	}
}

extern "C" {
	static mut wot_thing: wot_td_thing_t;
}

struct SendMutPtr<T>(*mut T);

impl<T> SendMutPtr<T> {
	pub fn new(data: *mut T) -> Self {
		Self(data)
	}
	pub fn get(&self) -> *mut T {
		self.0.clone()
	}
}

unsafe impl<T> Send for SendMutPtr<T> {}
unsafe impl<T> Sync for SendMutPtr<T> {}

static LED_CONTROLS: Mutex<LedControls<7>> = Mutex::new(LedControls::new());
lazy_static! {
	static ref TOGGLE_COAP_AFFORDANCE: SendMutPtr<wot_td_coap_action_affordance_t> =
		unsafe { SendMutPtr::new(wot_td_coap_find_action(cstr!("toggle").as_ptr())) };
}

#[no_mangle]
pub fn led_status_handler(pdu: &mut coap_pkt_t, buf: &mut u8, len: u32) -> i64 {
	let led_status = LED_CONTROLS.lock().get_led_status();
	let resp_len = unsafe {
		gcoap_resp_init(pdu, buf, len, COAP_CODE_CONTENT);
		coap_opt_add_uint(pdu, COAP_OPT_CONTENT_FORMAT as u16, COAP_FORMAT_TEXT);
		coap_opt_finish(pdu, COAP_OPT_FINISH_PAYLOAD as u16)
	};
	if pdu.payload_len as usize >= led_status.len() {
		pdu.payload = led_status.as_ptr() as *mut u8;
		return (resp_len + led_status.len() as i32) as i64;
	}
	println!("wot_coap: msg buffer too small");
	gcoap_response(pdu, buf, len.into(), COAP_CODE_INTERNAL_SERVER_ERROR)
}

#[no_mangle]
pub fn led_toggle_handler(pdu: &mut coap_pkt_t, buf: &mut u8, len: u32) -> i64 {
	let method_flag = coap_method2flag(unsafe { coap_get_code_detail(pdu) });
	if method_flag as u32 == COAP_POST {
		LED_CONTROLS.lock().toggle_led();
		return gcoap_response(pdu, buf, len.into(), COAP_CODE_CHANGED);
	}
	return 0;
}

#[no_mangle]
pub fn remove_add_endpoint_handler(pdu: &mut coap_pkt_t, buf: &mut u8, len: u32) -> i64 {
	let method_flag = coap_method2flag(unsafe { coap_get_code_detail(pdu) });
	if method_flag as u32 == COAP_POST {
		unsafe {
			wot_td_coap_action_affordance_toggle(TOGGLE_COAP_AFFORDANCE.get());
		}
		return gcoap_response(pdu, buf, len.into(), COAP_CODE_CHANGED);
	}
	return 0;
}

riot_wrappers::riot_main!(main);
fn main() {
	let mut main_msg_queue: Vec<msg_t> = vec![];
	unsafe {
		msg_init_queue(
			main_msg_queue.as_mut_ptr() as *mut msg_t,
			MAIN_QUEUE_SIZE.into(),
		);
		wot_td_coap_server_init();
	}
	println!("All up!");
}

#[cfg(test)]
mod tests {
	use coap::CoAPClient;

	#[test]
	fn call_coap_methods() {
		let res = CoAPClient::get("coap://127.0.0.1:5683/led_status").unwrap();
		println!(
			"Server reply: {}",
			String::from_utf8(res.message.payload).unwrap()
		);
	}
}
