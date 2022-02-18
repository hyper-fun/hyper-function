use gateway::gateway::Gateway;
use nanoid::nanoid;
use std::{env, fs::read_to_string, path::Path};

use once_cell::sync::OnceCell;
use tokio::runtime::{Builder, Runtime};

mod codec;
mod gateway;

#[derive(Debug)]
pub struct Instance {
    pub init_args: codec::InitArgs,
    pub json_config: codec::JsonConfig,
    pub upstream_id: String,
    pub runtime: Runtime,
}

pub static INSTANCE: OnceCell<Instance> = OnceCell::new();

// static CHAN: Lazy<(Sender<u32>, Receiver<u32>)> = Lazy::new(|| channel::<u32>(100));

pub fn init(args: Vec<u8>) -> Vec<u8> {
    // currently we only support one instance
    if INSTANCE.get().is_some() {
        panic!("Instance already initialized");
    }

    let args = codec::InitArgs::from_buf(args);
    let mut config_path;

    if env::var("HFN_CONFIG_PATH").is_ok() {
        let path = env::var("HFN_CONFIG_PATH").unwrap();
        config_path = Path::new(&path).to_owned();
    } else if let Some(hfn_config_path) = &args.hfn_config_path {
        config_path = Path::new(hfn_config_path).to_owned();
    } else {
        config_path = env::current_dir().unwrap();
        config_path.push("hfn.json");
    }

    if !config_path.exists() {
        panic!("hfn.json file not found: {}", config_path.display());
    }

    let json_config =
        codec::JsonConfig::from_str(read_to_string(config_path).expect("failed to read hfn.json"));

    let mut runtime_builder = Builder::new_multi_thread();

    if let Some(tokio_work_threads) = &args.tokio_work_threads {
        runtime_builder.worker_threads(*tokio_work_threads);
    }

    runtime_builder.thread_name("hfn-core-runtime-worker");
    runtime_builder.enable_all();
    let runtime = runtime_builder.build().expect("unable build tokio runtime");

    let (hfn_packages, hfn_modules, hfn_models, hfn_hfns, hfn_rpcs, hfn_schemas, hfn_fields) =
        json_config.to_hfn_struct();

    let upstream_id;
    if let Some(id) = &args.upstream_id {
        upstream_id = id.to_owned();
    } else {
        upstream_id = nanoid!();
    }

    INSTANCE
        .set(Instance {
            init_args: args,
            json_config,
            runtime,
            upstream_id: upstream_id.clone(),
        })
        .expect("unable to set instance");

    // let instance = INSTANCE.get().unwrap();

    let result = codec::InitResult {
        upstream_id: upstream_id.clone(),
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

pub fn run() {
    let instance = INSTANCE.get().unwrap();

    if instance.init_args.dev {
        let mut url = url::Url::parse(&instance.json_config.dev.devtools).unwrap();
        url.set_path("/us");

        url.query_pairs_mut()
            .append_pair("usid", &instance.upstream_id);

        url.query_pairs_mut()
            .append_pair("appid", &instance.json_config.appid);

        url.query_pairs_mut()
            .append_pair("ver", env!("CARGO_PKG_VERSION"));

        url.query_pairs_mut()
            .append_pair("sdk", &instance.init_args.sdk);

        instance.runtime.spawn(async move {
            let gateway = Gateway {
                dev: true,
                runway: url,
            };

            gateway.connect().await
        });

        // todo add package signature for querystring
    }
}
