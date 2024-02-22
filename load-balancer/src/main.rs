use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};

use deadpool::Runtime;
use futures::FutureExt;
use log::{error, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io;
use tokio::net::{TcpListener, TcpStream};

use crate::pool::{Manager, Pool, TcpConnWrapper};

mod pool;

static IDX: AtomicUsize = AtomicUsize::new(0);
static MAX_POOL_SIZE: usize = 40;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "warn");
    env_logger::init();

    let port = env::var("PORT").unwrap_or("9999".to_string());
    let listener = TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    println!("⚗️ 🛜moonshine-lb running at http://localhost:{}/", port);

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        std::process::exit(0)
    });

    run_server(listener).await
}

async fn run_server(listener: TcpListener) -> Result<(), Box<dyn Error>> {
    let api01_host_port = env::var("API01_HOST_PORT").unwrap_or("localhost:3042".to_string());
    let api02_host_port = env::var("API02_HOST_PORT").unwrap_or("localhost:3043".to_string());

    let pool01 = create_pool(api01_host_port.clone());
    let pool02 = create_pool(api02_host_port.clone());
    let pools: [Pool; 2] = [pool01, pool02];

    loop {
        let (socket, _) = listener.accept().await?;
        let backend_idx = IDX.fetch_add(1, Ordering::SeqCst) % pools.len();
        let pool = pools[backend_idx].clone();

        tokio::spawn(async move {
            if let Err(e) = proxy(socket, &pool).await {
                error!("Failed to forward to {}: {}", backend_idx, e);
            }
        });
    }
}

fn create_pool(url: String) -> Pool {
    let manager = Manager::new(url);
    return Pool::builder(manager)
        .max_size(MAX_POOL_SIZE) //30
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap();
}
async fn proxy(mut client: TcpStream, pool: &Pool) -> io::Result<()> {
    // Max buffer size is 4KB
    let mut buf = [0; 4096];

    let n = match client.read(&mut buf).await {
        Ok(n) if n == 0 => return Ok(()),
        Ok(n) => n,
        Err(e) => return Err(e),
    };

    let mut backend = TcpConnWrapper::new(pool.get().await.unwrap());
    backend.write_all(&buf[..n]).await.unwrap();

    let n = backend.read(&mut buf).await.unwrap();
    client.write_all(&buf[..n]).await.unwrap();

    // Exit as soon as the response is sent
    Ok(())
}
