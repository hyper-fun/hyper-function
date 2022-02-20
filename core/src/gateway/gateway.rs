use tokio::sync::mpsc;

use super::transport::Transport;

pub struct Gateway {
    pub dev: bool,
    pub runway: url::Url,
    pub read_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl Gateway {
    pub async fn connect(&self) {
        let mut transport = Transport::connect(self.runway.clone())
            .await
            .expect("failed to connect to devtools");

        while let Some(messages) = transport.next().await {
            for mut msg in messages {
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

                self.read_tx
                    .send(data)
                    .expect("failed to send message to read_tx");
            }
        }

        println!("transport disconnected");
    }
}
