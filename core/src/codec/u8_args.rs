use std::io::Cursor;

use rmp_serde;
use serde::{Deserialize, Serialize};

// Deserialize init options
#[derive(Debug, Deserialize)]
pub struct InitArgs {
    pub dev: bool,
    pub sdk: String,
    pub upstream_id: Option<String>,
    pub pkg_names: Vec<String>,
    pub hfn_config_path: Option<String>,
    pub tokio_work_threads: Option<usize>,
}

impl InitArgs {
    pub fn from_buf(data: Vec<u8>) -> Self {
        let mut de = rmp_serde::Deserializer::new(Cursor::new(&data));
        Deserialize::deserialize(&mut de).expect("failed to parse init args")
    }
}

#[derive(Debug, Serialize)]
pub struct InitResult {
    pub upstream_id: String,
    pub packages: Vec<HfnPackage>,
    pub modules: Vec<HfnModule>,
    pub models: Vec<HfnModel>,
    pub hfns: Vec<HfnHfn>,
    pub rpcs: Vec<HfnRpc>,
    pub schemas: Vec<HfnSchema>,
    pub fields: Vec<HfnField>,
}

impl InitResult {
    pub fn to_buf(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut ser = rmp_serde::Serializer::new(&mut buf).with_struct_map();
        self.serialize(&mut ser)
            .expect("failed to serialize init result");
        buf
    }
}

#[derive(Debug, Serialize)]
pub struct HfnPackage {
    pub id: u32,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HfnModule {
    pub id: u32,
    pub name: String,
    pub package_id: u32,
}

#[derive(Debug, Serialize)]
pub struct HfnModel {
    pub id: u32,
    pub name: String,
    pub schema_id: u32,
    pub package_id: u32,
    pub module_id: u32,
}

#[derive(Debug, Serialize)]
pub struct HfnHfn {
    pub id: u32,
    pub name: String,
    pub schema_id: u32,
    pub package_id: u32,
    pub module_id: u32,
}

#[derive(Debug, Serialize)]
pub struct HfnRpc {
    pub id: u32,
    pub name: String,
    pub req_schema_id: u32,
    pub res_schema_id: u32,
    pub package_id: u32,
}

#[derive(Debug, Serialize)]
pub struct HfnSchema {
    pub id: u32,
    pub package_id: u32,
}

#[derive(Debug, Serialize)]
pub struct HfnField {
    pub id: u32,
    pub name: String,
    pub t: String,
    pub is_array: bool,
    pub package_id: u32,
    pub schema_id: u32,
}
