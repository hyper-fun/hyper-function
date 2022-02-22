use dashmap::DashMap;
use gateway::gateway::Gateway;
use rusty_ulid::generate_ulid_string;
use server::{server::Server, socket::Socket};
use std::{env, fs::read_to_string, path::Path, sync::Arc};

use once_cell::sync::OnceCell;
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc,
};

mod codec;
mod gateway;
mod server;

pub static mut APP_ID: String = String::new();
pub static mut UPSTREAM_ID: String = String::new();

pub static RUNTIME: OnceCell<Runtime> = OnceCell::new();
pub static SOCKETS: OnceCell<DashMap<String, Socket>> = OnceCell::new();

pub static mut READ_CHAN_RX: OnceCell<mpsc::UnboundedReceiver<Vec<u8>>> = OnceCell::new();
pub static READ_CHAN_TX: OnceCell<mpsc::UnboundedSender<Vec<u8>>> = OnceCell::new();

pub static mut WRITE_CHAN_RX: OnceCell<mpsc::UnboundedReceiver<(String, Vec<u8>)>> =
    OnceCell::new();
pub static WRITE_CHAN_TX: OnceCell<mpsc::UnboundedSender<(String, Vec<u8>)>> = OnceCell::new();

pub static INIT_ARGS: OnceCell<codec::InitArgs> = OnceCell::new();
pub static JSON_CONFIG: OnceCell<codec::JsonConfig> = OnceCell::new();

pub fn init(args: Vec<u8>) -> Vec<u8> {
    if RUNTIME.get().is_some() {
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
    RUNTIME.set(runtime).unwrap();

    let (hfn_packages, hfn_modules, hfn_models, hfn_hfns, hfn_rpcs, hfn_schemas, hfn_fields) =
        json_config.to_hfn_struct();

    let upstream_id;
    if let Some(id) = &args.upstream_id {
        upstream_id = id.to_owned();
    } else {
        upstream_id = generate_ulid_string();
    }

    let (read_tx, read_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let (write_tx, write_rx) = mpsc::unbounded_channel::<(String, Vec<u8>)>();

    unsafe {
        APP_ID = json_config.appid.clone();
        UPSTREAM_ID = upstream_id.clone();

        READ_CHAN_RX.set(read_rx).unwrap();
        WRITE_CHAN_RX.set(write_rx).unwrap();
    }

    READ_CHAN_TX.set(read_tx).unwrap();
    WRITE_CHAN_TX.set(write_tx).unwrap();

    SOCKETS.set(DashMap::new()).unwrap();
    INIT_ARGS.set(args).unwrap();
    JSON_CONFIG.set(json_config).unwrap();

    let result = codec::InitResult {
        upstream_id,
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
    let init_args = INIT_ARGS.get().unwrap();
    let json_config = JSON_CONFIG.get().unwrap();
    let upstream_id = unsafe { UPSTREAM_ID.clone() };

    let runtime = RUNTIME.get().unwrap();

    let read_tx = READ_CHAN_TX.get().unwrap().clone();
    if !init_args.dev {
        let addr = init_args.addr.as_ref().unwrap().clone();
        runtime.spawn(async move {
            let server = Server { addr };
            server.listen().await
        });
    } else {
        let mut url = url::Url::parse(&json_config.dev.devtools).unwrap();
        url.set_path("/us");

        url.query_pairs_mut().append_pair("usid", &upstream_id);

        url.query_pairs_mut()
            .append_pair("appid", &json_config.appid);

        url.query_pairs_mut()
            .append_pair("ver", env!("CARGO_PKG_VERSION"));

        url.query_pairs_mut().append_pair("sdk", &init_args.sdk);

        let gateway = Gateway {
            dev: true,
            runway: url,
            read_tx,
        };

        runtime.spawn(async move {
            gateway.connect().await;
        });

        // todo add package signature for querystring
    }
}

pub fn read() {}

pub async fn read_async() -> Option<Vec<u8>> {
    let read_rx = unsafe { READ_CHAN_RX.get_mut().unwrap() };
    let data = read_rx.recv().await;

    data
}

pub fn send_message(socket_id: String, payload: Vec<u8>) {
    let sockets = SOCKETS.get().unwrap();
    if let Some(socket) = sockets.get(&socket_id) {
        // socket.send(payload).unwrap();
        socket.write_chan_tx.send(payload).unwrap();
        return;
    }

    let write_tx = WRITE_CHAN_TX.get().unwrap();
    write_tx.send((socket_id, payload)).unwrap();
}
