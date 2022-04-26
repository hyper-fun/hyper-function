use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::gateway::transport::Packet;

use super::transport::{PacketMessage, Transport};

pub struct Gateway {
    pub dev: bool,
    pub runway: url::Url,
    pub read_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl Gateway {
    pub async fn connect(&self, mut write_rx: mpsc::UnboundedReceiver<(String, Vec<u8>)>) {
        let stream = Transport::connect(self.runway.clone())
            .await
            .expect("failed to connect to devtools");

        let (mut sink, mut stream) = stream.split();

        let sink_task = tokio::spawn(async move {
            while let Some(data) = write_rx.recv().await {
                Transport::send_message(&mut sink, data).await.unwrap();
            }
        });

        let read_tx = self.read_tx.clone();
        while let Some(packets) = Transport::next(&mut stream).await {
            for packet in packets {
                match packet {
                    Packet::OPEN(open) => {
                        println!("gateway open: {:?}", open);
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
                    Packet::REDIRECT(redirect) => {
                        println!("redirect: {:?}", redirect);
                    }
                    Packet::MESSAGE(msg) => {
                        let data = Gateway::encode_message(msg);
                        read_tx
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
        println!("Devtools connection closed");
    }

    fn encode_message(mut msg: PacketMessage) -> Vec<u8> {
        let mut cap = 4 + 2 + msg.payload.len() + 2 + msg.socket_id.len();

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
        rmp::encode::write_str_len(&mut data, msg.socket_id.len() as u32).unwrap();
        data.append(&mut msg.socket_id);

        data
    }
}
