mod codec;

use std::{env, fs::read_to_string, path::Path};

use once_cell::sync::{Lazy, OnceCell};
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc::{channel, Receiver, Sender},
};

#[derive(Debug)]
pub struct Instance {
    runtime: Runtime,
}

pub static INSTANCE: OnceCell<Instance> = OnceCell::new();

static CHAN: Lazy<(Sender<u32>, Receiver<u32>)> = Lazy::new(|| channel::<u32>(100));

pub fn init(opts: Vec<u8>) -> Vec<u8> {
    // currently we only support one instance
    if INSTANCE.get().is_some() {
        panic!("Instance already initialized");
    }

    let opts = codec::InitArgs::from_buf(opts);
    let mut config_path;

    if env::var("HFN_CONFIG_PATH").is_ok() {
        let path = env::var("HFN_CONFIG_PATH").unwrap();
        config_path = Path::new(&path).to_owned();
    } else if let Some(hfn_config_path) = opts.hfn_config_path {
        config_path = Path::new(&hfn_config_path).to_owned();
    } else {
        config_path = env::current_dir().unwrap();
        config_path.push("hfn.json");
    }

    if !config_path.exists() {
        panic!("hfn.json file not found: {}", config_path.display());
    }

    let json_config =
        codec::JsonConfig::from_str(read_to_string(config_path).expect("failed to read hfn.json"));

    let (hfn_packages, hfn_modules, hfn_models, hfn_hfns, hfn_rpcs, hfn_schemas, hfn_fields) =
        json_config.to_hfn_struct();

    let mut runtime_builder = Builder::new_multi_thread();

    if let Some(tokio_work_threads) = opts.tokio_work_threads {
        runtime_builder.worker_threads(tokio_work_threads);
    }

    runtime_builder.enable_all();
    let runtime = runtime_builder.build().expect("unable build tokio runtime");

    let instance = Instance { runtime };
    INSTANCE.set(instance).expect("unable to set instance");

    let result = codec::InitResult {
        packages: hfn_packages,
        modules: hfn_modules,
        models: hfn_models,
        hfns: hfn_hfns,
        rpcs: hfn_rpcs,
        schemas: hfn_schemas,
        fields: hfn_fields,
    };
    result.to_buf()
}
