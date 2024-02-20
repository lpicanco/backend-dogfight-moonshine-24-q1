use log::info;
use tokio::io::{AsyncReadExt, BufWriter};
use tokio::net::TcpStream;

use crate::db::Db;

pub struct Put {
    partition_name: String,
    key: String,
    data: Vec<u8>,
    unlock: bool,
}

// echo '4303414243024241704156' | xxd -r -p | nc localhost 9942
impl Put {
    pub(crate) async fn parse_data(stream: &mut BufWriter<TcpStream>) -> crate::Result<Put> {
        let partition_name_size = stream.read_u8().await?;
        let mut partition_name = vec![0; partition_name_size as usize];
        stream.read_exact(&mut partition_name).await?;

        let key_size = stream.read_u8().await?;
        let mut key = vec![0; key_size as usize];
        stream.read_exact(&mut key).await?;

        let data_size = stream.read_u16().await?;
        let mut data = vec![0; data_size as usize];
        stream.read_exact(&mut data).await?;

        let unlock = stream.read_u8().await?;

        return Ok(Put {
            partition_name: String::from_utf8(partition_name).unwrap().trim_end().to_string(),
            key: String::from_utf8(key).unwrap().trim_end().to_string(),
            data,
            unlock: unlock == 1,
        });
    }

    pub(crate) async fn execute(self, db: &Db) -> crate::Result<()> {
        // self.data
        // let partition_name = String::from_utf8(self.data[1..].to_vec()).unwrap();
        // info!("OpenPartition::execute: {:?}", partition_name.trim_end());
        info!("Put::execute: partition_name: {:?}, key: {:?}, data: {:?}, unlock: {:?}", self.partition_name, self.key, self.data, self.unlock);
        db.clone().write_to_partition(self.partition_name, self.key, self.data, self.unlock).await?;
        Ok(())
    }
}