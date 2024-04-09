use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf};
use tracing::debug;

use crate::{
    cmd_parser::slave_cmd::SlaveCmd,
    database::{DatabaseEvent, DatabaseEventListener},
    resp_type::RESPType,
};
use SlaveCmd::*;

impl SlaveCmd {
    pub async fn process_slave_cmd(
        &self,
        writer: &mut WriteHalf<'_>,
        kv_chan: &DatabaseEventListener,
        bytes_received: usize,
    ) -> anyhow::Result<()> {
        match self {
            Ping => {
                // not need to process
            }
            Set { key, value, flags } => {
                let kv_cmd = DatabaseEvent::Set {
                    key: key.clone(),
                    value: value.clone(),
                    flags: flags.clone(),
                };
                kv_chan.send(kv_cmd).await?;
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
