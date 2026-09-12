use crate::app::gen_name;

pub fn init() -> (String, String, u16, bool, String) {
    // use std::str::FromStr;
    (
        gen_name(),
        web_sys::window().unwrap().location().hostname().unwrap(),
        // u16::from_str(
        //     web_sys::window()
        //         .unwrap()
        //         .location()
        //         .port()
        //         .unwrap()
        //         .as_str(),
        // )
        // .unwrap(),
        8080,
        #[cfg(not(debug_assertions))]
        true,
        #[cfg(debug_assertions)]
        false,
        #[cfg(not(debug_assertions))]
        "/dronoid/ws",
        #[cfg(debug_assertions)]
        "".to_string(),
    )
}
