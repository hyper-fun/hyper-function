use super::transport::Transport;

pub struct Gateway {
    pub dev: bool,
    pub runway: url::Url,
}

impl Gateway {
    pub async fn connect(&self) {
        let mut transport = Transport::connect(self.runway.clone())
            .await
            .expect("failed to connect to devtools");

        // while let Some(data) = transport.next().await {
        //     println!("{}", String::from_utf8_lossy(&data));
        // }
    }
}
