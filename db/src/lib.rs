pub use client::{Client, Pool};
pub use cmd::Command;

pub mod server;
pub mod db;

pub mod cmd;
pub mod client;

const MAX_CONNECTIONS: usize = 2048;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;