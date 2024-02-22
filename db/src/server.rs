use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use log::warn;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

use crate::{Command, MAX_CONNECTIONS};
use crate::db::Db;

struct Listener {
    db: Arc<Db>,
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
}

impl Listener {
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let (socket, _) = self.listener.accept().await?;
            let mut handler = Handler {
                db: self.db.clone(),
                stream: BufWriter::new(socket),
            };

            // Spawn a new task to process the connection
            tokio::spawn(async move {
                handler.process().await.expect("failed to process connection");
                drop(permit);
            });
        }
    }
}

struct Handler {
    db: Arc<Db>,
    stream: BufWriter<TcpStream>,
}

impl Handler {
    async fn process(&mut self) -> crate::Result<()> {
        loop {
            let mut cmd_op = vec![0; 1];

            let n = match self.stream.read(&mut cmd_op).await {
                Ok(n) => n,
                Err(e) => {
                    warn!("failed to read from socket; err = {:?}", e);
                    return Ok(());
                }
            };

            if n == 0 {
                warn!("socket closed");
                return Ok(());
            }

            let cmd = Command::from_data(cmd_op[0], &mut self.stream).await;

            match cmd {
                Ok(cmd) => cmd.execute(&self.db, &mut self.stream).await?,
                Err(e) => {
                    return Err(e);
                }
            }

            self.stream.flush().await?;
        }
    }
}

pub async fn run<P: AsRef<Path>>(listener: TcpListener, db_path: P) -> Result<(), Box<dyn Error>> {
    let mut server = Listener {
        listener,
        db: Arc::new(Db::new(db_path)),
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
    };

    server.run().await
}

