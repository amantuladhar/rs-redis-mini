use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::oneshot};

use crate::{
    app_config::AppConfig, cmd_parser::client_cmd::ClientCmd, kvstore::KvChan, resp_type::RESPType,
    KvStoreCmd, LINE_ENDING,
};
use ClientCmd::*;

impl ClientCmd {
    pub async fn process_client_cmd(
        &self,
        writer: &mut WriteHalf<'_>,
        kv_chan: &KvChan,
    ) -> anyhow::Result<()> {
        match self {
            Ping => {
                let resp_type = RESPType::SimpleString("PONG".to_string());
                writer.write_all(&resp_type.as_bytes()).await?;
            }
            Echo(value) => {
                let resp_type = RESPType::BulkString(value.clone());
                writer.write_all(&resp_type.as_bytes()).await?;
            }
            Set { key, value, flags } => {
                let kv_cmd = KvStoreCmd::Set {
                    key: key.clone(),
                    value: value.clone(),
                    flags: flags.clone(),
                };
                kv_chan.send(kv_cmd).await?;
                let resp_type = RESPType::SimpleString("OK".to_string());
                writer.write_all(&resp_type.as_bytes()).await?;
            }
            Get { key } => {
                let (tx, rx) = oneshot::channel::<Option<String>>();
                let kv_cmd = KvStoreCmd::Get {
                    resp: tx,
                    key: key.to_owned(),
                };
                kv_chan.send(kv_cmd).await?;
                match rx.await? {
                    None => {
                        let resp_type = RESPType::NullBulkString;
                        writer.write_all(&resp_type.as_bytes()).await?;
                    }
                    Some(value) => {
                        let resp_type = RESPType::BulkString(value);
                        writer.write_all(&resp_type.as_bytes()).await?;
                    }
                }
            }
            Info { .. } => {
                let is_master = AppConfig::is_master();
                let role = match is_master {
                    true => "master",
                    false => "slave",
                };
                let join = std::str::from_utf8(LINE_ENDING)?;
                let mut info_vec = vec!["# Replication".to_string(), format!("role:{}", role)];
                if is_master {
                    info_vec.push(format!("master_replid:{}", AppConfig::get_master_replid()));
                    info_vec.push(format!(
                        "master_repl_offset:{}",
                        AppConfig::get_master_repl_offset()
                    ));
                }
                let info_string = RESPType::BulkString(info_vec.join(join));
                writer.write_all(&info_string.as_bytes()).await?;
            }
            CustomNewLine | ExitConn => {}
        };
        Ok(())
    }
}
