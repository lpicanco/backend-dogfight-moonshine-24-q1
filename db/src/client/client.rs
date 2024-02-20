use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct Client {
    stream: BufWriter<TcpStream>,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> {
        let socket = TcpStream::connect(addr).await?;
        let stream = BufWriter::new(socket);
        Ok(Client { stream })
    }

    pub async fn echo(&mut self, msg: Vec<u8>) -> crate::Result<Vec<u8>> {
        self.stream.write_u8(crate::cmd::CMD_ECHO_OPCODE).await?;
        self.stream.write_u16(msg.len() as u16).await?;
        self.stream.write_all(&msg).await?;
        self.stream.flush().await?;

        Ok(self.read_response().await?)
    }

    pub async fn open_partition(&mut self, partition: &str) -> crate::Result<()> {
        self.stream.write_u8(crate::cmd::CMD_OPEN_OPCODE).await?;
        self.stream.write_u8(partition.len() as u8).await?;
        self.stream.write_all(partition.as_bytes()).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn put(&mut self, partition: &str, key: &str, value: Vec<u8>, unlock: bool) -> crate::Result<()> {
        self.stream.write_u8(crate::cmd::CMD_PUT_OPCODE).await?;
        self.stream.write_u8(partition.len() as u8).await?;
        self.stream.write_all(partition.as_bytes()).await?;

        self.stream.write_u8(key.len() as u8).await?;
        self.stream.write_all(key.as_bytes()).await?;

        self.stream.write_u16(value.len() as u16).await?;
        self.stream.write_all(&value).await?;
        self.stream.write_u8(unlock as u8).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn get(&mut self, partition: &str, key: &str) -> crate::Result<Vec<u8>> {
        self.stream.write_u8(crate::cmd::CMD_GET_OPCODE).await?;
        self.stream.write_u8(partition.len() as u8).await?;
        self.stream.write_all(partition.as_bytes()).await?;

        self.stream.write_u8(key.len() as u8).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream.flush().await?;
        Ok(self.read_response().await?)
    }

    pub async fn get_latest(&mut self, partition: &str, lock: bool) -> crate::Result<Option<Vec<u8>>> {
        self.stream.write_u8(crate::cmd::CMD_GET_LATEST_OPCODE).await?;
        self.stream.write_u8(partition.len() as u8).await?;
        self.stream.write_all(partition.as_bytes()).await?;

        self.stream.write_u8(lock as u8).await?;
        self.stream.flush().await?;

        return match self.read_response().await? {
            response if response.is_empty() => Ok(None),
            response => Ok(Some(response)),
        };
    }

    pub async fn unlock(&mut self, partition: &str) -> crate::Result<()> {
        self.stream.write_u8(crate::cmd::CMD_UNLOCK_OPCODE).await?;
        self.stream.write_u8(partition.len() as u8).await?;
        self.stream.write_all(partition.as_bytes()).await?;
        self.stream.flush().await?;

        return Ok(());
    }

    pub async fn get_last(&mut self, partition: &str, count: u16) -> crate::Result<Vec<Vec<u8>>> {
        self.stream.write_u8(crate::cmd::CMD_GET_LAST_OPCODE).await?;
        self.stream.write_u8(partition.len() as u8).await?;
        self.stream.write_all(partition.as_bytes()).await?;

        self.stream.write_u16(count).await?;
        self.stream.flush().await?;

        let response_len = self.stream.read_u16().await?;

        let mut response = Vec::with_capacity(response_len as usize);
        for _ in 0..response_len {
            let len = self.stream.read_u16().await?;
            let mut data = vec![0; len as usize];
            self.stream.read_exact(&mut data).await?;
            response.push(data);
        }
        Ok(response)
    }

    pub fn is_closed(&self) -> bool {
        return self.stream.get_ref().peer_addr().is_err()
    }

    async fn read_response(&mut self) -> crate::Result<Vec<u8>> {
        let response_len = self.stream.read_u16().await?;
        let mut response = vec![0; response_len as usize];
        self.stream.read_exact(&mut response).await?;
        Ok(response)
    }
}