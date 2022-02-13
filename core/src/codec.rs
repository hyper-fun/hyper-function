use std::io::Cursor;

use rmp_serde;
use serde::{Deserialize, Serialize};

// Deserialize init options
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct InitArgs {
    pub dev: bool,
    pub hfn_config_path: Option<String>,
    pub tokio_work_threads: Option<usize>,
}

impl InitArgs {
    pub fn from_buf(data: Vec<u8>) -> Self {
        let cur = Cursor::new(&data);
        let mut de = rmp_serde::Deserializer::new(cur);
        let opts: InitArgs = Deserialize::deserialize(&mut de).expect("failed to parse init args");
        opts
    }
}

// Deserialize hfn.json
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfig {
    pub name: String,
    pub description: Option<String>,
    pub appid: String,
    pub dev: JsonConfigDev,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub packages: Vec<JsonConfigPackage>,
}

impl JsonConfig {
    pub fn from_str(data: String) -> Self {
        let config: JsonConfig = serde_json::from_str(&data).expect("failed to parse hfn.json");
        config
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfigDev {
    pub devtools: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfigPackage {
    pub id: u32,
    pub name: String,
    pub modules: Vec<JsonConfigPackageModule>,
    pub schemas: Vec<JsonConfigPackageSchema>,
    pub rpcs: Vec<JsonConfigPackageRpc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfigPackageModule {
    pub id: u32,
    pub name: String,
    pub models: Vec<JsonConfigPackageModuleModel>,
    pub hfns: Vec<JsonConfigPackageModuleHfn>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfigPackageModuleModel {
    pub id: u32,
    pub name: String,
    #[serde(rename = "schemaId")]
    pub schema_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfigPackageModuleHfn {
    pub id: u32,
    pub name: String,
    #[serde(rename = "schemaId")]
    pub schema_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfigPackageRpc {
    pub id: u32,
    pub name: String,
    #[serde(rename = "reqSchemaId")]
    pub req_schema_id: u32,
    #[serde(rename = "resSchemaId")]
    pub res_schema_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfigPackageSchema {
    pub id: u32,
    pub fields: Vec<JsonConfigPackageSchemaField>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfigPackageSchemaField {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub t: String,
    #[serde(rename = "isArray")]
    pub is_array: bool,
}
