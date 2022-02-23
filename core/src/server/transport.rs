use std::io::Cursor;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use hyper::upgrade::Upgraded;
use hyper_tungstenite::{
    tungstenite::{Error, Message},
    WebSocketStream,
};

pub enum Packet {
    OPEN(PacketOpen),
    RETRY(PacketRetry),
    RESET(PacketReset),
    REDIRECT(PacketRedirect),
    CLOSE(PacketClose),
    PING(PacketPing),
    PONG(PacketPong),
    MESSAGE(PacketMessage),
    ACK(PacketAck),
}

#[derive(Debug)]
pub struct PacketOpen {
    pub ping_interval: u8, // ping interval second
    pub ping_timeout: u8,  // ping timeout second
}

#[derive(Debug)]
pub struct PacketClose {
    pub reason: String,
}

#[derive(Debug)]
pub struct PacketPing {}
#[derive(Debug)]
pub struct PacketPong {}

#[derive(Debug)]
pub struct PacketRetry {
    pub delay: u8,
}

#[derive(Debug)]
pub struct PacketReset {
    pub delay: u8,
}

#[derive(Debug)]
pub struct PacketRedirect {
    pub delay: u8,
    pub target: String,
}

#[derive(Debug)]
pub struct PacketMessage {
    pub id: i32,
    pub pkg_id: i32,
    pub headers: Vec<Vec<u8>>,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub struct PacketAck {
    pub id: i32,
    pub pkg_id: i32,
}

pub struct Transport {}

impl Transport {
    pub async fn next(stream: &mut SplitStream<WebSocketStream<Upgraded>>) -> Option<Vec<Packet>> {
        if let Some(msg) = stream.next().await {
            let mut packets = Vec::new();

            if msg.is_err() {
                // TODO handle error
                return Some(packets);
            }

            let data = msg.unwrap().into_data();
            let data_len = data.len() as u64;
            let mut cur = Cursor::new(&data);

            while cur.position() < data_len {
                if let Some(packet) = Transport::parse_packet(&mut cur) {
                    packets.push(packet);
                } else {
                    // unkonw packet
                    return Some(packets);
                }
            }

            Some(packets)
        } else {
            None
        }
    }

    pub async fn send_packet(
        sink: &mut SplitSink<WebSocketStream<Upgraded>, Message>,
        packet: Packet,
    ) {
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
            Packet::MESSAGE(msg) => {
                println!("msg: {:?}", msg);
            }
            Packet::ACK(ack) => {
                println!("ack: {:?}", ack);
            }
            _ => {}
        }
    }

    pub async fn send_open_packet(
        sink: &mut SplitSink<WebSocketStream<Upgraded>, Message>,
        ping_interval: i64,
        ping_timeout: i64,
    ) -> Result<(), Error> {
        let mut data = Vec::with_capacity(3);
        rmp::encode::write_sint(&mut data, 1).unwrap();
        rmp::encode::write_sint(&mut data, ping_interval).unwrap();
        rmp::encode::write_sint(&mut data, ping_timeout).unwrap();

        sink.send(Message::Binary(data)).await?;
        Ok(())
    }

    pub async fn send_close_packet(
        sink: &mut SplitSink<WebSocketStream<Upgraded>, Message>,
        reason: &str,
    ) -> Result<(), Error> {
        let mut data = Vec::with_capacity(3);
        rmp::encode::write_sint(&mut data, 5).unwrap();
        rmp::encode::write_str(&mut data, reason).unwrap();

        sink.send(Message::Binary(data)).await?;
        Ok(())
    }

    pub async fn send_ping_packet(
        sink: &mut SplitSink<WebSocketStream<Upgraded>, Message>,
    ) -> Result<(), Error> {
        let mut data = Vec::with_capacity(1);
        rmp::encode::write_sint(&mut data, 6).unwrap();

        sink.send(Message::Binary(data)).await?;
        Ok(())
    }

    pub async fn send_message_packet(
        sink: &mut SplitSink<WebSocketStream<Upgraded>, Message>,
        mut data: Vec<u8>,
    ) -> Result<(), Error> {
        let mut buf = Vec::with_capacity(1 + data.len());
        rmp::encode::write_pfix(&mut buf, 8).unwrap();
        buf.append(&mut data);

        sink.send(Message::Binary(buf)).await?;
        Ok(())
    }

    pub fn parse_packet(cur: &mut Cursor<&Vec<u8>>) -> Option<Packet> {
        let packet_type = match rmp::decode::read_pfix(cur) {
            Ok(v) => v,
            Err(_) => return None,
        };

        match packet_type {
            // open
            1 => {
                let ping_interval = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                let ping_timeout = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };

                return Some(Packet::OPEN(PacketOpen {
                    ping_interval,
                    ping_timeout,
                }));
            }
            // retry
            2 => {
                let delay = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };
                return Some(Packet::RETRY(PacketRetry { delay }));
            }
            // retry
            3 => {
                let delay = match rmp::decode::read_pfix(cur) {
                    Ok(v) => v,
                    Err(_) => return None,
                };
                return Some(Packet::RESET(PacketReset { delay }));
            }
            // redirect
            4 => {
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
            // close
            5 => {
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
            6 => {
                return Some(Packet::PING(PacketPing {}));
            }
            // pong
            7 => {
                return Some(Packet::PONG(PacketPong {}));
            }
            // message
            8 => {
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

                let packet = PacketMessage {
                    id,
                    pkg_id,
                    headers,
                    payload,
                };

                return Some(Packet::MESSAGE(packet));
            }
            // ack
            9 => {
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
