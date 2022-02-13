use hyper_function_core;
use napi::bindgen_prelude::Buffer;

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn init(buf: Buffer) -> Buffer {
    hyper_function_core::init(buf.into());

    Buffer::from(vec![0xab])
}
