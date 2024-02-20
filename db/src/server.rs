use std::error::Error;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use log::{error, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use crate::db::Db;
use crate::{Command, MAX_CONNECTIONS};


struct Listener {
/*    /// Shared database handle.
    ///
    /// Contains the key / value store as well as the broadcast channels for
    /// pub/sub.
    ///
    /// This holds a wrapper around an `Arc`. The internal `Db` can be
    /// retrieved and passed into the per connection state (`Handler`).*/
    // db_holder: DbDropGuard,
    db: Arc<Db>,

    /// TCP listener supplied by the `run` caller.
    listener: TcpListener,

    /// Limit the max number of connections.
    ///
    /// A `Semaphore` is used to limit the max number of connections. Before
    /// attempting to accept a new connection, a permit is acquired from the
    /// semaphore. If none are available, the listener waits for one.
    ///
    /// When handlers complete processing a connection, the permit is returned
    /// to the semaphore.
    limit_connections: Arc<Semaphore>,

/*    /// Broadcasts a shutdown signal to all active connections.
    ///
    /// The initial `shutdown` trigger is provided by the `run` caller. The
    /// server is responsible for gracefully shutting down active connections.
    /// When a connection task is spawned, it is passed a broadcast receiver
    /// handle. When a graceful shutdown is initiated, a `()` value is sent via
    /// the broadcast::Sender. Each active connection receives it, reaches a
    /// safe terminal state, and completes the task.*/
    // notify_shutdown: broadcast::Sender<()>,

/*    /// Used as part of the graceful shutdown process to wait for client
    /// connections to complete processing.
    ///
    /// Tokio channels are closed once all `Sender` handles go out of scope.
    /// When a channel is closed, the receiver receives `None`. This is
    /// leveraged to detect all connection handlers completing. When a
    /// connection handler is initialized, it is assigned a clone of
    /// `shutdown_complete_tx`. When the listener shuts down, it drops the
    /// sender held by this `shutdown_complete_tx` field. Once all handler tasks
    /// complete, all clones of the `Sender` are also dropped. This results in
    /// `shutdown_complete_rx.recv()` completing with `None`. At this point, it
    /// is safe to exit the server process.
*/    // shutdown_complete_tx: mpsc::Sender<()>,
}

impl Listener {
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            // Wait for a permit to become available
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            // Accept a new connection
            let (socket, _) = self.listener.accept().await?;
            let mut handler = Handler {
                // socket,
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
    // shutdown: broadcast::Receiver<()>,
    // connection: Connection,
    // socket: tokio::net::TcpStream,
    stream: BufWriter<TcpStream>,
}

impl Handler {
    async fn process(&mut self) -> crate::Result<()> {
        // let mut conn = Connection::new(socket, self.db.clone(), self.shutdown.clone());
        // conn.process().await?;
        // drop(permit);
        info!("socket: {:?}", self.stream.get_ref());
        // println!("socket: {:?}", self.socket);
        // println!("socket: {:?}", self.stream.get_ref());

        loop {
            // let mut buf = vec![0; 1024];
            let mut cmd_op = vec![0; 1];

            let n = match self.stream.read(&mut cmd_op).await {
                Ok(n) => n,
                Err(e) => {
                    warn!("failed to read from socket; err = {:?}", e);
                    return Ok(());
                }
            };
            // let n = self.stream
            //     .read(&mut cmd_op)
            //     .await;

            if n == 0 {
                warn!("socket closed");
                return Ok(());
            }

            info!("cmd_op: {:?}", cmd_op[0]);
            // let mut cmd = Command::from_data(&mut self.stream).await?;
            // let mut cmd = Command::from_data(buf[0..n].to_vec()).await?;
            let cmd = Command::from_data(cmd_op[0], &mut self.stream).await;

            match cmd {
                Ok(cmd) => cmd.execute(&self.db, &mut self.stream).await?,
                Err(e) => {
                    return Err(e);
                }
            }

            // info!("data: {:?}", &buf[0..n]);

            // self.stream
            //     // self.socket
            //     .write_all(&buf[0..n])
            //     .await
            //     .expect("failed to write data to socket");

            self.stream.flush().await?;
        }
    }
}

pub async fn run<P: AsRef<Path>>(listener: TcpListener, db_path: P,  signal: impl Future) {
    // let _ = shutdown_complete_rx.recv().await;
    let mut server = Listener {
        listener,
        db: Arc::new(Db::new(db_path)),
        // db_holder: DbDropGuard::new(),
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        // notify_shutdown,
        // shutdown_complete_tx,
    };

    tokio::select! {
        res = server.run() => {
            // If an error is received here, accepting connections from the TCP
            // listener failed multiple times and the server is giving up and
            // shutting down.
            //
            // Errors encountered when handling individual connections do not
            // bubble up to this point.
            if let Err(err) = res {
                error!("failed to accept: {}", err);
            }
        }
        _ = signal => {
            // The shutdown signal has been received.
            info!("shutting down");
        }
    }
}

