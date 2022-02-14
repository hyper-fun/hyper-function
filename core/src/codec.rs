use std::io::Cursor;

use rmp_serde;
use serde::{Deserialize, Serialize};

// Deserialize init options
#[derive(Debug, Deserialize)]
pub struct InitArgs {
    pub dev: bool,
    pub sdk: String,
    pub pkg_names: Vec<String>,
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

// Deserialize hfn.json
#[derive(Deserialize, Debug)]
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

    pub fn to_hfn_struct(
        &self,
    ) -> (
        Vec<HfnPackage>,
        Vec<HfnModule>,
        Vec<HfnModel>,
        Vec<HfnHfn>,
        Vec<HfnRpc>,
        Vec<HfnSchema>,
        Vec<HfnField>,
    ) {
        let mut hfn_packages = vec![];
        let mut hfn_modules = vec![];
        let mut hfn_models = vec![];
        let mut hfn_hfns = vec![];
        let mut hfn_rpcs = vec![];
        let mut hfn_schemas = vec![];
        let mut hfn_fields = vec![];
        let _ = &self.packages.iter().for_each(|p| {
            hfn_packages.push(HfnPackage {
                id: p.id,
                name: p.name.clone(),
                full_name: p.full_name.clone(),
            });

            p.modules.iter().for_each(|m| {
                hfn_modules.push(HfnModule {
                    id: m.id,
                    name: m.name.clone(),
                    package_id: p.id,
                });

                m.models.iter().for_each(|model| {
                    hfn_models.push(HfnModel {
                        id: model.id,
                        name: model.name.clone(),
                        schema_id: model.schema_id,
                        package_id: p.id,
                        module_id: m.id,
                    });
                });

                m.hfns.iter().for_each(|hfn| {
                    hfn_hfns.push(HfnHfn {
                        id: hfn.id,
                        name: hfn.name.clone(),
                        schema_id: hfn.schema_id,
                        package_id: p.id,
                        module_id: m.id,
                    });
                });
            });

            p.rpcs.iter().for_each(|rpc| {
                hfn_rpcs.push(HfnRpc {
                    id: rpc.id,
                    name: rpc.name.clone(),
                    package_id: p.id,
                    req_schema_id: rpc.req_schema_id,
                    res_schema_id: rpc.res_schema_id,
                });
            });

            p.schemas.iter().for_each(|schema| {
                hfn_schemas.push(HfnSchema {
                    id: schema.id,
                    package_id: p.id,
                });

                schema.fields.iter().for_each(|field| {
                    hfn_fields.push(HfnField {
                        id: field.id,
                        name: field.name.clone(),
                        schema_id: schema.id,
                        package_id: p.id,
                        t: field.t.clone(),
                        is_array: field.is_array,
                    });
                });
            });
        });

        (
            hfn_packages,
            hfn_modules,
            hfn_models,
            hfn_hfns,
            hfn_rpcs,
            hfn_schemas,
            hfn_fields,
        )
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
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
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
