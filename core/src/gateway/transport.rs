use std::io::Cursor;

use futures_util::{stream::Map, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::error::Error, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

pub enum Packet {
    OPEN(PacketOpen),
    CLOSE(PacketClose),
    PING(PacketPing),
    PONG(PacketPong),
    RETRY(PacketRetry),
    REDIRECT(PacketRedirect),
    MESSAGE(PacketMessage),
    ACK(PacketAck),
}

pub struct PacketOpen {
    pub pi: u8, // ping interval second
    pub pt: u8, // ping timeout second
    pub cs: u8, // min compress size kb, message payload great than this should be compress
    pub cm: u8, // compression method 0: no, 1: defalte
}

pub struct PacketClose {
    pub reason: String,
}

pub struct PacketPing {}
pub struct PacketPong {}

pub struct PacketRetry {
    pub delay: u8,
}

pub struct PacketRedirect {
    pub target: String,
}

pub struct PacketMessage {
    pub id: i32,
    pub pkg_id: i32,
    pub headers: Vec<Vec<u8>>,
    pub payload: Vec<u8>,
    pub socket_id: Vec<u8>,
    pub compress: u8,
}

pub struct PacketAck {
    pub id: i32,
    pub pkg_id: i32,
}

pub struct Transport {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Transport {
    pub async fn connect(mut url: url::Url) -> Result<Self, Error> {
        url.query_pairs_mut()
            .append_pair("ts", &chrono::Utc::now().timestamp_millis().to_string());

        println!("connecting to devtools: {}", url.to_string());
        let (stream, _) = connect_async(url).await?;

        Ok(Transport { stream })

        // while let Some(msg) = stream.next().await {
        //     if msg.is_err() {
        //         // TODO handle error
        //         continue;
        //     }

        //     let data = msg.unwrap().into_data();
        // }

        // todo handle close
    }

    pub async fn next(&mut self) -> Option<Vec<u8>> {
        if let Some(msg) = self.stream.next().await {
            if msg.is_err() {
                // TODO handle error
                // return None;
            }

            let data = msg.unwrap().into_data();

            return Some(data);
        } else {
            None
        }
    }

    fn read_map(cur: &mut Cursor<&Vec<u8>>) {}

    pub fn parse_packet(data: Vec<u8>) -> Vec<Packet> {
        let mut cur = Cursor::new(&data);
        let data_len = data.len() as u64;

        let mut packets: Vec<Packet> = Vec::new();

        while cur.position() < data_len {
            let packet_type = rmp::decode::read_pfix(&mut cur).unwrap();
            match packet_type {
                // open
                6 => {
                    let pi = rmp::decode::read_u8(&mut cur).unwrap();
                    let pt = rmp::decode::read_u8(&mut cur).unwrap();
                    let cs = rmp::decode::read_u8(&mut cur).unwrap();
                    let cm = rmp::decode::read_u8(&mut cur).unwrap();
                    packets.push(Packet::OPEN(PacketOpen { pi, pt, cs, cm }));
                }
                // close
                7 => {
                    let mut reason = Vec::new();
                    let reason = rmp::decode::read_str(&mut cur, &mut reason).unwrap();

                    packets.push(Packet::CLOSE(PacketClose {
                        reason: reason.to_string(),
                    }));
                }
                // ping
                8 => {
                    packets.push(Packet::PING(PacketPing {}));
                }
                // pong
                9 => {
                    packets.push(Packet::PONG(PacketPong {}));
                }
                // retry
                10 => {
                    let delay = rmp::decode::read_pfix(&mut cur).unwrap();
                    packets.push(Packet::RETRY(PacketRetry { delay }));
                }
                // redirect
                11 => {
                    let mut target = Vec::new();
                    let target = rmp::decode::read_str(&mut cur, &mut target).unwrap();
                    packets.push(Packet::REDIRECT(PacketRedirect {
                        target: target.to_string(),
                    }));
                }
                // message
                12 => {
                    let id: i32 = rmp::decode::read_int(&mut cur).unwrap();
                    let pkg_id: i32 = rmp::decode::read_int(&mut cur).unwrap();

                    let header_count = rmp::decode::read_map_len(&mut cur).unwrap();
                    let mut headers: Vec<Vec<u8>>;
                    if header_count != 0 {
                        headers = Vec::with_capacity((header_count * 2) as usize);
                        for _ in 0..header_count {
                            let key_len = rmp::decode::read_str_len(&mut cur).unwrap();
                            let key_end = cur.position() + key_len as u64;
                            let key = data[cur.position() as usize..key_end as usize].to_vec();
                            cur.set_position(key_end);
                            headers.push(key);

                            let val_len = rmp::decode::read_str_len(&mut cur).unwrap();
                            let val_end = cur.position() + val_len as u64;
                            let val = data[cur.position() as usize..val_end as usize].to_vec();
                            cur.set_position(val_end);
                            headers.push(val);
                        }
                    } else {
                        headers = vec![];
                    }

                    let payload_len = rmp::decode::read_bin_len(&mut cur).unwrap();
                    let payload_end = cur.position() + payload_len as u64;
                    let payload = data[cur.position() as usize..payload_end as usize].to_vec();
                    cur.set_position(payload_end);

                    let socket_id_len = rmp::decode::read_str_len(&mut cur).unwrap();
                    let socket_id_end = cur.position() + socket_id_len as u64;
                    let socket_id = data[cur.position() as usize..socket_id_end as usize].to_vec();
                    cur.set_position(socket_id_end);

                    let compress = rmp::decode::read_u8(&mut cur).unwrap();

                    let packet = PacketMessage {
                        id,
                        pkg_id,
                        headers,
                        payload,
                        socket_id,
                        compress,
                    };

                    packets.push(Packet::MESSAGE(packet));
                }
                // ack
                13 => {
                    let id: i32 = rmp::decode::read_int(&mut cur).unwrap();
                    let pkg_id: i32 = rmp::decode::read_int(&mut cur).unwrap();

                    let packet = PacketAck { id, pkg_id };

                    packets.push(Packet::ACK(packet));
                }
                _ => {
                    // unknown packet stop parsing
                    return packets;
                }
            }
        }

        packets
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::gateway::transport::Transport;

    #[test]
    fn parse_packet() {
        let mut buf = Vec::new();
        // message id
        rmp::encode::write_sint(&mut buf, 0).unwrap();
        // package id
        rmp::encode::write_sint(&mut buf, 1).unwrap();
        // headers
        rmp::encode::write_map_len(&mut buf, 2).unwrap();
        rmp::encode::write_str(&mut buf, "a").unwrap();
        rmp::encode::write_str(&mut buf, "1").unwrap();
        rmp::encode::write_str(&mut buf, "b").unwrap();
        rmp::encode::write_str(&mut buf, "2").unwrap();
        // payload
        rmp::encode::write_bin(&mut buf, &[0x01, 0x02, 0x03]).unwrap();

        println!("{:?}", buf);
        Transport::parse_packet(buf);
    }
}
