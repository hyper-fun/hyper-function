use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use futures_util::StreamExt;
use hyper::upgrade::Upgraded;
use hyper_tungstenite::WebSocketStream;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::sleep,
};

use super::transport::{Packet, PacketMessage, Transport};

#[derive(Debug)]
pub struct Socket {
    pub id: String,
    pub client_id: String,
    pub session_id: String,
    pub client_ts: u64,
    pub client_version: String,
}

pub enum Action {
    SendOpen(ActionSendOpen),
    SendPing(ActionSendPing),
    SendMessage(ActionSendMessage),
    Close(ActionClose),
}

#[derive(Debug)]
pub struct ActionSendOpen {
    pub ping_interval: i64,
    pub ping_timeout: i64,
}

#[derive(Debug)]
pub struct ActionSendPing {}

#[derive(Debug)]
pub struct ActionSendMessage {
    pub payload: Vec<u8>,
}

pub struct ActionClose {}

impl Socket {
    pub async fn accept_ws(
        &self,
        stream: WebSocketStream<Upgraded>,
        read_chan_tx: UnboundedSender<Vec<u8>>,
        socket_write_chan_tx: UnboundedSender<Action>,
        mut socket_write_chan_rx: UnboundedReceiver<Action>,
    ) {
        let (mut sink, mut stream) = stream.split();

        socket_write_chan_tx.send(Action::SendOpen(ActionSendOpen {
            ping_interval: 25,
            ping_timeout: 20,
        }));

        let (close_tx, mut close_rx) = mpsc::channel::<()>(1);

        let close_tx_clone = close_tx.clone();
        let sink_task = tokio::spawn(async move {
            while let Some(action) = socket_write_chan_rx.recv().await {
                match action {
                    Action::SendOpen(action) => {
                        Transport::send_open_packet(
                            &mut sink,
                            action.ping_interval,
                            action.ping_timeout,
                        )
                        .await;
                    }
                    Action::SendPing(action) => {
                        Transport::send_ping_packet(&mut sink).await;
                    }
                    Action::SendMessage(action) => {
                        Transport::send_message_packet(&mut sink, action.payload).await;
                    }
                    Action::Close(action) => {
                        Transport::send_close_packet(&mut sink, "").await;
                        close_tx_clone.send(()).await;
                    }
                }
            }
        });

        let socket_id = self.id.clone();
        let last_heartbeat = Arc::new(Mutex::new(chrono::Utc::now().timestamp()));

        let last_heartbeat_clone = last_heartbeat.clone();
        let close_tx_clone = close_tx.clone();

        let stream_task = tokio::spawn(async move {
            while let Some(packets) = Transport::next(&mut stream).await {
                for packet in packets {
                    match packet {
                        Packet::OPEN(open) => {
                            // println!("socket open: {:?}", open);
                        }
                        Packet::CLOSE(close) => {
                            close_tx_clone.send(()).await;
                            return;
                        }
                        Packet::PING(ping) => {
                            // println!("ping: {:?}", ping);
                        }
                        Packet::PONG(pong) => {
                            // println!("pong: {:?}", pong);
                        }
                        Packet::MESSAGE(msg) => {
                            let data = Socket::encode_message(&socket_id, msg);
                            read_chan_tx
                                .send(data)
                                .expect("failed to send message to read_tx");
                        }
                        Packet::ACK(ack) => {
                            // println!("ack: {:?}", ack);
                        }
                        // nothing todo
                        _ => {}
                    }
                }

                *last_heartbeat_clone.lock().unwrap() = chrono::Utc::now().timestamp();
            }

            close_tx_clone.send(()).await;
        });

        let socket_write_chan_tx = socket_write_chan_tx.clone();

        let close_tx_clone = close_tx.clone();
        let heartbeat_task = tokio::spawn(async move {
            loop {
                let now = chrono::Utc::now().timestamp();
                let heartbeat_at = *last_heartbeat.lock().unwrap();

                if now - heartbeat_at > 25 + 20 {
                    close_tx_clone.send(()).await;
                    return;
                }

                // send ping
                socket_write_chan_tx.send(Action::SendPing(ActionSendPing {}));
                sleep(Duration::from_secs(25)).await
            }
        });

        close_rx.recv().await;

        stream_task.abort();
        sink_task.abort();
        heartbeat_task.abort();
    }

    fn encode_message(socket_id: &str, mut msg: PacketMessage) -> Vec<u8> {
        let mut cap = 4 + 2 + msg.payload.len() + 2 + socket_id.len();

        cap += 2;
        if !msg.headers.is_empty() {
            for header in &msg.headers {
                cap += 5;
                cap += header.len();
            }
        }
        let mut data: Vec<u8> = Vec::with_capacity(cap);

        rmp::encode::write_sint(&mut data, msg.pkg_id as i64).unwrap();

        if msg.headers.is_empty() {
            rmp::encode::write_map_len(&mut data, 0).unwrap();
        } else {
            rmp::encode::write_map_len(&mut data, (msg.headers.len() / 2) as u32).unwrap();
            msg.headers.chunks_mut(2).for_each(|chunk| {
                rmp::encode::write_str_len(&mut data, chunk[0].len() as u32).unwrap();
                data.append(&mut chunk[0]);
                rmp::encode::write_str_len(&mut data, chunk[1].len() as u32).unwrap();
                data.append(&mut chunk[1]);
            });
        }

        rmp::encode::write_bin(&mut data, &msg.payload).unwrap();
        rmp::encode::write_str(&mut data, socket_id).unwrap();

        data
    }
}
