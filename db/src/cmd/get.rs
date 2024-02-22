use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::db::Db;

pub struct Get {
    partition_name: String,
    key: String,
}

impl Get {
    pub(crate) async fn parse_data(stream: &mut BufWriter<TcpStream>) -> crate::Result<Get> {
        let partition_name_size = stream.read_u8().await?;
        let mut partition_name = vec![0; partition_name_size as usize];
        stream.read_exact(&mut partition_name).await?;

        let key_size = stream.read_u8().await?;
        let mut key = vec![0; key_size as usize];
        stream.read_exact(&mut key).await?;

        return Ok(Get {
            partition_name: String::from_utf8(partition_name).unwrap().trim_end().to_string(),
            key: String::from_utf8(key).unwrap().trim_end().to_string(),
        });
    }

    pub(crate) async fn execute(self, buffer: &mut BufWriter<TcpStream>, db: &Db) -> crate::Result<()> {
        // info!("Get::execute: partition_name: {:?}, key: {:?}", self.partition_name, self.key);

        let data = db.read_from_partition(self.partition_name, self.key).await?;
        buffer.write_u16(data.len() as u16).await?;
        buffer.write_all(&data).await?;
        Ok(())
    }
}