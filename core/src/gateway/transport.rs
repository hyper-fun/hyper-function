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
    pub ping_interval: u8,   // ping interval second
    pub ping_timeout: u8,    // ping timeout second
    pub compress_size: u8, // min compress size kb, message payload great than this should be compress
    pub compress_method: u8, // compression method 0: no, 1: defalte
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
    pub delay: u8,
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

    pub async fn next(&mut self) -> Option<Vec<Packet>> {
        if let Some(msg) = self.stream.next().await {
            if msg.is_err() {
                // TODO handle error
                // return None;
            }

            let data = msg.unwrap().into_data();
            let mut cur = Cursor::new(&data);

            let data_len = data.len() as u64;
            let mut packets: Vec<Packet> = Vec::new();

            while cur.position() < data_len {
                let packet = Transport::parse_packet(&mut cur);
            }

            return Some(packets);
        } else {
            None
        }
    }

    pub fn parse_packet(cur: &mut Cursor<&Vec<u8>>) -> Option<Packet> {
        let packet_type = match rmp::decode::read_pfix(cur) {
            Ok(v) => v,
            Err(_) => return None,
        };

        match packet_type {
            // open
            6 => {
                let ping_interval = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let ping_timeout = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let compress_size = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let compress_method = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                return Some(Packet::OPEN(PacketOpen {
                    ping_interval,
                    ping_timeout,
                    compress_size,
                    compress_method,
                }));
            }
            // close
            7 => {
                let reason_len = match rmp::decode::read_str_len(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let reason_end = cur.position() + reason_len as u64;
                let reason =
                    cur.get_ref().as_slice()[cur.position() as usize..reason_end as usize].to_vec();
                cur.set_position(reason_end);

                let reason = match String::from_utf8(reason) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                return Some(Packet::CLOSE(PacketClose { reason }));
            }
            // ping
            8 => {
                return Some(Packet::PING(PacketPing {}));
            }
            // pong
            9 => {
                return Some(Packet::PONG(PacketPong {}));
            }
            // retry
            10 => {
                let delay = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };
                return Some(Packet::RETRY(PacketRetry { delay }));
            }
            // redirect
            11 => {
                let delay = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let target_len = match rmp::decode::read_str_len(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let target_end = cur.position() + target_len as u64;
                let target =
                    cur.get_ref().as_slice()[cur.position() as usize..target_end as usize].to_vec();
                cur.set_position(target_end);

                let target = match String::from_utf8(target) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                return Some(Packet::REDIRECT(PacketRedirect { delay, target }));
            }
            // message
            12 => {
                let data = cur.get_ref().as_slice();

                let id: i32 = match rmp::decode::read_int(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let pkg_id: i32 = match rmp::decode::read_int(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let header_count = match rmp::decode::read_map_len(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let mut headers: Vec<Vec<u8>>;
                if header_count != 0 {
                    headers = Vec::with_capacity((header_count * 2) as usize);
                    for _ in 0..header_count {
                        let key_len = match rmp::decode::read_str_len(cur) {
                            Ok(v) => v,
                            Err(_) => return None,
                        };

                        let key_end = cur.position() + key_len as u64;
                        let key = data[cur.position() as usize..key_end as usize].to_vec();
                        cur.set_position(key_end);
                        headers.push(key);

                        let val_len = match rmp::decode::read_str_len(cur) {
                            Ok(v) => v,
                            Err(_) => return None,
                        };

                        let val_end = cur.position() + val_len as u64;
                        let val = data[cur.position() as usize..val_end as usize].to_vec();
                        cur.set_position(val_end);
                        headers.push(val);
                    }
                } else {
                    headers = vec![];
                }

                let payload_len = match rmp::decode::read_bin_len(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let payload_end = cur.position() + payload_len as u64;
                let payload = data[cur.position() as usize..payload_end as usize].to_vec();
                cur.set_position(payload_end);

                let socket_id_len = match rmp::decode::read_str_len(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let socket_id_end = cur.position() + socket_id_len as u64;
                let socket_id = data[cur.position() as usize..socket_id_end as usize].to_vec();
                cur.set_position(socket_id_end);

                let compress = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let packet = PacketMessage {
                    id,
                    pkg_id,
                    headers,
                    payload,
                    socket_id,
                    compress,
                };

                return Some(Packet::MESSAGE(packet));
            }
            // ack
            13 => {
                let id: i32 = match rmp::decode::read_int(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let pkg_id: i32 = match rmp::decode::read_int(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let packet = PacketAck { id, pkg_id };

                return Some(Packet::ACK(packet));
            }
            _ => {
                // unknown packet stop parsing
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::gateway::transport::*;

    #[test]
    fn decode_packet_open() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 6).unwrap();
        rmp::encode::write_pfix(&mut data, 25).unwrap();
        rmp::encode::write_pfix(&mut data, 20).unwrap();
        rmp::encode::write_pfix(&mut data, 0).unwrap();
        rmp::encode::write_pfix(&mut data, 0).unwrap();

        let mut cur = Cursor::new(&data);
        let packet = Transport::parse_packet(&mut cur).expect("parse packet open failed");
        match packet {
            Packet::OPEN(packet) => {
                assert_eq!(packet.ping_interval, 25);
                assert_eq!(packet.ping_timeout, 20);
                assert_eq!(packet.compress_size, 0);
                assert_eq!(packet.compress_method, 0);
            }
            _ => panic!("parse packet open failed"),
        }
    }

    #[test]
    fn decode_packet_close() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 7).unwrap();
        rmp::encode::write_str(&mut data, "no reason").unwrap();

        let mut cur = Cursor::new(&data);
        let packet = Transport::parse_packet(&mut cur).expect("parse packet close failed");
        match packet {
            Packet::CLOSE(packet) => {
                assert_eq!(packet.reason, "no reason");
            }
            _ => panic!("parse packet open failed"),
        }
    }

    #[test]
    fn decode_packet_ping() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 8).unwrap();

        let mut cur = Cursor::new(&data);
        let packet = Transport::parse_packet(&mut cur).expect("parse packet ping failed");
        match packet {
            Packet::PING(packet) => {}
            _ => panic!("parse packet ping failed"),
        }
    }

    #[test]
    fn decode_packet_pong() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 9).unwrap();

        let mut cur = Cursor::new(&data);
        let packet = Transport::parse_packet(&mut cur).expect("parse packet pong failed");
        match packet {
            Packet::PONG(packet) => {}
            _ => panic!("parse packet pong failed"),
        }
    }

    #[test]
    fn decode_packet_retry() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 10).unwrap();
        rmp::encode::write_pfix(&mut data, 3).unwrap();

        let mut cur = Cursor::new(&data);
        let packet = Transport::parse_packet(&mut cur).expect("parse packet retry failed");
        match packet {
            Packet::RETRY(packet) => {
                assert_eq!(packet.delay, 3);
            }
            _ => panic!("parse packet retry failed"),
        }
    }

    #[test]
    fn decode_packet_redirect() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 11).unwrap();
        rmp::encode::write_pfix(&mut data, 6).unwrap();
        rmp::encode::write_str(&mut data, "123").unwrap();

        let mut cur = Cursor::new(&data);
        let packet = Transport::parse_packet(&mut cur).expect("parse packet redirect failed");
        match packet {
            Packet::REDIRECT(packet) => {
                assert_eq!(packet.delay, 6);
                assert_eq!(packet.target, "123");
            }
            _ => panic!("parse packet redirect failed"),
        }
    }

    #[test]
    fn decode_packet_message() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 12).unwrap();
        // message id
        rmp::encode::write_sint(&mut data, 1).unwrap();
        // package id
        rmp::encode::write_sint(&mut data, 2).unwrap();
        // headers
        rmp::encode::write_map_len(&mut data, 3).unwrap();
        rmp::encode::write_str(&mut data, "a").unwrap();
        rmp::encode::write_str(&mut data, "1").unwrap();
        rmp::encode::write_str(&mut data, "b").unwrap();
        rmp::encode::write_str(&mut data, "2").unwrap();
        rmp::encode::write_str(&mut data, "c").unwrap();
        rmp::encode::write_str(&mut data, "3").unwrap();
        // payload
        rmp::encode::write_bin(&mut data, &[0x01, 0x02, 0x03]).unwrap();
        // socket id
        rmp::encode::write_str(&mut data, "socketid:1").unwrap();
        // compress
        rmp::encode::write_pfix(&mut data, 0).unwrap();

        let mut cur = Cursor::new(&data);
        let msg = Transport::parse_packet(&mut cur).expect("parse packet message failed");
        match msg {
            Packet::MESSAGE(msg) => {
                assert_eq!(msg.id, 1);
                assert_eq!(msg.pkg_id, 2);
                assert_eq!(msg.headers.len(), 3 * 2);
                assert_eq!(msg.headers[0], "a".as_bytes());
                assert_eq!(msg.headers[1], "1".as_bytes());
                assert_eq!(msg.headers[2], "b".as_bytes());
                assert_eq!(msg.headers[3], "2".as_bytes());
                assert_eq!(msg.headers[4], "c".as_bytes());
                assert_eq!(msg.headers[5], "3".as_bytes());
                assert_eq!(msg.payload, vec![0x01, 0x02, 0x03]);
            }
            _ => panic!("should be message"),
        }
    }

    #[test]
    fn decode_packet_message_with_no_headers() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 12).unwrap();
        // message id
        rmp::encode::write_sint(&mut data, 1).unwrap();
        // package id
        rmp::encode::write_sint(&mut data, 2).unwrap();
        // headers
        rmp::encode::write_map_len(&mut data, 0).unwrap();
        // payload
        rmp::encode::write_bin(&mut data, &[0x01, 0x02, 0x03]).unwrap();
        // socket id
        rmp::encode::write_str(&mut data, "socketid:1").unwrap();
        // compress
        rmp::encode::write_pfix(&mut data, 0).unwrap();

        let mut cur = Cursor::new(&data);
        let msg = Transport::parse_packet(&mut cur).expect("parse packet message failed");
        match msg {
            Packet::MESSAGE(msg) => {
                assert_eq!(msg.id, 1);
                assert_eq!(msg.pkg_id, 2);
                assert_eq!(msg.headers.len(), 0);
                assert_eq!(msg.payload, vec![0x01, 0x02, 0x03]);
            }
            _ => panic!("should be message"),
        }
    }

    #[test]
    fn decode_packet_message_with_three_message_mixin() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 12).unwrap();
        // message id
        rmp::encode::write_sint(&mut data, 1).unwrap();
        // package id
        rmp::encode::write_sint(&mut data, 2).unwrap();
        // headers
        rmp::encode::write_map_len(&mut data, 0).unwrap();
        // payload
        rmp::encode::write_bin(&mut data, &[0x01, 0x02, 0x03]).unwrap();
        // socket id
        rmp::encode::write_str(&mut data, "socketid:1").unwrap();
        // compress
        rmp::encode::write_pfix(&mut data, 0).unwrap();

        let data = vec![data.as_slice(), data.as_slice(), data.as_slice()].concat();

        let mut msgs: Vec<PacketMessage> = Vec::new();
        let mut cur = Cursor::new(&data);

        while cur.position() < data.len() as u64 {
            let msg = Transport::parse_packet(&mut cur).unwrap();
            match msg {
                Packet::MESSAGE(msg) => {
                    msgs.push(msg);
                }
                _ => panic!("should be message"),
            }
        }

        assert_eq!(msgs.len(), 3);

        for msg in msgs {
            assert_eq!(msg.id, 1);
            assert_eq!(msg.pkg_id, 2);
            assert_eq!(msg.headers.len(), 0);
            assert_eq!(msg.payload, vec![0x01, 0x02, 0x03]);
        }
    }

    #[test]
    fn decode_packet_ack() {
        let mut data = Vec::new();
        // packet message
        rmp::encode::write_pfix(&mut data, 13).unwrap();
        rmp::encode::write_pfix(&mut data, 3).unwrap();
        rmp::encode::write_pfix(&mut data, 8).unwrap();

        let mut cur = Cursor::new(&data);
        let packet = Transport::parse_packet(&mut cur).expect("parse packet ack failed");
        match packet {
            Packet::ACK(packet) => {
                assert_eq!(packet.id, 3);
                assert_eq!(packet.pkg_id, 8);
            }
            _ => panic!("parse packet ack failed"),
        }
    }

    #[test]
    fn wrong_data_should_return_none() {
        let mut data = Vec::new();
        // packet ack
        rmp::encode::write_pfix(&mut data, 13).unwrap();
        rmp::encode::write_pfix(&mut data, 3).unwrap();
        // wrong type
        rmp::encode::write_str(&mut data, "bla").unwrap();

        let mut cur = Cursor::new(&data);
        let packet = Transport::parse_packet(&mut cur);
        assert_eq!(packet.is_none(), true);
    }
}
