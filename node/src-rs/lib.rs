use hyper_function_core;
use napi::bindgen_prelude::*;

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn init(buf: Buffer) -> Buffer {
    hyper_function_core::init(buf.into()).into()
}

#[napi]
pub fn run() {
    hyper_function_core::run();
}

#[napi]
pub async fn read() -> Buffer {
    let data = hyper_function_core::read_async().await.unwrap();

    data.into()
}

#[napi]
pub fn send_message(socket_id: String, payload: Buffer) {
    hyper_function_core::send_message(socket_id, payload.into());
}
