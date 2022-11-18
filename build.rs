use std::{
	env::{self, set_current_dir},
	ffi::OsString,
	fs,
	path::{Path, PathBuf},
};

use fragile::Fragile;
use lazy_static::lazy_static;

// const DEPS: &[&str] = &["make", "gcc"];
lazy_static! {
	// try not to change this two statics.
	// lazy_static instantiate the value once it's first called,
	// by calling the HOME static on RIOT_BASE we are making sure that
	// it'll always be the actual home directory.
	// since we change directories a lot in this script
	static ref HOME: String = env::current_dir().unwrap().as_os_str().to_str().unwrap().to_string();
	static ref RIOT_BASE: String = if let Ok(base) = env::var("RIOTBASE") { base }
		else { return format!("{}/riotos",	*HOME); };

	//					NAMES		COMMON
	static ref CPUS: (Vec<String>, Vec<String>) = {
		set_current_dir(format!("{}/sys/include/net/wot", *RIOT_BASE)).unwrap();
		let data = fs::read_dir(PathBuf::from(RIOT_BASE.clone() + "/cpu"))
			.expect("unable to list")
			.map(|entry| entry.expect("unable to get entry").file_name().to_str().unwrap().to_string()).collect::<Vec<String>>();

		(data
			.clone()
			.into_iter()
			.filter(|val| !val.contains("common"))
			.filter(|val| !val.eq("doc.txt"))
			.filter(|val| !val.eq("Kconfig"))
			.collect(),
		data
			.into_iter()
			.filter(|val| val.contains("common"))
			.collect())
	};

	static ref BINDINGS: Fragile<bindgen::Builder> = {
		let mut include = vec![];
		find_dirs(
			Path::new(&*RIOT_BASE),
			&mut include,
			"include",
			&vec![
				&format!("{}/sys/include/net/wot", *RIOT_BASE),
				&format!("{}/build", *RIOT_BASE),
				&format!("{}/cpu", *RIOT_BASE),
				&format!("{}/boards", *RIOT_BASE),
			],
		);

		set_current_dir(format!("{}/sys/include/net/wot", *RIOT_BASE)).unwrap();
		let bindings = bindgen::builder()
			// .detect_include_paths(false)
			.use_core()
			.layout_tests(false)
			.parse_callbacks(Box::new(bindgen::CargoCallbacks))
			.header("coap/config.h")
			.header("security/apikey.h")
			.header("security/basic.h")
			.header("security/bearer.h")
			.header("security/digest.h")
			.header("security/oauth2.h")
			.header("security/psk.h")
			.header("serialization/io.h")
			.header("serialization/json_keys.h")
			.header("serialization/json_writer.h")
			.header("coap.h")
			.header("config.h")
			.header("persistence.h")
			.header("serialization.h")
			.clang_args(
				include
					.iter()
					.map(|dir| format!("-I{}", dir.to_str().unwrap())),
			);
		Fragile::new(bindings)
	};

}

fn find_dirs(dir: &Path, include: &mut Vec<OsString>, pattern: &str, blacklist: &Vec<&str>) {
	fs::read_dir(dir)
		.expect("unable to list")
		.for_each(|entry| {
			let entry = entry.expect("unable to get entry");
			if entry.file_type().unwrap().is_dir() {
				if entry.file_name().to_str().unwrap() == pattern {
					include.push(entry.path().into_os_string());
				} else if !blacklist.contains(&entry.path().to_str().unwrap()) {
					find_dirs(&entry.path(), include, pattern, blacklist);
				}
			}
		});
}

// TODO: add all the other boards
fn _add_board(bindings: bindgen::Builder, board: &str) -> bindgen::Builder {
	if board.contains("esp32") {
		// there's no rust target for the esp32 family
		return bindings;
	}

	let bindings = match board {
		"lpc23xx" => bindings.clang_arg(format!("-I{}/cpu/arm7_common/include", *RIOT_BASE)),
		&_ => bindings,
	};

	bindings.clang_arg(format!("-I{}/cpu/{}/include", *RIOT_BASE, board))
}

// fn build_riot() -> Result<(), String> {
// 	if env::var("DID_RUN").is_err() {
// 		env::set_var("DID_RUN", "1");
// 		println!("this crate depends on: {}", DEPS.join(", "));
// 		let mut cmd = Command::new("make");
// 		if let Ok(board) = &*BOARD {
// 			cmd.arg("BOARD=".to_string() + board);
// 		}
// 		let res = cmd.output().unwrap();
// 		if res.stderr.len() > 0 {
// 			let mut buffer = String::new();
// 			res.stderr.as_slice().read_to_string(&mut buffer).unwrap();
// 			return Err(buffer);
// 		}
// 	}
// 	Ok(())
// }

fn main() {
	env::set_var("RIOTBASE", &*RIOT_BASE);
	/*if let Err(err) = build_riot() {
		eprintln!("{err}");
		std::process::exit(1);
	}*/
	/*if env::var("GEN").is_err() {
		return;
	}*/

	let out_path = format!("{}/gen", *HOME);

	// WARN: currently this is just generating bindings for the native platform
	// add_board(BINDINGS.get().clone(), "native")
	BINDINGS
		.get()
		.clone()
		.clang_arg(format!("-I{}/cpu/native/include", *RIOT_BASE))
		.clang_arg(format!("-I{}/boards/native/include", *RIOT_BASE))
		.generate()
		.expect("Unable to generate bindings")
		.write_to_file(Path::new(&out_path).join("bindings.rs"))
		.expect("could not write bindings!");

	/*CPUS.0.iter().for_each(|entry| {
		let board = entry.to_owned();
		let bindings = add_board(BINDINGS.get().clone(), &board);
		bindings
			.generate()
			.expect("Unable to generate bindings")
			.write_to_file(Path::new(&out_path).join(board + ".rs"))
			.expect("could not write bindings!")
	});*/
}
