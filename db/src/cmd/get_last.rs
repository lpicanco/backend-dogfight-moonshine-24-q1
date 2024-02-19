use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use crate::db::Db;

pub struct GetLast {
    partition_name: String,
    count: u16,
}

impl GetLast {
    pub(crate) async fn parse_data(stream: &mut BufWriter<TcpStream>) -> crate::Result<GetLast> {
        let partition_name_size = stream.read_u8().await?;
        let mut partition_name = vec![0; partition_name_size as usize];
        stream.read_exact(&mut partition_name).await?;

        let count = stream.read_u16().await?;

        return Ok(GetLast {
            partition_name: String::from_utf8(partition_name).unwrap().trim_end().to_string(),
            count,
        });
    }

    pub(crate) async fn execute(self, buffer: &mut BufWriter<TcpStream>, db: &Db) -> crate::Result<()> {
        info!("GetLast::execute: partition_name: {:?}, count: {:?}", self.partition_name, self.count);

        let data = db.clone().read_last_from_partition(self.partition_name, self.count).await?;
        buffer.write_u16(data.len() as u16).await?;

        for d in data {
            buffer.write_u16(d.len() as u16).await?;
            buffer.write_all(&d).await?;
        }

        Ok(())
    }
}