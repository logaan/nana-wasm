#![allow(dead_code)]
#![allow(unused_imports)]

mod utils;

use wasm_bindgen::prelude::*;

mod main;
use main::eval_once_off;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert(format!("(+ 1 2) = {:?}", eval_once_off("(+ 1 2)")).as_str());
}
