use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf};
use tracing::debug;

use crate::{cmd_parser::slave_command::SlaveCommand, database::Database, resp_type::RESPType};
use SlaveCommand::*;

impl SlaveCommand {
    pub async fn process_slave_cmd(
        &self,
        writer: &mut WriteHalf<'_>,
        bytes_received: usize,
    ) -> anyhow::Result<()> {
        match self {
            Ping => (),
            Set { key, value, flags } => {
                let _ = Database::set(key, value, flags).await?;
            }
            ReplConf { .. } => {
                let resp_type = RESPType::Array(vec![
                    RESPType::BulkString("REPLCONF".to_string()),
                    RESPType::BulkString("ACK".to_string()),
                    RESPType::BulkString(format!("{}", bytes_received)),
                ]);
                let content = String::from_utf8(resp_type.as_bytes())?;
                debug!("REpl conf content = {content:?}");
                writer.write_all(&resp_type.as_bytes()).await?;
            }
        };
        Ok(())
    }
}
