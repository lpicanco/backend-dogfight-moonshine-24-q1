use log::info;
use tokio::io::{AsyncReadExt, BufWriter};
use tokio::net::TcpStream;
use crate::db::Db;

pub struct OpenPartition {
    data: Vec<u8>,
}

impl OpenPartition {
    pub(crate) async fn parse_data(stream: &mut BufWriter<TcpStream>) -> crate::Result<OpenPartition> {
        let data_size = stream.read_u8().await?;
        let mut data = vec![0; data_size as usize];
        stream.read_exact(&mut data).await?;

        return Ok(OpenPartition {
            data,
        });
    }

    pub(crate) async fn execute(self, db: &Db) -> crate::Result<()> {
        let partition_name = String::from_utf8(self.data).unwrap();
        info!("OpenPartition::execute: {:?}", partition_name.trim_end());

        db.clone().open_partition(partition_name.trim_end().to_string()).await
    }
}