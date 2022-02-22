use futures_util::{SinkExt, StreamExt};
use hyper::upgrade::Upgraded;
use hyper_tungstenite::{tungstenite::Message, WebSocketStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::transport::{Packet, PacketMessage, Transport};

#[derive(Debug)]
pub struct Socket {
    pub id: String,
    pub client_id: String,
    pub session_id: String,
    pub client_ts: u64,
    pub client_version: String,
    pub write_chan_tx: UnboundedSender<Vec<u8>>,
}

impl Socket {
    pub async fn accept_ws(
        &self,
        stream: WebSocketStream<Upgraded>,
        read_chan_tx: UnboundedSender<Vec<u8>>,
        mut write_chan_rx: UnboundedReceiver<Vec<u8>>,
    ) {
        let (mut sink, mut stream) = stream.split();

        Transport::send_open_packet(&mut sink, 25, 20)
            .await
            .unwrap();

        let sink_task = tokio::spawn(async move {
            while let Some(data) = write_chan_rx.recv().await {
                Transport::send_message_packet(&mut sink, data)
                    .await
                    .unwrap();
            }
        });

        let socket_id = self.id.clone();
        let stream_task = tokio::spawn(async move {
            while let Some(packets) = Transport::next(&mut stream).await {
                for packet in packets {
                    match packet {
                        Packet::OPEN(open) => {
                            println!("open: {:?}", open);
                        }
                        Packet::CLOSE(close) => {
                            println!("close: {:?}", close);
                        }
                        Packet::PING(ping) => {
                            println!("ping: {:?}", ping);
                        }
                        Packet::PONG(pong) => {
                            println!("pong: {:?}", pong);
                        }
                        Packet::RETRY(retry) => {
                            println!("retry: {:?}", retry);
                        }
                        Packet::RESET(reset) => {
                            println!("reset: {:?}", reset);
                        }
                        Packet::REDIRECT(redirect) => {
                            println!("redirect: {:?}", redirect);
                        }
                        Packet::MESSAGE(msg) => {
                            let data = Socket::encode_message(&socket_id, msg);
                            read_chan_tx
                                .send(data)
                                .expect("failed to send message to read_tx");
                        }
                        Packet::ACK(ack) => {
                            println!("ack: {:?}", ack);
                        }
                    }
                }
            }

            sink_task.abort();
        });

        stream_task.await.unwrap();
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
