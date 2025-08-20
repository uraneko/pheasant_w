#![no_main]

use libfuzzer_sys::fuzz_target;
use pheasant::Request;

fuzz_target!(|data: &[u8]| {
    if let Ok(i) = std::str::from_utf8(data) {
        println!("{:#?}", Request::parse_from(i.to_string()).unwrap());
    }
});
