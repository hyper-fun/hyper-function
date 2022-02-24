use hyper_function_core::{self, TryReadRes};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pymodule]
fn hfn_core(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m)]
    fn init<'a>(py: Python<'a>, buf: Vec<u8>) -> PyResult<&'a PyBytes> {
        let result = hyper_function_core::init(buf);
        let result = PyBytes::new(py, &result);

        Ok(result)
    }

    #[pyfn(m)]
    fn run() {
        hyper_function_core::run();
    }

    #[pyfn(m)]
    fn read(py: Python) -> PyResult<Vec<u8>> {
        match hyper_function_core::try_read() {
            TryReadRes::DATA(buf) => Ok(buf),
            TryReadRes::EMPTY => py.allow_threads(|| {
                let result = hyper_function_core::read().unwrap();

                Ok(result)
            }),
            TryReadRes::CLOSED => Ok(vec![]),
        }
    }

    #[pyfn(m)]
    pub fn send_message(socket_id: String, payload: Vec<u8>) {
        hyper_function_core::send_message(socket_id, payload);
    }

    Ok(())
}
