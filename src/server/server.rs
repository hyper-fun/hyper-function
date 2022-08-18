use std::{collections::HashMap, convert::Infallible};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server as HyperServer, StatusCode,
};
use rusty_ulid::generate_ulid_string;
use tokio::sync::mpsc;

use crate::{APP_ID, READ_CHAN_TX, SOCKET_CHANS};

use super::socket::{Action, Socket};

pub struct Server {
    pub addr: String,
}

impl Server {
    pub async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Infallible> {
        if request.uri().path().eq("/hfn") {
            let bad_request = || {
                println!("bad request");
                return Ok(Response::builder()
                    .status(400)
                    .body(Body::from("Bad Request"))
                    .unwrap());
            };

            if !hyper_tungstenite::is_upgrade_request(&request) {
                return bad_request();
            }

            let qs = match request.uri().query() {
                Some(v) => v,
                None => return bad_request(),
            };

            let query: HashMap<String, String> = url::form_urlencoded::parse(&qs.as_bytes())
                .into_owned()
                .collect();

            let app_id = match query.get("aid") {
                Some(v) => v.to_string(),
                None => return bad_request(),
            };

            let client_id = match query.get("cid") {
                Some(v) => v.to_string(),
                None => return bad_request(),
            };

            let session_id = match query.get("sid") {
                Some(v) => v.to_string(),
                None => return bad_request(),
            };

            let client_version = match query.get("ver") {
                Some(v) => v.to_string(),
                None => return bad_request(),
            };

            let client_ts = match query.get("ts") {
                Some(v) => match v.parse::<u64>() {
                    Ok(v) => v,
                    Err(_) => return bad_request(),
                },
                None => return bad_request(),
            };

            if app_id.len() > 64
                || client_id.len() > 64
                || session_id.len() > 64
                || client_version.len() > 16
            {
                return bad_request();
            }

            if app_id.ne(unsafe { &APP_ID }) {
                return bad_request();
            }

            let (response, websocket) = match hyper_tungstenite::upgrade(request, None) {
                Ok(v) => v,
                Err(_) => return bad_request(),
            };

            tokio::spawn(async move {
                let stream = match websocket.await {
                    Ok(v) => v,
                    Err(_) => {
                        return ();
                    }
                };

                let socket = Socket {
                    id: generate_ulid_string(),
                    client_id,
                    session_id,
                    client_ts,
                    client_version,
                };

                let read_chan_tx = READ_CHAN_TX.get().unwrap().clone();
                let (socket_write_chan_tx, socket_write_chan_rx) =
                    mpsc::unbounded_channel::<Action>();

                let socket_id = socket.id.clone();
                let socket_chans = SOCKET_CHANS.get().unwrap();
                socket_chans.insert(socket_id.clone(), socket_write_chan_tx.clone());

                socket
                    .accept_ws(
                        stream,
                        read_chan_tx,
                        socket_write_chan_tx,
                        socket_write_chan_rx,
                    )
                    .await;

                // clean up
                socket_chans.remove(&socket_id);
            });

            // Return the response so the spawned future can continue.
            Ok(response)
        } else {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::NOT_FOUND;

            Ok(response)
        }
    }
    pub async fn listen(&self) {
        let addr: std::net::SocketAddr = self.addr.parse().expect("fail to parse addr");

        let server = HyperServer::bind(&addr).serve(make_service_fn(|_| async {
            Ok::<_, Infallible>(service_fn(Server::handle_request))
        }));

        println!("Listening on {}", self.addr);
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}
