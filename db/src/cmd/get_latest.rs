use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::db::Db;

pub struct GetLatest {
    partition_name: String,
    lock: bool,
}

impl GetLatest {
    pub(crate) async fn parse_data(stream: &mut BufWriter<TcpStream>) -> crate::Result<GetLatest> {
        let partition_name_size = stream.read_u8().await?;
        let mut partition_name = vec![0; partition_name_size as usize];
        stream.read_exact(&mut partition_name).await?;

        let lock = stream.read_u8().await?;

        return Ok(GetLatest {
            partition_name: String::from_utf8(partition_name).unwrap().trim_end().to_string(),
            lock: lock == 1,
        });
    }

    pub(crate) async fn execute(self, buffer: &mut BufWriter<TcpStream>, db: &Db) -> crate::Result<()> {
        // info!("GetLatest::execute: partition_name: {:?}, lock: {:?}", self.partition_name, self.lock);

        let data = db.read_latest_from_partition(self.partition_name, self.lock).await?;
        buffer.write_u16(data.len() as u16).await?;
        buffer.write_all(&data).await?;
        Ok(())
    }
}