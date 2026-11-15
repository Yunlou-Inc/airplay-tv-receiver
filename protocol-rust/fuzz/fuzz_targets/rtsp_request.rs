#![no_main]
use airplay_protocol::{parse_request, Limits};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = Limits { max_request_bytes: 8 * 1024, max_start_line_bytes: 1024,
        max_header_count: 32, max_header_line_bytes: 2048, max_body_bytes: 4096 };
    let _ = parse_request(data, limits);
});
