pub mod server;
pub mod db;

pub mod cmd;
pub use cmd::Command;

pub mod client;
pub use client::{Client, Pool};


const MAX_CONNECTIONS: usize = 600;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;