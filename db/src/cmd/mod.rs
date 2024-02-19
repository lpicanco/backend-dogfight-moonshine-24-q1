use tokio::io::{BufWriter};
use tokio::net::TcpStream;

pub use echo::Echo;
pub use open_partition::OpenPartition;
pub use put::Put;
pub use get::Get;
pub use get_last::GetLast;

use crate::db::Db;

mod echo;
mod open_partition;
mod put;
mod get;
mod get_last;

pub enum Command {
    Echo(Echo),
    Open(OpenPartition),
    Put(Put),
    Get(Get),
    GetLast(GetLast),
}

pub(crate) const CMD_ECHO_OPCODE: u8 = 65;
pub(crate) const CMD_OPEN_OPCODE: u8 = 66;
pub(crate) const CMD_PUT_OPCODE: u8 = 67;
pub(crate) const CMD_GET_OPCODE: u8 = 68;
pub(crate) const CMD_GET_LAST_OPCODE: u8 = 69;

impl Command {
    pub(crate) async fn execute(self, db: &Db, buffer: &mut BufWriter<TcpStream>) -> crate::Result<()> {
        match self {
            Command::Echo(cmd) => cmd.execute(buffer).await,
            Command::Open(cmd) => cmd.execute(db).await,
            Command::Put(cmd) => cmd.execute(db).await,
            Command::Get(cmd) => cmd.execute(buffer, db).await,
            Command::GetLast(cmd) => cmd.execute(buffer, db).await,
        }
    }
    pub(crate) async fn from_data(cmd: u8, data: &mut BufWriter<TcpStream>) -> crate::Result<Command> {

        let command = match cmd {
            CMD_ECHO_OPCODE => Command::Echo(Echo::parse_data(data).await?),
            CMD_OPEN_OPCODE => Command::Open(OpenPartition::parse_data(data).await?),
            CMD_PUT_OPCODE => Command::Put(Put::parse_data(data).await?),
            CMD_GET_OPCODE => Command::Get(Get::parse_data(data).await?),
            CMD_GET_LAST_OPCODE => Command::GetLast(GetLast::parse_data(data).await?),
            _ => return Err(format!("Unknown command: {}", cmd).into()),
        };

        Ok(command)
    }
}
