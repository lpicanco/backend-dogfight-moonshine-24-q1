use log::info;
use tokio::io::{AsyncReadExt, BufWriter};
use tokio::net::TcpStream;

use crate::db::Db;

pub struct Unlock {
    partition_name: String
}

impl Unlock {
    pub(crate) async fn parse_data(stream: &mut BufWriter<TcpStream>) -> crate::Result<Unlock> {
        let partition_name_size = stream.read_u8().await?;
        let mut partition_name = vec![0; partition_name_size as usize];
        stream.read_exact(&mut partition_name).await?;

        return Ok(Unlock {
            partition_name: String::from_utf8(partition_name).unwrap().trim_end().to_string()
        });
    }

    pub(crate) async fn execute(self, db: &Db) -> crate::Result<()> {
        info!("Unlock::execute: partition_name: {:?}", self.partition_name);

        db.unlock(self.partition_name).await;
        Ok(())
    }
}