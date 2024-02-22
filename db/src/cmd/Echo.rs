use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

pub struct Echo {
    data: Vec<u8>,
}

impl Echo {
    pub(crate) async fn parse_data(stream: &mut BufWriter<TcpStream>) -> crate::Result<Echo> {
        let data_size = stream.read_u16().await?;
        info!("data_size: {:?}", data_size);
        let mut data = vec![0; data_size as usize];
        stream.read_exact(&mut data).await?;

        return Ok(Echo {
            data
        });
    }

    pub(crate) async fn execute(&self, buffer: &mut BufWriter<TcpStream>) -> crate::Result<()> {
        let response = format!("PONG: {}", String::from_utf8(self.data.clone()).unwrap());
        buffer.write_u16(response.len() as u16).await?;

        buffer
            .write_all(response.as_bytes())
            .await
            .expect("failed to write data to socket");

        Ok(())
    }
}